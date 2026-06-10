pub mod central_panel;
pub mod separator;
pub mod sidebar;

use crate::app::NixobdoPdfApp;
use eframe::egui;

impl NixobdoPdfApp {
    pub(crate) fn ui_viewer(&mut self, ui: &mut egui::Ui) {
        self.ui_sidebar(ui);
        self.ui_separator(ui);
        self.ui_central_panel(ui);

        // Handle zoom with Ctrl + Mouse Wheel
        let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta.y);
        let zoom_delta = ui.ctx().input(|i| i.smooth_scroll_delta.y);
        let delta = if scroll_delta != 0.0 {
            scroll_delta
        } else {
            zoom_delta
        };

        let has_zoom_modifier = ui.ctx().input(|i| i.modifiers.command || i.modifiers.ctrl);

        if has_zoom_modifier && delta != 0.0 {
            if let Some(active_idx) = self.active_tab_index {
                if let Some(tab) = self.tabs.get_mut(active_idx) {
                    if delta > 0.0 {
                        tab.zoom += 0.1;
                    } else {
                        tab.zoom = (tab.zoom - 0.1).max(0.0);
                    }
                }
            }
        }

        // Handle zoom with Trackpad pinch
        let zoom_gesture = ui.ctx().input(|i| i.zoom_delta());
        if zoom_gesture != 1.0 {
            if let Some(active_idx) = self.active_tab_index {
                if let Some(tab) = self.tabs.get_mut(active_idx) {
                    let new_zoom = (1.0 + tab.zoom) * zoom_gesture - 1.0;
                    tab.zoom = new_zoom.max(0.0);
                }
            }
        }

        // Handle Copy selection shortcut (Cmd+C / Ctrl+C)
        let has_copy_modifier = ui.ctx().input(|i| i.modifiers.command || i.modifiers.ctrl);
        if has_copy_modifier && ui.ctx().input(|i| i.key_pressed(egui::Key::C)) {
            self.copy_selection(ui.ctx());
        }
    }
}
