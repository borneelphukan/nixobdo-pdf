# 05 — PDF Pipeline

Everything PDF-specific is split between `src/document.rs` (types, cache, rendering, extraction) and `src/worker/mod.rs` (task execution, saving). This document follows the data from “user opens a file” to “pixels are on screen”.

## 1. PDFium binding

The worker binds PDFium once at thread startup. The same fallback chain exists in `NixobdoPdfApp::default` for the toolbar warning badge:

```text
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

The first successful binding wins. The worker stores `Option<Pdfium>`. If binding fails:

- `Load` sends a `DocumentInfo` with error `"PDFium not initialized. Please ensure libpdfium is present."` and `page_count = 0`;
- `CheckUpdate`, `DownloadUpdate`, and `AiSummarize` still run;
- other tasks are silently ignored (`_ => {}`).

## 2. Load algorithm

`PdfDocumentState::background_load_with_pdfium(path, password, pdfium, tx, ctx)`:

1. Derive `file_name`.
2. Probe password protection by attempting `load_pdf_from_file(path, None)`:
   - `PdfiumInternalError::PasswordError` → `use_cache = false`;
   - anything else → `use_cache = true`.
3. Compute the cache key: `DefaultHasher` over `path`, file length, and mtime **seconds**.
4. If `use_cache` and `<cache>/metadata.bin` exists and deserialises:
   - send `DocumentInfo` with cached page count and `use_cache = true`;
   - for each page, decode `thumb_<i>.png` and send `PageData { image: None, thumbnail_image: Some(...), text/chars/links/page_size from metadata }`;
   - if every thumbnail decodes, touch `accessed.txt`, send `Finished`, and return;
   - if any thumbnail fails, continue to a full re-render (partial messages already sent are harmless; the full load re-sends them).
5. Full load with PDFium:
   - `load_pdf_from_file(path, password)`; on `PasswordError` map to `PasswordRequired` or `IncorrectPassword`;
   - send `DocumentInfo` immediately with page count, error `None`, password, `use_cache`;
   - create `PdfRenderConfig` with **target width 2400 px** and white clear colour;
   - create the cache directory and `accessed.txt` if caching;
   - for each page:
     - extract full text (`page.text().all()`);
     - read page width/height in PDF points;
     - extract every char with `loose_bounds()` and normalise;
     - extract links and targets (URL or page index);
     - render to an image;
     - downscale a **300 px wide** thumbnail with Lanczos3 *before* removing white;
     - remove the white background (see below);
     - save `page_<i>.png` and `thumb_<i>.png` if caching;
     - send `PageData` with full image, thumbnail, text, chars, links, and page size; request repaint.
6. After all pages, serialize metadata to `metadata.bin` if caching; send `Finished`.

### White-background removal

The renderer clears to white. For each pixel:

```text
alpha = max(255 - r, 255 - g, 255 - b)
if alpha < 255:
    pixel.a = alpha
    if alpha > 0: unpremultiply rgb by alpha
    else: rgb = 0
