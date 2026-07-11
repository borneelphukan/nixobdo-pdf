use crate::app::NixobdoPdfApp;
use eframe::egui;

pub struct PasswordPromptState {
    pub path: std::path::PathBuf,
    pub file_name: String,
    pub is_incorrect: bool,
    pub password_input: String,
    pub focus_input: bool,
}

impl NixobdoPdfApp {
    pub(crate) fn ui_password_dialog(&mut self, ui: &mut egui::Ui) {
        let prompt_open = self.password_prompt.is_some();
        if prompt_open {
            let mut prompt_state = self.password_prompt.take().unwrap();
            let mut submitted = false;
            let mut cancelled = false;

            // Center position based on parent
            let parent_outer_rect = ui.ctx().input(|i| i.viewport().outer_rect);
            let mut builder = egui::ViewportBuilder::default()
                .with_title("Password Required")
                .with_inner_size([400.0, 160.0])
                .with_resizable(false)
                .with_maximize_button(false)
                .with_minimize_button(false);

            if let Some(rect) = parent_outer_rect {
                let center = rect.center();
                builder = builder.with_position(center - egui::vec2(200.0, 80.0));
            }

            ui.ctx().show_viewport_immediate(
                egui::ViewportId::from_hash_of("password_prompt_viewport"),
                builder,
                |ctx, _class| {
                    if ctx.input(|i| i.viewport().close_requested()) {
                        cancelled = true;
                    }

                    let is_light = ctx.system_theme() == Some(egui::Theme::Light);
                    let mut style = (*ctx.global_style()).clone();
                    if is_light {
                        style.visuals = egui::Visuals::light();
                    }
                    let bg_fill = if is_light { egui::Color32::from_rgb(245, 245, 245) } else { ctx.global_style().visuals.window_fill };

                    #[allow(deprecated)]
                    egui::Panel::bottom("password_prompt_bottom_panel")
                        .frame(egui::Frame::default().inner_margin(egui::Margin { left: 16, right: 16, top: 8, bottom: 16 }).fill(bg_fill))
                        .show(ctx, |ui| {
                            ui.set_style(style.clone());
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(egui::Button::new("Cancel").min_size(egui::vec2(80.0, 28.0))).clicked() {
                                        cancelled = true;
                                    }
                                    ui.add_space(8.0);
                                    let can_submit = !prompt_state.password_input.is_empty();
                                    if ui.add_enabled(can_submit, egui::Button::new("OK").min_size(egui::vec2(80.0, 28.0))).clicked() {
                                        submitted = true;
                                    }
                                });
                            });
                        });

                    #[allow(deprecated)]
                    egui::CentralPanel::default()
                        .frame(egui::Frame::default().inner_margin(16).fill(bg_fill))
                        .show(ctx, |ui| {
                            ui.set_style(style.clone());
                            ui.vertical(|ui| {
                                let label_text = if prompt_state.is_incorrect {
                                    "Incorrect password. Please try again:"
                                } else {
                                    "This PDF is password-protected. Please enter password:"
                                };
                                ui.label(label_text);
                                ui.add_space(10.0);

                                let text_edit = egui::TextEdit::singleline(&mut prompt_state.password_input)
                                    .password(true)
                                    .margin(egui::Margin::symmetric(8, 8))
                                    .desired_width(f32::INFINITY);

                                let response = ui.add(text_edit);
                                if prompt_state.focus_input {
                                    response.request_focus();
                                    prompt_state.focus_input = false;
                                }

                                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !prompt_state.password_input.is_empty() {
                                    submitted = true;
                                }
                            });
                        });
                }
            );

            if cancelled {
                if let Some(idx) = self.tabs.iter().position(|t| t.path == prompt_state.path) {
                    self.close_tab(idx);
                }
                self.password_prompt = None;
            } else if submitted {
                let _ = self.pdf_task_tx.send(crate::worker::PdfWorkerTask::Load {
                    path: prompt_state.path.clone(),
                    password: Some(prompt_state.password_input.clone()),
                    ctx: ui.ctx().clone(),
                });
                self.password_prompt = None;
            } else {
                self.password_prompt = Some(prompt_state);
            }
        }
    }
}
