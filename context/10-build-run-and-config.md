# 10 — Building, Running, and Local Configuration

This document describes how to build, run, and configure `nixobdo-pdf` locally.

## 1. Prerequisites

- **Rust toolchain:** stable (as specified in `rust-toolchain.toml`), with
  `clippy` and `rustfmt` available.
- **PDFium binary:** required at runtime. The worker thread binds PDFium using
  this priority chain (also documented in `src/worker/mod.rs` and
  `src/app/mod.rs`):
  1. `<exe_dir>/libpdfium.dylib` (macOS)
  2. `<exe_dir>/pdfium.dll` (Windows)
  3. `<exe_dir>/libpdfium.so` (Linux)
  4. `./lib/libpdfium.dylib`
  5. `libpdfium.dylib`
  6. `./lib/pdfium.dll`
  7. `pdfium.dll`
  8. `./lib/libpdfium.so`
  9. `libpdfium.so`
  10. System library (`Pdfium::bind_to_system_library()`)
- If none of the above are found, `has_pdfium_bindings = false` and the app
  shows a red “⚠ PDFium library not found!” label in the toolbar.

### Platform notes

| Platform | Binary location |
|---|---|
| **Windows** | `pdfium.dll` alongside the exe or in `lib/` |
| **macOS** | `libpdfium.dylib` alongside the exe or in `lib/` |
| **Linux** | `libpdfium.so` alongside the exe or in `lib/`, or installed system‑wide |

## 2. Running locally

```bash
cargo run                # Debug run
cargo run -- <pdf_path> # Open a PDF on startup
cargo build --release    # Release build
cargo check              # Type check only
```

### First‑run behavior

- On first launch, the app creates `dirs::config_dir()/nixobdo-pdf/settings.json`
  if it does not exist (all fields default to empty/false).
- If a PDF path is passed as a CLI argument (`cargo run -- /path/to/doc.pdf`),
  `main.rs` forwards it to `load_pdf()` which opens a new tab.

### Configuration directory

- Path: `dirs::config_dir().join("nixobdo-pdf")` (e.g.
  `~/.config/nixobdo-pdf/` on Linux, `%APPDATA%/nixobdo-pdf/` on Windows,
  `~/Library/Application Support/nixobdo-pdf/` on macOS).
- Contains `settings.json` (JSON) and may contain other runtime files.

### Cache directory

- Path: `dirs::cache_dir().join("nixobdo-pdf-cache")` (e.g.
  `~/.cache/nixobdo-pdf-cache/`).
- Each PDF gets its own subdirectory named by a hash: `DefaultHasher` over
  `PathBuf`, `metadata.len()`, and `metadata.modified()` converted to seconds
  since UNIX epoch.
- A save that changes the file size or mtime (seconds) produces a new key.
- **Contents of each hash directory:**
  - `metadata.bin` — bincode tuple of page count, texts, chars, links, sizes.
  - `page_N.png` — transparent full‑page render, 2400 px target width.
  - `thumb_N.png` — 300 px wide thumbnail, white background retained.
  - `accessed.txt` — marker touched on cache hit.
- **Cleanup:** runs at startup; removes directories stale for >7 days or total
  >400 MB (oldest‑accessed first).

## 3. Building for development

```bash
# Debug build (run)
cargo run

# Release build (binary in target/release/nixobdo-pdf)
cargo build --release

# Run with specific PDF
cargo run -- path/to/file.pdf
```

### PDFium for development

- If running `cargo run` from the repo root, the executable looks for PDFium
  next to the binary or in `lib/`. For debug builds the binary path is the
  `target/debug/` directory; for release it is `target/release/`.
- It is common to copy the OS‑specific PDFium library into `lib/` at the repo
  root so that both debug and release builds find it:
  ```bash
  cp /path/to/libpdfium.dylib lib/
  # or on Windows
  copy libpdfium.dll lib\
  # or on Linux
  cp libpdfium.so lib\
  ```

## 4. Common local‑dev tasks

| Task | Command |
|---|---|
| Format check | `cargo fmt --all -- --check` |
| Lint (clippy) | `cargo clippy --locked --all-targets` |
| Dependency audit | `cargo audit` |
| Run all checks | `cargo fmt --check && cargo clippy --locked --all-targets && cargo build --locked && cargo test --locked --all-targets` |