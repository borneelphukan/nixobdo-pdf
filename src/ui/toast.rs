use crate::app::NixobdoPdfApp;
use eframe::egui;

impl NixobdoPdfApp {
    pub(crate) fn ui_toast(&mut self, ui: &mut egui::Ui) {
        if let Some(msg) = &self.toast_message {
            let now = ui.ctx().input(|i| i.time);
            if now < self.toast_timer {
                let remaining = self.toast_timer - now;
                // Fade out in the last second
                let alpha = if remaining < 1.0 { (remaining * 255.0) as u8 } else { 255 };
                
                let (bg_color, icon) = if self.toast_success {
                    (egui::Color32::from_rgba_unmultiplied(34, 139, 34, alpha), "✔")
                } else {
                    (egui::Color32::from_rgba_unmultiplied(200, 50, 50, alpha), "✖")
                };
                let text_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha);
                
                let toast_msg = msg.clone();
                egui::Area::new(egui::Id::new("export_toast"))
                    .anchor(egui::Align2::RIGHT_BOTTOM, [-16.0, -16.0])
                    .order(egui::Order::Foreground)
                    .show(ui.ctx(), |ui| {
                        egui::Frame::NONE
                            .fill(bg_color)
                            .corner_radius(egui::CornerRadius::same(6))
                            .inner_margin(egui::Margin::symmetric(16, 10))
                            .shadow(egui::epaint::Shadow {
                                offset: [0, 2],
                                blur: 8,
                                spread: 0,
                                color: egui::Color32::from_black_alpha(60),
                            })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(icon).color(text_color).size(16.0));
                                    ui.add_space(6.0);
                                    ui.label(egui::RichText::new(&toast_msg).color(text_color).size(13.0));
                                });
                            });
                    });
                
                ui.ctx().request_repaint(); // Keep repainting for animation
            } else {
                self.toast_message = None;
            }
        }
    }
}


