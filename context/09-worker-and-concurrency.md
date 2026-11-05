# 09 — Worker Thread and Concurrency

This document describes the single worker thread, task serialization, panic
handling, and how network work is off‑loaded to short‑lived sub‑threads.

## 1. Process and Thread Model

```
Native process
├─ Main / UI thread
│  eframe event loop
│  NixobdoPdfApp (all UI state, egui textures)
│  └─ task_tx → msg_rx (mpsc channels)
└─ Worker thread (one, started in Default)
   owns Pdfium; serialises Load/Export/Save*
   └─ short-lived sub‑threads:
        CheckUpdate / DownloadUpdate (HTTP)
        AiSummarize (LLM HTTP)
```

- **One worker thread** owns the `Pdfium` instance. `pdfium-render` bindings are
  **not** shared with the UI thread; all PDF operations (load, render, save)
  must go through this thread.
- **Channels:** `pdf_task_tx: Sender<PdfWorkerTask>` (main → worker) and
  `pdf_msg_tx: Sender<PdfWorkerMessage>` (worker → main, ad‑hoc).
- **UI thread** never calls PDFium directly; it sends tasks and receives messages.

## 2. Task Serialization

All tasks sent via `pdf_task_tx` are received and handled **serially** in the
worker's `while let Ok(task) = task_rx.recv()` loop. This means:

- Only one operation can be in progress at a time.
- A large export blocks opening another PDF until it completes.
- The worker processes tasks FIFO; `request_repaint()` is sent back to the UI
  after each page or step.

### Task enum (key variants)

| Variant | Purpose |
|---|---|
| `Load { path, password, ctx }` | Load PDF, render pages, populate cache. |
| `Export { format, stem, pages, options, cancel_flag, msg_tx, ctx }` | Export to PNG/JPEG/DOCX/DOC. |
| `SaveSignature { path, password, img_path, scale, ctx }` | Embed signature image into PDF. |
| `SaveRotation { path, rotation, ctx }` | Apply per‑page rotation delta and save. |
| `SaveAnnotations { path, password, annotations, ctx }` | Render annotations onto PDF. |
| `CheckUpdate { is_manual, ctx }` | Query GitHub Releases API. |
| `DownloadUpdate { url, progress_tx, ctx }` | Download installer binary. |
| `AiSummarize { is_chatbot, messages, endpoint_url, model, api_key, ctx }` | POST to LLM API. |

## 3. Short‑Lived Sub‑Threads

Network‑bound work (`CheckUpdate`, `DownloadUpdate`, `AiSummarize`) does **not**
run on the main worker thread. Instead:

- A **sub‑thread** is spawned that performs the HTTP call (via `ureq`).
- Progress or results are sent back via the existing `msg_tx` channel.
- The sub‑thread exits after the request completes; it does not own Pdfium.

This keeps the worker thread responsive for PDF operations while still allowing
update checks and AI calls.

## 4. Panic Handling

| Task | Panic protection |
|---|---|
| `Export` and `SaveAnnotations` | Wrapped in `std::panic::catch_unwind`. A panic becomes a failure message sent via `ExportComplete` / `AnnotationsSaved`. |
| `SaveSignature` and `SaveRotation` | **No** `catch_unwind`. If the worker thread panics, the UI loses the ability to save signatures/rotations until restarted. |
| `CheckUpdate`, `DownloadUpdate`, `AiSummarize` | Sub‑threads; if they panic, the message channel simply gets a broken sender; the UI shows a toast or falls back to last known state. |

### `SaveAnnotations` specifically

- The entire operation is wrapped in `catch_unwind`.
- On panic: the error is converted to `ExportComplete { success: false }`.
- **Save pattern:** opens a fresh PDFium document, writes to a temp file, drops
  the document (releases Windows lock), then retries copying over the original up
  to **10 times** with **300 ms** sleeps between attempts.
- On success: sends `AnnotationsSaved`; UI resets `is_saving_annotations = false`,
  clears `pending_annotations`, shows toast.
- On failure (including after all retries exhausted): sends
  `ExportComplete { success: false }`; UI shows toast with error message.

### `SaveSignature` / `SaveRotation` specifically

- **No retry logic.** Saves directly to the original path via a single
  `pdfium-render` call.
- If the original file is locked (e.g. by another process on Windows), the save
  may silently fail or corrupt the PDF. No retry or fallback is performed.
- **Bug:** On failure, `ExportComplete { success: false }` is sent, but
  `is_saving_signature` or `is_saving_rotation` can remain `true` (the UI does
  not clear them). The user must toggle "Add Signature" or "Rotate PDF" off and
  on to reset the flags.

## 5. Update Checks and Downloads

- **`CheckUpdate`:** sends a GET to GitHub Releases API; parses JSON for
  `"tag_name"` and compares it with the current version using **plain string
  inequality** (not semver). If a newer tag is found, `UpdateCheckResult` is sent
  with `is_available = true`.
- **`DownloadUpdate`:** downloads the `.exe` from a hard‑coded URL
  `https://github.com/borneelphukan/nixobdo-pdf/releases/download/nixobdo-pdfSetup/nixobdo-pdfSetup.exe`
  (Windows‑only) to the user's Downloads folder, showing progress via
  `UpdateDownloadProgress`.
- **`UpdateDownloadComplete`:** if the downloaded path is valid, spawns the
  installer (`std::process::Command::new(&path).spawn()`) and then exits the app
  (`std::process::exit(0)`). On failure, shows toast "Download failed: {err}".

## 6. Cache Cleanup (detached thread)

- Runs at app startup, independent of the worker thread.
- Deletes any subdirectory under
  `dirs::cache_dir()/nixobdo-pdf-cache/<hash>/` whose `accessed.txt` mtime is
  older than **7 days**.
- If total remaining directory size > **400 MB**, deletes oldest‑accessed
  directories first until under the limit.
- `accessed.txt` is touched on a successful cache hit; not updated on every page
  read.