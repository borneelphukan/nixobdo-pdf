use crate::app::NixobdoPdfApp;
use eframe::egui;

impl NixobdoPdfApp {
    pub(crate) fn ui_toolbar(&mut self, ui: &mut egui::Ui) {
        let has_search_modifier = ui.ctx().input(|i| i.modifiers.command || i.modifiers.ctrl);

        // Pre-calculate search matches
        let mut match_pages = Vec::new();
        if let Some(active_idx) = self.active_tab_index {
            if let Some(tab) = self.tabs.get(active_idx) {
                if !self.search_query.is_empty() {
                    let query_chars: Vec<char> = self.search_query.to_lowercase().chars().collect();
                    for (page_idx, page_chars) in tab.page_chars.iter().enumerate() {
                        let page_chars_lower: Vec<char> = page_chars
                            .iter()
                            .map(|char_info| {
                                char_info.c.to_lowercase().next().unwrap_or(char_info.c)
                            })
                            .collect();

                        let mut i = 0;
                        while i + query_chars.len() <= page_chars_lower.len()
                            && !query_chars.is_empty()
                        {
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

        egui::Panel::top("toolbar_panel").show(ui, |ui| {
            // Increase fonts and padding for the toolbar
            ui.style_mut()
                .text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(13.0));
            ui.style_mut()
                .text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(13.0));
            ui.style_mut().spacing.button_padding = egui::vec2(8.0, 6.0);

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let has_active_tab = self.active_tab_index.is_some();

                ui.add_enabled_ui(has_active_tab, |ui| {
                    let mut zoom_out = false;
                    let mut zoom_in = false;
                    let mut zoom_reset = false;
                    let mut page_up = false;
                    let mut page_down = false;
                    let mut rotate_left = false;
                    let mut rotate_right = false;

                    let page_disp = if let Some(active_idx) = self.active_tab_index {
                        if let Some(tab) = self.tabs.get(active_idx) {
                            format!("{}/{}", tab.selected_page + 1, tab.pages.len().max(1))
                        } else {
                            "0/0".to_string()
                        }
                    } else {
                        "0/0".to_string()
                    };

                    if ui.button("🔍-").on_hover_text("Zoom Out").clicked() {
                        zoom_out = true;
                    }

                    let mut current_zoom = if let Some(active_idx) = self.active_tab_index {
                        if let Some(tab) = self.tabs.get(active_idx) {
                            100.0 + tab.zoom
                        } else {
                            100.0
                        }
                    } else {
                        100.0
                    };
                    let zoom_response = ui.add_enabled(
                        has_active_tab,
                        egui::DragValue::new(&mut current_zoom)
                            .speed(1.0)
                            .max_decimals(0)
                            .suffix("%")
                            .range(0.0..=1000.0),
                    );

                    if ui.button("🔍+").on_hover_text("Zoom In").clicked() {
                        zoom_in = true;
                    }
                    if ui.button("Reset").clicked() {
                        zoom_reset = true;
                    }

                    ui.separator();

                    if ui.button("⬆").clicked()
                        || ui.input(|i| {
                            i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::ArrowLeft)
                        })
                    {
                        page_up = true;
                    }
                    ui.label(page_disp);
                    if ui.button("⬇").clicked()
                        || ui.input(|i| {
                            i.key_pressed(egui::Key::ArrowDown)
                                || i.key_pressed(egui::Key::ArrowRight)
                        })
                    {
                        page_down = true;
                    }

                    ui.separator();

                    if ui.button("⟲").clicked() {
                        rotate_left = true;
                    }
                    if ui.button("⟳").clicked() {
                        rotate_right = true;
                    }

                    ui.separator();

                    let is_fullscreen = ui.ctx().input(|i| i.viewport().fullscreen.unwrap_or(false));
                    let tooltip = if is_fullscreen { "Exit Fullscreen" } else { "Fullscreen" };
                    let fullscreen_icon = if is_fullscreen {
                        egui::Image::new(egui::include_image!("../../assets/icons/exit_fullscreen.svg"))
                    } else {
                        egui::Image::new(egui::include_image!("../../assets/icons/fullscreen.svg"))
                    };
                    if ui.add(egui::Button::image(fullscreen_icon.tint(ui.visuals().text_color()).max_height(16.0).max_width(16.0))).on_hover_text(tooltip).clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(!is_fullscreen));
                    }

                    if has_active_tab {
                        if let Some(active_idx) = self.active_tab_index {
                            if let Some(tab) = self.tabs.get_mut(active_idx) {
                                let step = if tab.layout_mode
                                    == crate::document::PageLayoutMode::TwoPage
                                {
                                    2
                                } else {
                                    1
                                };
                                if zoom_out {
                                    tab.zoom = (tab.zoom - 10.0).max(0.0);
                                } else if zoom_in {
                                    tab.zoom += 10.0;
                                } else if zoom_reset {
                                    tab.zoom = 0.0;
                                } else if zoom_response.changed() {
                                    tab.zoom = (current_zoom - 100.0).max(0.0);
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

                                if rotate_left || rotate_right {
                                    let d = if rotate_left { -90 } else { 90 };
                                    if tab.selected_page < tab.page_rotations.len() {
                                        tab.page_rotations[tab.selected_page] =
                                            (tab.page_rotations[tab.selected_page] + d)
                                                .rem_euclid(360);
                                    }
                                    if tab.layout_mode == crate::document::PageLayoutMode::TwoPage
                                        && tab.selected_page + 1 < tab.page_rotations.len()
                                    {
                                        tab.page_rotations[tab.selected_page + 1] =
                                            (tab.page_rotations[tab.selected_page + 1] + d)
                                                .rem_euclid(360);
                                    }
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
                    let sidebar_icon = egui::Image::new(egui::include_image!("../../assets/icons/toggle_sidebar.svg"))
                        .tint(ui.visuals().text_color())
                        .max_height(16.0)
                        .max_width(16.0);
                    
                    let tooltip = if self.ai_chatbot_open { "Hide AI Panel" } else { "Open AI Panel" };
                    if ui.add(egui::Button::image(sidebar_icon)).on_hover_text(tooltip).clicked() {
                        self.ai_chatbot_open = !self.ai_chatbot_open;
                    }
                    
                    ui.separator();
                    
                    ui.add_enabled_ui(has_active_tab, |ui| {
                    if self.active_tab_index.is_some() {
                        if !self.search_query.is_empty() {
                            if ui.small_button("Clear").clicked() {
                                self.search_query.clear();
                                self.search_active_match = 0;
                            }
                            if !match_pages.is_empty() {
                                let display_match = self
                                    .search_active_match
                                    .min(match_pages.len().saturating_sub(1));
                                ui.label(
                                    egui::RichText::new(format!(
                                        "({}/{})",
                                        display_match + 1,
                                        match_pages.len()
                                    ))
                                    .size(12.0)
                                    .weak(),
                                );
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

                        if response.lost_focus()
                            && ui.ctx().input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            if !match_pages.is_empty() {
                                self.search_active_match =
                                    (self.search_active_match + 1) % match_pages.len();
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

                        if has_search_modifier && ui.ctx().input(|i| i.key_pressed(egui::Key::F)) {
                            response.request_focus();
                        }

                        ui.label("🔍 Find:");
                        
                        }
                    });
                });
            });
            ui.add_space(8.0);
        });

        if self.is_annotation_mode {
            egui::Panel::top("annotation_toolbar_panel").show(ui, |ui| {
                ui.style_mut().text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(12.0));
                ui.style_mut().text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(12.0));
                ui.style_mut().spacing.button_padding = egui::vec2(8.0, 6.0);

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Annotation Tools:").strong());
                    ui.add_space(8.0);

                    let tool_selected = self.active_annotation_tool;

                    if ui.add(egui::Button::new("T").selected(tool_selected == Some(crate::document::AnnotationTool::Text))).on_hover_text("Edit Text").clicked() {
                        self.is_annotation_mode = true;
                        self.active_annotation_tool = if tool_selected == Some(crate::document::AnnotationTool::Text) { None } else { Some(crate::document::AnnotationTool::Text) };
                        if self.active_annotation_tool.is_none() {
                            self.is_annotation_mode = false;
                        }
                    }
                    ui.add_space(4.0);

                    let highlight_selected = tool_selected == Some(crate::document::AnnotationTool::Highlight);
                    if ui.add(
                        egui::Button::image(
                            egui::Image::new(egui::include_image!("../../assets/icons/highlight.svg"))
                                .max_height(16.0)
                                .max_width(16.0)
                                .tint(ui.visuals().text_color())
                        )
                        .selected(highlight_selected)
                    ).on_hover_text("Highlight").clicked() {
                        self.active_annotation_tool = Some(crate::document::AnnotationTool::Highlight);
                    }

                    let underline_selected = tool_selected == Some(crate::document::AnnotationTool::Underline);
                    if ui.add(
                        egui::Button::image(
                            egui::Image::new(egui::include_image!("../../assets/icons/underline.svg"))
                                .max_height(16.0)
                                .max_width(16.0)
                                .tint(ui.visuals().text_color())
                        )
                        .selected(underline_selected)
                    ).on_hover_text("Underline").clicked() {
                        self.active_annotation_tool = Some(crate::document::AnnotationTool::Underline);
                    }

                    let strike_selected = tool_selected == Some(crate::document::AnnotationTool::Strikethrough);
                    if ui.add(
                        egui::Button::image(
                            egui::Image::new(egui::include_image!("../../assets/icons/strikethrough.svg"))
                                .max_height(16.0)
                                .max_width(16.0)
                                .tint(ui.visuals().text_color())
                        )
                        .selected(strike_selected)
                    ).on_hover_text("Strikethrough").clicked() {
                        self.active_annotation_tool = Some(crate::document::AnnotationTool::Strikethrough);
                    }

                    let redact_selected = tool_selected == Some(crate::document::AnnotationTool::Redact);
                    if ui.add(
                        egui::Button::image(
                            egui::Image::new(egui::include_image!("../../assets/icons/redact.svg"))
                                .max_height(16.0)
                                .max_width(16.0)
                                .tint(ui.visuals().text_color())
                        )
                        .selected(redact_selected)
                    ).on_hover_text("Redact").clicked() {
                        self.active_annotation_tool = Some(crate::document::AnnotationTool::Redact);
                    }

                    // Unselect if clicked again
                    if self.active_annotation_tool == tool_selected && ui.input(|i| i.pointer.any_click()) {
                        // Handled by standard selectable_value logic above but we can also add escape logic in central_panel
                    }

                    let is_text_tool_active = self.active_annotation_tool == Some(crate::document::AnnotationTool::Text);
                    let has_pending_text = self.pending_annotations.iter().any(|a| a.tool == crate::document::AnnotationTool::Text);
                    let is_highlight_tool_active = self.active_annotation_tool == Some(crate::document::AnnotationTool::Highlight);
                    let has_pending_highlight = self.pending_annotations.iter().any(|a| a.tool == crate::document::AnnotationTool::Highlight);

                    if is_text_tool_active || has_pending_text || is_highlight_tool_active || has_pending_highlight {
                        ui.separator();

                        let mut size_changed = false;
                        let mut style_changed = false;
                        let mut color_changed = false;

                        if is_text_tool_active || has_pending_text {
                            let allowed_sizes = [
                                6.0, 7.0, 8.0, 9.0, 10.0, 10.5, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
                                18.0, 20.0, 21.0, 22.0, 24.0, 26.0, 28.0, 32.0, 36.0, 40.0, 42.0,
                                44.0, 48.0, 54.0, 60.0, 66.0, 72.0, 80.0, 88.0, 96.0
                            ];

                            egui::ComboBox::new("text_size_dropdown", "")
                                .selected_text(format!("{} px", self.text_annotation_size))
                                .show_ui(ui, |ui| {
                                    for &size in &allowed_sizes {
                                        if ui.selectable_value(&mut self.text_annotation_size, size, format!("{} px", size)).changed() {
                                            size_changed = true;
                                        }
                                    }
                                });

                            if ui.toggle_value(&mut self.text_annotation_bold, egui::RichText::new("B").strong()).on_hover_text("Bold").changed() {
                                style_changed = true;
                            }
                            if ui.toggle_value(&mut self.text_annotation_italic, egui::RichText::new("I").italics()).on_hover_text("Italic").changed() {
                                style_changed = true;
                            }
                            if ui.toggle_value(&mut self.text_annotation_underline, egui::RichText::new("U").underline()).on_hover_text("Underline").changed() {
                                style_changed = true;
                            }
                        }

                        let mut current_color = if is_text_tool_active || has_pending_text { self.text_annotation_color } else { self.annotation_color };
                        let icon_text = if is_text_tool_active || has_pending_text { "A" } else { "■" };

                        ui.menu_button(egui::RichText::new(icon_text).color(current_color).strong(), |ui| {
                            let mut predefined_colors = vec![
                                egui::Color32::BLACK,
                                egui::Color32::WHITE,
                                egui::Color32::LIGHT_GRAY,
                                egui::Color32::RED,
                                egui::Color32::from_rgb(255, 165, 0), // Orange
                                egui::Color32::from_rgb(255, 215, 0), // Yellow
                                egui::Color32::BLUE,
                                egui::Color32::from_rgb(128, 0, 128), // Purple
                                egui::Color32::GREEN,
                            ];

                            if is_highlight_tool_active || has_pending_highlight {
                                predefined_colors.retain(|&c| c != egui::Color32::BLACK);
                            }

                            egui::Grid::new("text_color_grid").num_columns(4).spacing([8.0, 8.0]).show(ui, |ui| {
                                for (i, &color) in predefined_colors.iter().enumerate() {
                                    let (rect, response) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::click());

                                    if ui.is_rect_visible(rect) {
                                        ui.painter().circle_filled(rect.center(), 10.0, color);
                                        ui.painter().circle_stroke(rect.center(), 10.0, egui::Stroke::new(1.0, egui::Color32::GRAY));
                                    }

                                    if response.clicked() {
                                        current_color = color;
                                        color_changed = true;
                                        ui.close();
                                    }

                                    if (i + 1) % 4 == 0 {
                                        ui.end_row();
                                    }
                                }

                                // Transparent
                                let (rect, response) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::click());
                                if ui.is_rect_visible(rect) {
                                    ui.painter().circle_filled(rect.center(), 10.0, egui::Color32::WHITE);
                                    ui.painter().circle_stroke(rect.center(), 10.0, egui::Stroke::new(1.0, egui::Color32::GRAY));
                                    ui.painter().line_segment([rect.left_bottom() + egui::vec2(4.0, -4.0), rect.right_top() + egui::vec2(-4.0, 4.0)], egui::Stroke::new(1.5, egui::Color32::RED));
                                }
                                if response.clicked() {
                                    current_color = egui::Color32::TRANSPARENT;
                                    color_changed = true;
                                    ui.close();
                                }

                                // Custom color button
                                let (rect, response) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::click());
                                if ui.is_rect_visible(rect) {
                                    let stroke_color = ui.visuals().text_color();
                                    ui.painter().circle_stroke(rect.center(), 10.0, egui::Stroke::new(1.0, stroke_color));
                                    ui.painter().line_segment([rect.center() + egui::vec2(-5.0, 0.0), rect.center() + egui::vec2(5.0, 0.0)], egui::Stroke::new(1.5, stroke_color));
                                    ui.painter().line_segment([rect.center() + egui::vec2(0.0, -5.0), rect.center() + egui::vec2(0.0, 5.0)], egui::Stroke::new(1.5, stroke_color));
                                }
                                if response.clicked() {
                                    self.is_custom_text_color_open = true;
                                    self.custom_text_color_temp = current_color;
                                    ui.close();
                                }
                            });
                        });

                        if color_changed {
                            if is_text_tool_active || has_pending_text {
                                self.text_annotation_color = current_color;
                            } else {
                                self.annotation_color = current_color;
                            }
                        }

                        if size_changed || style_changed || color_changed {
                            if is_text_tool_active || has_pending_text {
                                let target_index = self.active_text_annotation_index
                                    .or_else(|| self.pending_annotations.iter().enumerate().rev().find(|(_, a)| a.tool == crate::document::AnnotationTool::Text).map(|(i, _)| i));

                                if let Some(idx) = target_index {
                                    if let Some(action) = self.pending_annotations.get_mut(idx) {
                                        if size_changed {
                                            action.scale = Some(self.text_annotation_size);
                                        }
                                        if style_changed {
                                            action.bold = self.text_annotation_bold;
                                            action.italic = self.text_annotation_italic;
                                            action.underline = self.text_annotation_underline;
                                        }
                                        if color_changed {
                                            action.color = current_color;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    ui.separator();

                    if ui.add_enabled(!self.pending_annotations.is_empty(), egui::Button::new("Undo")).clicked() || ui.ctx().input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
                        if let Some(action) = self.pending_annotations.pop() {
                            self.redo_annotations.push(action);
                        }
                    }
                    if ui.add_enabled(!self.redo_annotations.is_empty(), egui::Button::new("Redo")).clicked() || ui.ctx().input(|i| i.modifiers.command && i.key_pressed(egui::Key::Y)) {
                        if let Some(action) = self.redo_annotations.pop() {
                            self.pending_annotations.push(action);
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            self.is_annotation_mode = false;
                            self.active_annotation_tool = None;
                            self.pending_annotations.clear();
                            self.redo_annotations.clear();
                        }

                        let can_save = !self.pending_annotations.is_empty() && self.active_tab_index.is_some() && !self.is_saving_annotations;
                        if ui.add_enabled(can_save, egui::Button::new(if self.is_saving_annotations { "Saving..." } else { "Save" })).clicked() {
                            let confirm = rfd::MessageDialog::new()
                                .set_title("Nixobdo PDF Reader")
                                .set_description("Annotation in a document is permanent. Are you sure you want to annotate the document ?")
                                .set_buttons(rfd::MessageButtons::YesNo)
                                .set_level(rfd::MessageLevel::Warning)
                                .show();

                            if confirm == rfd::MessageDialogResult::Yes {
                                if let Some(active_idx) = self.active_tab_index {
                                    if let Some(tab) = self.tabs.get(active_idx) {
                                        self.is_saving_annotations = true;
                                        let _ = self.pdf_task_tx.send(crate::worker::PdfWorkerTask::SaveAnnotations {
                                            path: tab.path.clone(),
                                            annotations: self.pending_annotations.clone(),
                                            ctx: ui.ctx().clone(),
                                        });
                                    }
                                }
                            }
                        }
                    });
                });
                ui.add_space(8.0);
            });
        }
    }
}
