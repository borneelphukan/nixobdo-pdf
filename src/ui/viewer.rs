use crate::app::PdfViewerApp;
use crate::document::{PageLayoutMode, PdfLinkTarget, find_closest_char, is_char_selected};
use eframe::egui;

impl PdfViewerApp {
    pub(crate) fn ui_viewer(&mut self, ctx: &egui::Context) {
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

                                            // Image rendering is moved to the bottom                                            // Draw blue text selection overlays
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
