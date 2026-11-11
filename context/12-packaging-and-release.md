# 12 — Packaging and Release

This document describes how Windows, macOS and Linux packages are produced, and
the current state of code signing and notarization.

## 1. Windows installer

- **Script:** `installer/nixobdo-pdf.iss` (Inno Setup 6), parameterised by
  preprocessor defines.
- **Defines supplied by CI:** `MyAppVersion`, `MyVersionInfo` (four-field
  numeric), `MyArch` (`x64`/`x86`/`arm64`), `MySourceExe`, `MyPdfiumDll` and
  `MyOutputBaseFilename`.
- **Output directory:** `Output/` at the repository root.
- **Contents:** `nixobdo-pdf.exe`, `pdfium.dll`, and `LICENSE`/`README.md`
  under `docs/`.
- **Architecture:** `x64` uses `ArchitecturesInstallIn64BitMode=x64compatible`;
  `arm64` installs in 64-bit ARM mode; `x86` installs in 32-bit mode so it also
  runs on 32-bit Windows.
- **Auto-update:** the updater queries the GitHub Releases API and downloads
  `nixobdo-pdfSetup.exe` from a hard-coded URL. The x64 installer therefore
  keeps that exact asset name; non-x64 installers append `-x86` or `-arm64`.
- **Signing:** not performed. The resulting installer may trigger Windows
  SmartScreen.

## 2. macOS app bundle

- **Script:** `packaging/macos/build-app.sh`.
- **Usage:** `build-app.sh [BINARY] [LIBPDFIUM] [VERSION] [OUT_DIR] [ARCH]`.
- **Output:** a `.app` bundle and a DMG named
  `nixobdo-pdf-<version>-macos-<arch>.dmg`. CI passes `universal`, giving
  `nixobdo-pdf-<version>-macos-universal.dmg`.
- **Bundle contents:**
  - `Contents/MacOS/nixobdo-pdf` — the Rust binary (universal in CI).
  - `Contents/MacOS/libpdfium.dylib` and `Contents/Resources/libpdfium.dylib` —
    PDFium is looked up next to the executable first.
  - `Contents/Resources/AppIcon.icns` — generated from `assets/logo.png`.
  - `Contents/Info.plist` — bundle identifier, version, minimum macOS 11.0 and
    `NSHighResolutionCapable`.
- **Code signing:** ad-hoc signed with `codesign --sign -`. Not notarized, so
  Gatekeeper still shows a warning for downloaded builds.
- **Default PDFium path:** `lib/libpdfium.dylib` (the checked-in development
  copy), overridable with the second argument.

## 3. Linux packaging

| Script | Output |
|---|---|
| `packaging/linux/build-deb.sh` | `.deb` for amd64 with dependencies `libc6`, GTK 3, XCB and xkbcommon. |
| `packaging/linux/build-tarball.sh` | Portable `nixobdo-pdf-<version>-linux-<arch>.tar.gz`. |
| `packaging/linux/build-appimage.sh` | `nixobdo-pdf-<version>-<arch>.AppImage`. |
| `packaging/rpm/build-rpm.sh` | `nixobdo-pdf-<version>-1.<arch>.rpm` from `packaging/rpm/nixobdo-pdf.spec.in`. |
| `packaging/linux/install.sh` | Installs a staged tarball into `/usr/local` or `~/.local`. |
| `packaging/linux/uninstall.sh` | Removes those installed files. |

`build-deb.sh` and `build-rpm.sh` take positional arguments
`BINARY LIBPDFIUM VERSION OUTPUT_DIR`; the tarball and AppImage builders read
`VERSION`, `ARCH`, `BINARY`, `LIBPDFIUM` and `OUT_DIR` from the environment.

All Linux packages place `libpdfium.so` next to the binary or under
`/usr/lib/nixobdo-pdf/`, because the application loads PDFium from the
executable directory first.

### AppImage

The AppImage contains:

- `usr/bin/nixobdo-pdf` and `usr/bin/libpdfium.so` (so the executable-relative
  lookup succeeds),
- a fallback copy under `usr/lib/nixobdo-pdf/`,
- the desktop entry and icon,
- an `AppRun` that execs `usr/bin/nixobdo-pdf`.

`appimagetool` is downloaded on demand unless `APPIMAGETOOL` points at an
existing binary.

## 4. apt repository and Pages

`packaging/apt/build-repo.py` assembles a static apt repository from the `.deb`
files:

```text
<out>/apt/dists/stable/Release
<out>/apt/dists/stable/main/binary-amd64/Packages[.gz|.xz]
<out>/apt/pool/main/n/nixobdo-pdf/*.deb
<out>/install.sh
<out>/index.html
```

`publish-pages` copies the website build over this output and deploys the merged
tree to GitHub Pages, so the repository is served from
`https://borneelphukan.github.io/nixobdo-pdf/apt/`.

Users install with:

```bash
curl -fsSL https://borneelphukan.github.io/nixobdo-pdf/install.sh | sudo bash
```

`packaging/apt/install.sh` supports `--print`, `--remove`, `--yes` and
`--help`. When a signing key is published it writes a
`Signed-By:` source; otherwise it falls back to `Trusted: yes`.

## 5. Release assets

A tagged release (`v<version>`) publishes:

| Asset | Platform | Format |
|---|---|---|
| `nixobdo-pdfSetup.exe` | Windows x64 | Inno Setup installer |
| `nixobdo-pdfSetup-x86.exe` | Windows x86 | Inno Setup installer |
| `nixobdo-pdfSetup-arm64.exe` | Windows ARM64 | Inno Setup installer |
| `nixobdo-pdf-<version>-windows-<arch>-portable.zip` | Windows | Portable zip |
| `nixobdo-pdf_<version>_amd64.deb` | Linux | Debian package |
| `nixobdo-pdf-<version>-1.x86_64.rpm` | Linux | RPM package |
| `nixobdo-pdf-<version>-linux-x86_64.tar.gz` | Linux | Portable tarball |
| `nixobdo-pdf-<version>-x86_64.AppImage` | Linux | AppImage |
| `nixobdo-pdf-<version>-macos-universal.dmg` | macOS | Universal DMG |
| `SHA256SUMS` | All | Checksums for every asset |

The website's download page classifies assets by extension and, for Windows, by
`x86`/`arm64` tokens in the file name. Keep those names stable when changing the
workflow.
