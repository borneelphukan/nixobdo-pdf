use crate::app::NixobdoPdfApp;
use crate::worker::{ExportFormat, PdfWorkerTask};
use eframe::egui;
use std::sync::atomic::Ordering;

impl NixobdoPdfApp {
    pub(crate) fn ui_export_dialog(&mut self, ui: &mut egui::Ui) {
        if !self.export_window_open {
            return;
        }

        let mut close_window = false;
        let mut perform_export = false;
        
        let window_title = "Export File";
        
        let window_frame = egui::Frame::window(&ui.ctx().global_style()).inner_margin(0.0);
        
        egui::Window::new(window_title)
            .collapsible(false)
            .resizable(false)
            .title_bar(false) // Hide default egui title bar
            .frame(window_frame)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                // Custom Header
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(window_title).strong().size(14.0));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let close_btn = egui::Button::new(egui::RichText::new("X").size(14.0)).frame(false);
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
                    ui.visuals().widgets.noninteractive.bg_stroke
                );

                // Main Content Body
                egui::Frame::NONE
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                            // Main Export UI
                            ui.set_min_width(380.0);

                            egui::Frame::group(&ui.ctx().global_style()).show(ui, |ui| {
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
                            
                            egui::Frame::group(&ui.ctx().global_style()).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.label("Format Settings");
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
                                
                                let save_btn = egui::Button::new("Save")
                                    .min_size(egui::vec2(100.0, 24.0));
                                    
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
                    });
            });

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
                            retain_layout: true,
                            include_images: true,
                            ctx: ui.ctx().clone(),
                            cancel_flag: self.export_cancel_flag.clone(),
                        });
                    }
                }
            }
        }
        if close_window {
            self.export_window_open = false;
        }
    }
}


