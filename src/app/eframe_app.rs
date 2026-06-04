use crate::app::{NixobdoPdfApp, UpdateState};
use crate::worker::PdfWorkerTask;
use crate::document::PdfWorkerMessage;
use eframe::egui;
use std::sync::atomic::Ordering;

impl eframe::App for NixobdoPdfApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Splash screen logic
        let elapsed = self.startup_time.elapsed().as_secs_f32();
        if elapsed < 2.5 {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(ctx.style().visuals.window_fill()))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() / 2.0 - 100.0);
                        
                        ui.add(egui::Image::new(egui::include_image!("../../assets/logo.svg")).max_width(250.0).max_height(250.0));
                        
                        ui.add_space(20.0);
                        ui.heading(egui::RichText::new("Nixobdo PDF Reader").size(28.0).strong());
                    });
                });
            
            ctx.request_repaint();
            return;
        }
        
        if !self.has_checked_for_updates {
            self.has_checked_for_updates = true;
            let _ = self.pdf_task_tx.send(PdfWorkerTask::CheckUpdate { is_manual: false, ctx: ctx.clone() });
        }

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
                                } else {
                                    rfd::MessageDialog::new()
                                        .set_title("Nixobdo PDF Reader")
                                        .set_description(&format!("Nixobdo PDF Reader could not open '{}' because it is either not a supported file type or because the file has been damaged.", file_name))
                                        .set_level(rfd::MessageLevel::Info)
                                        .show();
                                }
                                tab_to_remove = Some(i);
                            } else {
                                tab.file_name = file_name;
                                tab.pages = vec![None; page_count];
                                tab.thumbnails = vec![None; page_count];
                                tab.page_texts = vec![String::new(); page_count];
                                tab.page_chars = vec![Vec::new(); page_count];
                                tab.page_links = vec![Vec::new(); page_count];
                                tab.page_rotations = vec![0; page_count];
                                tab.is_loading = false; // Turn off main loading, pages will pop in
                            }
                            break;
                        }
                    }
                    if let Some(idx) = tab_to_remove {
                        self.close_tab(idx);
                    }
                }
                PdfWorkerMessage::PageData { path, index, image, thumbnail_image, text, chars, links } => {
                    for tab in self.tabs.iter_mut() {
                        if tab.path == path {
                            if index < tab.pages.len() {
                                let texture = ctx.load_texture(
                                    format!("doc_{}_page_{}", tab.file_name, index),
                                    image,
                                    egui::TextureOptions::LINEAR,
                                );
                                let thumb_texture = ctx.load_texture(
                                    format!("doc_{}_thumb_{}", tab.file_name, index),
                                    thumbnail_image,
                                    egui::TextureOptions::LINEAR,
                                );
                                tab.pages[index] = Some(texture);
                                tab.thumbnails[index] = Some(thumb_texture);
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
                PdfWorkerMessage::SignatureSaved { path } => {
                    self.toast_message = Some("Signature added successfully".to_string());
                    self.toast_success = true;
                    self.toast_timer = ctx.input(|i| i.time) + 4.0;
                    self.is_placing_signature = false;
                    self.is_saving_signature = false;
                    self.reload_pdf(ctx, path);
                }

                PdfWorkerMessage::UpdateCheckResult(is_available, version, is_manual) => {
                    if is_available {
                        self.update_state = UpdateState::Prompt(version.unwrap_or_else(|| "unknown".into()));
                    } else {
                        self.update_state = UpdateState::None;
                        if is_manual {
                            rfd::MessageDialog::new()
                                .set_title("No Update")
                                .set_description("No update available.")
                                .set_level(rfd::MessageLevel::Warning)
                                .show();
                        }
                    }
                }
                PdfWorkerMessage::UpdateDownloadProgress(progress) => {
                    self.update_state = UpdateState::Downloading(progress);
                }
                PdfWorkerMessage::UpdateDownloadComplete(result) => {
                    self.update_state = UpdateState::None;
                    match result {
                        Ok(path) => {
                            if let Err(e) = std::process::Command::new(&path).spawn() {
                                self.toast_message = Some(format!("Failed to start installer: {}", e));
                                self.toast_success = false;
                                self.toast_timer = ctx.input(|i| i.time) + 4.0;
                            } else {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                std::process::exit(0);
                            }
                        }
                        Err(e) => {
                            self.toast_message = Some(format!("Download failed: {}", e));
                            self.toast_success = false;
                            self.toast_timer = ctx.input(|i| i.time) + 4.0;
                        }
                    }
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
                        let rect = ui.available_rect_before_wrap();
                        let size = egui::vec2(rect.width(), 20.0);
                        let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
                        let corner_radius = egui::CornerRadius::same(4);
                        ui.painter().rect_filled(rect, corner_radius, ui.visuals().extreme_bg_color);
                        let fill_width = rect.width() * progress;
                        if fill_width > 0.0 {
                            let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, rect.height()));
                            ui.painter().rect_filled(fill_rect, corner_radius, ui.visuals().selection.bg_fill);
                        }
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{:.0}%", progress * 100.0),
                            egui::FontId::proportional(14.0),
                            ui.visuals().text_color(),
                        );
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
        match self.update_state.clone() {
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
            UpdateState::Prompt(version) => {
                egui::Area::new(egui::Id::new("update_banner"))
                    .anchor(egui::Align2::CENTER_TOP, [0.0, 10.0])
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        egui::Frame::window(&ctx.style())
                            .fill(egui::Color32::from_rgb(0, 120, 215))
                            .corner_radius(egui::CornerRadius::same(6))
                            .inner_margin(egui::Margin::symmetric(16, 10))
                            .shadow(egui::epaint::Shadow {
                                offset: [0, 4],
                                blur: 16,
                                spread: 0,
                                color: egui::Color32::from_black_alpha(80),
                            })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("New update available: v{}", version)).color(egui::Color32::WHITE).strong());
                                    ui.add_space(20.0);
                                    
                                    ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40);
                                    ui.visuals_mut().widgets.hovered.bg_fill = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 80);
                                    ui.visuals_mut().widgets.active.bg_fill = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 120);
                                    
                                    if ui.button(egui::RichText::new("Download Now").color(egui::Color32::WHITE)).clicked() {
                                        self.update_state = UpdateState::Downloading(0.0);
                                        let _ = self.pdf_task_tx.send(PdfWorkerTask::DownloadUpdate { version: version.clone(), ctx: ctx.clone() });
                                    }
                                    if ui.button(egui::RichText::new("Skip").color(egui::Color32::WHITE)).clicked() {
                                        self.update_state = UpdateState::None;
                                    }
                                });
                            });
                    });
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
                            let rect = ui.available_rect_before_wrap();
                            let size = egui::vec2(rect.width(), 20.0);
                            let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
                            let corner_radius = egui::CornerRadius::same(4);
                            ui.painter().rect_filled(rect, corner_radius, ui.visuals().extreme_bg_color);
                            let fill_width = rect.width() * progress;
                            if fill_width > 0.0 {
                                let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, rect.height()));
                                ui.painter().rect_filled(fill_rect, corner_radius, ui.visuals().selection.bg_fill);
                            }
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("{:.0}%", progress * 100.0),
                                egui::FontId::proportional(14.0),
                                ui.visuals().text_color(),
                            );
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
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("{} - nixobdo-pdf", tab.file_name)));
            }
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title("nixobdo-pdf".to_string()));
        }

        // Handle Ctrl+F / Cmd+F to focus search
        let has_ctrl_modifier = ctx.input(|i| i.modifiers.command || i.modifiers.ctrl);
        if has_ctrl_modifier && ctx.input(|i| i.key_pressed(egui::Key::F)) {
            ctx.memory_mut(|mem| mem.request_focus(egui::Id::new("search_bar")));
        }


        let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        if is_fullscreen && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        }

        if !is_fullscreen {
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
        }

        // Rename & Export Window Popups
        self.ui_dialogs(ctx);

        self.ui_viewer(ctx);
        
        let mut about_open = self.about_window_open;
        if about_open {
            egui::Window::new("About")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .open(&mut about_open)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new("Nixobdo PDF Reader").size(24.0).strong());
                        ui.add_space(12.0);
                        ui.label(egui::RichText::new("Developer: Borneel Bikash Phukan").size(16.0));
                        ui.label(egui::RichText::new(format!("Version: {}", env!("CARGO_PKG_VERSION"))).size(14.0));
                        ui.add_space(16.0);
                        ui.hyperlink_to(
                            egui::RichText::new("🌐 http://borneelphukan.com/").size(14.0),
                            "http://borneelphukan.com/"
                        );
                        ui.add_space(8.0);
                        ui.hyperlink_to(
                            egui::RichText::new("🤝 Contribute on GitHub").size(14.0),
                            "https://github.com/borneelphukan/nixobdo-pdf"
                        );
                        ui.add_space(10.0);
                    });
                });
            self.about_window_open = about_open;
        }
    }
}
