# nixobdo-pdf - AI Agent Context

## Project Overview

**nixobdo-pdf** is an open-source, distraction-free PDF viewer built with Rust using `eframe`/`egui` (immediate mode GUI) and `pdfium-render` for PDF processing. Designed to provide a clean, distraction-free PDF experience reminiscent of the old Adobe Reader.

**Key Details:**
- **Language**: Rust (Edition 2021)
- **GUI Framework**: `eframe` + `egui` (immediate mode)
- **PDF Engine**: `pdfium-render` (PDFium bindings)
- **Target**: Windows (primary), macOS, Linux
- **License**: MIT

## Architecture Overview

### Project Structure
```
src/
├── main.rs                 # Entry point, eframe setup, dark theme config
├── app/
│   ├── mod.rs              # Main app struct (NixobdoPdfApp), state management
│   ├── eframe_app.rs       # eframe::App implementation (update, UI layout)
│   ├── state.rs            # State management (load/close tabs, copy, recent files)
│   └── messages.rs         # Worker message processing (PDF load, export, annotations)
├── document.rs             # PDF document state, PDFium rendering, cache, annotations
├── worker/
│   ├── mod.rs              # Background worker thread, PDFium init, task dispatch
│   └── export.rs           # Export: PDF→Image(ZIP), DOCX, DOC(RTF)
├── ui/
│   ├── mod.rs              # UI module exports
│   ├── viewer/
│   │   ├── mod.rs          # Main viewer layout (sidebar, separator, central)
│   │   ├── central_panel.rs # PDF rendering, selection, annotations, signatures
│   │   ├── sidebar.rs      # Thumbnail sidebar navigation
│   │   └── separator.rs    # Draggable sidebar separator
│   ├── toolbar.rs          # Top toolbar (zoom, nav, search, annotations)
│   ├── tabs.rs             # Tab bar (multi-tab PDF support)
│   ├── menu_bar.rs         # Menu bar (File, Edit, View, Help)
│   ├── dialogs/
│   │   ├── mod.rs
│   │   ├── export.rs       # Export dialog (format, options)
│   │   ├── export_progress.rs
│   │   ├── update.rs       # Auto-update dialog
│   │   ├── about.rs        # About dialog
│   │   ├── custom_color.rs
│   │   ├── rename.rs
│   │   └── export.rs
│   ├── toast.rs            # Toast notifications
│   └── splash.rs           # Splash screen
```

### Core Types (document.rs)
```rust
// Document state per tab
PdfDocumentState {
    file_name, path, pages: Vec<TextureHandle>, thumbnails,
    page_texts, page_chars, page_links, page_sizes,
    page_rotations: Vec<i32>, zoom, selected_page, layout_mode,
    error, is_loading, ...
}

// Annotation system
AnnotationTool: Highlight | Underline | Strikethrough | Redact | Text
AnnotationAction { tool, page_index, rects, position, text, color, scale, bold, italic, underline }

// Worker messages (async communication)
PdfWorkerMessage: DocumentInfo, PageData, Finished, ExportProgress, ExportComplete,
    SignatureSaved, RotationSaved, AnnotationsSaved, UpdateCheckResult, UpdateDownloadProgress, ...
```

### Background Worker Architecture
- **Thread**: Dedicated worker thread (`spawn_worker_thread`)
- **Communication**: `mpsc` channels (task TX, message RX)
- **PDFium**: Initialized once in worker thread, reused for all operations
- **Tasks**: `Load`, `Export`, `CheckUpdate`, `DownloadUpdate`, `SaveSignature`, `SaveRotation`, `SaveAnnotations`

### Caching System
- Cache dir: `~/.cache/nixobdo-pdf-cache/{hash}/`
- Stores: rendered pages (PNG), thumbnails, metadata (bincode)
- TTL: 7 days, max cache size: 400MB
- Cache key: path hash + file size + mtime

## Features

### Core Viewing
- **Multi-tab PDF viewing** - Multiple PDFs in tabs
- **Continuous scroll**, Single page, Two-page layout modes
- **Zoom**: Ctrl+Wheel, pinch gesture, toolbar controls (10%-1000%)
- **Page navigation**: Arrow keys, toolbar buttons, sidebar thumbnails
- **Search**: Ctrl+F, highlights matches, navigation between matches
- **Text selection & copy**: Drag selection, Ctrl+C, right-click context menu
- **Rotation**: Per-page rotation (90° increments), save to PDF

### Annotations (Edit Mode)
Tools: **Highlight**, **Underline**, **Strikethrough**, **Redact**, **Text**
- **Highlight/Underline/Strikethrough**: Drag-select text → auto-merge adjacent chars into rects
- **Redact**: Draw black rectangles over content
- **Text**: Click to place, edit inline, style (size, bold, italic, underline, color)
- **Undo/Redo**: Stack-based for pending annotations
- **Save**: Commits annotations to PDF via PDFium (path objects, text objects)

