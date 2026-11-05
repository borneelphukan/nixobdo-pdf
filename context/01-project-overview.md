# 01 — Project Overview

## Product

**nixobdo-pdf** is a free, open-source, distraction-free PDF viewer. The goal is the focused reading experience of the classic Adobe Reader without the surrounding upsells, sidebars, or account prompts. It is a native desktop application written in Rust, not a web wrapper.

- **Repository:** `https://github.com/borneelphukan/nixobdo-pdf`
- **Author/maintainer:** Borneel B. Phukan
- **License:** MIT
- **Current version:** `0.1.38`
- **Primary platform:** Windows (installer + auto-update)
- **Other platforms:** macOS (`.app`/DMG), Linux (`.deb` + portable tarball)

## What the application does

| Capability | Summary | Main implementation |
|---|---|---|
| Multi-tab viewing | Open multiple PDFs; duplicate paths reuse the existing tab | `src/app/state.rs`, `src/ui/tabs.rs` |
| Layouts | Continuous scroll, single page, two page | `src/ui/viewer/central_panel.rs`, `src/document.rs` |
| Zoom | Toolbar, `Ctrl/Cmd` + wheel, trackpad pinch, 100 %–1000 % | `src/ui/toolbar.rs`, `src/ui/viewer/mod.rs`, `src/ui/viewer/central_panel.rs` |
| Text selection & copy | Drag selection across pages, `Ctrl/Cmd+C`, select all, context menu | `src/document.rs`, `src/app/state.rs`, `src/ui/viewer/central_panel.rs` |
| Search | `Ctrl/Cmd+F`, character-level search, match count, jump to matches | `src/ui/toolbar.rs`, `src/ui/viewer/central_panel.rs` |
| Page rotation | Per-page preview rotation; whole-document rotation with persistent save | `src/ui/toolbar.rs`, `src/ui/menu_bar.rs`, `src/worker/mod.rs` |
| Annotations | Highlight, underline, strikethrough, redact, text; undo/redo; permanent PDF save | `src/document.rs`, `src/ui/toolbar.rs`, `src/ui/viewer/central_panel.rs`, `src/worker/mod.rs` |
| Signatures | Load a PNG/JPG signature, drag/resize, embed as PDF image object | `src/ui/menu_bar.rs`, `src/ui/viewer/central_panel.rs`, `src/worker/mod.rs` |
| Export/conversion | PDF → DOCX, DOC (RTF), PNG/ZIP, JPEG/ZIP | `src/worker/export.rs`, `src/ui/dialogs/export.rs` |
| Password-protected PDFs | Prompt for a password; retry on incorrect password; bypass cache | `src/ui/dialogs/password_prompt.rs`, `src/document.rs` |
| Hyperlinks | Open external URLs; jump to internal page destinations | `src/document.rs`, `src/ui/viewer/central_panel.rs` |
| AI summary/chat | Optional OpenAI-compatible LLM: summarise a document or selection, chat with a session | `src/worker/mod.rs`, `src/ui/chatbot.rs`, `src/ui/dialogs/*` |
| Auto-update | Check GitHub releases; download and launch the Windows installer | `src/worker/mod.rs`, `src/ui/dialogs/update.rs` |
| Recent files | Last five successfully loaded paths | `src/app/state.rs` |
| Fullscreen | Hides chrome, offers an exit button and Esc hint | `src/app/eframe_app.rs`, `src/ui/viewer/central_panel.rs` |
| Cache | 7-day, 400 MB page/thumbnail/metadata cache | `src/document.rs` |

## Deliberate boundaries

- **No OCR.** Text comes from PDFium's text layer. Scanned image-only PDFs can be viewed/exported as images, but not searched or text-exported.
- **No cloud sync, accounts, telemetry, or analytics.** The only outbound traffic is GitHub Releases (update checks), explicit hyperlinks, and user-configured LLM endpoints.
- **No PDF form editor.** The Text annotation tool can place text visually, but this is not a full AcroForm implementation.
- **No real-time collaborative editing.** Saves rewrite the original file.
- **No platform-specific native UI.** All UI is egui; native dialogs come from `rfd`.
- **No test suite yet.** Verification is currently build + manual QA.

## Technology stack

From `Cargo.toml` (edition 2024):

| Crate | Version | Role |
|---|---|---|
| `eframe` | `0.37.0` | Native window, event loop, egui integration |
| `egui` | `0.37.0` | Immediate-mode GUI |
| `egui_extras` | `0.37.0` (`svg`, `image`) | SVG/image loaders |
| `pdfium-render` | `0.8.37` (`image`) | PDFium bindings, rendering, text/links, object editing |
| `rfd` | `0.17.2` | Native open/save/message dialogs |
| `webbrowser` | `1.2.4` | Open external links |
| `docx-rs` | `0.4.22` | DOCX generation |
| `zip` | `9.0.4` | ZIP archive for image export |
| `ureq` | `3.4.0` | Blocking HTTP for updates and LLM calls |
| `dirs` | `7.0.0` | Platform config/cache/download directories |
| `serde` / `serde_json` | `1.0.229` / `1.0.151` | Settings, chat history, LLM payloads |
| `bincode` | `1.3.3` | Compact PDF page metadata cache |
| `chrono` | `0.4.45` | Chat session names |
| `image` | `0.25.5` | Decode/resize/encode page images; icon loading |
| `winres` (build) | `0.1.12` | Windows `.exe` icon/manifest |

Notably **not** present: `rayon`, `tokio`, any async runtime, any logging crate, any analytics, any test framework.

## Runtime prerequisites

The binary dynamically binds PDFium. At least one of the following must exist:

- next to the executable: `libpdfium.dylib`, `pdfium.dll`, or `libpdfium.so`
- `./lib/libpdfium.dylib`, `./lib/pdfium.dll`, `./lib/libpdfium.so`
- the bare library name on the loader path
- a system-installed PDFium

The repository currently ships `lib/libpdfium.dylib` and `lib/pdfium.dll`; Linux builds download `libpdfium.so` in CI. The architecture must match the host (especially Apple Silicon vs Intel macs).

## High-level runtime shape

```text
main()
  └─ eframe window
       └─ NixobdoPdfApp (all UI state)
            ├─ task_tx ──────────────► worker thread (owns Pdfium)
            │                           ├─ Load / cache / render
            │                           ├─ Export (DOCX, RTF, ZIP)
            │                           └─ SaveSignature / SaveRotation / SaveAnnotations
            │                                └─ network sub-threads (update, LLM)
            └─ msg_rx ◄────────────── worker messages
                 └─ egui textures created on the UI thread only
```

## Where the product is heading

The recent changelog and code show active work in:

- AI/LLM integration (local LLM presets, API-key mode, chat sessions, selection explanation)
- release engineering (version-tag-driven stable releases, macOS artifacts, advisory CI gates)
- annotation reliability on Windows (panic recovery, temp-file retries)
- platform packaging (Debian package, portable tarball, macOS app bundle)

For exact historical detail, see `CHANGELOG.md`. Do not infer unreleased intent from `scratch/`; it contains superseded workflow drafts.