use crate::app::NixobdoPdfApp;
use crate::document::PageLayoutMode;
use eframe::egui;

impl NixobdoPdfApp {
    pub(crate) fn ui_menu_bar(&mut self, ctx: &egui::Context) {
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
                    

                    
                    if ui.add_enabled(self.active_tab_index.is_some(), egui::Button::new("Export...")).clicked() {
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
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Edit", |ui| {
                    let has_pdf = self.active_tab_index.is_some();
                    ui.horizontal(|ui| {
                        if ui.add_enabled(has_pdf, egui::Button::new("Add Signature")).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg"])
                                .pick_file()
                            {
                                if let Ok(img) = image::open(&path) {
                                    let rgba = img.to_rgba8();
                                    let size = [rgba.width() as usize, rgba.height() as usize];
                                    let pixels = rgba.as_flat_samples();
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                                    
                                    self.signature_texture = Some(ctx.load_texture(
                                        "signature",
                                        color_image,
                                        egui::TextureOptions::LINEAR,
                                    ));
                                    self.signature_image_path = Some(path.clone());
                                    
                                    // Start placing immediately
                                    self.is_placing_signature = true;
                                    self.signature_position = Some((0.5, 0.5));
                                    self.signature_scale = 1.0;
                                    self.signature_active_page = self.active_tab_index.and_then(|idx| self.tabs.get(idx).map(|t| t.selected_page));
                                }
                            }
                            ui.close_menu();
                        }
                    });
                    
                    ui.separator();
                    
                    if ui.add_enabled(has_pdf, egui::Button::image_and_text(
                        egui::Image::new(egui::include_image!("../../assets/rotate_left.svg")).max_height(14.0).tint(ui.visuals().text_color()),
                        "Rotate Left"
                    )).clicked()
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
                        ui.close_menu();
                    }
                    
                    if ui.add_enabled(has_pdf, egui::Button::image_and_text(
                        egui::Image::new(egui::include_image!("../../assets/rotate_right.svg")).max_height(14.0).tint(ui.visuals().text_color()),
                        "Rotate Right"
                    )).clicked()
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
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("View", |ui| {
                    ui.set_min_width(220.0);
                    let sidebar_text = if self.sidebar_open { "✔ Show Sidebar" } else { "    Show Sidebar" };
                    if ui.button(sidebar_text).clicked() {
                        self.sidebar_open = !self.sidebar_open;
                        ui.close_menu();
                    }
                    
                    let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                    let fullscreen_text = if is_fullscreen { "✔ Fullscreen" } else { "    Fullscreen" };
                    if ui.button(fullscreen_text).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!is_fullscreen));
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
                ui.menu_button("Help", |ui| {
                    ui.set_min_width(220.0);
                    if ui.button("Check for Updates").clicked() {
                        self.update_state = crate::app::UpdateState::Checking;
                        let _ = self.pdf_task_tx.send(crate::worker::PdfWorkerTask::CheckUpdate { is_manual: true, ctx: ctx.clone() });
                        ui.close_menu();
                    }
                    if ui.button("About").clicked() {
                        self.about_window_open = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }
}
