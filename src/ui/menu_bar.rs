use crate::app::NixobdoPdfApp;
use crate::document::PageLayoutMode;
use eframe::egui;

impl NixobdoPdfApp {
    pub(crate) fn ui_menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("menu_bar_panel").show(ui, |ui| {
            // Apply larger padding specifically for menu buttons to increase hover surface area
            ui.style_mut()
                .text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(13.0));
            ui.style_mut()
                .text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(13.0));
            ui.style_mut().spacing.button_padding = egui::vec2(16.0, 10.0);
            ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 0.0);

            ui.add_space(4.0);
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Some(paths) = rfd::FileDialog::new()
                            .add_filter("PDF files", &["pdf"])
                            .pick_files()
                        {
                            for path in paths {
                                self.load_pdf(ui.ctx(), path);
                            }
                        }
                        ui.close();
                    }

                    // Open Recent nested menu
                    ui.menu_button("Open Recent", |ui| {
                        if self.recent_files.is_empty() {
                            ui.label(egui::RichText::new("No recent files").weak());
                        } else {
                            let mut to_open = None;
                            for recent_path in &self.recent_files {
                                let name = recent_path
                                    .file_name()
                                    .map(|n| n.to_string_lossy())
                                    .unwrap_or_default();
                                if ui.button(name).clicked() {
                                    to_open = Some(recent_path.clone());
                                }
                            }
                            if let Some(path) = to_open {
                                self.load_pdf(ui.ctx(), path);
                                ui.close();
                            }
                        }
                    });

                    ui.separator();

                    if ui.button("Close Window").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    if ui
                        .add_enabled(
                            self.active_tab_index.is_some(),
                            egui::Button::new("Close PDF"),
                        )
                        .clicked()
                    {
                        if let Some(active_idx) = self.active_tab_index {
                            self.close_tab(active_idx);
                        }
                        ui.close();
                    }

                    if ui
                        .add_enabled(self.active_tab_index.is_some(), egui::Button::new("Rename"))
                        .clicked()
                    {
                        if let Some(active_idx) = self.active_tab_index {
                            if let Some(tab) = self.tabs.get(active_idx) {
                                self.rename_buffer = tab.file_name.clone();
                                self.rename_window_open = true;
                                self.focus_rename_input = true;
                            }
                        }
                        ui.close();
                    }

                    if ui
                        .add_enabled(
                            self.active_tab_index.is_some(),
                            egui::Button::new("Export..."),
                        )
                        .clicked()
                    {
                        if let Some(active_idx) = self.active_tab_index {
                            if let Some(tab) = self.tabs.get(active_idx) {
                                let name = if tab.file_name.to_lowercase().ends_with(".pdf") {
                                    tab.file_name[..tab.file_name.len() - 4].to_string()
                                } else {
                                    tab.file_name.clone()
                                }
                                .into();
                                self.export_name = name;
                                self.export_window_open = true;
                            }
                        }
                        ui.close();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    let has_pdf = self.active_tab_index.is_some();
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(has_pdf, egui::Button::new("Add Signature"))
                            .clicked()
                        {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg"])
                                .pick_file()
                            {
                                if let Ok(img) = image::open(&path) {
                                    let rgba = img.to_rgba8();
                                    let size = [rgba.width() as usize, rgba.height() as usize];
                                    let pixels = rgba.as_flat_samples();
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                        size,
                                        pixels.as_slice(),
                                    );

                                    self.signature_texture = Some(ui.ctx().load_texture(
                                        "signature",
                                        color_image,
                                        egui::TextureOptions::LINEAR,
                                    ));
                                    self.signature_image_path = Some(path.clone());

                                    // Start placing immediately
                                    self.is_placing_signature = true;
                                    self.signature_position = Some((0.5, 0.5));
                                    self.signature_scale = 1.0;
                                    self.signature_active_page =
                                        self.active_tab_index.and_then(|idx| {
                                            self.tabs.get(idx).map(|t| t.selected_page)
                                        });
                                }
                            }
                            ui.close();
                        }
                    });

                    let annot_text = if self.is_annotation_mode {
                        "✔ Annotation"
                    } else {
                        "    Annotation"
                    };
                    if ui
                        .add_enabled(has_pdf, egui::Button::new(annot_text))
                        .clicked()
                    {
                        self.is_annotation_mode = !self.is_annotation_mode;
                        if !self.is_annotation_mode {
                            // Reset state when turning off
                            self.active_annotation_tool = None;
                            self.pending_annotations.clear();
                            self.redo_annotations.clear();
                        }
                        ui.close();
                    }

                    ui.separator();

                    ui.add_enabled_ui(has_pdf, |ui| {
                        ui.menu_button("Rotate PDF", |ui| {
                            if ui
                                .add(egui::Button::image_and_text(
                                    egui::Image::new(egui::include_image!(
                                        "../../assets/icons/rotate_left.svg"
                                    ))
                                    .max_height(14.0),
                                    "Rotate Left",
                                ))
                                .clicked()
                            {
                                if let Some(active_idx) = self.active_tab_index {
                                    if let Some(tab) = self.tabs.get_mut(active_idx) {
                                        self.pending_rotation -= 90;
                                        for rot in &mut tab.page_rotations {
                                            *rot -= 90;
                                        }
                                        self.is_rotating_document = true;
                                    }
                                }
                                ui.close();
                            }

                            if ui
                                .add(egui::Button::image_and_text(
                                    egui::Image::new(egui::include_image!(
                                        "../../assets/icons/rotate_right.svg"
                                    ))
                                    .max_height(14.0),
                                    "Rotate Right",
                                ))
                                .clicked()
                            {
                                if let Some(active_idx) = self.active_tab_index {
                                    if let Some(tab) = self.tabs.get_mut(active_idx) {
                                        self.pending_rotation += 90;
                                        for rot in &mut tab.page_rotations {
                                            *rot += 90;
                                        }
                                        self.is_rotating_document = true;
                                    }
                                }
                                ui.close();
                            }
                        });
                    });
                });

                ui.menu_button("View", |ui| {
                    ui.set_min_width(220.0);
                    let sidebar_text = if self.sidebar_open {
                        "✔ Show Sidebar"
                    } else {
                        "    Show Sidebar"
                    };
                    if ui.button(sidebar_text).clicked() {
                        self.sidebar_open = !self.sidebar_open;
                        ui.close();
                    }
                    let utility_bar_text = if self.show_utility_bar {
                        "✔ Show Utility Bar"
                    } else {
                        "    Show Utility Bar"
                    };
                    if ui.button(utility_bar_text).clicked() {
                        self.show_utility_bar = !self.show_utility_bar;
                        ui.close();
                    }

                    let is_fullscreen =
                        ui.ctx().input(|i| i.viewport().fullscreen.unwrap_or(false));
                    let fullscreen_text = if is_fullscreen {
                        "✔ Fullscreen"
                    } else {
                        "    Fullscreen"
                    };
                    if ui.button(fullscreen_text).clicked() {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Fullscreen(!is_fullscreen));
                        ui.close();
                    }

                    ui.separator();

                    if let Some(active_idx) = self.active_tab_index {
                        if let Some(tab) = self.tabs.get_mut(active_idx) {
                            let cont_text = if tab.layout_mode == PageLayoutMode::ContinuousScroll {
                                "✔ Scroll Mode"
                            } else {
                                "   Scroll Mode"
                            };
                            if ui.button(cont_text).clicked() {
                                tab.layout_mode = PageLayoutMode::ContinuousScroll;
                                ui.close();
                            }

                            let single_text = if tab.layout_mode == PageLayoutMode::SinglePage {
                                "✔ Single Page"
                            } else {
                                "    Single Page"
                            };
                            if ui.button(single_text).clicked() {
                                tab.layout_mode = PageLayoutMode::SinglePage;
                                ui.close();
                            }

                            let two_text = if tab.layout_mode == PageLayoutMode::TwoPage {
                                "✔ Two Page"
                            } else {
                                "    Two Page"
                            };
                            if ui.button(two_text).clicked() {
                                tab.layout_mode = PageLayoutMode::TwoPage;
                                ui.close();
                            }
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("    Scroll Mode"));
                        ui.add_enabled(false, egui::Button::new("    Single Page"));
                        ui.add_enabled(false, egui::Button::new("    Two Page"));
                    }
                });
                ui.menu_button("Help", |ui| {
                    ui.set_min_width(220.0);
                    if ui.button("Check for Updates").clicked() {
                        self.update_state = crate::app::UpdateState::Checking;
                        let _ = self
                            .pdf_task_tx
                            .send(crate::worker::PdfWorkerTask::CheckUpdate {
                                is_manual: true,
                                ctx: ui.ctx().clone(),
                            });
                        ui.close();
                    }
                    if ui.button("About").clicked() {
                        self.about_window_open = true;
                        ui.close();
                    }
                });
            });
            ui.add_space(4.0);
        });
    }
}
