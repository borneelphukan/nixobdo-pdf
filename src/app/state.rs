use crate::app::NixobdoPdfApp;
use crate::document::PdfDocumentState;
use crate::worker::PdfWorkerTask;
use eframe::egui;
use std::path::PathBuf;

impl NixobdoPdfApp {
    pub(crate) fn get_selected_text(&self) -> Option<String> {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            if let Some(active_idx) = self.active_tab_index {
                if let Some(tab) = self.tabs.get(active_idx) {
                    let mut selected_text = String::new();
                    let (p_start, c_start, p_end, c_end) = if start.0 < end.0 {
                        (start.0, start.1, end.0, end.1)
                    } else if start.0 > end.0 {
                        (end.0, end.1, start.0, start.1)
                    } else {
                        let (c_min, c_max) = if start.1 <= end.1 {
                            (start.1, end.1)
                        } else {
                            (end.1, start.1)
                        };
                        (start.0, c_min, start.0, c_max)
                    };

                    for p_idx in p_start..=p_end {
                        if let Some(chars) = tab.page_chars.get(p_idx) {
                            let start_c = if p_idx == p_start { c_start } else { 0 };
                            let end_c = if p_idx == p_end {
                                c_end
                            } else {
                                chars.len().saturating_sub(1)
                            };
                            for c_idx in start_c..=end_c {
                                if let Some(char_info) = chars.get(c_idx) {
                                    selected_text.push(char_info.c);
                                }
                            }
                        }
                        if p_idx < p_end {
                            selected_text.push('\n');
                        }
                    }

                    if !selected_text.is_empty() {
                        return Some(selected_text);
                    }
                }
            }
        }
        None
    }

    pub(crate) fn copy_selection(&self, ctx: &egui::Context) {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            if let Some(active_idx) = self.active_tab_index {
                if let Some(tab) = self.tabs.get(active_idx) {
                    let mut selected_text = String::new();
                    let (p_start, c_start, p_end, c_end) = if start.0 < end.0 {
                        (start.0, start.1, end.0, end.1)
                    } else if start.0 > end.0 {
                        (end.0, end.1, start.0, start.1)
                    } else {
                        let (c_min, c_max) = if start.1 <= end.1 {
                            (start.1, end.1)
                        } else {
                            (end.1, start.1)
                        };
                        (start.0, c_min, start.0, c_max)
                    };

                    for p_idx in p_start..=p_end {
                        if let Some(chars) = tab.page_chars.get(p_idx) {
                            let start_c = if p_idx == p_start { c_start } else { 0 };
                            let end_c = if p_idx == p_end {
                                c_end
                            } else {
                                chars.len().saturating_sub(1)
                            };
                            for c_idx in start_c..=end_c {
                                if let Some(char_info) = chars.get(c_idx) {
                                    selected_text.push(char_info.c);
                                }
                            }
                        }
                        if p_idx < p_end {
                            selected_text.push('\n');
                        }
                    }

                    if !selected_text.is_empty() {
                        ctx.copy_text(selected_text);
                    }
                }
            }
        }
    }

    pub(crate) fn load_pdf(&mut self, ctx: &egui::Context, path: PathBuf) {
        // Manage recent files list
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path.clone());
        self.recent_files.truncate(5);
        self.save_settings();

        // Check if a tab with this path already exists
        if let Some(existing_idx) = self.tabs.iter().position(|t| t.path == path) {
            self.active_tab_index = Some(existing_idx);
            return;
        }

        let new_tab = PdfDocumentState::empty(path.clone());
        self.tabs.push(new_tab);
        self.active_tab_index = Some(self.tabs.len() - 1);

        let _ = self.pdf_task_tx.send(PdfWorkerTask::Load {
            path: path.clone(),
            ctx: ctx.clone(),
        });
    }

    pub(crate) fn reload_pdf(&mut self, ctx: &egui::Context, path: PathBuf) {
        if let Some(idx) = self.tabs.iter().position(|t| t.path == path.clone()) {
            self.tabs[idx].is_loading = true;
            let _ = self.pdf_task_tx.send(PdfWorkerTask::Load {
                path,
                ctx: ctx.clone(),
            });
        }
    }

    pub(crate) fn close_tab(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }
        self.tabs.remove(index);

        if self.tabs.is_empty() {
            self.active_tab_index = None;
        } else if let Some(active) = self.active_tab_index {
            if active == index {
                self.active_tab_index = Some(active.min(self.tabs.len() - 1));
            } else if active > index {
                self.active_tab_index = Some(active - 1);
            }
        }
    }

    pub(crate) fn save_settings(&self) {
        use serde::{Deserialize, Serialize};
        #[derive(Serialize, Deserialize)]
        struct AppSettings {
            recent_files: Vec<PathBuf>,
            llm_api_key: String,
            llm_model: String,
            llm_endpoint_url: String,
            llm_configured: bool,
            ai_chat_sessions: Vec<crate::app::AiChatSession>,
            ai_active_session_id: Option<String>,
        }
        let settings = AppSettings {
            recent_files: self.recent_files.clone(),
            llm_api_key: self.llm_api_key.clone(),
            llm_model: self.llm_model.clone(),
            llm_endpoint_url: self.llm_endpoint_url.clone(),
            llm_configured: self.llm_configured,
            ai_chat_sessions: self.ai_chat_sessions.clone(),
            ai_active_session_id: self.ai_active_session_id.clone(),
        };
        if let Some(config_dir) = dirs::config_dir() {
            let dir = config_dir.join("nixobdo-pdf");
            let _ = std::fs::create_dir_all(&dir);
            if let Ok(content) = serde_json::to_string_pretty(&settings) {
                let _ = std::fs::write(dir.join("settings.json"), content);
            }
        }
    }

}
