use crate::app::NixobdoPdfApp;
use crate::document::PdfWorkerMessage;
use eframe::egui;

impl NixobdoPdfApp {
    pub(crate) fn process_messages(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.pdf_receiver.try_recv() {
            match msg {
                PdfWorkerMessage::DocumentInfo { path, file_name, page_count, error } => {
                    let mut tab_to_remove = None;
                    for (i, tab) in self.tabs.iter_mut().enumerate() {
                        if tab.path == path {
                            if let Some(err) = error {
                                if err.contains("NotFound") || err.contains("cannot find the path specified") || err.contains("cannot find the file specified") {
                                    rfd::MessageDialog::new()
                                        .set_title("File Not Available")
                                        .set_description("The file you are trying to open is no longer available and cannot be opened.")
                                        .set_level(rfd::MessageLevel::Warning)
                                        .show();
                                } else {
                                    rfd::MessageDialog::new()
                                        .set_title("Nixobdo PDF Reader")
                                        .set_description(&format!("Nixobdo PDF Reader could not open '{}' because it is either not a supported file type or because the file has been damaged.", file_name))
                                        .set_level(rfd::MessageLevel::Info)
                                        .show();
                                }
                                tab_to_remove = Some(i);
                            } else {
                                tab.file_name = file_name;
                                tab.pages = vec![None; page_count];
                                tab.thumbnails = vec![None; page_count];
                                tab.page_texts = vec![String::new(); page_count];
                                tab.page_chars = vec![Vec::new(); page_count];
                                tab.page_links = vec![Vec::new(); page_count];
                                tab.page_rotations = vec![0; page_count];
                                tab.is_loading = false; // Turn off main loading, pages will pop in
                            }
                            break;
                        }
                    }
                    if let Some(idx) = tab_to_remove {
                        self.close_tab(idx);
                    }
                }
                PdfWorkerMessage::PageData { path, index, image, thumbnail_image, text, chars, links } => {
                    for tab in self.tabs.iter_mut() {
                        if tab.path == path {
                            if index < tab.pages.len() {
                                let texture = ctx.load_texture(
                                    format!("doc_{}_page_{}", tab.file_name, index),
                                    image,
                                    egui::TextureOptions::LINEAR,
                                );
                                let thumb_texture = ctx.load_texture(
                                    format!("doc_{}_thumb_{}", tab.file_name, index),
                                    thumbnail_image,
                                    egui::TextureOptions::LINEAR,
                                );
                                tab.pages[index] = Some(texture);
                                tab.thumbnails[index] = Some(thumb_texture);
                                tab.page_texts[index] = text;
                                tab.page_chars[index] = chars;
                                tab.page_links[index] = links;
                            }
                            break;
                        }
                    }
                }
                PdfWorkerMessage::Finished { path: _ } => {}
                PdfWorkerMessage::ExportProgress { progress } => {
                    self.export_progress = Some(progress);
                }
                PdfWorkerMessage::ExportComplete { success, message } => {
                    self.export_progress = None;
                    self.toast_message = Some(message);
                    self.toast_success = success;
                    self.toast_timer = ctx.input(|i| i.time) + 4.0; // show for 4 seconds
                }
                PdfWorkerMessage::SignatureSaved { path } => {
                    self.toast_message = Some("Signature added successfully".to_string());
                    self.toast_success = true;
                    self.toast_timer = ctx.input(|i| i.time) + 4.0;
                    self.is_placing_signature = false;
                    self.is_saving_signature = false;
                    self.reload_pdf(ctx, path);
                }
                PdfWorkerMessage::RotationSaved { path } => {
                    self.toast_message = Some("Document rotated successfully".to_string());
                    self.toast_success = true;
                    self.toast_timer = ctx.input(|i| i.time) + 4.0;
                    self.is_rotating_document = false;
                    self.is_saving_rotation = false;
                    self.pending_rotation = 0;
                    self.reload_pdf(ctx, path);
                }
                PdfWorkerMessage::AnnotationsSaved { path } => {
                    self.toast_message = Some("Annotations saved successfully".to_string());
                    self.toast_success = true;
                    self.toast_timer = ctx.input(|i| i.time) + 4.0;
                    self.is_saving_annotations = false;
                    self.is_annotation_mode = false;
                    self.active_annotation_tool = None;
                    self.pending_annotations.clear();
                    self.redo_annotations.clear();
                    self.reload_pdf(ctx, path);
                }

                PdfWorkerMessage::UpdateCheckResult(is_available, version, is_manual) => {
                    use crate::app::UpdateState;
                    if is_available {
                        self.update_state = UpdateState::Prompt(version.unwrap_or_else(|| "unknown".into()));
                    } else {
                        self.update_state = UpdateState::None;
                        if is_manual {
                            rfd::MessageDialog::new()
                                .set_title("No Update")
                                .set_description("No update available.")
                                .set_level(rfd::MessageLevel::Warning)
                                .show();
                        }
                    }
                }
                PdfWorkerMessage::UpdateDownloadProgress(progress) => {
                    use crate::app::UpdateState;
                    self.update_state = UpdateState::Downloading(progress);
                }
                PdfWorkerMessage::UpdateDownloadComplete(result) => {
                    use crate::app::UpdateState;
                    self.update_state = UpdateState::None;
                    match result {
                        Ok(path) => {
                            if let Err(e) = std::process::Command::new(&path).spawn() {
                                self.toast_message = Some(format!("Failed to start installer: {}", e));
                                self.toast_success = false;
                                self.toast_timer = ctx.input(|i| i.time) + 4.0;
                            } else {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                std::process::exit(0);
                            }
                        }
                        Err(e) => {
                            self.toast_message = Some(format!("Download failed: {}", e));
                            self.toast_success = false;
                            self.toast_timer = ctx.input(|i| i.time) + 4.0;
                        }
                    }
                }
            }
        }
    }
}
