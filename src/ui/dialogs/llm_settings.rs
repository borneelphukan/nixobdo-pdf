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
        if !self.show_llm_settings {
            return;
        }

        let mut open = true;
        egui::Window::new("AI Settings")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .order(egui::Order::Foreground)
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(440.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("LLM Service Configuration")
                            .size(16.0)
                            .strong(),
                    );
                    ui.add_space(12.0);

                    // Tabs
                    ui.horizontal(|ui| {
                        let local_selected = self.llm_settings_tab_index == 0;
                        let api_selected = self.llm_settings_tab_index == 1;

                        if ui.selectable_label(local_selected, "Local LLM").clicked() {
                            self.llm_settings_tab_index = 0;
                            self.init_llm_preset_from_model();
                        }
                        if ui.selectable_label(api_selected, "API Key").clicked() {
                            self.llm_settings_tab_index = 1;
                            self.init_llm_preset_from_model();
                        }
                    });
                    ui.add_space(12.0);

                    if self.llm_settings_tab_index == 0 {
                        ui.label("Endpoint URL:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.llm_endpoint_url)
                                .hint_text("http://localhost:1234/v1")
                                .desired_width(400.0),
                        );
                        ui.add_space(4.0);

                        ui.label(
                            egui::RichText::new("For local servers like LM Studio, Ollama, or vLLM.")
                                .size(11.0)
                                .weak(),
                        );
                        ui.add_space(12.0);

                        ui.label("Model:");
                        let mut selected = self.llm_selected_preset;
                        let preset_count = LLM_PRESETS.len();
                        egui::ComboBox::from_id_salt("llm_model_combo")
                            .width(400.0)
                            .selected_text(
                                if self.llm_is_custom_model {
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

                        if self.llm_is_custom_model {
                            ui.add_space(4.0);
                            ui.label("Custom Model Name:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.llm_custom_model)
                                    .hint_text("e.g. llama-3.2-3b")
                                    .desired_width(400.0),
                            );
                            if !self.llm_custom_model.is_empty() {
                                self.llm_model = self.llm_custom_model.clone();
                            }
                        }
                    } else {
                        ui.label("Model:");
                        let mut selected = self.llm_selected_preset;
                        let preset_count = LLM_PRESETS.len();
                        egui::ComboBox::from_id_salt("llm_model_combo_key")
                            .width(400.0)
                            .selected_text(
                                if self.llm_is_custom_model {
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

                        if self.llm_is_custom_model {
                            ui.add_space(4.0);
                            ui.label("Custom Model Name:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.llm_custom_model)
                                    .hint_text("e.g. gpt-4o-mini")
                                    .desired_width(400.0),
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
                                .desired_width(400.0),
                        );
                        ui.add_space(4.0);

                        ui.label(
                            egui::RichText::new("For cloud providers like OpenAI, Groq, Together AI, etc.")
                                .size(11.0)
                                .weak(),
                        );
                    }

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Cancel").clicked() {
                                self.show_llm_settings = false;
                            }
                            ui.add_space(8.0);
                            if ui.button("Add").clicked() {
                                if self.llm_is_custom_model && !self.llm_custom_model.is_empty() {
                                    self.llm_model = self.llm_custom_model.clone();
                                }
                                self.show_llm_settings = false;
                            }
                        });
                    });
                });
            });

        if !open {
            self.show_llm_settings = false;
        }
    }
}
