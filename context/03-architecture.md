# 03 — Architecture

## 1. Process and thread model

```text
┌────────────────────────────────────────────────────────────────────┐
│ Native process                                                      │
│                                                                    │
│  ┌──────────────────────────────┐                                  │
│  │ Main / UI thread             │                                  │
│  │ eframe event loop            │                                  │
│  │ NixobdoPdfApp (all UI state) │                                  │
│  │ egui textures                │                                  │
│  └──────────────┬───────────────┘                                  │
│        task_tx  │                       msg_rx                     │
│                 ▼                          ▲                       │
│  ┌─────────────────────────────────────────┴──────────────────┐   │
│  │ Worker thread (one, started in Default)                    │   │
│  │ owns Pdfium; serialises Load/Export/Save*                  │   │
│  │                                                            │   │
│  │  CheckUpdate / DownloadUpdate ──► short-lived sub-thread   │   │
│  │  AiSummarize ──────────────────► short-lived sub-thread    │   │
│  └────────────────────────────────────────────────────────────┘   │
│                                                                    │
│  Short-lived threads: sidebar thumbnail PNG decoding,              │
│  cache-page PNG decoding, update HTTP, LLM HTTP                    │
└────────────────────────────────────────────────────────────────────┘
```

Key properties:

- The `Pdfium` instance is created and used **only on the worker thread**. `pdfium-render`'s bindings are not shared with the UI thread.
- `PdfWorkerTask` is a single-consumer `std::sync::mpsc` queue. Loads, exports, and saves are therefore serialised. A long export blocks subsequent page loads and saves.
- `PdfWorkerMessage` is also an `mpsc` channel. The UI drains it once per frame with `try_recv()` until empty.
- `egui::Context` clones are sent with tasks so background code can call `request_repaint()` when data is ready or progress changes.
- `egui::TextureHandle` creation happens **only on the UI thread**, inside `process_messages`, using `ui.ctx().load_texture(...)`.
- Sidebar and lazy cache-page loading use ad-hoc `std::thread::spawn` with a cloned `Sender<PdfWorkerMessage>`; they do not own PDFium.

## 2. Startup sequence

1. `src/main.rs` loads `assets/logo.png` as the window icon.
2. Builds `eframe::NativeOptions` with an 800×1000 viewport, app id `nixobdo-pdf`, title `Nixobdo PDF`.
3. Sets a global dark `egui::Style` (panel/window fills, 8–12 px corner radii, gray selection).
4. Installs `egui_extras` image loaders (needed for SVG icons).
5. `NixobdoPdfApp::default()`:
   - probes every PDFium path (exe dir, `./lib`, bare name, system) and stores `has_pdfium_bindings`;
   - creates task/message channels;
   - spawns the worker thread;
   - spawns a background `clean_cache()` thread;
   - loads `settings.json` from `dirs::config_dir()/nixobdo-pdf/`;
   - fills all state fields.
6. If a CLI argument exists (`cargo run -- file.pdf`), calls `load_pdf` immediately.
7. eframe runs the app.

> Note: the main thread's PDFium probe is only for the warning badge. The worker performs its own independent bind and can still fail even if the badge says bindings exist (e.g. path visibility differences, architecture mismatch).

## 3. Frame lifecycle

`impl eframe::App for NixobdoPdfApp` is in `src/app/eframe_app.rs` and implements `fn ui(&mut self, ui, frame)` (not the older `update` shape).

Order per frame:

1. **Splash**: `ui_splash` returns `true` for the first 2.5 seconds and short-circuits the rest of the frame.
2. **Auto update check**: on the first non-splash frame, sends `CheckUpdate { is_manual: false }` once (`has_checked_for_updates`).
3. **Drain worker messages**: `process_messages(ui)` applies document info, page data, save/export results, AI results, and update state. Creating textures happens here.
4. **Toast**: draws the bottom-right notification and clears it after its timer.
5. **Window title**: active tab file name + `- nixobdo-pdf`, or just `nixobdo-pdf`.
6. **Global shortcuts**:
   - `Ctrl/Cmd+F` requests focus on the search text field (`Id::unique("search_bar")`).
   - `Escape` leaves fullscreen.
   - Fullscreen shows a 3-second “click Esc” hint.
