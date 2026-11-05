# 11 — CI/CD, Nightly Builds, and Stable Releases

This document describes the GitHub Actions workflows, release gates, and the
automated release process for `nixobdo-pdf`.

## 1. Workflow overview

Three primary workflows live in `.github/workflows/`:

| Workflow | Trigger | Purpose |
|---|---|---|
| `stable.yml` | Push to `main` | Stable release planning, bump PR, multi‑OS build, publish. |
| `nightly.yml` | Push to `dev` | Rolling `dev-release` prerelease, Ubuntu package build. |
| `review.yml` | Pull request | PR gates: fmt, clippy, build on 3 OSes, audit, test. |
| `website.yml` | Push to `main` or PR | Build/lint website (Vite + GitHub Pages). |
| `deploy.yml` | Push to `main` | Deploy GitHub Pages site. |

## 2. `.github/workflows/stable.yml`

Triggered on push to `main`. Main flow:

1. **Plan** — runs `scripts/release_plan.py` to determine the next version and
   mode (`bump`/`release`/`skip`).
2. **Bump version and changelog** — if mode is `bump`, runs
   `scripts/bump_version.py` to update `Cargo.toml`, `Cargo.lock`,
   `setup.iss`, and `CHANGELOG.md`; then commits, pushes, and opens a PR
   (`chore/bump-version-<VERSION>`) against `main`.
3. **Build Windows installer** — if mode is `release`, builds the Windows
   installer on `windows-latest` using `cargo build --locked` and Inno Setup
   (`setup.iss`).
4. **Build Linux/macOS** — builds release binaries on `ubuntu-latest` and
   `macos-latest`; packages Linux .deb via `packaging/linux/build-deb.sh` and
   macOS .app via `packaging/macos/build-app.sh`.
5. **Publish release** — if mode is `release`, creates a `v<version>` GitHub
   release and uploads Windows installer, Linux .deb, macOS DMG, and
   `SHA256SUMS` file.

### Version bump PR

- Branch name: `chore/bump-version-<VERSION>`.
- Updates: `Cargo.toml`, `Cargo.lock`, `setup.iss`, `CHANGELOG.md`.
- Commit message: `chore: bump version to <VERSION>`.
- Auto‑pr: `gh pr create --title "chore: bump version to $VERSION" --base main --head "$BRANCH"`.
- If a PR with that branch already exists, it is updated rather than creating a
  new one.

### Build gates (blocking)

- `cargo fmt --all -- --check` — **blocking**; fails the workflow.
- `cargo build --locked` on Ubuntu, Windows, macOS — **blocking**; each must
  succeed.
- `cargo test --locked --all-targets` — **blocking but empty** (no Rust tests);
  still runs and passes with zero tests.
- `rust-audit` (cargo‑audit) — **advisory** (`continue-on-error: true`);
  not currently blocking but the intent is to make it blocking once the
  dependency tree is clean.
- `clippy` — **advisory**; warnings do not fail the job but are tracked.

## 3. `.github/workflows/nightly.yml`

Triggered on push to `dev` branch. Flow:

1. **Build Windows dev installer** — produces
   `nixobdo-pdf-dev-<run_number>.exe` with 1‑day retention.
2. **Build Ubuntu packages** — cross‑compiles for Linux, builds .deb and
   tarball, stores under `dist/`.
3. **Publish dev-release prerelease** — the nightly build publishes a
   `dev-release` prerelease on GitHub, tracking the latest nightly commit.

### Key steps

- Installs system deps (`libgtk-3-dev`, etc.).
- Sets up Rust toolchain, caches Rust build.
- Downloads and extracts PDFium binaries from `bblanchon/pdfium-binaries`.
- Builds with `cargo build --release --locked`.
- Builds Ubuntu packages via `packaging/linux/build-deb.sh`.
- Builds macOS app bundle via `packaging/macos/build-app.sh`.

## 4. `.github/workflows/review.yml`

Runs on every Pull Request. Gates:

| Gate | Command | Status |
|---|---|---|
| **fmt** | `cargo fmt --all -- --check` | **blocking** |
| **clippy** | `cargo clippy --locked --all-targets` | **blocking** (warnings become errors via `-D warnings` in CI config) |
| **rust build** | `cargo build --locked` on Ubuntu, Windows, macOS (matrix) | **blocking** |
| **rust-audit** | `cargo audit` (advisory, `continue-on-error: true`) | **advisory** |
| **test** | `cargo test --locked --all-targets` | **blocking but empty** (0 tests) |

All gates must pass for the PR to be mergeable.

## 5. Release process (stable → main)

1. **Push to `main`** triggers `stable.yml`.
2. `scripts/release_plan.py` examines the git log since the last tag and decides
   one of three modes:
   - **bump** — increment the patch version, create a bump PR.
   - **release** — if the push includes a `v<version>` tag, or if the plan
     decides a release is warranted, proceed to full build+publish.
   - **skip** — no release this cycle; workflow exits early.
3. If mode is `bump`, the bump PR is opened. The PR author (or maintainer) merges
   it, which updates `Cargo.toml`, `Cargo.lock`, `setup.iss`, and
   `CHANGELOG.md` with an entry for the new version.
4. After the bump PR is merged (or if mode was `release`), the workflow builds
   installers for all three platforms, publishes a GitHub release at
   `v<version>`, and uploads:
   - Windows: `nixobdo-pdfSetup.exe`
   - Linux: `.deb` tarball and/or `.tar.gz`
   - macOS: `.app` DMG
   - `SHA256SUMS` file for checksum verification.
5. The new tag `v<version>` is created on `main`; future pushes compare against
   this tag.

## 6. Website CI

- `.github/workflows/website.yml` runs on pushes to `main` and on PRs.
- Uses Vite to build the React app with `base: '/nixobdo-pdf'`.
- Lints built files; deploys to GitHub Pages only from `main` (not from `dev`).
- No `404.html` fallback, so deep‑link refreshes on GitHub Pages may fail if the
  route is not exactly configured.
