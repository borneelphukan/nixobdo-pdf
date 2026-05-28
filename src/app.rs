use eframe::egui;
use pdfium_render::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::fs;

use crate::document::{PdfDocumentState, PdfWorkerMessage};

const RECENT_FILES_PATH: &str = ".recent_files.json";

#[derive(PartialEq, Clone)]
pub enum UpdateState {
    None,
    Checking,
    Prompt,
    Downloading(f32), // current progress
}

use crate::worker::{ExportFormat, PdfWorkerTask};

pub struct PdfViewerApp {
    pub has_pdfium_bindings: bool,
    pub tabs: Vec<PdfDocumentState>,
    pub active_tab_index: Option<usize>,
    pub search_query: String,
    pub sidebar_open: bool,
    pub selection_start: Option<(usize, usize)>, // (page_idx, char_idx)
    pub selection_end: Option<(usize, usize)>,   // (page_idx, char_idx)
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
    pub export_settings_open: bool,
    pub export_layout_retain_page: bool,
    pub export_include_images: bool,
    
    // Toast notification
    pub toast_message: Option<String>,
    pub toast_success: bool,
    pub toast_timer: f64,
    
    // Export Progress
    pub export_progress: Option<f32>,
    pub export_cancel_flag: Arc<AtomicBool>,
    
    // Updates
    pub update_state: UpdateState,
}

impl Default for PdfViewerApp {
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

        let recent_files = Self::load_recent_files();

        Self {
            has_pdfium_bindings,
            tabs: Vec::new(),
            active_tab_index: None,
            search_query: String::new(),
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
            export_settings_open: false,
            export_layout_retain_page: true,
            export_include_images: true,
            toast_message: None,
            toast_success: false,
            toast_timer: 0.0,
            export_progress: None,
            export_cancel_flag: Arc::new(AtomicBool::new(false)),
            update_state: UpdateState::None,
        }
    }
}

