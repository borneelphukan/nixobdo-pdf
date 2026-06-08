pub mod rename;
pub mod export;
pub mod about;
pub mod update;
pub mod export_progress;
pub mod custom_color;

use crate::app::NixobdoPdfApp;
use eframe::egui;

impl NixobdoPdfApp {
    pub(crate) fn ui_dialogs(&mut self, ctx: &egui::Context) {
        self.ui_rename_dialog(ctx);
        self.ui_export_dialog(ctx);
        self.ui_about_dialog(ctx);
        self.ui_update_dialog(ctx);
        self.ui_export_progress(ctx);
        self.ui_custom_color_dialog(ctx);
    }
}
