pub mod rename;
pub mod export;

use crate::app::PdfViewerApp;
use eframe::egui;

impl PdfViewerApp {
    pub(crate) fn ui_dialogs(&mut self, ctx: &egui::Context) {
        self.ui_rename_dialog(ctx);
        self.ui_export_dialog(ctx);
    }
}
