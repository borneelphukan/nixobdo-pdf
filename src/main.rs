use eframe::egui;
use pdfium_render::prelude::*;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let icon_bytes = include_bytes!("../assets/logo.png");
    let image = image::load_from_memory(icon_bytes).expect("Failed to load icon").into_rgba8();
    let (width, height) = image.dimensions();
    let icon_data = egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 1000.0])
            .with_title("PDF Viewer")
            .with_icon(icon_data),
        ..Default::default()
    };
    eframe::run_native(
        "PDF Viewer",
        options,
        Box::new(|_cc| Ok(Box::new(PdfViewerApp::default()))),
    )
}

struct PdfCharInfo {
    c: char,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

#[derive(Clone)]
enum PdfLinkTarget {
    Url(String),
    Page(usize),
}

#[derive(Clone)]
struct PdfLinkInfo {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    target: PdfLinkTarget,
}

fn find_closest_char(page_rect: egui::Rect, mouse_pos: egui::Pos2, chars: &[PdfCharInfo]) -> Option<usize> {
    if chars.is_empty() {
        return None;
    }
    let rx = ((mouse_pos.x - page_rect.min.x) / page_rect.width()) as f32;
    let ry = ((mouse_pos.y - page_rect.min.y) / page_rect.height()) as f32;

    let mut closest_idx = None;
    let mut min_dist = f32::MAX;

    for (idx, char_info) in chars.iter().enumerate() {
        let cx = (char_info.left + char_info.right) / 2.0;
        let cy = (char_info.top + char_info.bottom) / 2.0;

        let vertical_overlap = ry >= char_info.top && ry <= char_info.bottom;
        let horizontal_overlap = rx >= char_info.left && rx <= char_info.right;

        let dist = if vertical_overlap && horizontal_overlap {
            0.0
        } else if vertical_overlap {
            (rx - cx).abs()
        } else {
            let dx = rx - cx;
            let dy = ry - cy;
            (dx * dx + dy * dy).sqrt()
        };

        if dist < min_dist {
            min_dist = dist;
            closest_idx = Some(idx);
        }
    }

    closest_idx
}

fn is_char_selected(
    page_idx: usize,
    char_idx: usize,
    start: (usize, usize),
    end: (usize, usize),
) -> bool {
    let (p_min, c_min, p_max, c_max) = if start.0 < end.0 {
        (start.0, start.1, end.0, end.1)
    } else if start.0 > end.0 {
        (end.0, end.1, start.0, start.1)
    } else {
        let (c_min, c_max) = if start.1 <= end.1 {
            (start.1, end.1)
        } else {
            (end.1, start.1)
        };
        return page_idx == start.0 && char_idx >= c_min && char_idx <= c_max;
    };

    if page_idx < p_min || page_idx > p_max {
        false
    } else if page_idx > p_min && page_idx < p_max {
        true
    } else if page_idx == p_min {
        char_idx >= c_min
    } else {
        char_idx <= c_max
    }
}


struct PdfDocumentState {
    file_name: String,
    pages: Vec<egui::TextureHandle>,
    #[allow(dead_code)]
    page_texts: Vec<String>,
    page_chars: Vec<Vec<PdfCharInfo>>,
    page_links: Vec<Vec<PdfLinkInfo>>,
    zoom: f32,
    selected_page: usize,
    scroll_to_page: Option<usize>,
    error: Option<String>,
}

struct PdfViewerApp {
    pdfium: Option<Pdfium>,
    tabs: Vec<PdfDocumentState>,
    active_tab_index: Option<usize>,
    loading: bool,
    search_query: String,
    sidebar_open: bool,
    selection_start: Option<(usize, usize)>, // (page_idx, char_idx)
    selection_end: Option<(usize, usize)>,   // (page_idx, char_idx)
    is_selecting: bool,
}

impl Default for PdfViewerApp {
    fn default() -> Self {
        // Try to initialize pdfium from local lib folder first
        let exe_path = std::env::current_exe().ok().unwrap_or_default();
        let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new(""));
        
        let pdfium = match Pdfium::bind_to_library(exe_dir.join("libpdfium.dylib").to_str().unwrap_or_default())
            .or_else(|_| Pdfium::bind_to_library(exe_dir.join("pdfium.dll").to_str().unwrap_or_default()))
            .or_else(|_| Pdfium::bind_to_library("./lib/libpdfium.dylib"))
            .or_else(|_| Pdfium::bind_to_library("libpdfium.dylib"))
            .or_else(|_| Pdfium::bind_to_library("./lib/pdfium.dll"))
            .or_else(|_| Pdfium::bind_to_library("pdfium.dll"))
            .or_else(|_| Pdfium::bind_to_system_library())
        {
            Ok(bindings) => Some(Pdfium::new(bindings)),
            Err(e) => {
                eprintln!("Failed to bind to PDFium: {:?}", e);
                None
            }
        };

        Self {
            pdfium,
            tabs: Vec::new(),
            active_tab_index: None,
            loading: false,
            search_query: String::new(),
            sidebar_open: true,
            selection_start: None,
            selection_end: None,
            is_selecting: false,
        }
    }
}

