# 04 — Data Model and Contracts

This file is the quick reference for the Rust types that cross module boundaries. Field names are exact.

## 1. `NixobdoPdfApp` (`src/app/mod.rs`)

Public fields, grouped by purpose.

### Tabs and navigation

```rust
pub struct NixobdoPdfApp {
    pub has_pdfium_bindings: bool,
    pub tabs: Vec<PdfDocumentState>,
    pub active_tab_index: Option<usize>,
    pub sidebar_open: bool,
    pub pointer_mode: PointerMode,          // Select | Pan
    pub pending_scroll_delta: egui::Vec2,   // used by the floating utility bar
    ...
}
```

### Search and selection

```rust
pub search_query: String,
pub search_active_match: usize,             // index into the per-page match list
pub selection_start: Option<(usize, usize)>, // (page_index, char_index)
pub selection_end: Option<(usize, usize)>,
pub is_selecting: bool,
```

### Channel plumbing

```rust
pub pdf_task_tx: Sender<PdfWorkerTask>,
pub pdf_msg_tx: Sender<PdfWorkerMessage>,   // used by ad-hoc thumbnail/cache threads
pub pdf_receiver: Receiver<PdfWorkerMessage>,
```

### File/dialog state

```rust
pub recent_files: Vec<PathBuf>,             // max 5
pub rename_window_open: bool,
pub rename_buffer: String,
pub focus_rename_input: bool,
pub export_window_open: bool,
pub export_format: ExportFormat,            // Doc | Docx | Png | Jpeg
pub export_location: Option<PathBuf>,
```

### Feedback and updates

```rust
pub toast_message: Option<String>,
pub toast_success: bool,
pub toast_timer: f64                     // absolute egui time
pub export_progress: Option<f32>,        // 0.0..=1.0
pub export_cancel_flag: Arc<AtomicBool>,
pub update_state: UpdateState,           // None | Checking | Prompt(String) | Downloading(f32)
pub has_checked_for_updates: bool,
```

### Signature

```rust
pub signature_image_path: Option<PathBuf>,
pub signature_texture: Option<egui::TextureHandle>,
pub is_placing_signature: bool,
pub signature_position: Option<(f32, f32)>,  // normalised page top-left origin
pub signature_active_page: Option<usize>,
pub signature_scale: f32,                    // clamped 0.2..=3.0 during resize
pub is_saving_signature: bool,
```

### Rotation

```rust
pub is_rotating_document: bool,  // whole-document edit mode; gates the Save overlay
pub is_saving_rotation: bool,
pub pending_rotation: i32,       // accumulated -90/+90 delta for whole-document save
```

### Annotations

```rust
pub is_annotation_mode: bool,
pub active_annotation_tool: Option<AnnotationTool>,
pub pending_annotations: Vec<AnnotationAction>,
pub redo_annotations: Vec<AnnotationAction>,
pub active_text_annotation_index: Option<usize>,
pub annotation_color: egui::Color32,          // default yellow
pub is_saving_annotations: bool,
pub text_annotation_size: f32,                // default 12.0
pub text_annotation_bold: bool,
pub text_annotation_italic: bool,
pub text_annotation_underline: bool,
pub text_annotation_color: egui::Color32,     // default black
pub is_custom_text_color_open: bool,
pub custom_text_color_temp: egui::Color32,
```

### AI/LLM

```rust
pub llm_endpoint_url: String,        // default http://localhost:1234/v1
pub llm_model: String,
pub llm_api_key: String,
pub llm_configured: bool,
pub show_llm_settings: bool,
pub llm_settings_tab_index: usize,   // 0 = Local LLM, 1 = API Key
pub llm_selected_preset: usize,
pub llm_is_custom_model: bool,
pub llm_custom_model: String,
pub ai_summary_open: bool,
pub ai_summary_text: String,          // currently displayed (typing animation)
pub ai_summary_loading: bool,
pub ai_summary_error: Option<String>,
pub ai_summary_display_len: usize,
pub ai_summary_start_time: f64,
pub ai_summary_full_text: String,
pub password_prompt: Option<PasswordPromptState>,
pub ai_chatbot_open: bool,
pub ai_chat_sessions: Vec<AiChatSession>,
pub ai_active_session_id: Option<String>,
pub ai_chat_input: String,
pub ai_chat_loading: bool,
pub ai_chat_error: Option<String>,
pub ai_chat_display_len: usize,
pub ai_chat_start_time: f64,
```

### Fullscreen

```rust
pub was_fullscreen: bool,
pub fullscreen_toast_timer: f64,
```

### Enums in `app`

```rust
pub enum UpdateState { None, Checking, Prompt(String), Downloading(f32) }
pub enum PointerMode { Select, Pan }

pub struct ChatMessage {
    pub role: String,      // "system" | "user" | "assistant"
    pub content: String,
}

pub struct AiChatSession {
    pub id: String,        // epoch millis as string
    pub name: String,      // e.g. "Summary - 14:30"
    pub messages: Vec<ChatMessage>,
}
```

