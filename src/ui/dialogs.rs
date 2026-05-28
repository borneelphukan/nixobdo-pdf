use crate::app::PdfViewerApp;
use crate::worker::{ExportFormat, PdfWorkerTask};
use eframe::egui;
use std::fs;

impl PdfViewerApp {
    pub(crate) fn ui_dialogs(&mut self, ctx: &egui::Context) {
        // Rename Dialog
        if self.rename_window_open {
            let mut close_window = false;
            let mut perform_rename = false;

            egui::Window::new("Rename File")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(egui::Align2::CENTER_TOP, [0.0, 30.0])
                .frame(egui::Frame::popup(&ctx.style()).inner_margin(8.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        let response = ui.text_edit_singleline(&mut self.rename_buffer);
                        
                        if self.focus_rename_input {
                            response.request_focus();
                            self.focus_rename_input = false;
                        }
                        
                        // Save and close on Enter, or when clicking outside (losing focus)
                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            perform_rename = true;
                            close_window = true;
                        } else if response.lost_focus() && !ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            // Only save if it lost focus and we didn't just press escape (though escape would also close)
                            perform_rename = true;
                            close_window = true;
                        } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            close_window = true;
                        }
                    });
                });

            if perform_rename {
                if let Some(active_idx) = self.active_tab_index {
                    if let Some(tab) = self.tabs.get_mut(active_idx) {
                        let old_path = tab.path.clone();
                        let mut new_path = old_path.clone();
                        new_path.set_file_name(&self.rename_buffer);
                        
                        if fs::rename(&old_path, &new_path).is_ok() {
                            tab.path = new_path.clone();
                            tab.file_name = self.rename_buffer.clone();
                            
                            // Update recent files
                            self.recent_files.retain(|p| p != &old_path);
                            if !self.recent_files.contains(&new_path) {
                                self.recent_files.insert(0, new_path);
                                self.recent_files.truncate(5);
                                self.save_recent_files();
                            }
                        } else {
                            eprintln!("Failed to rename file on disk.");
                        }
                    }
                }
            }

            if close_window {
                self.rename_window_open = false;
            }
        }

        // Export Dialog
        if self.export_window_open {
            let mut close_window = false;
            let mut perform_export = false;
            
            egui::Window::new("Export File")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Export As:");
                        ui.text_edit_singleline(&mut self.export_name);
                    });
                    
                    ui.add_space(4.0);
                    
                    ui.horizontal(|ui| {
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
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("Format:");
                        egui::ComboBox::from_id_salt("format_dropdown")
                            .selected_text(format!("{:?}", self.export_format))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.export_format, ExportFormat::Doc, "Doc");
                                ui.selectable_value(&mut self.export_format, ExportFormat::Docx, "Docx");
                                ui.selectable_value(&mut self.export_format, ExportFormat::Png, "PNG");
                                ui.selectable_value(&mut self.export_format, ExportFormat::Jpeg, "JPEG");
                            });
                    });
                    
                    ui.add_space(8.0);
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Only enable Save if a location is selected
                        let save_enabled = self.export_location.is_some();
                        if ui.add_enabled(save_enabled, egui::Button::new("Save")).clicked() {
                            perform_export = true;
                            close_window = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_window = true;
                        }
                    });
                });
                
            if perform_export {
                if let Some(active_idx) = self.active_tab_index {
                    if let Some(tab) = self.tabs.get(active_idx) {
                        if let Some(location) = &self.export_location {
                            let final_name = format!("{}.{}", self.export_name, self.export_format.extension());
                            let out_path = location.join(final_name);
                            
                            let _ = self.pdf_task_tx.send(PdfWorkerTask::Export {
                                path: tab.path.clone(),
                                out_path,
                                format: self.export_format,
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
}