impl eframe::App for PdfViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Pre-calculate search matches to get dynamic match count
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

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("📂 Open PDF").clicked() {
                        // pick_files supporting multiple PDFs simultaneously
                        if let Some(paths) = rfd::FileDialog::new()
                            .add_filter("PDF files", &["pdf"])
                            .pick_files()
                        {
                            for path in paths {
                                self.load_pdf(ctx, path);
                            }
                        }
                    }
                    
                    ui.separator();
                    
                    ui.label("🔍 Zoom:");
                    if let Some(active_idx) = self.active_tab_index {
                        if let Some(tab) = self.tabs.get_mut(active_idx) {
                            if ui.button("➖").clicked() {
                                tab.zoom = (tab.zoom - 0.1).max(0.0);
                            }
                            ui.label(format!("{:.1}", tab.zoom));
                            if ui.button("➕").clicked() {
                                tab.zoom += 0.1;
                            }
                            if ui.button("Reset").clicked() {
                                tab.zoom = 0.0;
                            }
                        }
                    } else {
                        ui.label("-");
                    }

                    ui.separator();

                    if self.loading {
                        ui.spinner();
                        ui.label("Loading...");
                    }

                    if self.pdfium.is_none() {
                        ui.colored_label(egui::Color32::RED, "⚠ PDFium library not found!");
                    }

                    // Relocated search input box with stable ID and match counter
                    if self.active_tab_index.is_some() {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if !self.search_query.is_empty() {
                                if ui.small_button("Clear").clicked() {
                                    self.search_query.clear();
                                }
                                ui.label(egui::RichText::new(format!("({} matches)", total_matches)).size(12.0).weak());
                            }
                            
                            let text_edit = egui::TextEdit::singleline(&mut self.search_query)
                                .hint_text("Search PDF...")
                                .desired_width(150.0)
                                .id(egui::Id::new("search_bar")); // Stable ID to prevent focus loss!
                            ui.add(text_edit);
                            ui.label("🔍 Find:");
                        });
                    }
                });

                // Tab Bar UI
                if !self.tabs.is_empty() {
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);
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
                }
            });
        });

        // Left sidebar for page preview (strictly fixed width, no search tab or text results)
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
                                        for (index, texture) in tab.pages.iter().enumerate() {
                                            let thumb_w = (ui.available_width() - 16.0).max(40.0);
                                            let aspect = texture.size_vec2().y / texture.size_vec2().x;
                                            let thumb_h = thumb_w * aspect;
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
                                                    let img = egui::Image::new(egui::load::SizedTexture::new(texture.id(), thumb_size))
                                                        .sense(egui::Sense::click());
                                                    let response = ui.add(img);
                                                    if response.clicked() {
                                                        tab.selected_page = index;
                                                        tab.scroll_to_page = Some(index);
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

        // Draggable vertical separator panel to toggle, collapse or expand sidebar
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
                    } else if tab.pages.is_empty() && !self.loading {
                        ui.centered_and_justified(|ui| {
                            ui.label("No pages found in this PDF.");
                        });
                    } else if !tab.pages.is_empty() {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                let available_width = ui.available_width() - 24.0;
                                let page_width = available_width * (1.0 + tab.zoom);
                                
                                let mut scrolled = false;
                                
                                ui.vertical_centered(|ui| {
                                    for (index, texture) in tab.pages.iter().enumerate() {
                                        let aspect = texture.size_vec2().y / texture.size_vec2().x;
                                        let size = egui::vec2(page_width, page_width * aspect);
                                        
                                        // Allocate space for the page and get interaction response
                                        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
                                        
                                        if ui.is_rect_visible(rect) {
                                            // Draw solid white background for the page behind everything
                                            ui.painter().rect_filled(rect, 0.0, egui::Color32::WHITE);
                                        }
                                        
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
                                                                egui::Color32::from_rgba_unmultiplied(66, 165, 245, 80), // elegant semi-transparent blue
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        // Draw Y-axis-oriented yellow highlights overlay on matched words inside the page (fully legible soft yellow)
                                        if !self.search_query.is_empty() && index < tab.page_chars.len() {
                                            let query_lower = self.search_query.to_lowercase();
                                            
                                            // Reconstruct text string from characters
                                            let page_string: String = tab.page_chars[index].iter().map(|char_info| char_info.c).collect();
                                            let page_string_lower = page_string.to_lowercase();
                                            
                                            let mut start = 0;
                                            while let Some(pos) = page_string_lower[start..].find(&query_lower) {
                                                let absolute_pos = start + pos;
                                                
                                                // Draw yellow rectangles for all non-whitespace match indices
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
                                                                egui::Color32::from_rgba_unmultiplied(255, 255, 0, 75), // soft transparent yellow highlight for perfect text visibility
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
                                        
                                        if Some(index) == tab.scroll_to_page {
                                            response.scroll_to_me(Some(egui::Align::Center));
                                            scrolled = true;
                                        }
                                        
                                        ui.add_space(15.0);
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
                    if self.loading {
                        ui.label("Loading PDF...");
                    } else {
                        ui.label("Open a PDF file to start viewing.");
                    }
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

        // Handle zoom with Cmd/Ctrl + Mouse Wheel (command includes command key on macOS, Ctrl key on Windows/Linux)
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
        let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Untitled".to_string());
        
        let Some(pdfium) = &self.pdfium else {
            let state = PdfDocumentState {
                file_name,
                pages: Vec::new(),
                page_texts: Vec::new(),
                page_chars: Vec::new(),
                page_links: Vec::new(),
                zoom: 0.0,
                selected_page: 0,
                scroll_to_page: None,
                error: Some("PDFium not initialized. Please ensure libpdfium is present.".to_string()),
            };
            self.tabs.push(state);
            self.active_tab_index = Some(self.tabs.len() - 1);
            return;
        };

        self.loading = true;

        match pdfium.load_pdf_from_file(path.to_str().unwrap_or_default(), None) {
            Ok(doc) => {
                let render_config = PdfRenderConfig::new()
                    .set_target_width(2400) // High-resolution width for rendering crisp pages
                    .set_clear_color(PdfColor::new(255, 255, 255, 0)); // Transparent background

                let mut pages = Vec::new();
                let mut page_texts = Vec::new();
                let mut page_chars = Vec::new();
                let mut page_links = Vec::new();
                for (index, page) in doc.pages().iter().enumerate() {
                    // Extract text
                    let page_text = page.text().map(|t| t.all()).unwrap_or_default();
                    page_texts.push(page_text);

                    // Extract characters and normalized coordinates
                    let page_w = page.width().value;
                    let page_h = page.height().value;
                    let mut chars_list = Vec::new();
                    if let Ok(text) = page.text() {
                        for c in text.chars().iter() {
                            if let Ok(bounds) = c.loose_bounds() {
                                chars_list.push(PdfCharInfo {
                                    c: c.unicode_string().as_deref().and_then(|s| s.chars().next()).unwrap_or(' '),
                                    left: bounds.left().value / page_w,
                                    right: bounds.right().value / page_w,
                                    top: 1.0 - (bounds.top().value / page_h),
                                    bottom: 1.0 - (bounds.bottom().value / page_h),
                                });
                            }
                        }
                    }
                    page_chars.push(chars_list);

                    // Extract links
                    let mut links_list = Vec::new();
                    let links = page.links();
                    for link in links.iter() {
                            if let Ok(rect) = link.rect() {
                                let left = rect.left().value / page_w;
                                let right = rect.right().value / page_w;
                                let top = 1.0 - (rect.top().value / page_h);
                                let bottom = 1.0 - (rect.bottom().value / page_h);
                                
                                let mut target = None;
                                
                                if let Some(action) = link.action() {
                                    if let Some(uri) = action.as_uri_action() {
                                        if let Ok(uri_str) = uri.uri() {
                                            target = Some(PdfLinkTarget::Url(uri_str));
                                        }
                                    } else if let Some(local_dest) = action.as_local_destination_action() {
                                        if let Ok(dest) = local_dest.destination() {
                                            if let Ok(page_idx) = dest.page_index() {
                                                target = Some(PdfLinkTarget::Page(page_idx as usize));
                                            }
                                        }
                                    }
                                } else if let Some(dest) = link.destination() {
                                    if let Ok(page_idx) = dest.page_index() {
                                        target = Some(PdfLinkTarget::Page(page_idx as usize));
                                    }
                                }
                                
                                if let Some(t) = target {
                                    links_list.push(PdfLinkInfo {
                                        left,
                                        right,
                                        top,
                                        bottom,
                                        target: t,
                                    });
                                }
                            }
                        }
                    page_links.push(links_list);

                    // Render texture
                    if let Ok(bitmap) = page.render_with_config(&render_config) {
                        let image = bitmap.as_image();
                        let rgba = image.to_rgba8();
                        let pixels = rgba.as_flat_samples();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [rgba.width() as usize, rgba.height() as usize],
                            pixels.as_slice(),
                        );
                        let texture = ctx.load_texture(
                            format!("tab_{}_page_{}", self.tabs.len(), index),
                            color_image,
                            Default::default(),
                        );
                        pages.push(texture);
                    }
                }
                
                let state = PdfDocumentState {
                    file_name,
                    pages,
                    page_texts,
                    page_chars,
                    page_links,
                    zoom: 0.0,
                    selected_page: 0,
                    scroll_to_page: None,
                    error: None,
                };
                self.tabs.push(state);
                self.active_tab_index = Some(self.tabs.len() - 1);
                self.loading = false;
            }
            Err(e) => {
                let state = PdfDocumentState {
                    file_name,
                    pages: Vec::new(),
                    page_texts: Vec::new(),
                    page_chars: Vec::new(),
                    page_links: Vec::new(),
                    zoom: 0.0,
                    selected_page: 0,
                    scroll_to_page: None,
                    error: Some(format!("Failed to load PDF: {}", e)),
                };
                self.tabs.push(state);
                self.active_tab_index = Some(self.tabs.len() - 1);
                self.loading = false;
            }
        }
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
}
