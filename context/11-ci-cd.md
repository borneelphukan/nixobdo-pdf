# 11 — CI/CD and Releases

This document describes the GitHub Actions workflows and the release gates for
`nixobdo-pdf`. The repository uses four focused workflows instead of one large
build workflow.

## 1. Workflow overview

Four workflows live in `.github/workflows/`:

| Workflow | Trigger | Purpose |
|---|---|---|
| `audit.yml` | Weekly schedule, dependency changes, manual | RustSec audit of `Cargo.lock`. |
| `ci.yml` | Push to `main`, pull request, manual | fmt, Clippy, tests and packaging-asset checks. |
| `release.yml` | Push to `main` or `v*` tag, pull request, manual, weekly schedule | Build every package, publish the apt repository, attach assets to tagged releases. |
| `website.yml` | Pull request touching `website/**`, manual | Lint, type-check and build the landing page without the desktop matrix. |

## 2. `audit.yml`

- Runs `cargo audit` weekly (Sundays at 06:00 UTC) so a newly published advisory
  against a pinned dependency is reported even when the repository is idle.
- Also runs when `Cargo.toml`, `Cargo.lock`, `.cargo/audit.toml` or the workflow
  itself changes.
- Vulnerabilities fail the job. Informational categories (unmaintained, unsound,
  yanked) are printed but do not fail the run.
- Deliberate ignores live in `.cargo/audit.toml`, each with a reason and a
  revisit condition. The four `zip 8.6.0` extraction advisories are ignored
  because `docx-rs 0.4.22` pins `zip` 8 and the application only writes ZIP
  archives; no extraction API is reachable.

## 3. `ci.yml`

Two jobs run on every relevant push and pull request:

### `check`

1. Install GTK/X11/Wayland build dependencies.
2. Install the pinned Rust `1.99.1` toolchain with `rustfmt` and `clippy`.
3. `cargo fmt --all -- --check` — **blocking**.
4. `cargo clippy --all-targets --locked -- -D warnings` — **blocking**.
5. `cargo test --locked` — **blocking** (there are currently zero tests, but
   the job compiles the test target).

### `linux-assets`

- Validates `packaging/linux/nixobdo-pdf.desktop` with
  `desktop-file-validate`.
- Runs `bash -n` / `sh -n` over every packaging script, and
  `python3 -m py_compile` over the apt-repository and version scripts.
- Asserts that `installer/nixobdo-pdf.iss` remains parameterised by the
  release defines.

## 4. `release.yml`

The release pipeline is **tag-driven**. It reads the version from `Cargo.toml`,
and a `v*` tag must match that version exactly. There is no automatic version
bump PR.

### Jobs

| Job | What it produces |
|---|---|
| `metadata` | Version and four-field numeric version; validates tag/version match. |
| `debian-x86_64` | `.deb` for amd64 and apt repository input. Runs even on the weekly schedule. |
| `linux-packages` | Tarball, AppImage and RPM for x86_64. |
| `windows` | Inno Setup installer and portable zip for x64, x86 and ARM64. |
| `macos-universal` | Universal `.app` bundle and DMG. |
| `website` | Static Vite build for GitHub Pages. |
| `publish-pages` | apt repository + website deployed to GitHub Pages. |
| `ci` | The same fmt/Clippy/tests gate as `ci.yml`, enforced before publishing. |
| `release` | Verifies all deliverables, writes `SHA256SUMS`, uploads a workflow artifact, and attaches files to the GitHub release on a `v*` tag. |

### Artifact names

| Platform | Asset |
|---|---|
| Windows x64 | `nixobdo-pdfSetup.exe` (exact historical name required by the in-app updater) |
| Windows x86 | `nixobdo-pdfSetup-x86.exe` |
| Windows ARM64 | `nixobdo-pdfSetup-arm64.exe` |
| Windows portable | `nixobdo-pdf-<version>-windows-<arch>-portable.zip` |
| Debian | `nixobdo-pdf_<version>_amd64.deb` |
| RPM | `nixobdo-pdf-<version>-1.x86_64.rpm` |
| Tarball | `nixobdo-pdf-<version>-linux-x86_64.tar.gz` |
| AppImage | `nixobdo-pdf-<version>-x86_64.AppImage` |
| macOS | `nixobdo-pdf-<version>-macos-universal.dmg` |

The website's download page detects Windows architecture from the file name
(`x86`/`arm64`), so those suffixes are part of the contract.

### PDFium

Every packaging job downloads the matching binary from
`bblanchon/pdfium-binaries` at the latest release:

- Linux: `pdfium-linux-x64.tgz` → `libpdfium.so`
- Windows: `pdfium-win-{x64,x86,arm64}.tgz` → `pdfium.dll`
- macOS: `pdfium-mac-univ.tgz` → `libpdfium.dylib`

Downloads are extracted into a scratch directory to avoid picking up the
checked-in copies under `lib/`.

## 5. `website.yml`

- Runs on pull requests that touch `website/**` or the workflow itself.
- Installs pnpm (version from `website/package.json`) and Node (from
  `website/.nvmrc`), then runs `pnpm lint` and `pnpm build`.
- The release workflow performs the same build for GitHub Pages; this smaller
  workflow just avoids the desktop packaging matrix for website-only changes.

## 6. Release process

1. Update `Cargo.toml` (and ideally `CHANGELOG.md`) on `main`.
2. Run `scripts/bump_version.py --version <version>` to synchronise
   `Cargo.toml`, `Cargo.lock` and the changelog. The installer version comes
   from the release workflow defines, so it is not rewritten.
3. Commit, push `main`, and create/push a matching `v<version>` tag.
4. `release.yml` validates the tag against `Cargo.toml`, builds every package,
   and attaches the verified artifacts to the GitHub release.
5. Ordinary pushes to `main` still build and verify the full bundle, but only
   publish it as a workflow artifact; no GitHub release is created without a
   tag.
6. The weekly scheduled run refreshes the apt repository and Pages site without
   rebuilding the heavy packaging matrix.

## 7. GitHub Pages

`publish-pages` deploys two things to the same Pages site:

- the website build at the site root (`/nixobdo-pdf/`), and
- the apt repository under `/nixobdo-pdf/apt/` plus the installer script at
  `/nixobdo-pdf/install.sh`.

The apt repository is GPG-signed when `APT_REPO_PRIVATE_KEY`,
`APT_REPO_PASSPHRASE` and the `APT_REPO_KEY_ID` variable are configured;
otherwise it is published unsigned and `install.sh` falls back to
`Trusted: yes`.