### Digital Signatures
- Load PNG/JPG signature image
- Drag to position, resize with corner handle
- Save: embeds as image object in PDF via PDFium

### Export (Background Worker)
| Format | Description |
|--------|-------------|
| **PNG/JPEG** | Renders pages to images, zipped |
| **DOCX** | Text extraction with layout preservation option, image extraction |
| **DOC (RTF)** | Text + optional images as RTF |

Options: Retain layout, Include images, Progress dialog with cancel

### Auto-Update
- Checks GitHub Releases API on startup + manual check
- Downloads Windows installer (.exe) to Downloads folder
- Auto-launches installer, exits current app

### UI/UX
- **Dark theme** (default), respects system light/dark
- **Splash screen** (2.5s) with logo
- **Toast notifications** (success/error, auto-dismiss)
- **Drag-to-resize** sidebar separator
- **Fullscreen** support (hides sidebar/separator)
- **Recent files** (File → Open Recent, max 5)

## Key Dependencies (Cargo.toml)

| Crate | Purpose |
|-------|---------|
| `eframe` `0.34` | GUI framework (egui wrapper) |
| `egui` `0.34` | Immediate mode GUI |
| `egui_extras` | Image loaders, extras |
| `pdfium-render` `0.8` | PDF rendering (requires PDFium binary) |
| `image` `0.25` | Image processing |
| `docx-rs` `0.4` | DOCX generation |
| `zip` `8.6` | ZIP for image export |
| `ureq` `3.3` | HTTP (update checks/downloads) |
| `rfd` `0.17` | Native file dialogs |
| `rayon` `1.12` | Parallel processing (unused currently) |
| `serde` + `bincode` | Cache serialization |

## Build & Run Requirements

### PDFium Binary (Required)
- **Windows**: `pdfium.dll` next to exe or in `lib/`
- **macOS**: `libpdfium.dylib` next to exe or in `lib/`
- Download from: `bblanchon/pdfium-binaries` releases

### Build Commands
```bash
cargo run                    # Debug run
cargo build --release        # Release build
cargo check                  # Type check
```

### Windows Installer
- `setup.iss` (Inno Setup script) for Windows installer
- Build outputs to `target/release/nixobdo-pdf.exe`

## Entry Point (main.rs)
```rust
fn main() -> eframe::Result<()> {
    // 1. Load app icon
    // 2. Configure viewport (800x1000, title, icon)
    // 3. Set global dark theme with custom colors
    // 4. Run eframe with NixobdoPdfApp
    // 5. Handle CLI arg: cargo run -- <pdf_path>
}
```

## UI Layout Structure (eframe_app.rs)

```
Viewport
├── MenuBar (File, Edit, View, Help)
├── Toolbar (Zoom, Navigation, Rotation, Search)
├── TabBar (Multi-tab)
└── Viewer Area (when tab active)
    ├── Sidebar (Thumbnails) - resizable, toggleable
    ├── Separator (draggable, 1px)
    └── CentralPanel (PDF rendering)
        ├── Pages rendered as textures with mesh UV transforms for rotation
        ├── Highlights drawn BEFORE page image (transparent bg trick)
        ├── Text selection overlay (blue)
        ├── Search highlights (yellow)
        ├── Annotation tools (crosshair cursor, drag to create)
        ├── Signature placement (drag + resize handle)
        └── Rotation overlay window
├── Toast (bottom-right)
├── Export Progress Dialog (modal)
├── Update Dialog (banner/window)
├── About Dialog (separate viewport)
└── Splash Screen (2.5s on startup)
```

## Key Implementation Details

### PDF Rendering Pipeline
1. **Background thread**: PDFium loads PDF, renders pages to bitmaps (2400px width)
2. **White bg removal**: Mathematically removes white background → transparent (alpha blending highlights)
3. **Cache**: Saves PNG + thumb to cache dir
4. **Main thread**: Receives `PageData` message → creates egui textures
5. **Rendering**: Central panel uses `egui::Mesh` with custom UV coords for rotation

### Text Selection & Search
- `page_chars`: `Vec<PdfCharInfo>` per page (char + normalized bounds)
- `find_closest_char()`: Maps screen pos → char index
- `is_char_selected()`: Range check across pages
- Search: lowercase char matching on `page_chars`, highlights via yellow rects

### Annotation Rendering (PDF Save)
Worker thread uses PDFium to add:
- **Highlight**: Path objects with multiply blend, semi-transparent fill
- **Underline/Strikethrough**: Line path objects
- **Redact**: Black filled path objects
- **Text**: Text objects with font, size, color, position
- **Save**: Write to temp file → copy over original (retry loop for Windows locks)