7. **Fullscreen gate**: if not fullscreen, render `ui_menu_bar` → `ui_tabs` → `ui_toolbar`.
8. **Dialogs**: `ui_dialogs`, which dispatches rename, export, about, update, export progress, custom colour, LLM settings, AI summary, password.
9. **AI panel**: `ui_chatbot` (right `Panel`) when not fullscreen.
10. **Viewer**: `ui_viewer` → `manage_textures`, sidebar, separator, central panel, then zoom/copy shortcuts.

Because messages are drained before drawing, a worker result sent during frame *N* is normally visible in frame *N+1*.

## 4. State ownership

`NixobdoPdfApp` (`src/app/mod.rs`) is the single source of UI truth. State groups:

| Group | Examples |
|---|---|
| Tabs/document | `tabs: Vec<PdfDocumentState>`, `active_tab_index` |
| Navigation | `sidebar_open`, `pointer_mode`, `pending_scroll_delta` |
| Text interaction | `selection_start`, `selection_end`, `is_selecting`, `search_query`, `search_active_match` |
| Worker plumbing | `pdf_task_tx`, `pdf_msg_tx`, `pdf_receiver` |
| Files/UI windows | `recent_files`, `rename_*`, `export_*` |
| Feedback | `toast_*`, `export_progress`, `update_state` |
| Signature | `signature_*`, `is_placing_signature`, `is_saving_signature` |
| Rotation | `is_rotating_document`, `is_saving_rotation`, `pending_rotation` |
| Annotation | `is_annotation_mode`, `active_annotation_tool`, `pending_annotations`, `redo_annotations`, text style state |
| AI | `llm_*`, `ai_summary_*`, `ai_chat_*`, `ai_chat_sessions` |
| Fullscreen | `was_fullscreen`, `fullscreen_toast_timer` |

Per-tab state (`PdfDocumentState` in `src/document.rs`) holds textures, text, character boxes, links, page sizes, rotations, zoom, selected page, layout, password, cache mode, and loading sets. It has **no** access to `Pdfium`; rendering is done by the worker.

## 5. Channel contracts

```text
UI ── PdfWorkerTask ──► worker
UI ◄─ PdfWorkerMessage ─ worker / network sub-threads / thumbnail threads
```

`PdfWorkerTask` variants:

- `Load { path, password, ctx }`
- `Export { path, password, out_path, format, retain_layout, include_images, ctx, cancel_flag }`
- `CheckUpdate { is_manual, ctx }`
- `DownloadUpdate { version, ctx }`
- `SaveSignature { path, password, page_index, image_path, position, scale, ctx }`
- `SaveRotation { path, password, rotation, ctx }`
- `SaveAnnotations { path, password, annotations, ctx }`
- `AiSummarize { is_chatbot, messages, endpoint_url, model, api_key, ctx }`

`PdfWorkerMessage` variants:

- `DocumentInfo`, `PageData`, `PageDataLoaded`, `ThumbnailDataLoaded`, `Finished`
- `ExportProgress`, `ExportComplete`
- `SignatureSaved`, `RotationSaved`, `AnnotationsSaved`
- `UpdateCheckResult`, `UpdateDownloadProgress`, `UpdateDownloadComplete`
- `AiSummaryResult`

See [04-data-model.md](04-data-model.md) for field-level contracts and [09-worker-and-concurrency.md](09-worker-and-concurrency.md) for execution details.

## 6. Tab lifecycle

### Open

`load_pdf(ctx, path)`:

1. move path to front of `recent_files`, truncate to 5, `save_settings()`;
2. if a tab with the same `PathBuf` exists, activate it and stop;
3. create `PdfDocumentState::empty(path)` (`is_loading = true`) and make it active;
4. send `Load { password: None }`.

Duplicate detection is by exact `PathBuf`, not canonical path. Opening the same file via different relative/symlink paths creates two tabs.

### Load success

`DocumentInfo` without error initialises vectors sized to `page_count`, sets `is_loading = false`, stores the password and `use_cache`, clears loading sets. Page messages then fill textures/metadata as they arrive.