```

This makes the page background transparent so highlights drawn *before* the page texture are visible through the page, while text remains opaque and crisp. It also means cached page PNGs are RGBA with transparency; the UI paints a white rectangle behind each page.

### Failure mode

If `render_with_config` fails, the code inserts `egui::ColorImage::example()` (egui's test-pattern image) as both image and thumbnail. It does not report an error to the UI.

## 3. Cache protocol

### Location and key

```text
dirs::cache_dir()/nixobdo-pdf-cache/<hash>/
```

`<hash>` = `DefaultHasher` of:

- the `PathBuf`,
- `metadata.len()`,
- `metadata.modified()` converted to seconds since UNIX epoch.

A save that changes the file size or mtime (seconds) produces a new key. A save within the same second and with the exact same size can reuse a stale cache. Old cache directories are not deleted on save; `clean_cache()` removes them later.

### Contents

| File | Purpose |
|---|---|
| `metadata.bin` | bincode tuple of page count, texts, chars, links, sizes |
| `page_N.png` | transparent full-page render, 2400 px target width |
| `thumb_N.png` | 300 px wide thumbnail, white background retained |
| `accessed.txt` | marker touched on cache hit; used by cleanup |

### Cleanup

`clean_cache()` runs in a detached thread at startup:

- deletes any subdirectory whose `accessed.txt` (or directory) mtime is older than **7 days**;
- sums the remaining directories and, if total size > **400 MB**, deletes oldest-accessed directories first until under the limit.

`accessed.txt` is created on a full cached load and touched on a successful cache hit. It is not updated on every page read.

### Cache bypass

Password-protected PDFs set `use_cache = false`; no cache files are read or written. All pages are delivered with full images, and `manage_textures` returns early (so pages stay resident).

## 4. From `ColorImage` to texture

`process_messages` handles `PageData` by path lookup (not index into `tabs`):

- full page texture created only if:
  - `use_cache == false`, or
  - the page is within `selected_page ± 2`;
- thumbnail texture created only if:
  - `use_cache == false`, or
  - within `selected_page ± 15`;
- text/chars/links/size are always stored.

`PageDataLoaded` and `ThumbnailDataLoaded` are sent by ad-hoc decoder threads that read cache PNGs; these create textures unconditionally and remove the index from the corresponding loading set.

### Lazy texture management

`manage_textures(ctx)` runs at the start of every viewer frame:

- if no active tab, loading, empty pages, or `use_cache == false`, return;
- drop full page textures outside `selected_page ± 5`;
- drop thumbnails outside `selected_page ± 15`;
- for pages in `selected_page ± 3`, if the texture is missing and a cache PNG exists and no decoder is already running, spawn a thread to decode `page_N.png` and send `PageDataLoaded`.

Consequences:

- Continuous scroll can show a thumbnail fallback while the full page decodes.
- Rapid scrolling in a large document causes repeated load/drop churn but bounded memory.
- The sidebar independently asks for any visible missing thumbnail via the same cache.

## 5. Text and link extraction

### Character boxes

For each PDFium text char:

```text
left   = loose_bounds.left  / page_width
right  = loose_bounds.right / page_width
top    = 1 - loose_bounds.top    / page_height
bottom = 1 - loose_bounds.bottom / page_height
```

The inversion converts PDF's bottom-left origin to egui's top-left origin. Characters with no `loose_bounds()` are skipped. Whitespace is retained in `page_chars` (used for copy) but ignored when painting selection/annotation rectangles.

### Links

For each link rect, normalise identically. Target resolution order:

1. `action.as_uri_action()` → `PdfLinkTarget::Url`;
2. `action.as_local_destination_action().destination().page_index()` → `PdfLinkTarget::Page`;
3. bare `link.destination().page_index()` → `PdfLinkTarget::Page`.

Links without a target are not cached. Clicking a URL calls `webbrowser::open`; clicking a page link sets `scroll_to_page` (it does not change `selected_page` directly).

### Search

Search is character-sequence matching, not PDF text search:

- toolbar lowercases the query and each page's chars, then scans for contiguous matches;
- it records **page indices**, one per occurrence (the same page appears multiple times for multiple matches);
- Enter cycles `search_active_match` through the page list and scrolls to that page;
- central panel independently re-scans each page and paints yellow overlays on every match.

There is no regex, diacritic folding, whole-word option, or per-match highlight index. Search only works where a text layer exists.

## 6. Layout, zoom, and rendering geometry

### Layout modes

- `ContinuousScroll`: all page indices, one per row.
- `SinglePage`: `[selected_page]`.
- `TwoPage`: `[selected_page, selected_page + 1]` when available.

Navigation steps by 2 in TwoPage mode.

### Fit and zoom

For each page:

```text
aspect = texture_height / texture_width      (inverted if rotation is 90/270)
fit_page_width  = min(available_height / aspect, available_width / chunks)
fit_width_width = available_width / chunks
final_width = max(100,
    fit_page_width + (tab.zoom / 50.0) * (fit_width_width - fit_page_width))