impl eframe::App for PdfViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process background loaded PDFs
        while let Ok(msg) = self.pdf_receiver.try_recv() {
            match msg {
                PdfWorkerMessage::DocumentInfo { path, file_name, page_count, error } => {
                    let mut tab_to_remove = None;
                    for (i, tab) in self.tabs.iter_mut().enumerate() {
                        if tab.path == path {
                            if let Some(err) = error {
                                if err.contains("NotFound") || err.contains("cannot find the path specified") || err.contains("cannot find the file specified") {
                                    rfd::MessageDialog::new()
                                        .set_title("File Not Available")
                                        .set_description("The file you are trying to open is no longer available and cannot be opened.")
                                        .set_level(rfd::MessageLevel::Warning)
                                        .show();
                                    tab_to_remove = Some(i);
                                } else {
                                    tab.error = Some(err);
                                    tab.is_loading = false;
                                }
                            } else {
                                tab.file_name = file_name;
                                tab.pages = vec![None; page_count];
                                tab.page_texts = vec![String::new(); page_count];
                                tab.page_chars = vec![Vec::new(); page_count];
                                tab.page_links = vec![Vec::new(); page_count];
                                tab.is_loading = false; // Turn off main loading, pages will pop in
                            }
                            break;
                        }
                    }
                    if let Some(idx) = tab_to_remove {
                        self.close_tab(idx);
                    }
                }
                PdfWorkerMessage::PageData { path, index, image, text, chars, links } => {
                    for tab in self.tabs.iter_mut() {
                        if tab.path == path {
                            if index < tab.pages.len() {
                                let texture = ctx.load_texture(
                                    format!("doc_{}_page_{}", tab.file_name, index),
                                    image,
                                    Default::default(),
                                );
                                tab.pages[index] = Some(texture);
                                tab.page_texts[index] = text;
                                tab.page_chars[index] = chars;
                                tab.page_links[index] = links;
                            }
                            break;
                        }
                    }
                }
                PdfWorkerMessage::Finished { path: _ } => {}
                PdfWorkerMessage::ExportProgress { progress } => {
                    self.export_progress = Some(progress);
                }
                PdfWorkerMessage::ExportComplete { success, message } => {
                    self.export_progress = None;
                    self.toast_message = Some(message);
                    self.toast_success = success;
                    self.toast_timer = ctx.input(|i| i.time) + 4.0; // show for 4 seconds
                }
                PdfWorkerMessage::UpdateCheckResult(is_available) => {
                    if is_available {
                        self.update_state = UpdateState::Prompt;
                    } else {
                        self.update_state = UpdateState::None;
                        rfd::MessageDialog::new()
                            .set_title("No Update")
                            .set_description("No update available.")
                            .set_level(rfd::MessageLevel::Warning)
                            .show();
                    }
                }
                PdfWorkerMessage::UpdateDownloadProgress(progress) => {
                    self.update_state = UpdateState::Downloading(progress);
                }
                PdfWorkerMessage::UpdateDownloadComplete(result) => {
                    self.update_state = UpdateState::None;
                    match result {
                        Ok(path) => {
                            self.toast_message = Some(format!("Update downloaded to {}", path));
                            self.toast_success = true;
                        }
                        Err(e) => {
                            self.toast_message = Some(format!("Download failed: {}", e));
                            self.toast_success = false;
                        }
                    }
                    self.toast_timer = ctx.input(|i| i.time) + 4.0;
                }
            }
        }

        // Render export progress modal
        if let Some(progress) = self.export_progress {
            let mut is_open = true;
            egui::Window::new("Exporting...")
                .collapsible(false)
                .resizable(false)
                .open(&mut is_open)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(egui::Frame::window(&ctx.style()).inner_margin(16.0).corner_radius(8))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("Exporting document...").size(14.0));
                        ui.add_space(12.0);
                        let progress_bar = egui::ProgressBar::new(progress)
                            .show_percentage();
                        ui.add(progress_bar);
                        ui.add_space(16.0);
                        if ui.button("Cancel").clicked() {
                            self.export_cancel_flag.store(true, Ordering::Relaxed);
                        }
                    });
                });
            if !is_open {
                self.export_cancel_flag.store(true, Ordering::Relaxed);
            }
        }

        // Render toast notification (bottom-right)
        if let Some(msg) = &self.toast_message {
            let now = ctx.input(|i| i.time);
            if now < self.toast_timer {
                let remaining = self.toast_timer - now;
                // Fade out in the last second
                let alpha = if remaining < 1.0 { (remaining * 255.0) as u8 } else { 255 };
                
                let (bg_color, icon) = if self.toast_success {
                    (egui::Color32::from_rgba_unmultiplied(34, 139, 34, alpha), "✔")
                } else {
                    (egui::Color32::from_rgba_unmultiplied(200, 50, 50, alpha), "✖")
                };
                let text_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha);
                
                let toast_msg = msg.clone();
                egui::Area::new(egui::Id::new("export_toast"))
                    .anchor(egui::Align2::RIGHT_BOTTOM, [-16.0, -16.0])
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        egui::Frame::NONE
                            .fill(bg_color)
                            .corner_radius(egui::CornerRadius::same(6))
                            .inner_margin(egui::Margin::symmetric(16, 10))
                            .shadow(egui::epaint::Shadow {
                                offset: [0, 2],
                                blur: 8,
                                spread: 0,
                                color: egui::Color32::from_black_alpha(60),
                            })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(icon).color(text_color).size(16.0));
                                    ui.add_space(6.0);
                                    ui.label(egui::RichText::new(&toast_msg).color(text_color).size(13.0));
                                });
                            });
                    });
                
                ctx.request_repaint(); // Keep repainting for animation
            } else {
                self.toast_message = None;
            }
        }
        
        // Update check and download logic
        match self.update_state {
            UpdateState::None => {}
            UpdateState::Checking => {
                let mut is_open = true;
                egui::Window::new("Checking for Updates")
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut is_open)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .frame(egui::Frame::window(&ctx.style()).inner_margin(16.0).corner_radius(8))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label("Checking for newer version...");
                            ui.add_space(8.0);
                            ui.spinner();
                        });
                    });
                
                if !is_open {
                    self.update_state = UpdateState::None;
                }
            }
            UpdateState::Prompt => {
                let mut is_open = true;
                egui::Window::new("Update Available")
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut is_open)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .frame(egui::Frame::window(&ctx.style()).inner_margin(16.0).corner_radius(8))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("New update available. Download Now?").size(14.0));
                            ui.add_space(16.0);
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Cancel").clicked() {
                                        self.update_state = UpdateState::None;
                                    }
                                    if ui.button("Skip").clicked() {
                                        self.update_state = UpdateState::None;
                                    }
                                    if ui.button("Yes").clicked() {
                                        self.update_state = UpdateState::Downloading(0.0);
                                        let _ = self.pdf_task_tx.send(PdfWorkerTask::DownloadUpdate { ctx: ctx.clone() });
                                    }
                                });
                            });
                        });
                    });
                if !is_open {
                    self.update_state = UpdateState::None;
                }
            }
            UpdateState::Downloading(progress) => {
                let mut is_open = true;
                
                egui::Window::new("Downloading Update")
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut is_open)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .frame(egui::Frame::window(&ctx.style()).inner_margin(16.0).corner_radius(8))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("Downloading update...").size(14.0));
                            ui.add_space(12.0);
                            ui.add(egui::ProgressBar::new(progress).show_percentage());
                            ui.add_space(16.0);
                            // We don't support cancellation of the actual HTTP req yet, so we just close the dialog visually.
                            if ui.button("Cancel").clicked() {
                                self.update_state = UpdateState::None;
                            }
                        });
                    });
                
                if !is_open {
                    self.update_state = UpdateState::None;
                }
            }
        }

        if let Some(active_idx) = self.active_tab_index {
            if let Some(tab) = self.tabs.get(active_idx) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("{} - PDFViewer", tab.file_name)));
            }
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title("PDFViewer".to_string()));
        }

        // Handle Ctrl+F / Cmd+F to focus search
        let has_search_modifier = ctx.input(|i| i.modifiers.command || i.modifiers.ctrl);
        if has_search_modifier && ctx.input(|i| i.key_pressed(egui::Key::F)) {
            ctx.memory_mut(|mem| mem.request_focus(egui::Id::new("search_bar")));
        }



        self.ui_menu_bar(ctx);

        egui::TopBottomPanel::top("tab_bar_panel").show(ctx, |ui| {
            if !self.tabs.is_empty() {
                ui.horizontal(|ui| {
                    let mut tab_to_close = None;
                    for (index, tab) in self.tabs.iter().enumerate() {
                        let is_active = Some(index) == self.active_tab_index;
                        
                        let text = format!("📄 {}", tab.file_name);
                        let text_style = if is_active {
                            egui::RichText::new(text).strong()
                        } else {
                            egui::RichText::new(text)
                        };
                        
                        if ui.selectable_label(is_active, text_style).clicked() {
                            self.active_tab_index = Some(index);
                        }
                        
                        let close_btn = egui::Button::new(egui::RichText::new("×").size(14.0))
                            .frame(false);
                        if ui.add(close_btn).clicked() {
                            tab_to_close = Some(index);
                        }
                        ui.add_space(8.0);
                    }
                    
                    if let Some(close_idx) = tab_to_close {
                        self.close_tab(close_idx);
                    }
                });
            } else {
                ui.add_space(16.0);
            }
        });

        self.ui_toolbar(ctx);

        // Rename Window Popup
        self.ui_dialogs(ctx);

        self.ui_viewer(ctx);
    }
}

