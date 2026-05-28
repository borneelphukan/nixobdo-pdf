use eframe::egui;
use pdfium_render::prelude::*;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::fs;

use crate::document::{find_closest_char, is_char_selected, PdfDocumentState, PdfLinkTarget, PdfWorkerMessage, PageLayoutMode};

const RECENT_FILES_PATH: &str = ".recent_files.json";

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ExportFormat {
    Doc,
    Docx,
    Png,
    Jpeg,
}
impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Doc => "doc",
            ExportFormat::Docx => "docx",
            ExportFormat::Png => "png",
            ExportFormat::Jpeg => "jpeg",
        }
    }
}

pub enum PdfWorkerTask {
    Load { path: PathBuf, ctx: egui::Context },
    Export { path: PathBuf, out_path: PathBuf, format: ExportFormat },
}

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
        
        // Dedicated Background Worker Thread
        thread::spawn(move || {
            let exe_path = std::env::current_exe().ok().unwrap_or_default();
            let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new(""));
            
            let pdfium_result = Pdfium::bind_to_library(exe_dir.join("libpdfium.dylib").to_str().unwrap_or_default())
                .or_else(|_| Pdfium::bind_to_library(exe_dir.join("pdfium.dll").to_str().unwrap_or_default()))
                .or_else(|_| Pdfium::bind_to_library("./lib/libpdfium.dylib"))
                .or_else(|_| Pdfium::bind_to_library("libpdfium.dylib"))
                .or_else(|_| Pdfium::bind_to_library("./lib/pdfium.dll"))
                .or_else(|_| Pdfium::bind_to_library("pdfium.dll"))
                .or_else(|_| Pdfium::bind_to_system_library());

            let pdfium = match pdfium_result {
                Ok(bindings) => Some(Pdfium::new(bindings)),
                Err(_) => None,
            };

            while let Ok(task) = task_rx.recv() {
                if let Some(pdf) = &pdfium {
                    match task {
                        PdfWorkerTask::Load { path, ctx } => {
                            PdfDocumentState::background_load_with_pdfium(path, pdf, msg_tx_clone.clone(), ctx);
                        }
                        PdfWorkerTask::Export { path, out_path, format } => {
                            if format == ExportFormat::Png || format == ExportFormat::Jpeg {
                                if let Ok(doc) = pdf.load_pdf_from_file(&path, None) {
                                    let pages = doc.pages();
                                    if let Ok(page) = pages.get(0) {
                                        let render_config = PdfRenderConfig::new()
                                            .set_target_width(2000)
                                            .set_clear_color(PdfColor::new(255, 255, 255, 255));
                                            
                                        if let Ok(bitmap) = page.render_with_config(&render_config) {
                                            let _ = bitmap.as_image().save(&out_path);
                                        }
                                    }
                                }
                            } else {
                                if let Ok(doc) = pdf.load_pdf_from_file(&path, None) {
                                    let pages = doc.pages();
                                    let mut content = String::new();
                                    for page in pages.iter() {
                                        if let Ok(text) = page.text() {
                                            content.push_str(&text.all());
                                            content.push_str("\n\n");
                                        }
                                    }
                                    
                                    if format == ExportFormat::Docx {
                                        if let Ok(file) = std::fs::File::create(&out_path) {
                                            let mut docx = docx_rs::Docx::new();
                                            for line in content.split('\n') {
                                                let text = if line.is_empty() { " " } else { line };
                                                docx = docx.add_paragraph(
                                                    docx_rs::Paragraph::new().add_run(docx_rs::Run::new().add_text(text))
                                                );
                                            }
                                            let _ = docx.build().pack(file);
                                        }
                                    } else {
                                        // Write valid RTF for .doc format
                                        let mut rtf = String::from(r"{\rtf1\ansi\ansicpg1252\deff0\nouicompat{\fonttbl{\f0\fnil\fcharset0 Calibri;}}\viewkind4\uc1\pard\f0\fs22 ");
                                        for c in content.chars() {
                                            match c {
                                                '\\' => rtf.push_str(r"\\"),
                                                '{' => rtf.push_str(r"\{"),
                                                '}' => rtf.push_str(r"\}"),
                                                '\n' => rtf.push_str("\\par\n"),
                                                _ => rtf.push(c),
                                            }
                                        }
                                        rtf.push('}');
                                        let _ = std::fs::write(&out_path, rtf);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    if let PdfWorkerTask::Load { path, ctx } = task {
                        let _ = msg_tx_clone.send(PdfWorkerMessage::DocumentInfo {
                            path: path.clone(),
                            file_name: String::new(),
                            page_count: 0,
                            error: Some("PDFium not initialized. Please ensure libpdfium is present.".into()),
                        });
                        ctx.request_repaint();
                    }
                }
            }
        });

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
        }
    }
}

impl eframe::App for PdfViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process background loaded PDFs
        while let Ok(msg) = self.pdf_receiver.try_recv() {
            match msg {
                PdfWorkerMessage::DocumentInfo { path, file_name, page_count, error } => {
                    for tab in self.tabs.iter_mut() {
                        if tab.path == path {
                            if let Some(err) = error {
                                tab.error = Some(err);
                                tab.is_loading = false;
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

        // Pre-calculate search matches
        let mut total_matches = 0;
        if let Some(active_idx) = self.active_tab_index {
            if let Some(tab) = self.tabs.get(active_idx) {
                if !self.search_query.is_empty() {
                    let query_lower = self.search_query.to_lowercase();
                    for page_chars in &tab.page_chars {
                        let page_string: String = page_chars.iter().map(|char_info| char_info.c).collect();
                        let page_string_lower = page_string.to_lowercase();
                        
                        let mut start = 0;
                        while let Some(pos) = page_string_lower[start..].find(&query_lower) {
                            total_matches += 1;
                            start = start + pos + query_lower.len();
                        }
                    }
                }
            }
        }

        egui::TopBottomPanel::top("menu_bar_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Some(paths) = rfd::FileDialog::new()
                            .add_filter("PDF files", &["pdf"])
                            .pick_files()
                        {
                            for path in paths {
                                self.load_pdf(ctx, path);
                            }
                        }
                        ui.close_menu();
                    }
                    
                    // Open Recent nested menu
                    ui.menu_button("Open Recent", |ui| {
                        if self.recent_files.is_empty() {
                            ui.label(egui::RichText::new("No recent files").weak());
                        } else {
                            let mut to_open = None;
                            for recent_path in &self.recent_files {
                                let name = recent_path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                                if ui.button(name).clicked() {
                                    to_open = Some(recent_path.clone());
                                }
                            }
                            if let Some(path) = to_open {
                                self.load_pdf(ctx, path);
                                ui.close_menu();
                            }
                        }
                    });
                    
                    ui.separator();
                    
                    if ui.button("Close Window").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    
                    if ui.add_enabled(self.active_tab_index.is_some(), egui::Button::new("Close Selected PDF Document")).clicked() {
                        if let Some(active_idx) = self.active_tab_index {
                            self.close_tab(active_idx);
                        }
                        ui.close_menu();
                    }
                    
                    if ui.add_enabled(self.active_tab_index.is_some(), egui::Button::new("Rename")).clicked() {
                        if let Some(active_idx) = self.active_tab_index {
                            if let Some(tab) = self.tabs.get(active_idx) {
                                self.rename_buffer = tab.file_name.clone();
                                self.rename_window_open = true;
                                self.focus_rename_input = true;
                            }
                        }
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.add_enabled(self.active_tab_index.is_some(), egui::Button::new("Export...")).clicked() {
                        if let Some(active_idx) = self.active_tab_index {
                            if let Some(tab) = self.tabs.get(active_idx) {
                                let name = if tab.file_name.to_lowercase().ends_with(".pdf") {
                                    tab.file_name[..tab.file_name.len() - 4].to_string()
                                } else {
                                    tab.file_name.clone()
                                };
                                self.export_name = name;
                                self.export_window_open = true;
                            }
                        }
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("View", |ui| {
                    let sidebar_text = if self.sidebar_open { "Hide Sidebar" } else { "Show Sidebar" };
                    if ui.button(sidebar_text).clicked() {
                        self.sidebar_open = !self.sidebar_open;
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if let Some(active_idx) = self.active_tab_index {
                        if let Some(tab) = self.tabs.get_mut(active_idx) {
                            let cont_text = if tab.layout_mode == PageLayoutMode::ContinuousScroll { "✔ Scroll Mode" } else { "   Scroll Mode" };
                            if ui.button(cont_text).clicked() {
                                tab.layout_mode = PageLayoutMode::ContinuousScroll;
                                ui.close_menu();
                            }
                            
                            let single_text = if tab.layout_mode == PageLayoutMode::SinglePage { "✔ Single Page" } else { "    Single Page" };
                            if ui.button(single_text).clicked() {
                                tab.layout_mode = PageLayoutMode::SinglePage;
                                ui.close_menu();
                            }
                            
                            let two_text = if tab.layout_mode == PageLayoutMode::TwoPage { "✔ Two Page" } else { "    Two Page" };
                            if ui.button(two_text).clicked() {
                                tab.layout_mode = PageLayoutMode::TwoPage;
                                ui.close_menu();
                            }
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("    Scroll Mode"));
                        ui.add_enabled(false, egui::Button::new("    Single Page"));
                        ui.add_enabled(false, egui::Button::new("    Two Page"));
                    }
                });
            });
        });

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

        egui::TopBottomPanel::top("toolbar_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("🔍 Zoom:");
                if let Some(active_idx) = self.active_tab_index {
                    if let Some(tab) = self.tabs.get_mut(active_idx) {
                        if ui.button("➖ Zoom Out").clicked() {
                            tab.zoom = (tab.zoom - 0.1).max(0.0);
                        }
                        ui.label(format!("{:.1}x", tab.zoom + 1.0));
                        if ui.button("➕ Zoom In").clicked() {
                            tab.zoom += 0.1;
                        }
                        if ui.button("Reset").clicked() {
                            tab.zoom = 0.0;
                        }
                        
                        ui.separator();
                        
                        ui.label("Page:");
                        if ui.button("⬆").clicked() {
                            if tab.selected_page > 0 {
                                tab.selected_page -= 1;
                                tab.scroll_to_page = Some(tab.selected_page);
                            }
                        }
                        ui.label(format!("{}/{}", tab.selected_page + 1, tab.pages.len().max(1)));
                        if ui.button("⬇").clicked() {
                            if tab.selected_page + 1 < tab.pages.len() {
                                tab.selected_page += 1;
                                tab.scroll_to_page = Some(tab.selected_page);
                            }
                        }
                    }
                } else {
                    ui.label("-");
                }

                ui.separator();

                let any_loading = self.tabs.iter().any(|t| t.is_loading);
                if any_loading {
                    ui.spinner();
                    ui.label("Initializing...");
                }

                if !self.has_pdfium_bindings {
                    ui.colored_label(egui::Color32::RED, "⚠ PDFium library not found!");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.active_tab_index.is_some() {
                        if !self.search_query.is_empty() {
                            if ui.small_button("Clear").clicked() {
                                self.search_query.clear();
                            }
                            ui.label(egui::RichText::new(format!("({} matches)", total_matches)).size(12.0).weak());
                        }
                        
                        let text_edit = egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Search PDF... (Ctrl+F)")
                            .desired_width(150.0)
                            .id(egui::Id::new("search_bar"));
                        let response = ui.add(text_edit);
                        
                        if has_search_modifier && ctx.input(|i| i.key_pressed(egui::Key::F)) {
                            response.request_focus();
                        }
                        
                        ui.label("🔍 Find:");
                    }
                });
            });
            ui.add_space(4.0);
        });

        // Rename Window Popup
        if self.rename_window_open {
            let mut close_window = false;
            let mut perform_rename = false;

            egui::Window::new("Rename File")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(egui::Align2::CENTER_TOP, [0.0, 30.0])
                .frame(egui::Frame::popup(&ctx.style()).inner_margin(8.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        let response = ui.text_edit_singleline(&mut self.rename_buffer);
                        
                        if self.focus_rename_input {
                            response.request_focus();
                            self.focus_rename_input = false;
                        }
                        
                        // Save and close on Enter, or when clicking outside (losing focus)
                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            perform_rename = true;
                            close_window = true;
                        } else if response.lost_focus() && !ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            // Only save if it lost focus and we didn't just press escape (though escape would also close)
                            perform_rename = true;
                            close_window = true;
                        } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            close_window = true;
                        }
                    });
                });

            if perform_rename {
                if let Some(active_idx) = self.active_tab_index {
                    if let Some(tab) = self.tabs.get_mut(active_idx) {
                        let old_path = tab.path.clone();
                        let mut new_path = old_path.clone();
                        new_path.set_file_name(&self.rename_buffer);
                        
                        if fs::rename(&old_path, &new_path).is_ok() {
                            tab.path = new_path.clone();
                            tab.file_name = self.rename_buffer.clone();
                            
                            // Update recent files
                            self.recent_files.retain(|p| p != &old_path);
                            if !self.recent_files.contains(&new_path) {
                                self.recent_files.insert(0, new_path);
                                self.recent_files.truncate(5);
                                self.save_recent_files();
                            }
                        } else {
                            eprintln!("Failed to rename file on disk.");
                        }
                    }
                }
            }

            if close_window {
                self.rename_window_open = false;
            }
        }

        // Export Dialog
        if self.export_window_open {
            let mut close_window = false;
            let mut perform_export = false;
            
            egui::Window::new("Export File")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Export As:");
                        ui.text_edit_singleline(&mut self.export_name);
                    });
                    
                    ui.add_space(4.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Location:");
                        let location_text = self.export_location.as_ref()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Select Folder...".to_string());
                            
                        if ui.button(&location_text).clicked() {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                self.export_location = Some(folder);
                            }
                        }
                    });
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("Format:");
                        egui::ComboBox::from_id_salt("format_dropdown")
                            .selected_text(format!("{:?}", self.export_format))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.export_format, ExportFormat::Doc, "Doc");
                                ui.selectable_value(&mut self.export_format, ExportFormat::Docx, "Docx");
                                ui.selectable_value(&mut self.export_format, ExportFormat::Png, "PNG");
                                ui.selectable_value(&mut self.export_format, ExportFormat::Jpeg, "JPEG");
                            });
                    });
                    
                    ui.add_space(8.0);
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Only enable Save if a location is selected
                        let save_enabled = self.export_location.is_some();
                        if ui.add_enabled(save_enabled, egui::Button::new("Save")).clicked() {
                            perform_export = true;
                            close_window = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_window = true;
                        }
                    });
                });
                
            if perform_export {
                if let Some(active_idx) = self.active_tab_index {
                    if let Some(tab) = self.tabs.get(active_idx) {
                        if let Some(location) = &self.export_location {
                            let final_name = format!("{}.{}", self.export_name, self.export_format.extension());
                            let out_path = location.join(final_name);
                            
                            let _ = self.pdf_task_tx.send(PdfWorkerTask::Export {
                                path: tab.path.clone(),
                                out_path,
                                format: self.export_format,
                            });
                        }
                    }
                }
            }
            if close_window {
                self.export_window_open = false;
            }
        }

        // Left sidebar for page preview
        if self.sidebar_open {
            egui::SidePanel::left("preview_panel")
                .resizable(false)
                .exact_width(180.0)
                .show(ctx, |ui| {
                    let mut pages_empty = true;
                    if let Some(active_idx) = self.active_tab_index {
                        if let Some(tab) = self.tabs.get_mut(active_idx) {
                            pages_empty = tab.pages.is_empty();
                            
                            if !pages_empty {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        for (index, texture_opt) in tab.pages.iter().enumerate() {
                                            let thumb_w = (ui.available_width() - 16.0).max(40.0);
                                            let thumb_h = thumb_w * 1.414; // Default A4 ratio for placeholder
                                            let thumb_size = egui::vec2(thumb_w, thumb_h);
                                            
                                            let is_selected = tab.selected_page == index;
                                            let stroke_color = if is_selected {
                                                ui.visuals().selection.bg_fill
                                            } else {
                                                ui.visuals().widgets.noninteractive.bg_stroke.color
                                            };
                                            let bg_color = if is_selected {
                                                ui.visuals().selection.bg_fill.gamma_multiply(0.15)
                                            } else {
                                                ui.visuals().widgets.noninteractive.bg_fill
                                            };

                                            egui::Frame::NONE
                                                .stroke(egui::Stroke::new(if is_selected { 2.0 } else { 1.0 }, stroke_color))
                                                .fill(bg_color)
                                                .corner_radius(4.0)
                                                .inner_margin(egui::Margin::symmetric(4, 3))
                                                .show(ui, |ui| {
                                                    if let Some(texture) = texture_opt {
                                                        let actual_aspect = texture.size_vec2().y / texture.size_vec2().x;
                                                        let actual_h = thumb_w * actual_aspect;
                                                        
                                                        let img = egui::Image::new(egui::load::SizedTexture::new(texture.id(), egui::vec2(thumb_w, actual_h)))
                                                            .sense(egui::Sense::click());
                                                        let response = ui.add(img);
                                                        if response.clicked() {
                                                            tab.selected_page = index;
                                                            tab.scroll_to_page = Some(index);
                                                        }
                                                    } else {
                                                        // Placeholder spinner for loading pages
                                                        let (rect, response) = ui.allocate_exact_size(thumb_size, egui::Sense::click());
                                                        if ui.is_rect_visible(rect) {
                                                            ui.painter().rect_filled(rect, 2.0, egui::Color32::from_gray(40));
                                                            ui.painter().text(
                                                                rect.center(),
                                                                egui::Align2::CENTER_CENTER,
                                                                "...",
                                                                egui::FontId::proportional(14.0),
                                                                egui::Color32::GRAY,
                                                            );
                                                        }
                                                        if response.clicked() {
                                                            tab.selected_page = index;
                                                            tab.scroll_to_page = Some(index);
                                                        }
                                                    }
                                                    
                                                    ui.vertical_centered(|ui| {
                                                        ui.add_space(4.0);
                                                        ui.label(format!("{}", index + 1));
                                                    });
                                                });
                                            ui.add_space(8.0);
                                        }
                                    });
                                });
                            }
                        }
                    }
                    
                    if pages_empty {
                        ui.centered_and_justified(|ui| {
                            ui.label("No PDF loaded");
                        });
                    }
                });
        }

        // Draggable vertical separator panel
        egui::SidePanel::left("separator_panel")
            .resizable(false)
            .exact_width(1.0)
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let (response, painter) = ui.allocate_painter(rect.size(), egui::Sense::click_and_drag());
                
                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }
                
                if response.clicked() {
                    self.sidebar_open = !self.sidebar_open;
                } else if response.dragged() {
                    let delta_x = response.drag_delta().x;
                    if self.sidebar_open && delta_x < -2.0 {
                        self.sidebar_open = false;
                    } else if !self.sidebar_open && delta_x > 2.0 {
                        self.sidebar_open = true;
                    }
                }
                
                let is_active = response.hovered() || response.dragged();
                let color = if is_active {
                    ui.visuals().widgets.active.bg_fill
                } else {
                    ui.visuals().widgets.noninteractive.bg_stroke.color
                };
                
                let stroke_width = if is_active { 2.0 } else { 0.5 };
                let line_x = rect.center().x;
                painter.line_segment(
                    [egui::pos2(line_x, rect.min.y), egui::pos2(line_x, rect.max.y)],
                    egui::Stroke::new(stroke_width, color),
                );
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut show_placeholder = true;
            
            let mut select_all_triggered = false;
            let mut copy_triggered = false;
            let mut exit_triggered = false;
            
            if let Some(active_idx) = self.active_tab_index {
                if let Some(tab) = self.tabs.get_mut(active_idx) {
                    show_placeholder = false;
                    
                    if let Some(error) = &tab.error {
                        ui.centered_and_justified(|ui| {
                            ui.colored_label(egui::Color32::RED, error);
                        });
                    } else if tab.is_loading {
                        ui.centered_and_justified(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.spinner();
                                ui.add_space(10.0);
                                ui.label(egui::RichText::new("Parsing PDF Document...").weak());
                            });
                        });
                    } else if tab.pages.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label("No pages found in this PDF.");
                        });
                    } else {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                let available_width = ui.available_width() - 24.0;
                                let chunks = if tab.layout_mode == PageLayoutMode::TwoPage { 2 } else { 1 };
                                let page_width = if chunks == 2 { available_width * (1.0 + tab.zoom) / 2.0 - 10.0 } else { available_width * (1.0 + tab.zoom) };
                                
                                let mut scrolled = false;
                                
                                let mut page_indices = Vec::new();
                                match tab.layout_mode {
                                    PageLayoutMode::ContinuousScroll => {
                                        for i in 0..tab.pages.len() { page_indices.push(i); }
                                    },
                                    PageLayoutMode::SinglePage => {
                                        if tab.selected_page < tab.pages.len() { page_indices.push(tab.selected_page); }
                                    },
                                    PageLayoutMode::TwoPage => {
                                        if tab.selected_page < tab.pages.len() { page_indices.push(tab.selected_page); }
                                        if tab.selected_page + 1 < tab.pages.len() { page_indices.push(tab.selected_page + 1); }
                                    }
                                }
                                
                                ui.vertical_centered(|ui| {
                                    for chunk in page_indices.chunks(chunks) {
                                        ui.horizontal_centered(|ui| {
                                            for &index in chunk {
                                                let texture_opt = &tab.pages[index];
                                        // Default aspect ratio for A4
                                        let aspect = if let Some(texture) = texture_opt {
                                            texture.size_vec2().y / texture.size_vec2().x
                                        } else {
                                            1.414 
                                        };
                                        let size = egui::vec2(page_width, page_width * aspect);
                                        
                                        // Allocate space for the page and get interaction response
                                        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
                                        
                                        if ui.is_rect_visible(rect) {
                                            // Draw solid white background for the page behind everything
                                            ui.painter().rect_filled(rect, 0.0, egui::Color32::WHITE);
                                        }
                                        
                                        if let Some(texture) = texture_opt {
                                            // Handle drag/selection input on the page
                                            if index < tab.page_chars.len() {
                                                if response.drag_started() {
                                                    if let Some(mouse_pos) = ctx.pointer_interact_pos() {
                                                        if let Some(char_idx) = find_closest_char(response.rect, mouse_pos, &tab.page_chars[index]) {
                                                            self.selection_start = Some((index, char_idx));
                                                            self.selection_end = Some((index, char_idx));
                                                            self.is_selecting = true;
                                                        }
                                                    }
                                                }
                                                
                                                if self.is_selecting && response.dragged() {
                                                    if let Some(mouse_pos) = ctx.pointer_interact_pos() {
                                                        if let Some(char_idx) = find_closest_char(response.rect, mouse_pos, &tab.page_chars[index]) {
                                                            self.selection_end = Some((index, char_idx));
                                                        }
                                                    }
                                                }
                                                
                                                if self.is_selecting && response.drag_stopped() {
                                                    self.is_selecting = false;
                                                }
                                                
                                                if response.clicked() && !response.dragged() {
                                                    self.selection_start = None;
                                                    self.selection_end = None;
                                                }
                                                
                                                // Copy content on right-click
                                                if response.secondary_clicked() {
                                                    copy_triggered = true;
                                                }
                                                
                                                // Attach custom right-click context menu
                                                let has_selection = self.selection_start.is_some();
                                                let mut zoom_in_clicked = false;
                                                let mut zoom_out_clicked = false;
                                                
                                                response.context_menu(|ui| {
                                                    if has_selection {
                                                        if ui.button("📋 Copy Selected Text").clicked() {
                                                            copy_triggered = true;
                                                            ui.close_menu();
                                                        }
                                                    }
                                                    if ui.button("📖 Select All").clicked() {
                                                        select_all_triggered = true;
                                                        ui.close_menu();
                                                    }
                                                    if ui.button("🔍 Zoom In").clicked() {
                                                        zoom_in_clicked = true;
                                                        ui.close_menu();
                                                    }
                                                    if ui.button("🔍 Zoom Out").clicked() {
                                                        zoom_out_clicked = true;
                                                        ui.close_menu();
                                                    }
                                                    ui.separator();
                                                    if ui.button("🚪 Exit").clicked() {
                                                        exit_triggered = true;
                                                        ui.close_menu();
                                                    }
                                                });
                                                
                                                if zoom_in_clicked {
                                                    tab.zoom += 0.1;
                                                }
                                                if zoom_out_clicked {
                                                    tab.zoom = (tab.zoom - 0.1).max(0.0);
                                                }
                                            }

                                            // Draw blue text selection overlays
                                            if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                                                if index < tab.page_chars.len() {
                                                    for char_idx in 0..tab.page_chars[index].len() {
                                                        if is_char_selected(index, char_idx, start, end) {
                                                            let char_info = &tab.page_chars[index][char_idx];
                                                            if !char_info.c.is_whitespace() {
                                                                let rect_min = egui::pos2(
                                                                    response.rect.min.x + char_info.left * response.rect.width(),
                                                                    response.rect.min.y + char_info.top * response.rect.height(),
                                                                );
                                                                let rect_max = egui::pos2(
                                                                    response.rect.min.x + char_info.right * response.rect.width(),
                                                                    response.rect.min.y + char_info.bottom * response.rect.height(),
                                                                );
                                                                let highlight_rect = egui::Rect::from_min_max(rect_min, rect_max);
                                                                
                                                                ui.painter().rect_filled(
                                                                    highlight_rect,
                                                                    0.0,
                                                                    egui::Color32::from_rgba_unmultiplied(66, 165, 245, 80),
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            
                                            // Draw Y-axis-oriented yellow highlights overlay on matched words
                                            if !self.search_query.is_empty() && index < tab.page_chars.len() {
                                                let query_lower = self.search_query.to_lowercase();
                                                
                                                let page_string: String = tab.page_chars[index].iter().map(|char_info| char_info.c).collect();
                                                let page_string_lower = page_string.to_lowercase();
                                                
                                                let mut start = 0;
                                                while let Some(pos) = page_string_lower[start..].find(&query_lower) {
                                                    let absolute_pos = start + pos;
                                                    
                                                    for char_idx in absolute_pos..(absolute_pos + query_lower.len()) {
                                                        if let Some(char_info) = tab.page_chars[index].get(char_idx) {
                                                            if !char_info.c.is_whitespace() {
                                                                let rect_min = egui::pos2(
                                                                    response.rect.min.x + char_info.left * response.rect.width(),
                                                                    response.rect.min.y + char_info.top * response.rect.height(),
                                                                );
                                                                let rect_max = egui::pos2(
                                                                    response.rect.min.x + char_info.right * response.rect.width(),
                                                                    response.rect.min.y + char_info.bottom * response.rect.height(),
                                                                );
                                                                let highlight_rect = egui::Rect::from_min_max(rect_min, rect_max);
                                                                
                                                                ui.painter().rect_filled(
                                                                    highlight_rect,
                                                                    0.0,
                                                                    egui::Color32::from_rgba_unmultiplied(255, 255, 0, 75),
                                                                );
                                                            }
                                                        }
                                                    }
                                                    
                                                    start = start + pos + query_lower.len();
                                                }
                                            }
                                            
                                            // Draw the actual PDF page image with transparent background over the top
                                            if ui.is_rect_visible(response.rect) {
                                                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                                                ui.painter().image(texture.id(), response.rect, uv, egui::Color32::WHITE);
                                            }
                                            
                                            // Handle PDF links interaction
                                            if index < tab.page_links.len() {
                                                for (link_idx, link_info) in tab.page_links[index].iter().enumerate() {
                                                    let link_rect = egui::Rect::from_min_max(
                                                        egui::pos2(
                                                            response.rect.min.x + link_info.left * response.rect.width(),
                                                            response.rect.min.y + link_info.top * response.rect.height(),
                                                        ),
                                                        egui::pos2(
                                                            response.rect.min.x + link_info.right * response.rect.width(),
                                                            response.rect.min.y + link_info.bottom * response.rect.height(),
                                                        ),
                                                    );
                                                    
                                                    let link_response = ui.interact(link_rect, ui.id().with(format!("link_{}_{}", index, link_idx)), egui::Sense::click());
                                                    if link_response.hovered() {
                                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                                    }
                                                    if link_response.clicked() {
                                                        match &link_info.target {
                                                            PdfLinkTarget::Url(url) => {
                                                                let _ = webbrowser::open(url);
                                                            }
                                                            PdfLinkTarget::Page(page_idx) => {
                                                                tab.scroll_to_page = Some(*page_idx);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            // Loading placeholder
                                            if ui.is_rect_visible(rect) {
                                                ui.painter().text(
                                                    rect.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    format!("Loading Page {}...", index + 1),
                                                    egui::FontId::proportional(16.0),
                                                    egui::Color32::GRAY,
                                                );
                                            }
                                        }
                                        
                                        if Some(index) == tab.scroll_to_page {
                                            response.scroll_to_me(Some(egui::Align::Center));
                                            scrolled = true;
                                        }
                                        
                                        ui.add_space(15.0); // Horizontal spacing between pages
                                    }
                                });
                                ui.add_space(15.0); // Vertical spacing between rows
                            }
                        });
                                
                                if scrolled {
                                    tab.scroll_to_page = None;
                                }
                            });
                    }
                }
            }
            
            if show_placeholder {
                ui.centered_and_justified(|ui| {
                    ui.label("Open a PDF file from the File menu to start viewing.");
                });
            }

            if copy_triggered {
                self.copy_selection(ctx);
            }
            if select_all_triggered {
                if let Some(active_idx) = self.active_tab_index {
                    if let Some(tab) = self.tabs.get_mut(active_idx) {
                        if !tab.page_chars.is_empty() {
                            self.selection_start = Some((0, 0));
                            let last_page_idx = tab.pages.len() - 1;
                            let last_page_chars_len = tab.page_chars[last_page_idx].len();
                            self.selection_end = Some((last_page_idx, last_page_chars_len.saturating_sub(1)));
                        }
                    }
                }
            }
            if exit_triggered {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        // Handle zoom with Cmd/Ctrl + Mouse Wheel
        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
        let zoom_delta = ctx.input(|i| i.raw_scroll_delta.y);
        let delta = if scroll_delta != 0.0 { scroll_delta } else { zoom_delta };
        
        let has_zoom_modifier = ctx.input(|i| i.modifiers.command || i.modifiers.ctrl);
        
        if has_zoom_modifier && delta != 0.0 {
            if let Some(active_idx) = self.active_tab_index {
                if let Some(tab) = self.tabs.get_mut(active_idx) {
                    if delta > 0.0 {
                        tab.zoom += 0.1;
                    } else {
                        tab.zoom = (tab.zoom - 0.1).max(0.0);
                    }
                }
            }
        }

        // Handle zoom with Trackpad pinch
        let zoom_gesture = ctx.input(|i| i.zoom_delta());
        if zoom_gesture != 1.0 {
            if let Some(active_idx) = self.active_tab_index {
                if let Some(tab) = self.tabs.get_mut(active_idx) {
                    let new_zoom = (1.0 + tab.zoom) * zoom_gesture - 1.0;
                    tab.zoom = new_zoom.max(0.0);
                }
            }
        }

        // Handle Copy selection shortcut (Cmd+C / Ctrl+C)
        let has_copy_modifier = ctx.input(|i| i.modifiers.command || i.modifiers.ctrl);
        if has_copy_modifier && ctx.input(|i| i.key_pressed(egui::Key::C)) {
            self.copy_selection(ctx);
        }
    }
}

impl PdfViewerApp {
    fn copy_selection(&self, ctx: &egui::Context) {
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

    fn load_pdf(&mut self, ctx: &egui::Context, path: PathBuf) {
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

    fn close_tab(&mut self, index: usize) {
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
    fn load_recent_files() -> Vec<PathBuf> {
        if let Ok(data) = fs::read_to_string(RECENT_FILES_PATH) {
            let paths: Vec<String> = data.lines().map(|s| s.to_string()).collect();
            paths.into_iter().map(PathBuf::from).collect()
        } else {
            Vec::new()
        }
    }
    
    fn save_recent_files(&self) {
        let data = self.recent_files.iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<String>>()
            .join("\n");
        let _ = fs::write(RECENT_FILES_PATH, data);
    }
}
