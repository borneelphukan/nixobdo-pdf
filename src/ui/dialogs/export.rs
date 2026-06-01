use crate::app::nixobdo-pdfApp;
use crate::worker::{ExportFormat, PdfWorkerTask};
use eframe::egui;
use std::sync::atomic::Ordering;

impl nixobdo-pdfApp {
    pub(crate) fn ui_export_dialog(&mut self, ctx: &egui::Context) {
        if !self.export_window_open {
            return;
        }

        let mut close_window = false;
        let mut perform_export = false;
        let mut restore_defaults = false;
        
        let window_title = if self.export_settings_open { "Save As DOCX Settings" } else { "Export File" };
        
        let window_frame = egui::Frame {
            fill: egui::Color32::from_rgb(240, 240, 240),
            stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(160, 160, 160)),
            corner_radius: egui::CornerRadius::ZERO,
            inner_margin: egui::Margin::ZERO,
            shadow: egui::epaint::Shadow {
                offset: [0, 4],
                blur: 8,
                spread: 0,
                color: egui::Color32::from_black_alpha(40),
            },
            ..Default::default()
        };
        
        egui::Window::new(window_title)
            .collapsible(false)
            .resizable(false)
            .title_bar(false) // Hide default egui title bar
            .frame(window_frame)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                // Force light theme visuals to match native Windows dialog look
                let mut style = (*ctx.style()).clone();
                style.visuals = egui::Visuals::light();
                ui.set_style(style);

                // Custom Native-looking Header
                egui::Frame::NONE
                    .fill(egui::Color32::WHITE)
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(window_title).color(egui::Color32::BLACK).size(14.0));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let close_btn = egui::Button::new(egui::RichText::new("✕").color(egui::Color32::BLACK)).frame(false);
                                if ui.add(close_btn).clicked() {
                                    close_window = true;
                                }
                            });
                        });
                    });
                    
                // Thin separator line below header
                ui.painter().hline(
                    0.0..=ui.available_width(),
                    ui.cursor().top(),
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(210, 210, 210))
                );

                // Main Content Body
                egui::Frame::NONE
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        if self.export_settings_open {
                            // DOCX Settings UI
                            ui.set_min_width(380.0);
                            
                            egui::Frame::group(&ctx.style()).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label("Layout Settings");
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.add_space(20.0);
                                    ui.radio_value(&mut self.export_layout_retain_page, false, "Retain Flowing Text");
                                    ui.add_space(40.0);
                                    ui.radio_value(&mut self.export_layout_retain_page, true, "Retain Page Layout");
                                });
                                ui.add_space(2.0);
                            });
                            
                            ui.add_space(8.0);
                            
                            egui::Frame::group(&ctx.style()).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label("Image Settings");
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.add_space(20.0);
                                    ui.checkbox(&mut self.export_include_images, "Include Images");
                                });
                                ui.add_space(2.0);
                            });
                            
                            ui.add_space(16.0);
                            
                            ui.horizontal(|ui| {
                                // OK button with prominent border
                                let ok_btn = egui::Button::new("OK")
                                    .min_size(egui::vec2(100.0, 24.0))
                                    .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 120, 215)));
                                    
                                if ui.add(ok_btn).clicked() {
                                    self.export_settings_open = false; // Go back to Export window
                                }
                                
                                ui.add_space(10.0);
                                
                                if ui.add(egui::Button::new("Restore Defaults").min_size(egui::vec2(120.0, 24.0))).clicked() {
                                    restore_defaults = true;
                                }
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(egui::Button::new("Cancel").min_size(egui::vec2(100.0, 24.0))).clicked() {
                                        self.export_settings_open = false; // Go back to Export window
                                    }
                                });
                            });
                        } else {
                            // Main Export UI
                            ui.set_min_width(380.0); // Make width consistent with settings

                            egui::Frame::group(&ctx.style()).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label("File Information");
                                ui.add_space(4.0);
                                
                                ui.horizontal(|ui| {
                                    ui.add_space(20.0);
                                    ui.label("Name:");
                                    ui.text_edit_singleline(&mut self.export_name);
                                });
                                
                                ui.add_space(4.0);
                                
                                ui.horizontal(|ui| {
                                    ui.add_space(20.0);
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
                                ui.add_space(2.0);
                            });
                            
                            ui.add_space(8.0);
                            
                            egui::Frame::group(&ctx.style()).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.label("Format Settings");
                                    let is_docx = self.export_format == ExportFormat::Docx;
                                    let gear_btn = egui::Button::new("⚙").frame(false);
                                    
                                    if ui.add_enabled(is_docx, gear_btn).on_hover_text("DOCX Settings").clicked() {
                                        self.export_settings_open = true;
                                    }
                                });
                                
                                ui.add_space(4.0);
                                
                                ui.horizontal(|ui| {
                                    ui.add_space(20.0);
                                    ui.radio_value(&mut self.export_format, ExportFormat::Doc, "Doc");
                                    ui.add_space(10.0);
                                    ui.radio_value(&mut self.export_format, ExportFormat::Docx, "Docx");
                                    ui.add_space(10.0);
                                    ui.radio_value(&mut self.export_format, ExportFormat::Png, "PNG");
                                    ui.add_space(10.0);
                                    ui.radio_value(&mut self.export_format, ExportFormat::Jpeg, "JPEG");
                                });
                                ui.add_space(2.0);
                            });
                            
                            ui.add_space(16.0);
                            
                            ui.horizontal(|ui| {
                                let save_enabled = self.export_location.is_some();
                                
                                let border_color = if save_enabled { 
                                    egui::Color32::from_rgb(0, 120, 215) 
                                } else { 
                                    egui::Color32::from_rgb(180, 180, 180) 
                                };
                                
                                let save_btn = egui::Button::new("Save")
                                    .min_size(egui::vec2(100.0, 24.0))
                                    .stroke(egui::Stroke::new(2.0, border_color));
                                    
                                if ui.add_enabled(save_enabled, save_btn).clicked() {
                                    perform_export = true;
                                    close_window = true;
                                }
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(egui::Button::new("Cancel").min_size(egui::vec2(100.0, 24.0))).clicked() {
                                        close_window = true;
                                    }
                                });
                            });
                        }
                    });
            });
            
        if restore_defaults {
            self.export_layout_retain_page = true;
            self.export_include_images = true;
        }
            
        if perform_export {
            if let Some(active_idx) = self.active_tab_index {
                if let Some(tab) = self.tabs.get(active_idx) {
                    if let Some(location) = &self.export_location {
                        let final_name = format!("{}.{}", self.export_name, self.export_format.extension());
                        let out_path = location.join(final_name);
                        
                        self.export_cancel_flag.store(false, Ordering::Relaxed);
                        
                        let _ = self.pdf_task_tx.send(PdfWorkerTask::Export {
                            path: tab.path.clone(),
                            out_path,
                            format: self.export_format,
                            retain_layout: self.export_layout_retain_page,
                            include_images: self.export_include_images,
                            ctx: ctx.clone(),
                            cancel_flag: self.export_cancel_flag.clone(),
                        });
                    }
                }
            }
        }
        if close_window {
            self.export_window_open = false;
            self.export_settings_open = false; // Reset for next time
        }
    }
}
