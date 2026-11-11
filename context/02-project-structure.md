# 02 — Project Structure

## Top level

```text
nixobdo-pdf/
├── .github/
│   ├── CODEOWNERS                  # Everything owned by @borneelphukan
│   ├── dependabot.yml              # Weekly cargo + actions updates into dev
│   └── workflows/
│       ├── audit.yml               # Weekly RustSec audit + dependency changes
│       ├── ci.yml                  # fmt, clippy, tests, packaging-asset checks
│       ├── release.yml             # Packages, apt repo, Pages, tagged releases
│       └── website.yml             # PR lint/build for website-only changes
├── assets/
│   ├── cover.png                   # README banner
│   ├── logo.png                    # App icon (also converted to ICO in CI)
│   ├── Screenshot-1.png            # README screenshot
│   ├── Screenshot-2.png            # README screenshot
│   └── icons/
│       ├── logo.svg                # Splash/about/website source logo
│       ├── fullscreen.svg
│       ├── exit_fullscreen.svg
│       ├── toggle_sidebar.svg
│       ├── highlight.svg
│       ├── underline.svg
│       ├── strikethrough.svg
│       ├── redact.svg
│       ├── rotate_left.svg
│       ├── rotate_right.svg
│       ├── github.svg
│       └── website.svg
├── .cargo/
│   └── audit.toml                  # Triaged cargo-audit ignores with reasons
├── installer/
│   └── nixobdo-pdf.iss             # Parameterised Inno Setup installer
├── lib/
│   ├── libpdfium.dylib             # macOS PDFium binary shipped in repo
│   └── pdfium.dll                  # Windows PDFium binary shipped in repo
├── packaging/
│   ├── apt/
│   │   ├── build-repo.py           # Builds the static, optionally signed apt repo
│   │   └── install.sh              # curl | sudo bash repository installer
│   ├── linux/
│   │   ├── build-deb.sh            # Builds the .deb staging tree
│   │   ├── build-tarball.sh        # Portable tarball
│   │   ├── build-appimage.sh       # AppImage
│   │   ├── install.sh              # Portable tarball installer
│   │   ├── uninstall.sh            # Portable uninstaller
│   │   └── nixobdo-pdf.desktop     # XDG desktop entry
│   ├── rpm/
│   │   ├── build-rpm.sh            # RPM builder
│   │   └── nixobdo-pdf.spec.in     # RPM spec template
│   └── macos/
│       └── build-app.sh            # .app bundle + Info.plist + icns + DMG
├── scripts/
│   └── bump_version.py             # Updates Cargo.toml, Cargo.lock, CHANGELOG
├── src/                            # Rust application (see below)
├── website/                        # React/Vite documentation + marketing site
├── Cargo.toml / Cargo.lock         # Rust manifest / locked dependency graph
├── build.rs                        # Windows icon resources + GIT_BRANCH env
├── rust-toolchain.toml             # pinned Rust 1.99.1 + clippy + rustfmt
├── AGENTS.md                       # OLD agent notes; prefer context/
├── README.md                       # User-facing quick start
├── CHANGELOG.md                    # Release history
└── LICENSE                         # MIT
```

### `src/` module map

```text
src/
├── main.rs                         # Binary entry, window/theme setup, CLI file arg
├── document.rs                     # PdfDocumentState, worker messages, annotation types,
│                                   # layout enum, text/link extraction, cache, PDFium render
├── app/
│   ├── mod.rs                      # NixobdoPdfApp struct + Default construction
│   ├── eframe_app.rs               # eframe::App::ui frame orchestration
│   ├── state.rs                    # Tab lifecycle, copy/select text, settings, texture lazy-load
│   └── messages.rs                 # Consumer for every PdfWorkerMessage
├── worker/
│   ├── mod.rs                      # Task enum, worker thread, PDF saves, update, LLM
│   └── export.rs                   # DOCX / DOC(RTF) / PNG / JPEG export
└── ui/
    ├── mod.rs                      # Module exports
    ├── menu_bar.rs                 # File/Edit/View/AI/Help menus
    ├── tabs.rs                     # Tab bar and close buttons
    ├── toolbar.rs                  # Main toolbar + annotation toolbar + search
    ├── chatbot.rs                  # Right-hand AI chat panel
    ├── splash.rs                   # 2.5-second startup splash
    ├── toast.rs                    # Bottom-right transient notifications
    ├── viewer/
    │   ├── mod.rs                  # Viewer composition, zoom gestures, copy shortcut
    │   ├── sidebar.rs              # Thumbnail sidebar + lazy thumbnail loading
    │   ├── separator.rs            # Draggable 1-px sidebar separator
    │   └── central_panel.rs        # Page layout, mesh rendering, selection, annotation
    │                               # previews, links, signature placement, utility bar
    └── dialogs/
        ├── mod.rs                  # Dialog dispatch order
        ├── about.rs                # About viewport
        ├── ai_summary.rs           # Floating AI summary viewport (selection explain)
        ├── custom_color.rs         # Custom HSV colour picker
        ├── export.rs               # Export options window
        ├── export_progress.rs      # Export progress + cancel
        ├── llm_settings.rs         # Local/API-key LLM config viewport
        ├── password_prompt.rs      # Password viewport
        ├── rename.rs               # Inline rename window
        └── update.rs               # Checking/downloading/prompt update UI
```

