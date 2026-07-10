use crate::app::NixobdoPdfApp;
use eframe::egui;

const LLM_PRESETS: &[(&str, &str)] = &[
    ("Llama 3.2 3B", "llama-3.2-3b"),
    ("Llama 3.1 8B", "llama-3.1-8b"),
    ("Mistral 7B", "mistral-7b"),
    ("Gemma 2 9B", "gemma-2-9b"),
    ("Phi-3 Mini", "phi-3-mini"),
    ("Qwen 2.5 7B", "qwen-2.5-7b"),
    ("Qwen 2.5 14B", "qwen-2.5-14b"),
    ("DeepSeek-V2", "deepseek-v2"),
];

impl NixobdoPdfApp {
    pub(crate) fn init_llm_preset_from_model(&mut self) {
        self.llm_is_custom_model = true;
        self.llm_selected_preset = LLM_PRESETS.len();
        self.llm_custom_model = self.llm_model.clone();

        for (i, (_, model_id)) in LLM_PRESETS.iter().enumerate() {
            if *model_id == self.llm_model {
                self.llm_is_custom_model = false;
                self.llm_selected_preset = i;
                self.llm_custom_model.clear();
                break;
            }
        }
    }

    pub(crate) fn ui_llm_settings_dialog(&mut self, ui: &mut egui::Ui) {
        let mut settings_open = self.show_llm_settings;
        if settings_open {
            ui.ctx().show_viewport_immediate(
                egui::ViewportId::from_hash_of("ai_settings_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("AI Settings")
                    .with_inner_size([540.0, 400.0])
                    .with_resizable(false)
                    .with_maximize_button(false)
                    .with_minimize_button(false),
                |ctx, _class| {
                    if ctx.input(|i| i.viewport().close_requested()) {
                        settings_open = false;
                        self.save_settings();
                    }

                    let is_light = ctx.system_theme() == Some(egui::Theme::Light);
                    let mut style = (*ctx.global_style()).clone();
                    if is_light {
                        style.visuals = egui::Visuals::light();
                    }
                    let bg_fill = if is_light { egui::Color32::from_rgb(245, 245, 245) } else { ctx.global_style().visuals.window_fill };

                    #[allow(deprecated)]
                    egui::Panel::bottom("ai_settings_bottom_panel")
                        .frame(egui::Frame::default().inner_margin(egui::Margin { left: 16, right: 16, top: 8, bottom: 16 }).fill(bg_fill))
                        .show(ctx, |ui| {
                            ui.set_style(style.clone());
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(egui::Button::new("Cancel").min_size(egui::vec2(80.0, 28.0))).clicked() {
                                        settings_open = false;
                                        self.save_settings();
                                    }
                                    ui.add_space(8.0);
                                    if ui.add(egui::Button::new("Add").min_size(egui::vec2(80.0, 28.0))).clicked() {
                                        if self.llm_is_custom_model && !self.llm_custom_model.is_empty() {
                                            self.llm_model = self.llm_custom_model.clone();
                                        }
                                        settings_open = false;
                                        self.save_settings();
                                    }
                                });
                            });
                        });

                    #[allow(deprecated)]
                    egui::CentralPanel::default()
                        .frame(egui::Frame::default().inner_margin(16).fill(bg_fill))
                        .show(ctx, |ui| {
                            let mut central_style = style.clone();
                            central_style.spacing.interact_size.y = 32.0;
                            ui.set_style(central_style);
                            
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("LLM Service Configuration")
                                        .size(16.0)
                                        .strong(),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("Configure your preferred AI model provider for PDF processing and chat features.")
                                        .size(12.0)
                                        .weak(),
                                );
                                ui.add_space(12.0);

                                egui::Frame::group(ui.style())
                                    .inner_margin(16.0)
                                    .show(ui, |ui| {
                                        // Right-aligned Tabs
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                let local_selected = self.llm_settings_tab_index == 0;
                                                let api_selected = self.llm_settings_tab_index == 1;

                                                ui.scope(|ui| {
                                                    ui.spacing_mut().item_spacing.x = 0.0;
                                                    
                                                    let right_btn = egui::Button::new("API Key")
                                                        .selected(api_selected)
                                                        .corner_radius(egui::CornerRadius { nw: 0, ne: 8, sw: 0, se: 8 });
                                                    if ui.add_sized([80.0, 28.0], right_btn).clicked() {
                                                        self.llm_settings_tab_index = 1;
                                                        if self.llm_endpoint_url == "http://localhost:1234/v1" || self.llm_endpoint_url.is_empty() {
                                                            self.llm_endpoint_url = "https://api.openai.com/v1".to_string();
                                                        }
                                                        self.init_llm_preset_from_model();
                                                    }

                                                    let left_btn = egui::Button::new("Local LLM")
                                                        .selected(local_selected)
                                                        .corner_radius(egui::CornerRadius { nw: 8, ne: 0, sw: 8, se: 0 });
                                                    if ui.add_sized([80.0, 28.0], left_btn).clicked() {
                                                        self.llm_settings_tab_index = 0;
                                                        if self.llm_endpoint_url == "https://api.openai.com/v1" || self.llm_endpoint_url.is_empty() {
                                                            self.llm_endpoint_url = "http://localhost:1234/v1".to_string();
                                                        }
                                                        self.init_llm_preset_from_model();
                                                    }
                                                });
                                            });
                                        });
                                        ui.add_space(16.0);

