# 15 — Task Recipes (Step‑by‑Step Change Guides)

This document provides concrete, step‑by‑step recipes for common modification
scenarios. Each recipe lists the exact files to touch and the minimal changes
needed.

## 1. Adding a New Annotation Tool

**Goal:** Add a new annotation tool (e.g. "Freehand Draw") to the UI and have it
persist into the PDF.

| Step | File | Change |
|---|---|---|
| 1 | `src/document.rs` | Add a new variant to the `AnnotationTool` enum, e.g.
  `FreehandDraw`. |
| 2 | `src/ui/toolbar.rs` | Add a button in the annotation toolbar (below the existing
  tool buttons) that toggles the new tool on/off, following the pattern of
  `Highlight`/`Underline`/`Strikethrough`. |
| 3 | `src/ui/viewer/central_panel.rs` | Add preview rendering logic for the new tool. The annotation
  system draws rects before the page image; for a freehand tool you would likely
  draw a path or free‑form shape. |
| 4 | `src/worker/mod.rs` | Add a match arm for `PdfWorkerTask::SaveAnnotations` that handles the
  new tool type when rendering `AnnotationAction` objects. For `FreehandDraw`,
  this would mean rendering a free‑form path object via PDFium. |
| 5 | `src/ui/toolbar.rs` (optional) | If the tool needs extra UI (color, size, opacity), add a
  dropdown/grid in the annotation toolbar’s style‑changed section, mirroring the
  Text tool’s color/Size/Bold/Italic/Underline controls. |

**Tip:** The annotation rendering in the worker uses `AnnotationAction { tool,
page_index, rects, position, text, color, scale, bold, italic, underline }`. For a
freehand tool you could store the free‑form points in a custom field or reuse the
`rects` vector as a list of line segments.

---

## 2. Adding a New Export Format (e.g. BMP)

**Goal:** Add a BMP export option alongside PNG/JPEG.

| Step | File | Change |
|---|---|---|
| 1 | `src/worker/export.rs` | Add a new variant to the `ExportFormat` enum, e.g.
  `Bmp`. Then implement the export function body: render each page, write the
  bitmap to a `buf` via `image::ImageFormat::Bmp`, add to the ZIP. |
| 2 | `src/ui/dialogs/export.rs` | Add `Bmp` to the format selector UI and expose any format‑specific
  options (e.g. BMP has no quality setting, unlike JPEG). |
| 3 | `src/app/mod.rs` (or `state.rs`) | If the UI needs to track the selected format across sessions,
  ensure the new variant is serialised in the `ExportFormat` field (it already is
  via `#[derive(Serialize, Deserialize)]`). |
| 4 | Test | Run `cargo build --release` and manually export a PDF to BMP; verify the
  ZIP contains `.bmp` files. |

**Note:** The export worker currently supports PNG, JPEG, and the document formats
(DOCX, DOC). Adding a raster format requires handling the image correctly; BMP is
straightforward because `image` crate supports both reading and writing BMP.

---

## 3. Adding a Menu Item

**Goal:** Add a new menu item (e.g. "Toggle Dark Mode") to the menu bar.

| Step | File | Change |
|---|---|---|
| 1 | `src/ui/menu_bar.rs` | Add a new `ui.menu_button("...", |ui| { ... })` entry in the appropriate
  menu section (File, Edit, View, AI, or Help). Follow the existing patterns:
  • Use `ui.add_enabled_ui(condition, egui::Button::new("Label"))`. |
| 2 | `src/app/mod.rs` (if state needed) | If the item toggles a global state field (e.g. `show_dark_mode`),
  add the field to `NixobdoPdfApp` and initialize it in `Default::default()`. |
| 3 | Inline action | The handler can be inline within the menu button closure; no separate
  module is needed for simple actions. For example, toggling a boolean and calling
  `ui.ctx().request_repaint()`. |
| 4 | Test | Run the app, open the menu, verify the new item appears and fires the
  expected action. |

**Pattern example** (from existing "Rotate PDF" menu):
```rust
ui.menu_button("Rotate PDF", |ui| {
    if ui.button("Rotate Left").clicked() { ... }
    if ui.button("Rotate Right").clicked() { ... }
});
```

---

## 4. Modifying PDF Rendering Configuration

**Goal:** Change the page render target width, clear color, or thumbnail size.

| Step | File | Change |
|---|---|---|
| 1 | `src/document.rs` | Locate `PdfRenderConfig` construction. The target width is set via
  `.set_target_width(2400)` (full pages) or `.set_target_width(2000)` (export).
  The clear color is set via `.set_clear_color(PdfColor::new(255, 255, 255, 255))`. |
| 2 | Thumbnail size | Controlled by the code that generates `thumb_N.png` — 300 px wide is
  hard‑coded in `background_load_with_pdfium` (see `src/document.rs` line ~53). |
| 3 | Rebuild | After changing `document.rs`, run `cargo build --release` to pick up the
  change. The cache will be invalidated automatically because the hash depends on
  the file, but changing the render width does **not** retroactively re‑render
  already‑cached pages — they’ll keep the old width. |
| 4 | Cache note | If you want to force re‑rendering of all pages, bump the app version or
  clear `~/.cache/nixobdo-pdf-cache/`. |

---

## 5. Common UI Pattern: `ui.scope()` Isolation

Many UI sections use `ui.scope(|ui| { ... })` or `ui.scope_builder(...)` to
localize style changes (font sizes, colors, corner radius). This prevents one
section's visual tweaks from leaking into other parts of the UI.

**Examples from the codebase:**

- `src/ui/tabs.rs`: tabs visual style (dark vs light mode) is scoped per‑panel.
- `src/ui/toolbar.rs`: annotation toolbar uses `ui.style_mut()` changes inside the
  annotation `if` block.
- `src/ui/dialogs/export.rs`: header separator and body frame are wrapped in
  `ui.ctx().show_viewport_immediate(...)` with isolated styling.

**Rule of thumb:** Wrap any style‑mutating code (text_styles, visuals, spacing)
in a scope if it only applies to that particular subsection.

---

## 6. Debugging Worker‑Thread Panics

If a background operation seems to silently fail:

1. Check whether the task is `Export`, `SaveAnnotations`, `SaveSignature`, or
   `SaveRotation`.
2. `Export` and `SaveAnnotations` are wrapped in `catch_unwind`; a panic will be
   caught and reported as a failure toast.
3. `SaveSignature` and `SaveRotation` have **no** panic protection; if the worker
   thread panics, the UI loses those capabilities entirely until restart.
4. Look at the toast message: `ExportComplete { success: false }` indicates the
   worker caught an error or panicked (for Export/Annotations). For signature/rotation
   failures, the flag `is_saving_signature`/`is_saving_rotation` may remain stuck.

---

## 7. Verifying Internal Links in Context Docs

After adding or editing a `.md` file in `context/`:

- Open `context/README.md` and verify the "What to read first" table links point
  to the correct file names.
- Run a internal‑link check (e.g. `grep -r "\[.*\](.md)" context/` to ensure no
  orphaned references).
- The markdown files are self‑contained; they do not depend on each other beyond
  the index in `README.md`.