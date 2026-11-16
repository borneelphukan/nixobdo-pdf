<div align="center">

<img src="./assets/cover.png" alt="nixobdo-pdf Banner" title="nixobdo-pdf" width="100%"/>

# nixobdo-pdf

**An open-source PDF viewer with the sole intention to provide distraction-free PDF experience that we enjoyed in the old Adobe PDF reader, before they added multiple entities and made it too distracting. Its free and will always remain free.**

[![Release](https://img.shields.io/github/release/borneelphukan/nixobdo-pdf?color=fed114&label=Release&style=flat-square)](https://github.com/borneelphukan/nixobdo-pdf/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/borneelphukan/nixobdo-pdf/total?label=Downloads&style=flat-square&color=lightgreen)](https://github.com/borneelphukan/nixobdo-pdf/releases)

</div>

---

## Quick Start

### Windows
1. Download `nixobdo-pdfSetup.exe` from **[GitHub Releases](https://github.com/borneelphukan/nixobdo-pdf/releases/latest)**.
2. Run the installer and launch the app.

### Ubuntu / Debian
Install from the signed apt repository:
```bash
curl -fsSL https://borneelphukan.github.io/nixobdo-pdf/install.sh | sudo bash
```

Or download the `.deb` package from **[GitHub Releases](https://github.com/borneelphukan/nixobdo-pdf/releases/latest)** and install it manually:
```bash
sudo apt install ./nixobdo-pdf_*_amd64.deb
```
*Alternatively, download the portable tarball (`nixobdo-pdf-*-linux-x86_64.tar.gz`), extract it, and run `./install.sh`.*

> [!NOTE]
> The application bundles the PDFium library in installer packages. For manual builds, PDFium must be placed alongside the executable or in the `lib/` directory.

---

## Features

- **Distraction-Free Viewing**: Open and view any standard PDF document in a clean, minimalistic interface.
- **Secure Signatures**: Add your digital signatures to documents securely.
- **Annotation Tools**: Easily highlight, redact, underline content within your documents, including fill forms with the `Add Text` feature.
- **File Conversion**: Convert your PDF to .doc, .docx, .png or .jpg

---

## Snapshots

<div align="center">

<img src="./assets/Screenshot-1.png" alt="Application Snapshot 1" width="80%" />
<br />
<br />
<img src="./assets/Screenshot-2.png" alt="Application Snapshot 2" width="80%" />

</div>

---

## Setup & Build

### Windows
1. Ensure Rust is installed on your system.
2. Download `pdfium-win-x64.tgz` from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries/releases).
3. Extract `pdfium.dll` and place it in the `lib/` directory or next to the executable.
4. Run:
   ```bash
   cargo run
   ```

### Linux (Ubuntu / Debian)
1. Install system prerequisites:
   ```bash
   sudo apt update
   sudo apt install -y libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev pkg-config
   ```
2. Download `pdfium-linux-x64.tgz` from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries/releases) and extract `libpdfium.so` into the project directory (or `lib/`).
3. Run:
   ```bash
   cargo run
   ```

### Docker
The container runs the viewer on a virtual display and streams it to your browser via noVNC, so you do not need to install Rust, GTK or PDFium locally.

```bash
docker build -t nixobdo-pdf .
docker run --rm -p 6080:6080 nixobdo-pdf
```

Or with Compose:
```bash
docker compose up --build
```

Then open **http://localhost:6080/vnc.html?autoconnect=true&resize=scale** in a browser.

The noVNC browser port (`6080`) is the only published port — the VNC server itself stays bound to loopback inside the container, so the browser is the intended way in.

- To open a PDF on startup, mount a folder and pass the path as an argument:
  ```bash
  docker run --rm -p 6080:6080 -v "$PWD:/docs:ro" nixobdo-pdf /docs/report.pdf
  ```
- Set `VNC_PASSWORD` to password-protect the desktop (noVNC will prompt for it), and `RESOLUTION` to change the window size (default `1600x1000x24`).

> [!NOTE]
> The image is built for `linux/amd64`, because the bundled PDFium binary is x64 only. The build compiles the app from source, so the first `docker build` takes a while; subsequent builds reuse the Cargo cache.

---

## Technical Details

This project leverages the following technologies:

- **`eframe`/`egui`**: For a fast, immediate-mode GUI.
- **`pdfium-render`**: For robust PDF processing and rendering.
- **`rfd`**: For native file dialogs.