                                        if self.llm_settings_tab_index == 0 {
                                            let has_api_key = !self.llm_api_key.trim().is_empty();
                                            ui.add_enabled_ui(!has_api_key, |ui| {
                                                ui.label("Model:");
                                                let mut selected = self.llm_selected_preset;
                                                let preset_count = LLM_PRESETS.len();
                                                egui::ComboBox::from_id_salt("llm_model_combo")
                                                    .width(480.0)
                                                    .selected_text(
                                                        if self.llm_is_custom_model && self.llm_custom_model.is_empty() {
                                                            "Select your LLM Model"
                                                        } else if self.llm_is_custom_model {
                                                            &self.llm_custom_model
                                                        } else {
                                                            LLM_PRESETS.get(self.llm_selected_preset)
                                                                .map(|(name, _)| *name)
                                                                .unwrap_or("Custom")
                                                        }
                                                    )
                                                    .show_ui(ui, |ui| {
                                                        for (i, (name, id)) in LLM_PRESETS.iter().enumerate() {
                                                            let label = format!("{} ({})", name, id);
                                                            if ui.selectable_value(&mut selected, i, label).clicked() {
                                                                self.llm_selected_preset = i;
                                                                self.llm_is_custom_model = false;
                                                                self.llm_model = id.to_string();
                                                                self.llm_custom_model.clear();
                                                            }
                                                        }
                                                        if ui.selectable_value(&mut selected, preset_count, "Custom").clicked() {
                                                            self.llm_selected_preset = preset_count;
                                                            self.llm_is_custom_model = true;
                                                            self.llm_custom_model = self.llm_model.clone();
                                                        }
                                                    });

                                                if self.llm_selected_preset == preset_count {
                                                    ui.add_space(12.0);
                                                    ui.label("Custom Model Name:");
                                                    ui.add(
                                                        egui::TextEdit::singleline(&mut self.llm_custom_model)
                                                            .hint_text("e.g. llama-3.2-3b")
                                                            .desired_width(f32::INFINITY)
                                                            .margin(egui::Margin::symmetric(8, 8)),
                                                    );
                                                    if !self.llm_custom_model.is_empty() {
                                                        self.llm_model = self.llm_custom_model.clone();
                                                    }
                                                }

                                                ui.add_space(12.0);

                                                ui.label("Endpoint URL:");
                                                ui.add(
                                                    egui::TextEdit::singleline(&mut self.llm_endpoint_url)
                                                        .hint_text("http://localhost:1234/v1")
                                                        .desired_width(f32::INFINITY)
                                                        .margin(egui::Margin::symmetric(8, 8)),
                                                );
                                                ui.add_space(4.0);
                                            });

                                        } else {
                                            let has_local_endpoint = !self.llm_endpoint_url.trim().is_empty() && self.llm_endpoint_url != "https://api.openai.com/v1";
                                            ui.add_enabled_ui(!has_local_endpoint, |ui| {
                                                ui.label("Model:");
                                                let mut selected = self.llm_selected_preset;
                                                let preset_count = LLM_PRESETS.len();
                                                egui::ComboBox::from_id_salt("llm_model_combo_key")
                                                    .width(480.0)
                                                    .selected_text(
                                                        if self.llm_is_custom_model && self.llm_custom_model.is_empty() {
                                                            "Select your LLM Model"
                                                        } else if self.llm_is_custom_model {
                                                            &self.llm_custom_model
                                                        } else {
                                                            LLM_PRESETS.get(self.llm_selected_preset)
                                                                .map(|(name, _)| *name)
                                                                .unwrap_or("Select your LLM Model")
                                                        }
                                                    )
                                                    .show_ui(ui, |ui| {
                                                        for (i, (name, id)) in LLM_PRESETS.iter().enumerate() {
                                                            let label = format!("{} ({})", name, id);
                                                            if ui.selectable_value(&mut selected, i, label).clicked() {
                                                                self.llm_selected_preset = i;
                                                                self.llm_is_custom_model = false;
                                                                self.llm_model = id.to_string();
                                                                self.llm_custom_model.clear();
                                                            }
                                                        }
                                                        if ui.selectable_value(&mut selected, preset_count, "Custom").clicked() {
                                                            self.llm_selected_preset = preset_count;
                                                            self.llm_is_custom_model = true;
                                                            self.llm_custom_model = self.llm_model.clone();
                                                        }
                                                    });

                                                if self.llm_selected_preset == preset_count {
                                                    ui.add_space(12.0);
                                                    ui.label("Custom Model Name:");
                                                    ui.add(
                                                        egui::TextEdit::singleline(&mut self.llm_custom_model)
                                                            .hint_text("e.g. gpt-4o-mini")
                                                            .desired_width(f32::INFINITY)
                                                            .margin(egui::Margin::symmetric(8, 8)),
                                                    );
                                                    if !self.llm_custom_model.is_empty() {
                                                        self.llm_model = self.llm_custom_model.clone();
                                                    }
                                                }

                                                ui.add_space(12.0);

                                                ui.label("API Key:");
                                                ui.add(
                                                    egui::TextEdit::singleline(&mut self.llm_api_key)
                                                        .hint_text("sk-...")
                                                        .password(true)
                                                        .desired_width(f32::INFINITY)
                                                        .margin(egui::Margin::symmetric(8, 8)),
                                                );
                                            });
                                        }
                                    });
                            });
                        });
                }
            );
            self.show_llm_settings = settings_open;
        }
    }
}
