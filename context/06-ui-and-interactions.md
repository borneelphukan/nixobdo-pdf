# 06 — UI and Interactions

This document describes the immediate-mode egui UI, keyboard/mouse interactions,
and per-feature UI flow in `nixobdo-pdf`.

## 1. UI Architecture

- **Immediate mode.** Every frame the UI is rebuilt by calling `impl NixobdoPdfApp`
  methods. There is no retained widget tree; local `bool` flags (e.g.
  `zoom_in`, `perform_export`, `send_msg`) defer mutations until after a UI
  closure.
- **`eframe::Area`** is used for floating UI: toasts (bottom-right), update
  banner, AI settings viewport, custom color picker.
- **`ui.scope()`** isolates style changes for tabs, annotation toolbar, and
  color grids.
- **Keyboard focus** is managed via `ui.input(|i| ...)` and
  `response.request_focus()` / `response.lost_focus()`.

## 2. Toolbar

The top toolbar (defined in `src/ui/toolbar.rs`) always visible and contains:

| Section | Controls |
|---|---|
| **Zoom** | Drag value (10%–1000%), `🔍-` out, `🔍+` in, `Reset`. Can also be adjusted with `Ctrl/Cmd` + mouse wheel, trackpad pinch, arrow keys (1-step in TwoPage mode, 2-step), utility bar ±10. |
| **Navigation** | `⬆` / `⬇` (page up/down, moves `selected_page` by step; 1 in Single/Continuous, 2 in TwoPage). |
| **Rotate** | `⟲` / `⟳` buttons, or `Edit → Rotate PDF` submenu (left/right 90°). Only changes preview `page_rotations`; persistence requires saving via the menu. |
| **Fullscreen** | Toggles viewport fullscreen; shows “To exit Full screen, click Esc” toast when entering. |
| **Search (Ctrl+F)** | Text field with hint “Search PDF… (Ctrl+F)”. Lowers case and scans each page's `page_chars` for contiguous character matches. Results recorded as page indices. |
| **Search navigation** | Enter (when focus is lost and re‑pressed) cycles `search_active_match` through `match_pages` and scrolls to that page. Pressing `Ctrl+F` again refocuses the field. |
| **AI Panel toggle** | Opens/closes the right‑side chatbot panel. |
| **Sidebar toggle** | Shows/hides the thumbnail sidebar. |
| **Annotation toolbar** | Appears when `is_annotation_mode` is true. Tools: Text, Highlight, Underline, Strikethrough, Redo, Undo, Color palette, Size dropdown, Bold/Italic/Underline toggles, Save/Cancel. |
| **Undo/Redo** | Button or `Cmd+Z` / `Cmd+Y`. Only operates on `pending_annotations` stack; uses `modifiers.command` (not `modifiers.ctrl`). |

### Annotation toolbar detail

- **Tool selection buttons** set `active_annotation_tool` enum. Clicking the
  active tool again deselects it.
- **Text tool** additionally shows a dropdown for size (px), toggle buttons for
  Bold/Italic/Underline, and a color grid with predefined colors + Transparent +
  Custom color… picker.
- **Undo** pops `pending_annotations` onto `redo_annotations`; **Redo** pops
  back. Both use `ui.ctx().input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z))` /
  `Key::Y`.
- **Save** is enabled only when `pending_annotations` is non‑empty, a tab is
  active, and `!self.is_saving_annotations`. Prompts with a yes/no dialog;
  on Yes sends `SaveAnnotations` task and sets `is_saving_annotations = true`.

### Signature placement

- **Add Signature** (Edit menu) opens a file picker for PNG/JPG. The image is
  loaded, a texture created, and the user is put in “placing” mode:
  `is_placing_signature = true`. The signature can be dragged and resized with a
  corner handle.
- **Save** is triggered by `Ctrl+S` (code checks `modifiers.ctrl` only, not
  `modifiers.command`). On save the worker embeds the image as a PDF image
  object and sends `SignatureSaved`. The UI then resets `is_placing_signature =
  false`, `is_saving_signature = false`.

## 3. Menu Bar

- **File → Open**, **Open Recent**, **Export…**, **Rename**, **Close PDF**,
  **Close Window**.
- **Edit → Add Signature**, **Annotation** (toggle on/off), **Rotate PDF** (left/
  right 90° submenu), **Delete** (not implemented — see gotchas).
- **View → Show Sidebar**, **Show Utility Bar**, **Show AI Panel**, **Fullscreen**,
  **Scroll Mode / Single Page / Two Page** (layout toggles).
- **AI → Settings…**, **Summarize with AI** (sends `AiSummarize` task).
- **Help → Check for Updates**, **About**.

## 4. Tabs

- Tab bar at the top (defined in `src/ui/tabs.rs`). Visual style differs between
  dark and light mode; active tab has a custom bg_fill; corner radius 4px.
- Middle‑click or `Close PDF` removes the tab; recent files max 5.
- `active_tab_index` tracks the current tab; `None` when no PDF is open.

## 5. Shortcut Inconsistencies (gotchas)

| Action | Documented | Actual code behavior |
|---|---|---|
| Save annotations | `Ctrl+S` | Code only checks `modifiers.ctrl`, not `modifiers.command` (macOS). |
| Undo annotation | `Cmd+Z` | Code uses `modifiers.command` exclusively. |
| Redo annotation | `Cmd+Y` | Code uses `modifiers.command` exclusively. |
| Search Enter | Direct | Must lose focus then press Enter (`response.lost_focus()` + `key_pressed(Enter)`). |
| Copy (`Ctrl+C` / `Cmd+C`) | Both | Code checks `modifiers.command || modifiers.ctrl` in `ui/viewer/mod.rs`. |