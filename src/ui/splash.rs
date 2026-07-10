use crate::app::NixobdoPdfApp;
use eframe::egui;

impl NixobdoPdfApp {
    pub(crate) fn ui_splash(&mut self, ui: &mut egui::Ui) -> bool {
        let elapsed = self.startup_time.elapsed().as_secs_f32();
        if elapsed < 2.5 {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(ui.ctx().global_style().visuals.window_fill()))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() / 2.0 - 100.0);

                        ui.add(
                            egui::Image::new(egui::include_image!("../../assets/icons/logo.svg"))
                                .max_width(250.0)
                                .max_height(250.0),
                        );

                        ui.add_space(20.0);
                        ui.heading(
                            egui::RichText::new("Nixobdo PDF Reader")
                                .size(28.0)
                                .strong(),
                        );
                    });
                });

            ui.ctx().request_repaint();
            true
        } else {
            false
        }
    }
}