impl PdfViewerApp {
    pub(crate) fn copy_selection(&self, ctx: &egui::Context) {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            if let Some(active_idx) = self.active_tab_index {
                if let Some(tab) = self.tabs.get(active_idx) {
                    let mut selected_text = String::new();
                    let (p_start, c_start, p_end, c_end) = if start.0 < end.0 {
                        (start.0, start.1, end.0, end.1)
                    } else if start.0 > end.0 {
                        (end.0, end.1, start.0, start.1)
                    } else {
                        let (c_min, c_max) = if start.1 <= end.1 { (start.1, end.1) } else { (end.1, start.1) };
                        (start.0, c_min, start.0, c_max)
                    };
                    
                    for p_idx in p_start..=p_end {
                        if let Some(chars) = tab.page_chars.get(p_idx) {
                            let start_c = if p_idx == p_start { c_start } else { 0 };
                            let end_c = if p_idx == p_end { c_end } else { chars.len().saturating_sub(1) };
                            for c_idx in start_c..=end_c {
                                if let Some(char_info) = chars.get(c_idx) {
                                    selected_text.push(char_info.c);
                                }
                            }
                        }
                        if p_idx < p_end {
                            selected_text.push('\n');
                        }
                    }
                    
                    if !selected_text.is_empty() {
                        ctx.copy_text(selected_text);
                    }
                }
            }
        }
    }

    pub(crate) fn load_pdf(&mut self, ctx: &egui::Context, path: PathBuf) {
        // Manage recent files list
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path.clone());
        self.recent_files.truncate(5);
        self.save_recent_files();

        // Check if a tab with this path already exists
        if let Some(existing_idx) = self.tabs.iter().position(|t| t.path == path) {
            self.active_tab_index = Some(existing_idx);
            return;
        }
        
        let new_tab = PdfDocumentState::empty(path.clone());
        self.tabs.push(new_tab);
        self.active_tab_index = Some(self.tabs.len() - 1);
        
        let _ = self.pdf_task_tx.send(PdfWorkerTask::Load {
            path: path.clone(),
            ctx: ctx.clone(),
        });
    }

    pub(crate) fn close_tab(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }
        self.tabs.remove(index);
        
        if self.tabs.is_empty() {
            self.active_tab_index = None;
        } else if let Some(active) = self.active_tab_index {
            if active == index {
                self.active_tab_index = Some(active.min(self.tabs.len() - 1));
            } else if active > index {
                self.active_tab_index = Some(active - 1);
            }
        }
    }

    // Recent Files persistence
    pub(crate) fn load_recent_files() -> Vec<PathBuf> {
        if let Ok(data) = fs::read_to_string(RECENT_FILES_PATH) {
            let paths: Vec<String> = data.lines().map(|s| s.to_string()).collect();
            paths.into_iter().map(PathBuf::from).collect()
        } else {
            Vec::new()
        }
    }
    
    pub(crate) fn save_recent_files(&self) {
        let data = self.recent_files.iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<String>>()
            .join("\n");
        let _ = fs::write(RECENT_FILES_PATH, data);
    }
}