`ChatMessage` and `AiChatSession` derive `Serialize`, `Deserialize`, `Clone`, `Debug`.

## 2. `PdfDocumentState` (`src/document.rs`)

```rust
pub struct PdfDocumentState {
    pub file_name: String,
    pub path: PathBuf,
    pub pages: Vec<Option<egui::TextureHandle>>,
    pub thumbnails: Vec<Option<egui::TextureHandle>>,
    pub page_texts: Vec<String>,              // collected but not currently displayed
    pub page_chars: Vec<Vec<PdfCharInfo>>,
    pub page_links: Vec<Vec<PdfLinkInfo>>,
    pub page_sizes: Vec<egui::Vec2>,          // PDF points (unrotated)
    pub page_rotations: Vec<i32>,             // degrees, multiples of 90, normalised
    pub zoom: f32,                            // percentage offset: displayed = 100 + zoom
    pub selected_page: usize,                 // zero-based
    pub scroll_to_page: Option<usize>,        // one-shot scroll request
    pub layout_mode: PageLayoutMode,
    pub error: Option<String>,                // currently never populated by messages.rs
    pub is_loading: bool,
    pub last_page_change_time: f64,           // throttle for scroll/wheel page changes
    pub password: Option<String>,
    pub use_cache: bool,                      // false for password-protected PDFs
    pub pages_loading: HashSet<usize>,        // cache-page decoder threads in flight
    pub thumbnails_loading: HashSet<usize>,   // thumbnail decoder threads in flight
}
```

Constructor: `PdfDocumentState::empty(path)` — filename from the path, empty vectors, `zoom = 0.0`, `is_loading = true`, `use_cache = true`.

Cache directory: `cache_dir()` hashes path + file length + mtime seconds and returns `dirs::cache_dir()/nixobdo-pdf-cache/<hex>`.

### Supporting types

```rust
pub struct PdfCharInfo {
    pub c: char,
    pub left: f32,   // normalised 0..1 from left
    pub right: f32,
    pub top: f32,    // normalised 0..1 from top (PDF y inverted)
    pub bottom: f32,
}

pub enum PdfLinkTarget {
    Url(String),
    Page(usize),
}

pub struct PdfLinkInfo {
    pub left: f32, pub right: pub top: f32, pub bottom: f32,
    pub target: PdfLinkTarget,
}

pub enum PageLayoutMode {
    ContinuousScroll, // default
    SinglePage,
    TwoPage,
}
```

`PdfCharInfo` and `PdfLinkInfo` derive `Serialize`, `Deserialize`, `Clone` because they are cached with `bincode`.

## 3. Annotation types

```rust
pub enum AnnotationTool {
    Highlight,
    Underline,
    Strikethrough,
    Redact,
    Text,
}

pub struct AnnotationAction {
    pub tool: AnnotationTool,
    pub page_index: usize,
    pub rects: Vec<egui::Rect>,      // normalised 0..1 page rects
    pub position: Option<egui::Pos2>, // normalised text top-left
    pub text: Option<String>,
    pub color: egui::Color32,
    pub scale: Option<f32>,          // text font size in PDF points (UI calls it px)
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}
```

`AnnotationAction` is `Clone + Debug` but **not** serialized; pending annotations live only in memory until saved into the PDF.

## 4. Worker tasks (`src/worker/mod.rs`)

```rust
pub enum PdfWorkerTask {
    Load {
        path: PathBuf,
        password: Option<String>,
        ctx: egui::Context,
    },
    Export {
        path: PathBuf,
        password: Option<String>,
        out_path: PathBuf,
        format: ExportFormat,
        retain_layout: bool,
        include_images: bool,
        ctx: egui::Context,
        cancel_flag: Arc<AtomicBool>,
    },
    CheckUpdate {
        is_manual: bool,
        ctx: egui::Context,
    },
    DownloadUpdate {
        version: String,
        ctx: egui::Context,
    },
    SaveSignature {
        path: PathBuf,
        password: Option<String>,
        page_index: usize,
        image_path: PathBuf,
        position: (f32, f32),   // normalised page centre
        scale: f32,
        ctx: egui::Context,
    },
    SaveRotation {
        path: PathBuf,
        password: Option<String>,
        rotation: i32,          // delta in degrees
        ctx: egui::Context,
    },
    SaveAnnotations {
        path: PathBuf,
        password: Option<String>,
        annotations: Vec<AnnotationAction>,
        ctx: egui::Context,
    },
    AiSummarize {
        is_chatbot: bool,
        messages: Vec<ChatMessage>,
        endpoint_url: String,
        model: String,
        api_key: String,
        ctx: egui::Context,
    },
}
```

