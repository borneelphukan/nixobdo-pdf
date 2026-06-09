use crate::app::NixobdoPdfApp;
use eframe::egui;
use std::sync::atomic::Ordering;

impl NixobdoPdfApp {
    pub(crate) fn ui_export_progress(&mut self, ui: &mut egui::Ui) {
        if let Some(progress) = self.export_progress {
            let mut is_open = true;
            egui::Window::new("Exporting...")
                .collapsible(false)
                .resizable(false)
                .open(&mut is_open)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(egui::Frame::window(&ui.ctx().global_style()).inner_margin(16.0).corner_radius(8))
                .show(ui.ctx(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("Exporting document...").size(14.0));
                        ui.add_space(12.0);
                        let rect = ui.available_rect_before_wrap();
                        let size = egui::vec2(rect.width(), 20.0);
                        let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
                        let corner_radius = egui::CornerRadius::same(4);
                        ui.painter().rect_filled(rect, corner_radius, ui.visuals().extreme_bg_color);
                        let fill_width = rect.width() * progress;
                        if fill_width > 0.0 {
                            let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, rect.height()));
                            ui.painter().rect_filled(fill_rect, corner_radius, ui.visuals().selection.bg_fill);
                        }
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{:.0}%", progress * 100.0),
                            egui::FontId::proportional(14.0),
                            ui.visuals().text_color(),
                        );
                        ui.add_space(16.0);
                        if ui.button("Cancel").clicked() {
                            self.export_cancel_flag.store(true, Ordering::Relaxed);
                        }
                    });
                });
            if !is_open {
                self.export_cancel_flag.store(true, Ordering::Relaxed);
            }
        }
    }
}


