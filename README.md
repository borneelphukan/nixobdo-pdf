<div align="center">

<img src="./assets/cover.png" alt="nixobdo-pdf Banner" title="nixobdo-pdf" width="100%"/>

# nixobdo-pdf

**An open-source PDF viewer with the sole intention to provide distraction-free PDF experience that we enjoyed in the old Adobe PDF reader, before they added multiple entities and made it too distracting. Its free and will always remain free.**

[![Release](https://img.shields.io/github/release/borneelphukan/nixobdo-pdf?color=fed114&label=Release&style=flat-square)](https://github.com/borneelphukan/nixobdo-pdf/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/borneelphukan/nixobdo-pdf/total?label=Downloads&style=flat-square&color=lightgreen)](https://github.com/borneelphukan/nixobdo-pdf/releases)
[![CI](https://github.com/borneelphukan/nixobdo-pdf/actions/workflows/windows-build.yml/badge.svg)](https://github.com/borneelphukan/nixobdo-pdf/actions/workflows/windows-build.yml)

<a href="https://www.buymeacoffee.com/borneelphukan" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" style="height: 50px !important;width: 200px !important;" ></a>

</div>

---

## Quick Start

1. Download the latest version from **[GitHub Releases](https://github.com/borneelphukan/nixobdo-pdf/releases/latest)**.
2. Run the application directly.

> [!NOTE]
> The project requires the PDFium library to be present alongside the executable or in the `lib/` directory.

---

## Features

- **View PDF files**: Open and view any standard PDF document.
- **Scroll**: Scroll up and down pages smoothly using the mouse wheel.
- **Zoom**: Zoom in and out effortlessly with `Cmd/Ctrl + Mouse Wheel` or by using the built-in Zoom slider.

---

## Setup & Build

### macOS

1. Ensure Rust is installed on your system.
2. The project requires `libpdfium.dylib`. Ensure it has been downloaded into the `lib/` directory.
3. Run the application:
   ```bash
   cargo run
   ```

### Windows

To build for Windows, you will need the Windows version of `pdfium.dll`.

1. Download `pdfium-win-x64.tgz` (or x86) from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries/releases).
2. Extract `pdfium.dll` and place it in the `lib/` directory or next to the executable.
3. If cross-compiling from Mac, use `cargo-xwin`:
   ```bash
   cargo xwin build --target x86_64-pc-windows-msvc
   ```

---

## Technical Details

This project leverages the following technologies:

- **`eframe`/`egui`**: For a fast, immediate-mode GUI.
- **`pdfium-render`**: For robust PDF processing and rendering.
- **`rfd`**: For native file dialogs.

---

## License

<div align="center">

See the [LICENSE](./LICENSE) file for details.

</div>
