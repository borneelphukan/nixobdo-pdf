use crate::app::NixobdoPdfApp;
use eframe::egui;
impl eframe::App for NixobdoPdfApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Splash screen logic
        if self.ui_splash(ctx) {
            return;
        }
        
        if !self.has_checked_for_updates {
            self.has_checked_for_updates = true;
            let _ = self.pdf_task_tx.send(crate::worker::PdfWorkerTask::CheckUpdate { is_manual: false, ctx: ctx.clone() });
        }

        // Process background loaded PDFs and worker messages
        self.process_messages(ctx);

        // Render toast notification (bottom-right)
        self.ui_toast(ctx);

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
            self.ui_tabs(ctx);
            self.ui_toolbar(ctx);
        }

        // Rename, Export, About, Update, Export Progress Windows
        self.ui_dialogs(ctx);

        self.ui_viewer(ctx);
    }
}