## Ownership by concern

| Concern | Primary files |
|---|---|
| App-level state and defaults | `src/app/mod.rs` |
| Frame order and global shortcuts | `src/app/eframe_app.rs` |
| Tab open/close/reload, copy/select, recent files, settings persistence | `src/app/state.rs` |
| Worker-message side effects | `src/app/messages.rs` |
| PDF state, cache, render, text/link extraction | `src/document.rs` |
| Worker task contracts and save/network logic | `src/worker/mod.rs` |
| Export implementations | `src/worker/export.rs` |
| Menu actions | `src/ui/menu_bar.rs` |
| Tabs | `src/ui/tabs.rs` |
| Zoom/navigation/rotation/search/annotation toolbar | `src/ui/toolbar.rs` |
| Page drawing and per-page interaction | `src/ui/viewer/central_panel.rs` |
| Thumbnail navigation | `src/ui/viewer/sidebar.rs` |
| Sidebar visibility gesture | `src/ui/viewer/separator.rs` |
| AI chat panel | `src/ui/chatbot.rs` |
| Modal/native auxiliary windows | `src/ui/dialogs/` |
| Release version/changelog edits | `scripts/bump_version.py` |
| Windows installer definition | `installer/nixobdo-pdf.iss` |
| Linux packaging | `packaging/linux/` |
| RPM packaging | `packaging/rpm/` |
| apt repository | `packaging/apt/` |
| macOS packaging | `packaging/macos/build-app.sh` |
| CI/CD workflows | `.github/workflows/` |
| Website | `website/src/` |

## Module dependency direction

```text
main
 └─ app
     ├─ document  (types + PDF/cache)
     ├─ worker    (task enum + execution; re-exports ExportFormat)
     └─ ui
         ├─ (calls NixobdoPdfApp methods/fields, document types, worker tasks)
         └─ dialogs (same)

document ──► worker sends PdfWorkerMessage
worker   ──► document types + app::ChatMessage
```

There is a deliberate two-way type dependency between `app` and `worker` (`worker` uses `crate::app::ChatMessage`), so the crate is not layered strictly. In practice:

- `document.rs` defines the data/message vocabulary and PDF rendering.
- `worker/mod.rs` defines the task vocabulary and performs side effects.
- `app/` owns state and reacts to messages.
- `ui/` is presentation plus user-intent generation; it mutates `NixobdoPdfApp` directly.

## File-size hotspots

If you are changing behaviour, these are the files most likely to need edits:

| File | Approx. lines | Why it matters |
|---|---:|---|
| `src/ui/viewer/central_panel.rs` | ~1,170 | Largest file; all page drawing and interaction |
| `src/worker/mod.rs` | ~910 | All background side effects |
| `src/document.rs` | ~640 | PDF render/cache and message types |
| `src/ui/toolbar.rs` | ~600 | Main toolbar, search, annotation toolbar |
| `src/worker/export.rs` | ~510 | All export formats |
| `src/ui/menu_bar.rs` | ~390 | All menu actions |

## Notes for agents

- `src/ui/mod.rs` only declares modules; each module adds `impl NixobdoPdfApp` methods. There are no separate widget structs for most UI.
- `src/ui/dialogs/mod.rs` fixes the dialog dispatch order: rename → export → about → update → export progress → custom colour → LLM settings → AI summary → password.
- `assets/logo.ico` is **not** committed (it is ignored). The Windows release job generates it with ImageMagick before compilation; `build.rs` safely skips resources if absent.
- `website/.vite/` is a local Vite cache, not application source.
- There is no `tests/` directory and no `#[test]` in `src/`.