```

`tab.zoom` is a percentage offset, not a multiplier. Displayed zoom is `100 + tab.zoom`, clamped to 0–1000 % by the toolbar drag value. At `zoom = 0` the page fits the available height; at `zoom = 50` it fits the available width.

Multiple input paths adjust zoom:

- toolbar buttons/drag: ±10;
- `Ctrl/Cmd` + wheel: `src/ui/viewer/mod.rs` applies ±0.1, while `central_panel.rs` applies ±10 with a 50 ms throttle — both can observe the same scroll;
- trackpad pinch: `(1 + zoom) * gesture - 1`, floor 0;
- right-click menu: ±10;
- utility bar: ±10.

### Rotation rendering

Preview rotation is `page_rotations[i]` in degrees. The page is drawn as an `egui::Mesh` with rotated UVs:

| Rotation | UV order (TL, TR, BR, BL) |
|---|---|
| 0 | (0,0), (1,0), (1,1), (0,1) |
| 90 | (0,1), (0,0), (1,0), (1,1) |
| 180 | (1,1), (0,1), (0,0), (1,0) |
| 270 | (1,0), (1,1), (0,1), (0,0) |

Aspect ratio is inverted for 90/270. Interaction coordinates are un-rotated with `transform_pos_to_unrot`, and normalised rects are re-rotated with `transform_rect_to_rot`. When adding overlays, always apply these transforms.

### Drawing order inside a page

1. white page rectangle;
2. highlight annotations (semi-transparent, under the page image);
3. blue text-selection rects;
4. yellow search rects;
5. the PDF page mesh (transparent background lets 2–4 show through);
6. signature overlay;
7. link hit areas (invisible);
8. underline/strikethrough/redact/text annotations on top;
9. loading text if no texture.

## 7. Selection model

Selection is a pair of `(page, char)` indices. On primary drag start, `find_closest_char` maps the pointer to the nearest char:

- convert screen point to normalised page coordinates;
- prefer a char whose box contains the point (distance 0);
- otherwise choose the smallest vertical/euclidean distance.

During drag, `selection_end` updates for whatever page the pointer is over (multi-page selections work). On release, `is_selecting` becomes false. `is_char_selected` implements a lexicographic range compare. `get_selected_text` and `copy_selection` walk pages/chars and insert `\n` between pages. Selection rectangles merge adjacent boxes on the same line for a continuous highlight.

`Ctrl/Cmd+A` sets the selection from `(0, 0)` to the last char of the last page. It is gated on `!egui_wants_keyboard_input()` so it does not hijack text fields.

Auto-scroll during selection: in continuous mode, dragging within 40 px of the top/bottom edge scrolls the viewport with a speed proportional to proximity.

## 8. Saving back into the PDF

All save paths open a fresh PDFium document and write to the original path (or a temp then copy). None of them use the cache.

### Annotations (`SaveAnnotations`)

- Iterates actions, fetches the page, converts normalised rects to PDF points with `(1 - y)` inversion.
- Highlight: filled path, alpha 100 fill, `Multiply` blend mode.
- Underline: 2 pt line at `y = (1 - max.y) * h`.
- Strikethrough: 2 pt line at vertical centre.
- Redact: opaque black filled path.
- Text: Helvetica text object, font size = `scale`, fill colour, translated so `position` is the visual centre.
- Saves to a temp PDF, drops the document (Windows lock release), then retries copying over the original up to 10 times with 300 ms sleeps.
- On success sends `AnnotationsSaved`; on failure sends `ExportComplete { success: false }`.
- Entire operation is wrapped in `catch_unwind`; a panic becomes a failure message.

### Signature (`SaveSignature`)

- Opens the image to compute aspect ratio.
- Target width = `150 * scale` PDF points; height = width × aspect.
- Position is the normalised page centre:
  - `x = position.x * page_width - target_width / 2`
  - `y = page_height - position.y * page_height - target_height / 2`
- Creates `PdfPageImageObject::new_with_width`, translates, adds to the page, saves **directly** to the original path.
- Sends `SignatureSaved` or `ExportComplete { success: false }`.

### Rotation (`SaveRotation`)

- Opens the document, reads each page's current rotation, adds the `rotation` delta, normalises to 0/90/180/270, sets it on every page, and saves directly to the original path.
- Sends `RotationSaved` or `ExportComplete { success: false }`.

### Invalidation

The UI calls `reload_pdf` after each success. Because the PDF bytes/metadata changed, the next load generally hashes to a new cache directory. Old directories remain until cleanup.

## 9. Known pipeline limitations

- No OCR and no fallback text extraction from rendered images.
- `render_with_config` failure is silent and displays egui's example image.
- Password-protected files never use cache.
- Cache key uses mtime seconds; fast same-size saves can appear stale.
- `metadata.bin` has no schema version; changing cached types requires a cache-key/schema change.
- Search/link/char boxes assume PDFium's text/link layer is present and correctly ordered.
- Rotation transforms are non-trivial; any new overlay must be routed through both `transform_pos_to_unrot` and `transform_rect_to_rot`.
- Link clicks scroll to the destination page but do not update `selected_page` immediately; the next frame's overlap detection normally corrects it.
- The worker serialises loads/exports/saves, so a large export blocks opening another PDF.
