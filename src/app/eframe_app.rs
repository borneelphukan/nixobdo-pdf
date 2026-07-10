use crate::app::NixobdoPdfApp;
use eframe::egui;
impl eframe::App for NixobdoPdfApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Splash screen logic
        if self.ui_splash(ui) {
            return;
        }

        if !self.has_checked_for_updates {
            self.has_checked_for_updates = true;
            let _ = self
                .pdf_task_tx
                .send(crate::worker::PdfWorkerTask::CheckUpdate {
                    is_manual: false,
                    ctx: ui.ctx().clone(),
                });
        }

        // Process background loaded PDFs and worker messages
        self.process_messages(ui);

        // Render toast notification (bottom-right)
        self.ui_toast(ui);

        if let Some(active_idx) = self.active_tab_index {
            if let Some(tab) = self.tabs.get(active_idx) {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Title(format!(
                        "{} - nixobdo-pdf",
                        tab.file_name
                    )));
            }
        } else {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Title("nixobdo-pdf".to_string()));
        }

        // Handle Ctrl+F / Cmd+F to focus search
        let has_ctrl_modifier = ui.ctx().input(|i| i.modifiers.command || i.modifiers.ctrl);
        if has_ctrl_modifier && ui.ctx().input(|i| i.key_pressed(egui::Key::F)) {
            ui.ctx()
                .memory_mut(|mem| mem.request_focus(egui::Id::new("search_bar")));
        }

        let is_fullscreen = ui.ctx().input(|i| i.viewport().fullscreen.unwrap_or(false));
        if is_fullscreen && ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        }

        self.ui_menu_bar(ui);
        self.ui_tabs(ui);
        self.ui_toolbar(ui);

        // Rename, Export, About, Update, Export Progress Windows
        self.ui_dialogs(ui);

        self.ui_viewer(ui);
    }
}
