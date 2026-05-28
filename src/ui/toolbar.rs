use crate::app::PdfViewerApp;
use eframe::egui;

impl PdfViewerApp {
    pub(crate) fn ui_toolbar(&mut self, ctx: &egui::Context) {
        let has_search_modifier = ctx.input(|i| i.modifiers.command || i.modifiers.ctrl);

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

        egui::TopBottomPanel::top("toolbar_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let has_active_tab = self.active_tab_index.is_some();
                
                ui.add_enabled_ui(has_active_tab, |ui| {
                    let mut zoom_out = false;
                    let mut zoom_in = false;
                    let mut zoom_reset = false;
                    let mut page_up = false;
                    let mut page_down = false;
                    
                    let (zoom_disp, page_disp) = if let Some(active_idx) = self.active_tab_index {
                        if let Some(tab) = self.tabs.get(active_idx) {
                            (tab.zoom + 1.0, format!("{}/{}", tab.selected_page + 1, tab.pages.len().max(1)))
                        } else {
                            (1.0, "0/0".to_string())
                        }
                    } else {
                        (1.0, "0/0".to_string())
                    };

                    if ui.button("➖ Zoom Out").clicked() { zoom_out = true; }
                    ui.label(format!("{:.1}x", zoom_disp));
                    if ui.button("➕ Zoom In").clicked() { zoom_in = true; }
                    if ui.button("Reset").clicked() { zoom_reset = true; }
                    
                    ui.separator();
                    
                    if ui.button("⬆").clicked() { page_up = true; }
                    ui.label(page_disp);
                    if ui.button("⬇").clicked() { page_down = true; }
                    
                    if has_active_tab {
                        if let Some(active_idx) = self.active_tab_index {
                            if let Some(tab) = self.tabs.get_mut(active_idx) {
                                if zoom_out { tab.zoom = (tab.zoom - 0.1).max(0.0); }
                                if zoom_in { tab.zoom += 0.1; }
                                if zoom_reset { tab.zoom = 0.0; }
                                if page_up && tab.selected_page > 0 {
                                    tab.selected_page -= 1;
                                    tab.scroll_to_page = Some(tab.selected_page);
                                }
                                if page_down && tab.selected_page + 1 < tab.pages.len() {
                                    tab.selected_page += 1;
                                    tab.scroll_to_page = Some(tab.selected_page);
                                }
                            }
                        }
                    }
                });
                
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
    }
}
