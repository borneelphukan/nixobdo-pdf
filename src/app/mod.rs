pub mod eframe_app;
pub mod messages;
pub mod state;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use pdfium_render::prelude::Pdfium;

use crate::document::{PdfDocumentState, PdfWorkerMessage};
use crate::worker::{ExportFormat, PdfWorkerTask};

#[derive(PartialEq, Clone)]
pub enum UpdateState {
    None,
    Checking,
    Prompt(String),
    Downloading(f32),
}

pub struct NixobdoPdfApp {
    pub has_pdfium_bindings: bool,
    pub tabs: Vec<PdfDocumentState>,
    pub active_tab_index: Option<usize>,
    pub search_query: String,
    pub search_active_match: usize,
    pub sidebar_open: bool,
    pub selection_start: Option<(usize, usize)>,
    pub selection_end: Option<(usize, usize)>,
    pub is_selecting: bool,
    
    // Background Loading
    pub pdf_task_tx: Sender<PdfWorkerTask>,
    pub pdf_receiver: Receiver<PdfWorkerMessage>,
    
    // File Menu features
    pub recent_files: Vec<PathBuf>,
    pub rename_window_open: bool,
    pub rename_buffer: String,
    pub focus_rename_input: bool,
    pub export_window_open: bool,
    pub export_name: String,
    pub export_format: ExportFormat,
    pub export_location: Option<PathBuf>,
    
    // Toast notification
    pub toast_message: Option<String>,
    pub toast_success: bool,
    pub toast_timer: f64,
    
    // Export Progress
    pub export_progress: Option<f32>,
    pub export_cancel_flag: Arc<AtomicBool>,
    
    // Updates
    pub update_state: UpdateState,
    
    // About Window
    pub about_window_open: bool,
    
    // Splash screen
    pub startup_time: std::time::Instant,
    
    // Auto update
    pub has_checked_for_updates: bool,
    
    // Signature feature
    pub signature_image_path: Option<PathBuf>,
    pub signature_texture: Option<egui::TextureHandle>,
    pub is_placing_signature: bool,
    pub signature_position: Option<(f32, f32)>, // Normalized (x, y) relative to page top-left
    pub signature_active_page: Option<usize>,
    pub signature_scale: f32,
    pub is_saving_signature: bool,
    
    // Rotation feature
    pub is_rotating_document: bool,
    pub is_saving_rotation: bool,
    pub pending_rotation: i32,
    
    // Annotation feature
    pub is_annotation_mode: bool,
    pub active_annotation_tool: Option<crate::document::AnnotationTool>,
    pub pending_annotations: Vec<crate::document::AnnotationAction>,
    pub redo_annotations: Vec<crate::document::AnnotationAction>,
    pub annotation_color: egui::Color32,
    pub is_saving_annotations: bool,
    pub text_annotation_size: f32,
    pub text_annotation_bold: bool,
    pub text_annotation_italic: bool,
    pub text_annotation_underline: bool,
    pub text_annotation_color: egui::Color32,
    pub is_custom_text_color_open: bool,
    pub custom_text_color_temp: egui::Color32,
}

impl Default for NixobdoPdfApp {
    fn default() -> Self {
        let exe_path = std::env::current_exe().ok().unwrap_or_default();
        let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new(""));
        
        let has_pdfium_bindings = Pdfium::bind_to_library(exe_dir.join("libpdfium.dylib").to_str().unwrap_or_default())
            .or_else(|_| Pdfium::bind_to_library(exe_dir.join("pdfium.dll").to_str().unwrap_or_default()))
            .or_else(|_| Pdfium::bind_to_library("./lib/libpdfium.dylib"))
            .or_else(|_| Pdfium::bind_to_library("libpdfium.dylib"))
            .or_else(|_| Pdfium::bind_to_library("./lib/pdfium.dll"))
            .or_else(|_| Pdfium::bind_to_library("pdfium.dll"))
            .or_else(|_| Pdfium::bind_to_system_library())
            .is_ok();

        let (task_tx, task_rx) = channel::<PdfWorkerTask>();
        let (msg_tx, msg_rx) = channel::<PdfWorkerMessage>();

        let msg_tx_clone = msg_tx.clone();
        
        crate::worker::spawn_worker_thread(task_rx, msg_tx_clone);
        
        // Clean up old or oversized cache entries in the background
        std::thread::spawn(|| {
            crate::document::clean_cache();
        });

        let recent_files = Self::load_recent_files();

        Self {
            has_pdfium_bindings,
            tabs: Vec::new(),
            active_tab_index: None,
            search_query: String::new(),
            search_active_match: 0,
            sidebar_open: true,
            selection_start: None,
            selection_end: None,
            is_selecting: false,
            pdf_task_tx: task_tx,
            pdf_receiver: msg_rx,
            recent_files,
            rename_window_open: false,
            rename_buffer: String::new(),
            focus_rename_input: false,
            export_window_open: false,
            export_name: String::new(),
            export_format: ExportFormat::Docx,
            export_location: None,
            toast_message: None,
            toast_success: false,
            toast_timer: 0.0,
            export_progress: None,
            export_cancel_flag: Arc::new(AtomicBool::new(false)),
            update_state: UpdateState::None,
            about_window_open: false,
            startup_time: std::time::Instant::now(),
            has_checked_for_updates: false,
            signature_image_path: None,
            signature_texture: None,
            is_placing_signature: false,
            signature_position: None,
            signature_active_page: None,
            signature_scale: 1.0,
            is_saving_signature: false,
            is_rotating_document: false,
            is_saving_rotation: false,
            pending_rotation: 0,
            is_annotation_mode: false,
            active_annotation_tool: None,
            pending_annotations: Vec::new(),
            redo_annotations: Vec::new(),
            annotation_color: egui::Color32::from_rgb(0, 0, 0), // Black by default
            is_saving_annotations: false,
            text_annotation_size: 12.0,
            text_annotation_bold: false,
            text_annotation_italic: false,
            text_annotation_underline: false,
            text_annotation_color: egui::Color32::BLACK,
            is_custom_text_color_open: false,
            custom_text_color_temp: egui::Color32::BLACK,
        }
    }
}
