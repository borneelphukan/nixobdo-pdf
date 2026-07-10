use crate::app::NixobdoPdfApp;
use eframe::egui;

impl NixobdoPdfApp {
    pub fn ui_chatbot(&mut self, ui: &mut egui::Ui) {
        if !self.ai_chatbot_open {
            return;
        }

        let mut send_msg = false;

        egui::Panel::right("ai_chatbot_panel")
            .resizable(true)
            .min_size(300.0)
            .max_size(800.0)
            .default_size(350.0)
            .show(ui, |ui| {
                if !self.llm_configured {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        ui.label(egui::RichText::new("No LLM Service Available").size(16.0).strong());
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Please configure an LLM service\nin the settings to use the AI assistant.").weak());
                        ui.add_space(16.0);
                        if ui.button("Open Settings").clicked() {
                            self.show_llm_settings = true;
                        }
                    });
                    return;
                }

                if self.active_tab_index.is_none() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        ui.label(egui::RichText::new("No Document Opened").size(16.0).strong());
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Please open a PDF document\nto interact with the AI assistant.").weak());
                    });
                    return;
                }

                ui.horizontal(|ui| {
                    // Session Selector
                    let active_name = if let Some(active_id) = &self.ai_active_session_id {
                        self.ai_chat_sessions.iter().find(|s| &s.id == active_id).map(|s| s.name.clone()).unwrap_or_else(|| "No Session".to_string())
                    } else {
                        "No Session".to_string()
                    };

                    let mut selected_id = self.ai_active_session_id.clone();
                    
                    ui.scope(|ui| {
                        ui.style_mut().spacing.interact_size.y = 32.0; // Taller dropdown button
                        egui::ComboBox::from_id_salt("ai_chat_session_select")
                            .width(220.0)
                            .selected_text(active_name)
                            .show_ui(ui, |ui| {
                                if self.ai_chat_sessions.is_empty() {
                                    ui.label(egui::RichText::new("No sessions available").weak());
                                } else {
                                    let mut delete_id = None;
                                    for session in &self.ai_chat_sessions {
                                        ui.horizontal(|ui| {
                                            ui.set_min_height(28.0); // Taller dropdown items
                                            ui.style_mut().spacing.interact_size.y = 28.0;
                                            
                                            let is_selected = selected_id.as_ref() == Some(&session.id);
                                            if is_selected {
                                                let is_dark = ui.visuals().dark_mode;
                                                ui.style_mut().visuals.selection.bg_fill = if is_dark {
                                                    egui::Color32::from_rgb(65, 65, 65)
                                                } else {
                                                    egui::Color32::from_rgb(210, 210, 210)
                                                };
                                            }
                                            
                                            if ui.add_sized(
                                                [ui.available_width() - 32.0, 28.0],
                                                egui::Button::new(&session.name).selected(is_selected)
                                            ).clicked() {
                                                selected_id = Some(session.id.clone());
                                            }
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("🗑").on_hover_text("Delete Session").clicked() {
                                            delete_id = Some(session.id.clone());
                                        }
                                    });
                                    });
                                }
                                if let Some(id) = delete_id {
                                    self.ai_chat_sessions.retain(|s| s.id != id);
                                    if selected_id.as_ref() == Some(&id) {
                                        selected_id = self.ai_chat_sessions.last().map(|s| s.id.clone());
                                    }
                                    self.save_settings();
                                }
                            }
                        });
                    });
                    
                    if selected_id != self.ai_active_session_id {
                        self.ai_active_session_id = selected_id;
                        self.save_settings();
                    }

                });
                ui.separator();

                egui::Panel::bottom("ai_chat_input_panel")
                    .frame(egui::Frame::default().inner_margin(egui::Margin::symmetric(0, 8)))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let res = ui.add_sized(
                                [ui.available_width() - 44.0, 36.0],
                                egui::TextEdit::multiline(&mut self.ai_chat_input)
                                    .hint_text("Follow up question...")
                                    .margin(egui::Margin::symmetric(8, 8)),
                            );

                            if res.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift) {
                                send_msg = true;
                            }

                            if ui.add_enabled(
                                !self.ai_chat_loading && !self.ai_chat_input.trim().is_empty() && self.ai_active_session_id.is_some(),
                                egui::Button::new(egui::RichText::new("▶").color(egui::Color32::WHITE).size(16.0))
                                    .fill(egui::Color32::from_rgb(0, 122, 255))
                                    .corner_radius(18.0)
                                    .min_size(egui::vec2(36.0, 36.0)),
                            ).clicked() {
                                send_msg = true;
                            }
                        });
                    });
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if let Some(active_id) = &self.ai_active_session_id {
                            if let Some(session) = self.ai_chat_sessions.iter().find(|s| &s.id == active_id) {
                                let history_len = session.messages.len();
                                for (i, msg) in session.messages.iter().enumerate() {
                                    if msg.role == "system" || i == 1 {
                                        continue;
                                    }

                                    ui.add_space(8.0);
                                    if msg.role == "user" {
                                        ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                                            let text = msg.content.clone();
                                            let frame = egui::Frame::NONE
                                                .fill(egui::Color32::from_rgb(0, 122, 255))
                                                .corner_radius(8.0)
                                                .inner_margin(egui::Margin::same(8))
                                                .outer_margin(egui::Margin { left: 0, right: 12, top: 0, bottom: 0 });
                                            
                                            frame.show(ui, |ui| {
                                                for paragraph in text.split('\n') {
                                                    if paragraph.trim().is_empty() {
                                                        ui.add_space(4.0);
                                                    } else {
                                                        ui.label(egui::RichText::new(paragraph).color(egui::Color32::WHITE).size(14.0));
                                                        ui.add_space(6.0); // Space between lines of generated content
                                                    }
                                                }
                                            });
                                        });
                                    } else {
                                        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                            let mut text = msg.content.clone();
                                            
                                            // Animation logic for the last assistant message
                                            if i == history_len - 1 && !self.ai_chat_loading && self.ai_chat_display_len < text.len() {
                                                let chars_per_second = 800.0;
                                                let elapsed = ui.ctx().input(|i| i.time) - self.ai_chat_start_time;
                                                let expected_len = (elapsed * chars_per_second) as usize;
                                                
                                                self.ai_chat_display_len = expected_len.min(text.len());
                                                text = text[..self.ai_chat_display_len].to_string();
                                                
                                                if self.ai_chat_display_len < msg.content.len() {
                                                    ui.ctx().request_repaint();
                                                }
                                            }

                                            let frame = egui::Frame::NONE
                                                .fill(ui.visuals().faint_bg_color)
                                                .corner_radius(8.0)
                                                .inner_margin(egui::Margin::same(8));
                                            
                                            frame.show(ui, |ui| {
                                                for paragraph in text.split('\n') {
                                                    if paragraph.trim().is_empty() {
                                                        ui.add_space(4.0);
                                                    } else {
                                                        ui.label(egui::RichText::new(paragraph).color(ui.visuals().text_color()).size(14.0));
                                                        ui.add_space(6.0); // Space between lines of generated content
                                                    }
                                                }
                                            });
                                        });
                                    }
                                }
                            }
                        }

                        if self.ai_chat_loading {
                            ui.add_space(16.0);
                            ui.horizontal(|ui| {
                                ui.spinner();
                                let mut text = "Thinking...";
                                if let Some(active_id) = &self.ai_active_session_id {
                                    if let Some(session) = self.ai_chat_sessions.iter().find(|s| &s.id == active_id) {
                                        if session.messages.len() <= 2 {
                                            text = "Summarizing...";
                                        }
                                    }
                                }
                                ui.label(text);
                            });
                            ui.ctx().request_repaint(); // keep animating spinner
                        } else if let Some(err) = &self.ai_chat_error {
                            ui.add_space(16.0);
                            ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                        }
                        
                        ui.add_space(8.0);
                    });

                // Handle sending messages
                if send_msg {
                    if !self.ai_chat_input.trim().is_empty() {
                        if let Some(active_id) = &self.ai_active_session_id {
                            let text = self.ai_chat_input.trim().to_string();
                            self.ai_chat_input.clear();
                        
                        let mut messages_to_send = Vec::new();
                        
                        if let Some(session) = self.ai_chat_sessions.iter_mut().find(|s| &s.id == active_id) {
                            session.messages.push(crate::app::ChatMessage {
                                role: "user".to_string(),
                                content: text,
                            });
                            messages_to_send = session.messages.clone();
                            self.save_settings();
                        }
                        
                        self.ai_chat_loading = true;
                        self.ai_chat_error = None;

                        let _ = self.pdf_task_tx.send(crate::worker::PdfWorkerTask::AiSummarize {
                            is_chatbot: true,
                            messages: messages_to_send,
                            endpoint_url: self.llm_endpoint_url.clone(),
                            model: self.llm_model.clone(),
                            api_key: self.llm_api_key.clone(),
                            ctx: ui.ctx().clone(),
                        });
                        }
                    } else {
                        // If they pressed send on empty input (e.g. just pressed Enter), clear the stray newline
                        self.ai_chat_input.clear();
                    }
                }
            });
    }
}
