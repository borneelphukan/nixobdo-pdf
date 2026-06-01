use crate::app::PdfViewerApp;
use crate::document::{PageLayoutMode, PdfLinkTarget, find_closest_char, is_char_selected};
use eframe::egui;

impl PdfViewerApp {
    pub(crate) fn ui_central_panel(&mut self, ctx: &egui::Context) {
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
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("⚠️").size(64.0));
                                ui.add_space(20.0);
                                ui.label(egui::RichText::new("Failed to Load Document").size(24.0).strong());
                                ui.add_space(10.0);
                                ui.label(error);
                            });
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
                        let ctrl_pressed = ui.input(|i| i.modifiers.ctrl);
                        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
                        let current_time = ui.input(|i| i.time);
                        
                        if ui.rect_contains_pointer(ui.max_rect()) {
                            if ctrl_pressed && scroll_delta != 0.0 {
                                if current_time - tab.last_page_change_time > 0.05 {
                                    if scroll_delta > 0.0 {
                                        tab.zoom += 10.0;
                                    } else {
                                        tab.zoom = (tab.zoom - 10.0).max(0.0);
                                    }
                                    tab.last_page_change_time = current_time;
                                }
                            } else if tab.layout_mode != PageLayoutMode::ContinuousScroll && scroll_delta != 0.0 {
                                if current_time - tab.last_page_change_time > 0.3 {
                                    let step = if tab.layout_mode == PageLayoutMode::TwoPage { 2 } else { 1 };
                                    if scroll_delta > 0.0 && tab.selected_page > 0 {
                                        tab.selected_page = tab.selected_page.saturating_sub(step);
                                        tab.scroll_to_page = Some(tab.selected_page);
                                        tab.last_page_change_time = current_time;
                                    } else if scroll_delta < 0.0 && tab.selected_page + step < tab.pages.len() {
                                        tab.selected_page += step;
                                        tab.scroll_to_page = Some(tab.selected_page);
                                        tab.last_page_change_time = current_time;
                                    } else if scroll_delta < 0.0 && tab.selected_page + 1 < tab.pages.len() {
                                        tab.selected_page += 1;
                                        tab.scroll_to_page = Some(tab.selected_page);
                                        tab.last_page_change_time = current_time;
                                    }
                                }
                            }
                        }
                        
                        let available_height_before_scroll = ui.available_height() - 20.0;
                        
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                if ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle)) {
                                    if let Some(press_origin) = ui.input(|i| i.pointer.press_origin()) {
                                        if let Some(current_pos) = ui.input(|i| i.pointer.interact_pos()) {
                                            let delta_y = current_pos.y - press_origin.y;
                                            
                                            // Apply a small deadzone
                                            if delta_y.abs() > 5.0 {
                                                // Exponential velocity scale for comfortable endless joystick scrolling
                                                let speed = (delta_y.abs() - 5.0).powf(1.15) * 0.08 * delta_y.signum();
                                                ui.scroll_with_delta(egui::vec2(0.0, -speed));
                                                ui.ctx().request_repaint(); // Keep repainting to allow endless scroll
                                            }
                                        }
                                    }
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::AllScroll);
                                }
                                
                                let available_width = ui.available_width() - 24.0;
                                let chunks = if tab.layout_mode == PageLayoutMode::TwoPage { 2 } else { 1 };
                                let page_width = if chunks == 2 { available_width * (1.0 + tab.zoom) / 2.0 - 10.0 } else { available_width * (1.0 + tab.zoom) };
                                
                                let mut scrolled = false;
                                let mut best_page = tab.selected_page;
                                let mut max_overlap_area = 0.0;
                                
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
                                        let mut total_row_width = 0.0;
                                        for &index in chunk {
                                            let texture_opt = &tab.pages[index];
                                            let aspect = texture_opt.as_ref().map(|t| t.size_vec2().y / t.size_vec2().x).unwrap_or(1.414);
                                            let fit_page_width = (available_height_before_scroll / aspect).min(available_width / chunks as f32);
                                            let fit_width_width = available_width / chunks as f32;
                                            let final_width = (fit_page_width + (tab.zoom / 50.0) * (fit_width_width - fit_page_width)).max(100.0);
                                            total_row_width += final_width;
                                        }
                                        total_row_width += (chunk.len().saturating_sub(1)) as f32 * 15.0;
                                        
                                        ui.horizontal_centered(|ui| {
                                            let extra_space = (ui.available_width() - total_row_width).max(0.0) / 2.0;
                                            ui.add_space(extra_space);
                                            
                                            for &index in chunk {
                                                let texture_opt = &tab.pages[index];
                                                // Default aspect ratio for A4
                                                let aspect = if let Some(texture) = texture_opt {
                                                    texture.size_vec2().y / texture.size_vec2().x
                                                } else {
                                                    1.414 
                                                };
                                                
                                                let fit_page_width = (available_height_before_scroll / aspect).min(available_width / chunks as f32);
                                                let fit_width_width = available_width / chunks as f32;
                                                let final_width = (fit_page_width + (tab.zoom / 50.0) * (fit_width_width - fit_page_width)).max(100.0);
                                                
                                                let size = egui::vec2(final_width, final_width * aspect);
                                                
                                                // Allocate space for the page and get interaction response
                                                let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
                                                
                                                let overlap = rect.intersect(ui.clip_rect());
                                                if overlap.is_positive() {
                                                    let area = overlap.width() * overlap.height();
                                                    if area > max_overlap_area {
                                                        max_overlap_area = area;
                                                        best_page = index;
                                                    }
                                                }
                                                
                                                if ui.is_rect_visible(rect) {
                                                    // Draw solid white background for the page behind everything
                                                    ui.painter().rect_filled(rect, 0.0, egui::Color32::WHITE);
                                                }
                                                
                                                if let Some(texture) = texture_opt {
                                                    // Handle drag/selection input on the page
                                                    if index < tab.page_chars.len() {
                                                        if response.drag_started_by(egui::PointerButton::Primary) {
                                                            if let Some(mouse_pos) = ctx.pointer_interact_pos() {
                                                                if let Some(char_idx) = find_closest_char(response.rect, mouse_pos, &tab.page_chars[index]) {
                                                                    self.selection_start = Some((index, char_idx));
                                                                    self.selection_end = Some((index, char_idx));
                                                                    self.is_selecting = true;
                                                                }
                                                            }
                                                        }
                                                        
                                                        if self.is_selecting && response.dragged_by(egui::PointerButton::Primary) {
                                                            if let Some(mouse_pos) = ctx.pointer_interact_pos() {
                                                                if let Some(char_idx) = find_closest_char(response.rect, mouse_pos, &tab.page_chars[index]) {
                                                                    self.selection_end = Some((index, char_idx));
                                                                }
                                                            }
                                                        }
                                                        
                                                        if self.is_selecting && response.drag_stopped_by(egui::PointerButton::Primary) {
                                                            self.is_selecting = false;
                                                        }
                                                        
                                                        if response.clicked() && !response.dragged_by(egui::PointerButton::Primary) {
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
                                                        let query_chars: Vec<char> = self.search_query.to_lowercase().chars().collect();
                                                        let page_chars_lower: Vec<char> = tab.page_chars[index].iter().map(|char_info| {
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
                                                                for char_idx in i..(i + query_chars.len()) {
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
                                                                i += query_chars.len();
                                                            } else {
                                                                i += 1;
                                                            }
                                                        }
                                                    }
                                                    
                                                    // Draw the actual PDF page image LAST so text is drawn cleanly ON TOP of highlights,
                                                    // while the transparent background lets the highlights show through.
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
                                
                                if tab.scroll_to_page.is_none() {
                                    tab.selected_page = best_page;
                                }
                                        
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
    }
}
