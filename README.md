# PDFViewer

A simple PDF viewer built with Rust, egui, and PDFium.

## Features
- View PDF files.
- Scroll up and down with the mouse wheel.
- Zoom in and out with `Cmd/Ctrl + Mouse Wheel` or the Zoom slider.

## Setup (macOS)
1. Ensure Rust is installed.
2. The project requires `libpdfium.dylib`. It has been downloaded into the `lib/` directory.
3. Run the application:
   ```bash
   cargo run
   ```

## Setup (Windows)
To build for Windows, you will need the Windows version of `pdfium.dll`.
1. Download `pdfium-win-x64.tgz` (or x86) from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries/releases).
2. Extract `pdfium.dll` and place it in the `lib/` directory or next to the executable.
3. If cross-compiling from Mac, use `cargo-xwin`:
   ```bash
   cargo xwin build --target x86_64-pc-windows-msvc
   ```

## Development
This project uses:
- `eframe`/`egui` for the GUI.
- `pdfium-render` for PDF processing.
- `rfd` for file dialogs.
