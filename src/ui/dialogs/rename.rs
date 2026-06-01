use crate::app::nixobdo-pdfApp;
use eframe::egui;
use std::fs;

impl nixobdo-pdfApp {
    pub(crate) fn ui_rename_dialog(&mut self, ctx: &egui::Context) {
        if !self.rename_window_open {
            return;
        }

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
}
