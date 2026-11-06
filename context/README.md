# nixobdo-pdf — AI Agent Context Pack

This directory is the **authoritative, current context** for anyone (human or AI) working on `nixobdo-pdf`. It describes the application as it exists at:

| Field | Value |
|---|---|
| Repository | `github.com/borneelphukan/nixobdo-pdf` |
| Branch verified | `main` |
| Commit verified | `1333e78 chore: bump version to 0.1.38 (#103)` |
| Version | `0.1.38` (`Cargo.toml`, `setup.iss`) |
| Rust edition | `2024` |
| UI stack | `eframe` / `egui` `0.37` |
| PDF engine | `pdfium-render` `0.8.37` + bundled/OS PDFium |
| Platforms | Windows (primary), macOS, Linux |
| License | MIT |
| Website | React 19 + TypeScript + Vite 8, GitHub Pages |

> **Important:** the root `AGENTS.md` is older than the code. It still says edition 2021, version 0.1.28, eframe 0.34, and does not describe the AI/LLM features. Use this `context/` directory instead, and treat `AGENTS.md` as a historical summary.

---

## What to read first

| Question | Read |
|---|---|
| What does the app do and what are its boundaries? | [01-project-overview.md](01-project-overview.md) |
| Which file owns which behaviour? | [02-project-structure.md](02-project-structure.md) |
| How do the process, threads, UI, worker, and state fit together? | [03-architecture.md](03-architecture.md) |
| What are the core Rust types and message/task contracts? | [04-data-model.md](04-data-model.md) |
| How are PDFs loaded, rendered, cached, selected, and rotated? | [05-pdf-pipeline.md](05-pdf-pipeline.md) |
| How is the egui UI assembled and how do shortcuts behave? | [06-ui-and-interactions.md](06-ui-and-interactions.md) |
| How does each user-facing feature work end to end? | [07-features-and-flows.md](07-features-and-flows.md) |
| How do AI summaries, chat, and LLM configuration work? | [08-ai-and-llm.md](08-ai-and-llm.md) |
| How does background work and messaging work? | [09-worker-and-concurrency.md](09-worker-and-concurrency.md) |
| How do I build, run, and configure it locally? | [10-build-run-and-config.md](10-build-run-and-config.md) |
| How do CI, nightly builds, and stable releases work? | [11-ci-cd.md](11-ci-cd.md) |
| How are Windows/Linux/macOS installers produced? | [12-packaging-and-release.md](12-packaging-and-release.md) |
| How does the GitHub Pages website work? | [13-website.md](13-website.md) |
| What patterns, bugs, and platform traps should I avoid? | [14-conventions-and-gotchas.md](14-conventions-and-gotchas.md) |
| I need a concrete step-by-step change recipe | [15-task-recipes.md](15-task-recipes.md) |

---

## Critical facts at a glance

1. **There is one native process and one primary background worker.** `NixobdoPdfApp` owns all UI state. A dedicated worker thread owns the `Pdfium` instance and serialises `Load`, `Export`, `SaveSignature`, `SaveRotation`, and `SaveAnnotations`. Network work (update checks/downloads, LLM calls) is spawned onto short-lived sub-threads.
2. **The UI is immediate mode.** Every frame rebuilds the UI by calling `impl NixobdoPdfApp` methods. There is no retained widget tree. Local `bool` flags (e.g. `zoom_in`, `perform_export`, `send_msg`) are the standard way to defer mutations until after a UI closure.
3. **PDFium is a runtime dependency, not a compile-time one.** It must be next to the executable, in `lib/`, in the current directory, or installed system-wide. The app reports `has_pdfium_bindings` in the toolbar and the worker independently binds the library.
4. **The PDF page pipeline is cache-first.** Rendered pages are 2400 px wide with the white background mathematically removed to alpha, so highlights can be drawn underneath the text. Cached pages are PNGs plus a `bincode` metadata blob under `dirs::cache_dir()/nixobdo-pdf-cache/<hash>/`.
5. **Coordinates are mixed but deliberate.** Character/link bounds are stored normalised `(0..1)` with a top-left origin; page sizes are in PDF points; drawing uses `egui::Rect` with a top-left origin; PDF writes convert back to points with bottom-left origin. Rotation transforms are explicit.
6. **State persistence is JSON, not a database.** Settings and AI chat history live in `dirs::config_dir()/nixobdo-pdf/settings.json`. The LLM API key is stored in plaintext in that file.
7. **`main` is the stable branch, `dev` is the nightly branch.** Stable releases are driven by `scripts/release_plan.py` and version tags; pushes to `dev` publish a rolling `dev-release` prerelease. The website deploys from `main` only.
8. **There are currently no Rust unit/integration tests.** `cargo test` succeeds with zero tests. CI gates are `cargo fmt --check` (blocking), `cargo build` on three OSes (blocking), `cargo test` (blocking but empty), clippy (advisory), and `cargo audit` (advisory).
9. **The `scratch/` directory contains obsolete copies of the release workflows.** Do not use them as the source of truth; `.github/workflows/` is current.
10. **Only the Windows build has a real updater path.** The update download URL is hard-coded to `nixobdo-pdfSetup.exe`, even though the check/download code is compiled on every platform.

---

## Repository entry points

- Application entry: `src/main.rs` → `app::NixobdoPdfApp` → `eframe::App::ui` in `src/app/eframe_app.rs`.
- Worker entry: `src/worker/mod.rs::spawn_worker_thread`.
- PDF engine/cache: `src/document.rs`.
- Main feature logic: `src/app/state.rs`, `src/app/messages.rs`, `src/ui/`.
- Release automation: `scripts/release_plan.py`, `scripts/bump_version.py`, `.github/workflows/build.yml`, `.github/workflows/nightly.yml`.

## Keeping this pack accurate

When behaviour changes, update the relevant file here **in the same PR**. Prefer updating function names, module paths, and contracts over copying large code blocks; line numbers drift quickly. If `Cargo.toml`, `Cargo.lock`, `setup.iss`, or the release scripts change, re-check [12-packaging-and-release.md](12-packaging-and-release.md) and [11-ci-cd.md](11-ci-cd.md).