## 5. Worker messages (`src/document.rs`)

```rust
pub enum PdfWorkerMessage {
    DocumentInfo {
        path: PathBuf,
        file_name: String,
        page_count: usize,
        error: Option<String>,    // "PasswordRequired" | "IncorrectPassword" | descriptive
        password: Option<String>,
        use_cache: bool,
    },
    PageData {
        path: PathBuf,
        index: usize,
        image: Option<egui::ColorImage>,
        thumbnail_image: Option<egui::ColorImage>,
        text: String,
        chars: Vec<PdfCharInfo>,
        links: Vec<PdfLinkInfo>,
        page_size: egui::Vec2,    // PDF points
    },
    PageDataLoaded {
        path: PathBuf,
        index: usize,
        image: egui::ColorImage,  // decoded lazily from cache PNG
    },
    ThumbnailDataLoaded {
        path: PathBuf,
        index: usize,
        image: egui::ColorImage,  // decoded lazily from cache PNG
    },
    Finished {
        path: PathBuf,            // currently ignored by the UI
    },
    ExportProgress {
        progress: f32,            // 0.0..=1.0
    },
    ExportComplete {
        success: bool,
        message: String,
    },
    UpdateCheckResult(bool, Option<String>, bool), // available, version, is_manual
    UpdateDownloadProgress(f32),
    UpdateDownloadComplete(Result<String, String>), // Ok(installer path) / Err(reason)
    SignatureSaved { path: PathBuf },
    RotationSaved { path: PathBuf },
    AnnotationsSaved { path: PathBuf },
    AiSummaryResult {
        is_chatbot: bool,
        success: bool,
        text: String,
        error: Option<String>,
    },
}
```

## 6. Export types (`src/worker/export.rs`)

```rust
pub enum ExportFormat { Doc, Docx, Png, Jpeg }

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        // "doc" | "docx" | "png" | "jpeg"
    }
}
```

`ExportFormat` is re-exported from `worker/mod.rs`.

## 7. Password prompt state

```rust
pub struct PasswordPromptState {
    pub path: PathBuf,
    pub file_name: String,
    pub is_incorrect: bool,
    pub password_input: String,
    pub focus_input: bool,
}
```

## 8. State-transition invariants

1. `active_tab_index` is either `None` or a valid index into `tabs`.
2. A tab's vectors (`pages`, `thumbnails`, `page_texts`, `page_chars`, `page_links`, `page_sizes`, `page_rotations`) are all sized to `page_count` after a successful `DocumentInfo`.
3. `is_loading = true` from tab creation until successful `DocumentInfo`, a password prompt is open, or a load error closes the tab.
4. `use_cache = false` for password-protected PDFs, so all pages are rendered and `manage_textures` returns early.
5. `selection_start`/`selection_end` are `(page_index, char_index)` pairs, not screen coordinates. They may span pages.
6. `pending_annotations` and `redo_annotations` are a two-stack undo model. Saving clears both and exits annotation mode.
7. `is_placing_signature` requires an active tab and a texture; `signature_active_page` pins placement to one page.
8. `is_rotating_document` is set by **Edit → Rotate PDF**, not by the toolbar rotation buttons (toolbar rotation is preview-only).
9. `export_cancel_flag` is reset before each export; a cancel sets it to `true`, and the worker checks once per page.
10. `update_state == Downloading(_)` means a sub-thread is downloading; cancel only resets UI state, it does not abort the HTTP transfer.
11. `ai_chat_loading` / `ai_summary_loading` are cleared by the corresponding `AiSummaryResult`.
12. Chat sessions and the active session id are persisted whenever they change; summary-viewport text is not persisted.

## 9. Settings schema

Defined inline in both `NixobdoPdfApp::default` and `save_settings`:

```rust
struct AppSettings {
    recent_files: Vec<PathBuf>,
    llm_api_key: String,
    llm_model: String,
    llm_endpoint_url: String,
    llm_configured: bool,           // serde(default)
    ai_chat_sessions: Vec<AiChatSession>, // serde(default)
    ai_active_session_id: Option<String>, // serde(default)
}
```

`default()` fills a missing endpoint with `http://localhost:1234/v1`. `save_settings()` writes pretty JSON under `dirs::config_dir()/nixobdo-pdf/settings.json`.

## 10. Cache metadata schema

`metadata.bin` is a `bincode` encoding of:

```rust
(
    usize,                         // page_count
    Vec<String>,                   // page_texts
    Vec<Vec<PdfCharInfo>>,         // page_chars
    Vec<Vec<PdfLinkInfo>>,         // page_links
    Vec<(f32, f32)>,               // page_sizes as (width, height)
)
```

There is no version field. Changing any cached type's layout requires invalidating or versioning the cache key.