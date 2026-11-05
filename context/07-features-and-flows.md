# 07 — Features and End‑to‑End Flows

This document surveys every user‑facing feature from opening a PDF to exporting
it, including annotation, signature, AI, and export workflows.

## 1. Multi‑Tab PDF Viewing

- **Open PDF:** `File → Open` or `Ctrl+O` (not coded explicitly; handled by
  `rfd::FileDialog`). `src/app/state.rs::load_pdf()` creates a new
  `PdfDocumentState` tab, adds it to `tabs[]`, sends a `Load` task to the
  worker, and sets `active_tab_index`.
- **Reuse tab:** If the same path is opened again, the existing tab is reused
  (position check in `load_pdf`).
- **Close tab:** `Edit → Close PDF` or middle‑click on tab tab removes it;
  `state.rs::close_tab()` updates `active_tab_index` accordingly.
- **Recent files:** max 5, stored in `settings.json`; managed by
  `state.rs::save_settings()`.
- **Layout modes:** `ContinuousScroll`, `SinglePage`, `TwoPage` (configured per
  tab via `layout_mode`; navigation steps by 2 in TwoPage mode). Toggled from
  `View` submenu or toolbar page up/down.

## 2. Zoom and Page Fitting

- **Toolbar drag** sets `tab.zoom` (percentage offset, 0–1000%). Displayed zoom
  is `100 + tab.zoom`.
- **`Ctrl/Cmd` + mouse wheel** adjusts `tab.zoom` by ±0.1 (in `src/ui/viewer/mod.rs`);
  `central_panel.rs` also applies ±10 with a 50 ms throttle — both can observe
  the same scroll.
- **Trackpad pinch:** `(1 + zoom) * gesture - 1`, floored at 0.
- **Fit options:** toolbar buttons + utility bar ±10; right‑click menu ±10.
- **Zoom geometry:** `fit_page_width = min(available_height / aspect, available_width / chunks)`;
  `final_width = max(100, fit_page_width + (tab.zoom / 50.0) * (fit_width_width - fit_page_width))`.

## 3. Text Selection and Copy

- **Drag selection** starts when user primary‑drags over characters. `find_closest_char()`
  maps screen point → nearest char index. During drag, `selection_end` updates
  for whatever page the pointer is over (multi‑page selections work).
- **On release:** `is_selecting` becomes false. If `is_annotation_mode` is true,
  the selection is converted into annotation rects (highlight/underline/etc.)
  based on the active tool.
- **`Ctrl/Cmd+A`** selects from `(0,0)` to the last char of the last page; gated
  on `!egui_wants_keyboard_input()`.
- **Copy** (`Ctrl+C` / `Cmd+C`): `copy_selection()` walks `page_chars` ranges and
  inserts `\n` between pages; calls `ctx.copy_text()`.
- **Selection rectangles** merge adjacent boxes on the same line for a continuous
  highlight.

## 4. Search (Ctrl+F)

- Toolbar text field lowercases the query and each page's `page_chars`. Scans for
  contiguous character matches (no regex, no diacritic folding, no whole‑word
  option).
- Results recorded as **page indices**, one per occurrence (same page can appear
  multiple times for multiple matches).
- **Navigation:** Enter (after focus lost + re‑press) cycles `search_active_match`
  through `match_pages` and scrolls to that page. The central panel independently
  re‑scans each page and paints yellow overlays on every match.
- **No regex, diacritic folding, or per‑match highlight index.** Search only works
  where a text layer exists (i.e. not scanned/image‑only PDFs).

## 5. Page Rotation

- **Preview rotation** (`page_rotations[i]` in degrees) is changed by toolbar
  `⟲`/`⟳` buttons or `Edit → Rotate PDF` submenu. Only affects the egui mesh UV
  transforms; the PDF bytes are *not* modified until saved.
- **Persistent save:** `Edit → Rotate PDF` opens a submenu; the setting
  `is_rotating_document` / `pending_rotation` gates whether the rotation is
  saved. When the user chooses to save, `worker/mod.rs::SaveRotation` task adds
  the delta to every page's rotation (normalised to 0/90/180/270) and writes
  directly to the original path.
- After a successful save the UI calls `reload_pdf()`, which generally hashes to
  a new cache directory.

## 6. Annotations (Highlight, Underline, Strikethrough, Redact, Text)

All annotation tools are available when the user toggles **Edit → Annotation** (or
presses the annotation button in the toolbar). The system tracks:

- `is_annotation_mode: bool`
- `active_annotation_tool: Option<AnnotationTool>` (Highlight \| Underline
  \| Strikethrough \| Redact \| Text)
- `pending_annotations: Vec<AnnotationAction>` (undo/redo stack)
- `redo_annotations: Vec<AnnotationAction>`
- `active_text_annotation_index: Option<usize>` (for in‑place editing)
- `annotation_color: egui::Color32` (default Yellow)
- `text_annotation_size: f32`, `text_annotation_bold`, `text_annotation_italic`,
  `text_annotation_underline`
- `is_custom_text_color_open`, `custom_text_color_temp`

### How annotation creation works

1. User drag‑selects text → `find_closest_char()` maps start, and during drag
   `selection_end` tracks the trailing edge.
2. On release, if `is_annotation_mode` is active, the selection rects are
   converted into `AnnotationAction` entries based on the active tool.
3. **Highlight:** filled path with `Multiply` blend, semi‑transparent fill (alpha ~100).
   Drawn **before** the page image so it shows through transparent areas.
