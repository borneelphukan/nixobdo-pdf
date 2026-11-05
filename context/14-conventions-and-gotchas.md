# 14 — Conventions, Gotchas, and Platform Traps

This document collects quirks, known bugs, and platform‑specific caveats that
future AI agents (and humans) should be aware of when working with this codebase.

## 1. Shortcut Inconsistencies

| Action | Documented | Actual code behavior |
|---|---|---|
| **Save annotations** | `Ctrl+S` | Code in `src/ui/toolbar.rs` checks `modifiers.ctrl` only; `modifiers.command`
  (macOS) is **not** recognized. Users on macOS cannot save annotations with
  `Cmd+S`. |
| **Undo annotation** | `Cmd+Z` | Code uses `ui.ctx().input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z))`. |
| **Redo annotation** | `Cmd+Y` | Code uses `ui.ctx().input(|i| i.modifiers.command && i.key_pressed(egui::Key::Y))`. |
| **Copy selection** | `Ctrl+C` / `Cmd+C` | Code in `src/ui/viewer/mod.rs` checks `modifiers.command || modifiers.ctrl` — both work. |
| **Search Enter** | Press Enter | Must lose focus first (`response.lost_focus()`), then press Enter.
  Direct `Key::Enter` press does not navigate. |

## 2. Rotation Persistence Gap

- Toolbar `⟲`/`⟳` buttons and `Edit → Rotate PDF` submenu only change the **preview**
  `page_rotations` array in the tab state. The PDF bytes are **not** modified until
  the user explicitly chooses to save via the menu.
- Persistence is gated by the `Edit → Rotate PDF` setting
  `is_rotating_document` / `pending_rotation`. If the user rotates preview but
  never saves, the rotations reset on next PDF open.
- The save operation (`SaveRotation` worker task) adds the delta to every page's
  current rotation and normalises to 0/90/180/270, then writes directly to the
  original path.

## 3. Save Flag Leaks on Failure

| Failed task | UI state after failure |
|---|---|
| `SaveSignature` | `is_saving_signature` remains `true`; `ExportComplete { success: false }` is sent but the UI does not clear it. User must toggle “Add Signature” off and on to reset. |
| `SaveRotation` | `is_saving_rotation` remains `true`; same leak as above. |
| `SaveAnnotations` | `is_saving_annotations` is **always** cleared (success or failure). UI correctly resets `is_annotation_mode = false` and clears pending/redo stacks. |

## 4. Update Check String Matching

- The update check parses the GitHub Releases JSON and searches for `"tag_name"`.
- Comparison is a **plain string inequality** against the current version; it does
  **not** use semver parsing. This means `v0.1.10` would be considered “older”
  than `v0.1.9` if compared lexicographically, which happens to work for the
  current zero‑padded version scheme but is technically incorrect for general
  semver.

## 5. Download URL Hard‑coding

- The Windows update downloader hard‑codes the URL
  `https://github.com/borneelphukan/nixobdo-pdf/releases/download/nixobdo-pdfSetup/nixobdo-pdfSetup.exe`.
- This URL is embedded in the download logic and does not dynamically construct
  from the tag name. If the release asset name changes, the downloader will fail.

## 6. AI Endpoint Rewrites

- If the user provides a **non‑empty API key** and an endpoint of
  `localhost`, `127.0.0.1`, or **empty**, the effective endpoint is rewritten to
  `https://api.openai.com/v1`.
- If the endpoint is anything else (e.g. a custom Ollama server), the URL is used
  as‑provided, **without** the Bearer token rewrite. The API key is still sent as
  `Authorization: Bearer <key>`.

## 7. PDFium Binding Fallback Chain

The binding lookup order (hard‑coded, multiple locations):

```
<exe_dir>/libpdfium.dylib
<exe_dir>/pdfium.dll
<exe_dir>/libpdfium.so
./lib/libpdfium.dylib
libpdfium.dylib
./lib/pdfium.dll
pdfium.dll
./lib/libpdfium.so
libpdfium.so
Pdfium::bind_to_system_library()
```

If none bind successfully, `has_pdfium_bindings = false` and the app renders a
red warning label in the toolbar. No PDF operations work without PDFium.

## 8. Cache Key Staleness

- Cache key = `DefaultHasher` over `PathBuf`, `metadata.len()`, and
  `metadata.modified()` in **seconds** since UNIX epoch.
- A save that changes the file size or mtime (seconds) produces a **new** key.
- **Fast same‑size saves** within the same second can reuse a stale cache entry,
  because the mtime hasn't changed. Old cache directories are not automatically
  deleted on save; `clean_cache()` runs at startup and removes stale entries.

## 9. Metadata Version Skew

- `metadata.bin` is a bincode‑serialized tuple with **no schema version field**.
- If the types stored in metadata change (e.g. new fields in `PdfCharInfo`),
  old cache directories may become incompatible. There is no automatic migration;
  the cache key would change (because the hash depends on the serialized bytes),
  but old directories remain until `clean_cache()` removes them.

## 10. No Rust Unit/Integration Tests

- `cargo test --locked --all-targets` passes with **zero tests**.
- CI gate `cargo test` is **blocking** but always passes; no test files exist in
  the project.
- If adding tests, remember there is no test infrastructure currently.

## 11. Selection Model Edge Cases

- `Ctrl+A` selects from `(0, 0)` to the last char of the last page; gated on
  `!egui_wants_keyboard_input()` so it does not hijack text fields.
- Selection rectangles **merge adjacent boxes on the same line** for a continuous
  highlight. Non‑adjacent boxes are drawn separately.
- `page_chars` retains whitespace characters (used for copy), but they are
  ignored when painting selection/annotation rectangles.

## 12. Search-Only Text Layer

- Search scans `page_chars` for contiguous lowercase character matches.
- **No regex, no diacritic folding, no whole‑word option.**
- Search **only works** where a text layer exists (i.e. PDFs with selectable
  text). Scanned/image‑only PDFs will find zero matches.

## 13. Annotation Tool Rendering Order

Inside a page the drawing order is:

1. White page rectangle
2. **Highlight** annotations (semi‑transparent, drawn **before** page image;
   transparent background lets them show through)
3. Blue text‑selection rects
4. Yellow search rects
5. The PDF page mesh (transparent background lets 2‑4 show through)
6. Signature overlay
7. Link hit areas (invisible)
8. **Underline / Strikethrough / Redact / Text** annotations (on top)
9. Loading text if no texture

This order ensures highlights appear underneath text and the page, while
underlining/strikethrough/redact/text appear on top of everything.

## 14. `SaveAnnotations` Retry vs `SaveSignature`/`SaveRotation` Single‑Attempt

- `SaveAnnotations` opens a temp file, drops the document (releases Windows
  lock), then retries copying the original file up to **10 times** with **300 ms**
  sleeps. This is the only save path with a retry loop.
- `SaveSignature` and `SaveRotation` write directly to the original path in a
  single attempt. If the file is locked (common on Windows when the app has the
  file open), the save may silently fail or corrupt the PDF. No retry or
  fallback is performed.

## 15. `metadata.bin` and `accessed.txt` Nuances

- `metadata.bin` is written **after all pages are loaded** during a cached load.
- `accessed.txt` is touched on a **successful cache hit** (i.e. when the cache
  is used without a full re‑render). It is **not** updated on every page read.
- `accessed.txt` mtime is what `clean_cache()` checks against the 7‑day TTL.
- If `accessed.txt` exists but is stale (old mtime), the directory will be
  deleted during cleanup even if the PNG files inside are still valid.