### Load failure

- `PasswordRequired` / `IncorrectPassword` → open `password_prompt`; the tab stays loading.
- Missing path errors → native warning and remove the tab.
- Other PDFium errors → native “could not open/damaged” dialog and remove the tab.

The `PdfDocumentState.error` field exists and `central_panel` can render it, but `messages.rs` does **not** populate it in the current code; load failures generally close the tab instead.

### Close

`close_tab(index)` removes the tab and repairs `active_tab_index`; it does not cancel in-flight worker work. A later message for the removed path is ignored because handlers look up by path.

### Reload

`reload_pdf(ctx, path)` sets `is_loading = true` and sends a fresh `Load`. It is used after successful signature/rotation/annotation saves. Cache invalidation relies on the file size/mtime changing and producing a new cache key.

## 7. Persistence

### Settings

Path: `dirs::config_dir()/nixobdo-pdf/settings.json`

Schema (written with `serde_json::to_string_pretty`):

```json
{
  "recent_files": ["/path/to/a.pdf"],
  "llm_api_key": "sk-...",
  "llm_model": "llama-3.2-3b",
  "llm_endpoint_url": "http://localhost:1234/v1",
  "llm_configured": true,
  "ai_chat_sessions": [
    {
      "id": "1712345678901",
      "name": "Summary - 14:30",
      "messages": [
        { "role": "system", "content": "..." },
        { "role": "user", "content": "..." },
        { "role": "assistant", "content": "..." }
      ]
    }
  ],
  "ai_active_session_id": "1712345678901"
}
```

Notes:

- Missing keys are `#[serde(default)]`; malformed files fall back to defaults silently.
- Settings are written on PDF open, rename (recent list), AI configuration, chat mutation, and some dialog closes.
- There is no atomic-write/temp-file strategy for settings; a crash mid-write can truncate it.
- The API key is plaintext. See [14-conventions-and-gotchas.md](14-conventions-and-gotchas.md).

### Page cache

Path: `dirs::cache_dir()/nixobdo-pdf-cache/<hash>/`

```text
<hash>/
├── accessed.txt        # touched on cache use; drives TTL cleanup
├── metadata.bin        # bincode (page_count, texts, chars, links, sizes)
├── page_0.png ...      # transparent-background full page renders
└── thumb_0.png ...     # 300 px wide thumbnails
```

The hash is `DefaultHasher` over the path, file length, and modification time in **seconds**. See [05-pdf-pipeline.md](05-pdf-pipeline.md) for the exact cache protocol.

## 8. Why the architecture looks like this

| Decision | Consequence |
|---|---|
| Immediate-mode egui | UI code is plain functions over `&mut self`; no observer/event bus; local bools defer mutations. |
| Single worker with one `Pdfium` | Avoids FFI thread-safety concerns and repeated binding; costs serialised loads/exports. |
| Messages instead of shared state | Keeps FFI/bitmap work off the UI thread; requires per-frame draining and explicit repaint requests. |
| Normalised page coordinates | Resolution-independent selection/link/annotation overlays; conversion happens at draw/save boundaries. |
| Transparent page renders | Highlights can be painted beneath text while text remains crisp; requires alpha-aware assets/cache. |
| Cache first, network-free | Fast reopening and offline use; cache staleness is mitigated by file size/mtime keys. |
| Separate native viewports for some dialogs | Modality-ish UX and independent sizing; state is still driven from the main app struct. |

## 9. Error handling style

There is no global error type or logging framework. The code uses:

- `Option` + `if let` for recoverable UI/worker operations;
- `Result<_, String>` inside export/save helpers, converted to `ExportComplete { success, message }`;
- `std::panic::catch_unwind` around export and annotation save so a panic does not kill the worker;
- `rfd::MessageDialog` for user-facing load failures and confirmations;
- toasts for success/failure of asynchronous operations;
- `eprintln!` in a couple of synchronous UI paths (e.g. rename failure);
- silent failures for many filesystem/cache writes (`let _ = ...`).

When adding features, prefer propagating a `Result` back through a worker message and showing a toast/dialog; do not panic the worker.