4. **Underline:** 2 pt line at `y = (1 - max.y) * h` (PDF bottom‑left origin).
5. **Strikethrough:** 2 pt line at vertical centre of the char box.
6. **Redact:** opaque black filled path (covers content).
7. **Text:** Helvetica text object, font size = `scale`, colour, positioned so
   `position` is the visual centre. Edited inline via a text buffer.

### Undo / Redo

- **Undo** pops the last `AnnotationAction` from `pending_annotations` and pushes
  it onto `redo_annotations`. Uses `modifiers.command` + `Key::Z`.
- **Redo** pops from `redo_annotations` back onto `pending_annotations`. Uses
  `modifiers.command` + `Key::Y`.
- Button `Undo`/`Redo` in the annotation toolbar also works.

### Saving annotations

- When the user clicks **Save** in the annotation toolbar, a confirmation dialog
  appears (“Annotation in a document is permanent. Are you sure you want to
  annotate the document?”).
- On Yes, `is_saving_annotations = true` and `SaveAnnotations` task is sent to
  the worker.
- **Worker:** iterates actions, fetches the page, converts normalised rects to PDF
  points with `(1 - y)` inversion. Highlight → filled path, alpha 100 fill,
  Multiply blend. Underline/Strikethrough → 2 pt line. Redact → opaque black
  filled path. Text → Helvetica text object.
- **Save pattern:** opens a fresh PDFium document, writes to a temp file, drops
  the document (releases Windows lock), then retries copying over the original up
  to 10 times with 300 ms sleeps.
- On success: sends `AnnotationsSaved`; UI resets `is_annotation_mode = false`,
  clears pending/redo annotations, shows toast “Annotations saved successfully”.
- On failure: sends `ExportComplete { success: false }`; UI shows toast with
  error message. Note: `is_saving_annotations` is cleared but
  `is_saving_signature`/`is_saving_rotation` can remain stuck if their saves
  fail (see gotchas).

## 7. Digital Signatures

- **Add Signature** (`Edit → Add Signature`): file picker for PNG/JPG. Image
  loaded, texture created, user enters “placing” mode. Signature can be dragged
  and resized by corner handle.
- **Save:** triggered by `Ctrl+S` (code checks `modifiers.ctrl` only, not
  `modifiers.command`). The worker embeds the image as a `PdfPageImageObject`
  at the normalised position, using the computed aspect‑ratio dimensions
  (`target_width = 150 * scale` PDF points).
- **Save path:** writes directly to the original PDF path (no temp file retry).
- On success: sends `SignatureSaved`; UI shows toast, resets
  `is_placing_signature = false`, `is_saving_signature = false`, reloads PDF.
- **Failure mode:** if the save fails, `ExportComplete { success: false }` is sent
  but `is_saving_signature` can remain `true` (the UI does not automatically
  clear it). The user must manually toggle “Add Signature” off and on to reset.

## 8. Export

Three export formats, dispatched from the toolbar’s **Export…** dialog:

| Format | Implementation |
|---|---|
| **PNG / JPEG** | Renders each page to a bitmap (2000 px target width), writes to
  `buf` via `image::ImageFormat::Png`/`Jpeg`, adds to a `zip`. ZIP finished at
  end. Progress sent as `ExportProgress` messages. |
| **DOCX** | Text extraction with optional layout preservation and image extraction.
  Uses `docx-rs` crate. |
| **DOC (RTF)** | Text + optional images as RTF. Uses custom RTF generation. |

- Export dialog (`src/ui/dialogs/export.rs`) lets the user set a file name,
  choose format, and optionally “Include images”.
- Cancellation: `export_cancel_flag: Arc<AtomicBool>` checked each page in the
  worker; cancels with error “Export cancelled by user.”
- Progress: `export_progress: Option<f32>` (0.0–1.0) displayed in a modal dialog.

## 9. AI Summaries and Chat

### Configuration

- **LLM endpoint normalization:** If the API key is non‑empty **and** the endpoint
  is `localhost`, `127.0.0.1`, or empty, the effective endpoint is
  `https://api.openai.com/v1`. Otherwise the endpoint is used as‑provided.
- **Presets:** `llama-3.2-3b`, `llama-3.1-8b`, `mistral-7b`, `gemma-2-9b`,
  `phi-3-mini`, `qwen-2.5-7b`, `qwen-2.5-14b`, `deepseek-v2`, plus **Custom**.
- **Settings:** `src/ui/dialogs/llm_settings.rs` provides a dialog with preset
  buttons and a custom‑model text field.

### Request format (POST)

- `POST /chat/completions` (endpoint normalized per above rules).
- Body: `temperature: 0.3`, `max_tokens: 1500`, `stream: false`,
  `Authorization: Bearer <key>`.
- Model field comes from the preset or the custom model string.

### AI Summary (non‑chatbot)

- Triggered from `AI → Summarize with AI` menu. Gathers all page text via
  `tab.page_texts.join("\n")`.
- Sends `AiSummarize` task with the full text, endpoint, model, and API key.
- On success: stores `ai_summary_full_text`, clears `ai_summary_text`, shows
  toast.
- On failure: stores `ai_summary_error`.

### AI Chatbot

- Opens via **AI → Summarize with AI** (creates a session) or via the AI panel
  toggle.
- Session model: `AiChatSession { id, name, messages: Vec<ChatMessage> }`.
- User types in a send field; `send_msg` is set when the user presses Enter.
- Messages stored per‑session; on successful reply the assistant message is
  pushed and `save_settings()` is called.
- If LLM is not configured, shows “No LLM Service Available” with “Open Settings”
  button.
- If no document open, shows “No Document Opened”.