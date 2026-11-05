# 12 — Packaging and Release

This document describes how Windows, macOS, and Linux installers are produced,
and the current state of code signing and notarization.

## 1. Windows Installer

- **Script:** `setup.iss` (Inno Setup 6).
- **Build output:** `target/release/nixobdo-pdf.exe` — the Inno Setup compiler
  produces `nixobdo-pdf.exe` (the installer) alongside the release binary.
- **Contents:** the application binary, PDFium `pdfium.dll`, `settings.json`,
  `Cargo.toml`/`Cargo.lock` (optional), icons, and a shortcut in the Start
  menu.
- **Auto‑update:** The updater checks GitHub Releases API for a newer
  `tag_name`; if newer than the installed version, it downloads the new
  `nixobdo-pdfSetup.exe` from a hard‑coded URL and launches it.
- **Signing:** The Inno Setup script can be run with `/si` (sign) and `/vi`
  (verify) flags if a code‑signing certificate is available; currently the repo
  does not enforce signing, but the resulting installer may be flagged by
  Windows SmartScreen.

## 2. macOS App Bundle

- **Script:** `packaging/macos/build-app.sh`.
- **Output:** An `.app` bundle (e.g. `nixobdo-pdf.app`) placed beside the
  release binary.
- **Bundle contents:**
  - `nixobdo-pdf` executable (the Rust binary).
  - `libpdfium.dylib` — PDFium binary copied from the build environment.
  - `Resources/AppIcon.icns` — ICNS generated from `assets/logo.png` at sizes
    16/32/128/256/512 with retina doubles.
  - `Info.plist` — standard macOS plist with bundle identifier,
    `LSApplicationCategoryType: productivity`, `NSHighResolutionCapable: true`.
- **Code signing:** The app is **ad‑hoc signed** (notarization is not performed).
  The signing identity is expected to be set in the build environment; without
  a valid signature the app may show a warning on macOS 10.15+.
- **Entitlements:** Minimal; `NSHighResolutionCapable` is the only special
  entitlement.

## 3. Linux Packaging

Three sub‑scripts under `packaging/linux/`:

| Script | Output |
|---|---|
| `build-deb.sh` | `.deb` package for amd64, depends on `libc6, libgtk-3-0 | libgtk-3-0t64, libxcb1, libxcb-render0, libxcb-shape0, libxcb-xfixes0, libxkbcommon0`. |
| `install.sh` | Portable tarball installer; extracts to `opt/` and creates a `.desktop` entry. |
| `uninstall.sh` | Removes the tarball, `.desktop`, and resets icon cache. |

### `build-deb.sh` flow

1. Copies `assets/logo.png` to the debian package icon path
   (`/usr/share/icons/hicolor/512x512/apps/nixobdo-pdf.png`).
2. Computes `Installed-Size` via `du -sk`.
3. Generates `DEBIAN/control` with package name, version, dependencies, and
   description.
4. Generates `DEBIAN/postinst` and `DEBIAN/postrm` scripts for icon cache
   updates.
5. Packages the whole staging tree into a `.deb` using `dpkg-deb`.

### Packaging script commonalities

- All scripts expect `chmod +x` to have been run once.
- They reference `$REPO_ROOT` (the repository root) for assets and lib PDFium.
- The Linux build in CI (`nightly.yml`) runs `./packaging/linux/build-deb.sh
  target/release/nixobdo-pdf libpdfium.so "$VERSION" dist`.

## 4. Release Assets

A successful stable release publishes the following assets under a `v<version>`
tag:

| Asset | Platform | Format |
|---|---|---|
| Installer | Windows | `.exe` (Inno Setup built) |
| Package | Linux | `.deb` (via `build-deb.sh`) + `.tar.gz` portable tarball |
| Bundle | macOS | `.app` DMG (hand‑built from `build-app.sh`) |
| Metadata | All | `SHA256SUMS` file with checksums for each artifact |

## 5. Obsolete workflows

- `scratch/nightly.yml` and `scratch/stable.yml` are **obsolete copies** of the
  release workflows and **must not** be used as the source of truth. The current
  workflows are in `.github/workflows/`.