### Coordinate Systems
- **PDF points**: 72 DPI, origin bottom-left
- **egui**: Logical pixels, origin top-left
- **Normalized (0-1)**: Used for char bounds, signature position, selections
- **Transforms**: `transform_pos_to_unrot`, `transform_rect_to_rot` handle 90/180/270° rotation

## Common Tasks for AI Agents

### Adding a New Annotation Tool
1. Add variant to `AnnotationTool` enum (`document.rs`)
2. Add button in `toolbar.rs` annotation toolbar
3. Handle selection in `central_panel.rs` (drag logic)
4. Add rendering in `central_panel.rs` (preview)
5. Add PDF save logic in `worker/mod.rs` `SaveAnnotations` task

### Adding Export Format
1. Add variant to `ExportFormat` enum (`worker/export.rs`)
2. Implement export function in `worker/export.rs`
3. Dispatch in `worker/mod.rs` `Export` task match arm
4. Add UI option in `dialogs/export.rs`

### Adding Menu Item
1. Add handler in `menu_bar.rs` appropriate menu
2. Add state fields in `app/mod.rs` if needed
3. Implement action in `state.rs` or inline

### Modifying PDF Rendering
- Render config: `PdfRenderConfig` in `document.rs:393`
- Cache logic: `background_load_with_pdfium` in `document.rs:291`
- Page display: `central_panel.rs` mesh UV transforms

## Testing & Quality
```bash
cargo check          # Type check
cargo build --release # Release build
cargo run            # Run debug
```

## Large PDF Memory Management

To prevent GPU/process OOM on large PDFs (e.g., 498+ pages), the following safeguards are in place:

### Texture Window (GPU Memory Budget)
- **Window size**: `TEXTURE_WINDOW = 30` pages ahead and behind `selected_page`
- `texture_window_range()` computes the range of pages that should have GPU textures
- `sync_texture_window(ctx)` is called each frame after `process_messages()`:
  - Loads textures from disk cache for pages entering the window
  - Frees textures (`pages[i] = None, thumbnails[i] = None`) for pages leaving the window
- `load_page_texture_from_cache(ctx, index)` reads cached PNGs from disk and creates textures on demand

### Channel Backpressure
- Message channel (`PdfWorkerMessage`) uses `sync_channel(8)` instead of unbounded `channel()`
- Worker blocks when the channel is full, preventing unbounded memory growth

### PageData Handling
- In `messages.rs::PageData` handler: metadata (text, chars, links, size) is always stored
- GPU textures are only created if `index` is within `texture_window_range()`
- ColorImage for out-of-window pages is immediately dropped (memory freed)
- The worker already cached the page to disk, so it's available for lazy loading

### Render Resolution
- Full pages: rendered at 1200px width (down from 2400px) → ~3.9 MB per page
- Thumbnails: rendered at 150px width (down from 300px) → ~0.125 MB per page

### Cache Loading Path
- When loading from disk cache, all page metadata+images still flow through the bounded channel
- The main thread filters texture creation using the same texture window logic

### Windows
- Requires `pdfium.dll` alongside exe or in `lib/`
- Uses `winres` for .exe icon/manifest (build.rs)
- File locking: temp file + retry copy pattern for saves

### macOS
- Requires `libpdfium.dylib` in `lib/` or bundled
- Dark/light theme follows system

### Linux
- Requires `pdfium` shared library (system or bundled)
- Uses `dirs` crate for cache/download directories

## GitHub Actions / CI
- `.github/workflows/` - Check for build/release workflows
- Releases: Tag `v*` triggers build, uploads artifacts

## Common Patterns in Codebase

| Pattern | Example Location |
|---------|------------------|
| `ui.scope()` for style isolation | `tabs.rs`, `toolbar.rs` |
| `ui.ctx().request_repaint()` for async updates | `messages.rs`, `central_panel.rs` |
| `mpsc` channels for thread communication | `app/mod.rs`, `worker/mod.rs` |
| `Arc<AtomicBool>` for cancel flags | `export.rs`, `worker/mod.rs` |
| `egui::Area` for floating UI (toasts, dialogs) | `toast.rs`, `central_panel.rs` |
| `ui.input(|i| ...)` for input handling | `central_panel.rs`, `toolbar.rs` |
| `Option<PathBuf>` for optional file paths | `app/mod.rs` state fields |

## Assets
- `assets/logo.png` - App icon
- `assets/icons/*.svg` - Toolbar icons (highlight, underline, strikethrough, redact, rotate)
- `assets/cover.png` - README banner

## Version & Metadata
- Version: `0.1.28` (Cargo.toml)
- Repo: `github.com/borneelphukan/nixobdo-pdf`
- Author: Borneel B. Phukan