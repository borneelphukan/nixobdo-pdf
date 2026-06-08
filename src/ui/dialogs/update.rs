use crate::app::{NixobdoPdfApp, UpdateState};
use eframe::egui;

impl NixobdoPdfApp {
    pub(crate) fn ui_update_dialog(&mut self, ctx: &egui::Context) {
        match self.update_state.clone() {
            UpdateState::None => {}
            UpdateState::Checking => {
                let mut is_open = true;
                egui::Window::new("Checking for Updates")
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut is_open)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .frame(egui::Frame::window(&ctx.style()).inner_margin(16.0).corner_radius(8))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label("Checking for newer version...");
                            ui.add_space(8.0);
                            ui.spinner();
                        });
                    });
                
                if !is_open {
                    self.update_state = UpdateState::None;
                }
            }
            UpdateState::Prompt(version) => {
                let is_dark = ctx.system_theme().unwrap_or(egui::Theme::Dark) == egui::Theme::Dark;
                
                let bg_color = if is_dark { egui::Color32::from_rgb(45, 45, 55) } else { egui::Color32::from_rgb(240, 240, 245) };
                let text_color = if is_dark { egui::Color32::from_rgb(240, 240, 245) } else { egui::Color32::from_rgb(20, 20, 25) };
                
                let btn_inactive = if is_dark { egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20) } else { egui::Color32::from_rgba_unmultiplied(0, 0, 0, 15) };
                let btn_hovered = if is_dark { egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40) } else { egui::Color32::from_rgba_unmultiplied(0, 0, 0, 30) };
                let btn_active = if is_dark { egui::Color32::from_rgba_unmultiplied(255, 255, 255, 60) } else { egui::Color32::from_rgba_unmultiplied(0, 0, 0, 45) };

                egui::Area::new(egui::Id::new("update_banner"))
                    .anchor(egui::Align2::CENTER_TOP, [0.0, 10.0])
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        egui::Frame::window(&ctx.style())
                            .fill(bg_color)
                            .corner_radius(egui::CornerRadius::same(6))
                            .inner_margin(egui::Margin::symmetric(16, 10))
                            .shadow(egui::epaint::Shadow {
                                offset: [0, 4],
                                blur: 16,
                                spread: 0,
                                color: egui::Color32::from_black_alpha(80),
                            })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("New update available: v{}", version)).color(text_color).strong());
                                    ui.add_space(20.0);
                                    
                                    ui.visuals_mut().widgets.inactive.bg_fill = btn_inactive;
                                    ui.visuals_mut().widgets.hovered.bg_fill = btn_hovered;
                                    ui.visuals_mut().widgets.active.bg_fill = btn_active;
                                    
                                    if ui.button(egui::RichText::new("Download Now").color(text_color)).clicked() {
                                        self.update_state = UpdateState::Downloading(0.0);
                                        let _ = self.pdf_task_tx.send(crate::worker::PdfWorkerTask::DownloadUpdate { version: version.clone(), ctx: ctx.clone() });
                                    }
                                    if ui.button(egui::RichText::new("Skip").color(text_color)).clicked() {
                                        self.update_state = UpdateState::None;
                                    }
                                });
                            });
                    });
            }
            UpdateState::Downloading(progress) => {
                let mut is_open = true;
                
                egui::Window::new("Downloading Update")
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut is_open)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .frame(egui::Frame::window(&ctx.style()).inner_margin(16.0).corner_radius(8))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("Downloading update...").size(14.0));
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
                                self.update_state = UpdateState::None;
                            }
                        });
                    });
                
                if !is_open {
                    self.update_state = UpdateState::None;
                }
            }
        }
    }
}
