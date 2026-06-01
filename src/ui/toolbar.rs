use crate::app::PdfViewerApp;
use eframe::egui;

impl PdfViewerApp {
    pub(crate) fn ui_toolbar(&mut self, ctx: &egui::Context) {
        let has_search_modifier = ctx.input(|i| i.modifiers.command || i.modifiers.ctrl);

        // Pre-calculate search matches
        let mut match_pages = Vec::new();
        if let Some(active_idx) = self.active_tab_index {
            if let Some(tab) = self.tabs.get(active_idx) {
                if !self.search_query.is_empty() {
                    let query_chars: Vec<char> = self.search_query.to_lowercase().chars().collect();
                    for (page_idx, page_chars) in tab.page_chars.iter().enumerate() {
                        let page_chars_lower: Vec<char> = page_chars.iter().map(|char_info| {
                            char_info.c.to_lowercase().next().unwrap_or(char_info.c)
                        }).collect();
                        
                        let mut i = 0;
                        while i + query_chars.len() <= page_chars_lower.len() && !query_chars.is_empty() {
                            let mut is_match = true;
                            for j in 0..query_chars.len() {
                                if page_chars_lower[i + j] != query_chars[j] {
                                    is_match = false;
                                    break;
                                }
                            }
                            if is_match {
                                match_pages.push(page_idx);
                                i += query_chars.len();
                            } else {
                                i += 1;
                            }
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
                    
                    let page_disp = if let Some(active_idx) = self.active_tab_index {
                        if let Some(tab) = self.tabs.get(active_idx) {
                            format!("{}/{}", tab.selected_page + 1, tab.pages.len().max(1))
                        } else {
                            "0/0".to_string()
                        }
                    } else {
                        "0/0".to_string()
                    };

                    if ui.button("➖ Zoom Out").clicked() { zoom_out = true; }
                    
                    let mut current_zoom = if let Some(active_idx) = self.active_tab_index {
                        if let Some(tab) = self.tabs.get(active_idx) {
                            tab.zoom
                        } else { 100.0 }
                    } else { 100.0 };
                    
                    let zoom_response = ui.add_enabled(
                        has_active_tab,
                        egui::DragValue::new(&mut current_zoom)
                            .speed(1.0)
                            .suffix("%")
                            .range(0.0..=1000.0)
                    );
                    
                    if ui.button("➕ Zoom In").clicked() { zoom_in = true; }
                    if ui.button("Reset").clicked() { zoom_reset = true; }
                    
                    ui.separator();
                    
                    if ui.button("⬆").clicked() || ui.input(|i| i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::ArrowLeft)) { page_up = true; }
                    ui.label(page_disp);
                    if ui.button("⬇").clicked() || ui.input(|i| i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::ArrowRight)) { page_down = true; }
                    
                    if has_active_tab {
                        if let Some(active_idx) = self.active_tab_index {
                            if let Some(tab) = self.tabs.get_mut(active_idx) {
                                let step = if tab.layout_mode == crate::document::PageLayoutMode::TwoPage { 2 } else { 1 };
                                if zoom_out { 
                                    tab.zoom = (tab.zoom - 10.0).max(0.0); 
                                } else if zoom_in { 
                                    tab.zoom += 10.0; 
                                } else if zoom_reset { 
                                    tab.zoom = 0.0; 
                                } else if zoom_response.changed() {
                                    tab.zoom = current_zoom;
                                }
                                if page_up && tab.selected_page > 0 {
                                    tab.selected_page = tab.selected_page.saturating_sub(step);
                                    tab.scroll_to_page = Some(tab.selected_page);
                                }
                                if page_down && tab.selected_page + step < tab.pages.len() {
                                    tab.selected_page += step;
                                    tab.scroll_to_page = Some(tab.selected_page);
                                } else if page_down && tab.selected_page + 1 < tab.pages.len() {
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
                                self.search_active_match = 0;
                            }
                            if !match_pages.is_empty() {
                                let display_match = self.search_active_match.min(match_pages.len().saturating_sub(1));
                                ui.label(egui::RichText::new(format!("({}/{})", display_match + 1, match_pages.len())).size(12.0).weak());
                            } else {
                                ui.label(egui::RichText::new("(0 matches)").size(12.0).weak());
                            }
                        }
                        
                        let text_edit = egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Search PDF... (Ctrl+F)")
                            .desired_width(150.0)
                            .id(egui::Id::new("search_bar"));
                        let response = ui.add(text_edit);
                        
                        if response.changed() {
                            self.search_active_match = 0;
                            // Immediately navigate to first match if available
                            if !match_pages.is_empty() {
                                if let Some(active_idx) = self.active_tab_index {
                                    if let Some(tab) = self.tabs.get_mut(active_idx) {
                                        let target_page = match_pages[0];
                                        tab.scroll_to_page = Some(target_page);
                                        tab.selected_page = target_page;
                                    }
                                }
                            }
                        }
                        
                        if response.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if !match_pages.is_empty() {
                                self.search_active_match = (self.search_active_match + 1) % match_pages.len();
                                if let Some(active_idx) = self.active_tab_index {
                                    if let Some(tab) = self.tabs.get_mut(active_idx) {
                                        let target_page = match_pages[self.search_active_match];
                                        tab.scroll_to_page = Some(target_page);
                                        tab.selected_page = target_page;
                                    }
                                }
                                response.request_focus(); // keep focus to allow rapid pressing
                            }
                        }
                        
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
