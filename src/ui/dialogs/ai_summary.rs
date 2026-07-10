#![allow(deprecated)]
use crate::app::NixobdoPdfApp;
use eframe::egui;

impl NixobdoPdfApp {
    pub(crate) fn ui_ai_summary_dialog(&mut self, ui: &mut egui::Ui) {
        let mut summary_open = self.ai_summary_open;
        if summary_open {
            ui.ctx().show_viewport_immediate(
                egui::ViewportId::from_hash_of("ai_summary_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("AI Summary")
                    .with_inner_size([500.0, 450.0])
                    .with_resizable(true)
                    .with_maximize_button(true)
                    .with_minimize_button(false),
                |ctx, _class| {
                    if ctx.input(|i| i.viewport().close_requested()) {
                        summary_open = false;
                        self.ai_summary_text = String::new();
                        self.ai_summary_full_text = String::new();
                        self.ai_summary_error = None;
                        self.ai_summary_loading = false;
                    }

                    let is_light = ctx.system_theme() == Some(egui::Theme::Light);
                    let mut style = (*ctx.global_style()).clone();
                    if is_light {
                        style.visuals = egui::Visuals::light();
                    }
                    let bg_fill = if is_light { egui::Color32::from_rgb(245, 245, 245) } else { ctx.global_style().visuals.window_fill };

                    egui::Panel::bottom("ai_summary_bottom_panel")
                        .frame(egui::Frame::default().inner_margin(egui::Margin { left: 16, right: 16, top: 8, bottom: 16 }).fill(bg_fill))
                        .show(ctx, |ui| {
                            ui.set_style(style.clone());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(egui::Button::new("Close").min_size(egui::vec2(80.0, 28.0))).clicked() {
                                    summary_open = false;
                                    self.ai_summary_text = String::new();
                                    self.ai_summary_full_text = String::new();
                                    self.ai_summary_error = None;
                                    self.ai_summary_loading = false;
                                }
                            });
                        });

                    egui::CentralPanel::default()
                        .frame(egui::Frame::default().inner_margin(16).fill(bg_fill))
                        .show(ctx, |ui| {
                            ui.set_style(style);

                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚡ AI Summary").size(18.0).strong());
                            });
                            ui.add_space(12.0);

                            if self.ai_summary_loading {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(40.0);
                                    ui.spinner();
                                    ui.add_space(12.0);
                                    ui.label(
                                        egui::RichText::new("Generating summary...")
                                            .size(13.0)
                                            .weak(),
                                    );
                                    ui.add_space(40.0);
                                });
                                // Keep repainting so the spinner animates
                                ctx.request_repaint();
                            } else if let Some(error) = self.ai_summary_error.clone() {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(30.0);
                                    ui.label(egui::RichText::new("⚠️").size(32.0));
                                    ui.add_space(12.0);
                                    ui.label(
                                        egui::RichText::new(&error)
                                            .size(13.0)
                                            .color(egui::Color32::from_rgb(255, 100, 100)),
                                    );
                                    ui.add_space(30.0);
                                });
                            } else if !self.ai_summary_full_text.is_empty() {
                                // Animate text reveal
                                let now = ctx.input(|i| i.time);
                                let elapsed = (now - self.ai_summary_start_time) as f32;
                                let chars_per_second = 800.0;
                                let target_len = (elapsed * chars_per_second) as usize;
                                let full_len = self.ai_summary_full_text.chars().count();

                                if target_len >= full_len {
                                    self.ai_summary_display_len = full_len;
                                } else {
                                    self.ai_summary_display_len = target_len;
                                    ctx.request_repaint();
                                }

                                let display_text: String = self
                                    .ai_summary_full_text
                                    .chars()
                                    .take(self.ai_summary_display_len)
                                    .collect();

                                egui::ScrollArea::vertical()
                                    .max_height(f32::INFINITY)
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(display_text)
                                                .size(14.0)
                                                .line_height(Some(20.0)),
                                        );
                                    });
                            }
                        });
                }
            );
            self.ai_summary_open = summary_open;
        }
    }
}
