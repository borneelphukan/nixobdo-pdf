pub mod about;
pub mod custom_color;
pub mod export;
pub mod export_progress;
pub mod rename;
pub mod update;

use crate::app::NixobdoPdfApp;
use eframe::egui;

impl NixobdoPdfApp {
    pub(crate) fn ui_dialogs(&mut self, ui: &mut egui::Ui) {
        self.ui_rename_dialog(ui);
        self.ui_export_dialog(ui);
        self.ui_about_dialog(ui);
        self.ui_update_dialog(ui);
        self.ui_export_progress(ui);
        self.ui_custom_color_dialog(ui);
    }
}
