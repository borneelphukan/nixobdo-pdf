pub mod export;

use eframe::egui;
use pdfium_render::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;

use crate::document::{PdfDocumentState, PdfWorkerMessage};
pub use export::ExportFormat;

pub enum PdfWorkerTask {
    Load {
        path: PathBuf,
        ctx: egui::Context,
    },
    Export {
        path: PathBuf,
        out_path: PathBuf,
        format: ExportFormat,
        retain_layout: bool,
        include_images: bool,
        ctx: egui::Context,
        cancel_flag: Arc<AtomicBool>,
    },
    CheckUpdate {
        is_manual: bool,
        ctx: egui::Context,
    },
    DownloadUpdate {
        version: String,
        ctx: egui::Context,
    },
    SaveSignature {
        path: PathBuf,
        page_index: usize,
        image_path: PathBuf,
        position: (f32, f32),
        scale: f32,
        ctx: egui::Context,
    },
    SaveRotation {
        path: PathBuf,
        rotation: i32,
        ctx: egui::Context,
    },
    SaveAnnotations {
        path: PathBuf,
        annotations: Vec<crate::document::AnnotationAction>,
        ctx: egui::Context,
    },
    AiSummarize {
        is_chatbot: bool,
        messages: Vec<crate::app::ChatMessage>,
        endpoint_url: String,
        model: String,
        api_key: String,
        ctx: egui::Context,
    },
}

pub fn spawn_worker_thread(task_rx: Receiver<PdfWorkerTask>, msg_tx: Sender<PdfWorkerMessage>) {
    let msg_tx_clone = msg_tx.clone();

    thread::spawn(move || {
        let exe_path = std::env::current_exe().ok().unwrap_or_default();
        let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new(""));

        let pdfium_result =
            Pdfium::bind_to_library(exe_dir.join("libpdfium.dylib").to_str().unwrap_or_default())
                .or_else(|_| {
                    Pdfium::bind_to_library(exe_dir.join("pdfium.dll").to_str().unwrap_or_default())
                })
                .or_else(|_| {
                    Pdfium::bind_to_library(
                        exe_dir.join("libpdfium.so").to_str().unwrap_or_default(),
                    )
                })
                .or_else(|_| Pdfium::bind_to_library("./lib/libpdfium.dylib"))
                .or_else(|_| Pdfium::bind_to_library("libpdfium.dylib"))
                .or_else(|_| Pdfium::bind_to_library("./lib/pdfium.dll"))
                .or_else(|_| Pdfium::bind_to_library("pdfium.dll"))
                .or_else(|_| Pdfium::bind_to_library("./lib/libpdfium.so"))
                .or_else(|_| Pdfium::bind_to_library("libpdfium.so"))
                .or_else(|_| Pdfium::bind_to_system_library());

        let pdfium = match pdfium_result {
            Ok(bindings) => Some(Pdfium::new(bindings)),
            Err(_) => None,
        };

        while let Ok(task) = task_rx.recv() {
            if let Some(pdf) = &pdfium {
                match task {
                    PdfWorkerTask::Load { path, ctx } => {
                        PdfDocumentState::background_load_with_pdfium(
                            path,
                            pdf,
                            msg_tx_clone.clone(),
                            ctx,
                        );
                    }
                    PdfWorkerTask::Export {
                        path,
                        out_path,
                        format,
                        retain_layout,
                        include_images,
                        ctx,
                        cancel_flag,
                    } => {
                        let tx = msg_tx_clone.clone();
                        // Wrap in catch_unwind so a panic in export doesn't kill the worker thread
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            if format == ExportFormat::Png || format == ExportFormat::Jpeg {
                                export::export_image(
                                    pdf,
                                    &path,
                                    &out_path,
                                    format,
                                    &tx,
                                    &ctx,
                                    &cancel_flag,
                                )
                            } else if format == ExportFormat::Docx {
                                export::export_docx(
                                    pdf,
                                    &path,
                                    &out_path,
                                    retain_layout,
                                    include_images,
                                    &tx,
                                    &ctx,
                                    &cancel_flag,
                                )
                            } else {
                                export::export_doc_rtf(
                                    pdf,
                                    &path,
                                    &out_path,
                                    retain_layout,
                                    include_images,
                                    &tx,
                                    &ctx,
                                    &cancel_flag,
                                )
                            }
                        }));

                        let (success, message) = match result {
                            Ok(Ok(msg)) => (true, msg),
                            Ok(Err(msg)) => (false, msg),
                            Err(_) => (
                                false,
                                "Export failed unexpectedly (internal error).".to_string(),
                            ),
                        };

                        let _ = msg_tx_clone
                            .send(PdfWorkerMessage::ExportComplete { success, message });
                        ctx.request_repaint();
                    }
                    PdfWorkerTask::CheckUpdate { is_manual, ctx } => {
                        let tx = msg_tx_clone.clone();
                        std::thread::spawn(move || {
                            let url = "https://api.github.com/repos/borneelphukan/nixobdo-pdf/releases/latest";
                            let mut is_available = false;
                            let mut version = None;
                            if let Ok(response) =
                                ureq::get(url).header("User-Agent", "nixobdo-pdf").call()
                            {
                                use std::io::Read;
                                let mut json = String::new();
                                if response
                                    .into_body()
                                    .into_reader()
                                    .read_to_string(&mut json)
                                    .is_ok()
                                {
                                    if let Some(tag_idx) = json.find("\"tag_name\":") {
                                        let rest = &json[tag_idx + 11..];
                                        if let Some(start_quote) = rest.find('"') {
                                            let rest = &rest[start_quote + 1..];
                                            if let Some(end_quote) = rest.find('"') {
                                                let tag = &rest[..end_quote];
                                                let latest_version = tag.trim_start_matches('v');
                                                if latest_version != env!("CARGO_PKG_VERSION") {
                                                    is_available = true;
                                                    version = Some(latest_version.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            let _ = tx.send(PdfWorkerMessage::UpdateCheckResult(
                                is_available,
                                version,
                                is_manual,
                            ));
                            ctx.request_repaint();
                        });
                    }
                    PdfWorkerTask::DownloadUpdate { version, ctx } => {
                        let tx = msg_tx_clone.clone();
                        std::thread::spawn(move || {
                            let url = format!("https://github.com/borneelphukan/nixobdo-pdf/releases/download/v{}/nixobdo-pdfSetup.exe", version);
                            match ureq::get(&url).header("User-Agent", "nixobdo-pdf").call() {
                                Ok(response) => {
                                    let len: Option<u64> = response
                                        .headers()
                                        .get("Content-Length")
                                        .and_then(|h| h.to_str().ok())
                                        .and_then(|s| s.parse().ok());
                                    let mut reader = response.into_body().into_reader();
                                    let download_dir =
                                        dirs::download_dir().unwrap_or_else(|| PathBuf::from("."));
                                    let out_path = download_dir.join("nixobdo-pdfSetup.exe");
                                    if let Ok(mut file) = std::fs::File::create(&out_path) {
                                        use std::io::Read;
                                        let mut buf = [0; 8192];
                                        let mut downloaded = 0;
                                        loop {
                                            match reader.read(&mut buf) {
                                                Ok(0) => break,
                                                Ok(n) => {
                                                    use std::io::Write;
                                                    let _ = file.write_all(&buf[..n]);
                                                    downloaded += n as u64;
                                                    if let Some(total) = len {
                                                        let progress =
                                                            (downloaded as f32) / (total as f32);
                                                        let _ = tx.send(PdfWorkerMessage::UpdateDownloadProgress(progress));
                                                        ctx.request_repaint();
                                                    }
                                                }
                                                Err(_) => {
                                                    let _ = tx.send(
                                                        PdfWorkerMessage::UpdateDownloadComplete(
                                                            Err("Download failed".into()),
                                                        ),
                                                    );
                                                    ctx.request_repaint();
                                                    return;
                                                }
                                            }
                                        }
                                        let _ = tx.send(PdfWorkerMessage::UpdateDownloadComplete(
                                            Ok(out_path.to_string_lossy().into()),
                                        ));
                                    } else {
                                        let _ = tx.send(PdfWorkerMessage::UpdateDownloadComplete(
                                            Err("Failed to create file".into()),
                                        ));
                                    }
                                    ctx.request_repaint();
                                }
                                Err(_) => {
                                    let _ = tx.send(PdfWorkerMessage::UpdateDownloadComplete(Err(
                                        "Download failed".into(),
                                    )));
                                    ctx.request_repaint();
                                }
                            }
                        });
                    }
                    PdfWorkerTask::SaveSignature {
                        path,
                        page_index,
                        image_path,
                        position,
                        scale,
                        ctx,
                    } => {
                        let tx = msg_tx_clone.clone();
                        // Open the image to get its dimensions
                        if let Ok(img) = image::open(&image_path) {
                            let img_w = img.width() as f32;
                            let img_h = img.height() as f32;
                            let aspect = img_h / img_w;

                            // Let's assume signature is 200px wide for layout, but we need PDF points
                            // Standard PDF points: 72 per inch
                            let target_w = 150.0_f32 * scale; // 150 points width * scale
                            let target_h = target_w * aspect;

                            match pdf.load_pdf_from_file(path.to_str().unwrap_or_default(), None) {
                                Ok(doc) => {
                                    if let Ok(mut page) = doc.pages().get(page_index as u16) {
                                        let page_w = page.width().value;
                                        let page_h = page.height().value;

                                        // Calculate the position in PDF coordinates
                                        let x = position.0 * page_w - target_w / 2.0;
                                        let y = page_h - (position.1 * page_h) - target_h / 2.0;

                                        if let Ok(mut object) = pdfium_render::prelude::PdfPageImageObject::new_with_width(
                                            &doc,
                                            &img,
                                            pdfium_render::prelude::PdfPoints::new(target_w)
                                        ) {
                                            let _ = object.translate(
                                                pdfium_render::prelude::PdfPoints::new(x as f32),
                                                pdfium_render::prelude::PdfPoints::new(y as f32)
                                            );

                                            let _ = page.objects_mut().add_image_object(object);

                                            // Save the document back
                                            if let Ok(_) = doc.save_to_file(path.to_str().unwrap_or_default()) {
                                                let _ = tx.send(PdfWorkerMessage::SignatureSaved { path: path.clone() });
                                            } else {
                                                let _ = tx.send(PdfWorkerMessage::ExportComplete { success: false, message: "Failed to save PDF".into() });
                                            }
                                        } else {
                                            let _ = tx.send(PdfWorkerMessage::ExportComplete { success: false, message: "Failed to create image object".into() });
                                        }
                                    }
                                }
                                Err(_) => {
                                    let _ = tx.send(PdfWorkerMessage::ExportComplete {
                                        success: false,
                                        message: "Failed to load PDF for saving".into(),
                                    });
                                }
                            }
                        }
                        ctx.request_repaint();
                    }

                    PdfWorkerTask::SaveRotation {
                        path,
                        rotation,
                        ctx,
                    } => {
                        let tx = msg_tx_clone.clone();
                        match pdf.load_pdf_from_file(path.to_str().unwrap_or_default(), None) {
                            Ok(doc) => {
                                for i in 0..doc.pages().len() {
                                    if let Ok(mut page) = doc.pages().get(i) {
                                        let current_rot = match page.rotation().unwrap_or(pdfium_render::prelude::PdfPageRenderRotation::None) {
                                            pdfium_render::prelude::PdfPageRenderRotation::None => 0,
                                            pdfium_render::prelude::PdfPageRenderRotation::Degrees90 => 90,
                                            pdfium_render::prelude::PdfPageRenderRotation::Degrees180 => 180,
                                            pdfium_render::prelude::PdfPageRenderRotation::Degrees270 => 270,
                                        };
                                        let mut new_deg = (current_rot + rotation) % 360;
                                        if new_deg < 0 {
                                            new_deg += 360;
                                        }
                                        let new_rot = match new_deg {
                                            90 => pdfium_render::prelude::PdfPageRenderRotation::Degrees90,
                                            180 => pdfium_render::prelude::PdfPageRenderRotation::Degrees180,
                                            270 => pdfium_render::prelude::PdfPageRenderRotation::Degrees270,
                                            _ => pdfium_render::prelude::PdfPageRenderRotation::None,
                                        };
                                        page.set_rotation(new_rot);
                                    }
                                }
                                if let Ok(_) = doc.save_to_file(path.to_str().unwrap_or_default()) {
                                    let _ = tx.send(PdfWorkerMessage::RotationSaved {
                                        path: path.clone(),
                                    });
                                } else {
                                    let _ = tx.send(PdfWorkerMessage::ExportComplete {
                                        success: false,
                                        message: "Failed to save PDF".into(),
                                    });
                                }
                            }
                            Err(_) => {
                                let _ = tx.send(PdfWorkerMessage::ExportComplete {
                                    success: false,
                                    message: "Failed to load PDF for saving".into(),
                                });
                            }
                        }
                        ctx.request_repaint();
                    }
                    PdfWorkerTask::SaveAnnotations {
                        path,
                        annotations,
                        ctx,
                    } => {
                        let tx = msg_tx_clone.clone();
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            match pdf.load_pdf_from_file(path.to_str().unwrap_or_default(), None) {
                                Ok(mut doc) => {
                                    for action in annotations {
                                        if let Ok(mut page) =
                                            doc.pages().get(action.page_index as u16)
                                        {
                                            let page_w = page.width().value;
                                            let page_h = page.height().value;

                                            let pdf_color = pdfium_render::prelude::PdfColor::new(
                                                action.color.r(),
                                                action.color.g(),
                                                action.color.b(),
                                                255,
                                            );

                                            match action.tool {
                                                crate::document::AnnotationTool::Highlight => {
                                                    if action.rects.is_empty() {
                                                        continue;
                                                    }
                                                    for r in &action.rects {
                                                        let x1 =
                                                            pdfium_render::prelude::PdfPoints::new(
                                                                r.min.x * page_w,
                                                            );
                                                        let y1 =
                                                            pdfium_render::prelude::PdfPoints::new(
                                                                (1.0 - r.max.y) * page_h,
                                                            );
                                                        let x2 =
                                                            pdfium_render::prelude::PdfPoints::new(
                                                                r.max.x * page_w,
                                                            );
                                                        let y2 =
                                                            pdfium_render::prelude::PdfPoints::new(
                                                                (1.0 - r.min.y) * page_h,
                                                            );

                                                        let highlight_color =
                                                            pdfium_render::prelude::PdfColor::new(
                                                                action.color.r(),
                                                                action.color.g(),
                                                                action.color.b(),
                                                                100,
                                                            );
                                                        if let Ok(mut path) = pdfium_render::prelude::PdfPagePathObject::new(&doc, x1, y1, None, None, Some(highlight_color)) {
                                                              let _ = path.line_to(x2, y1);
                                                              let _ = path.line_to(x2, y2);
                                                              let _ = path.line_to(x1, y2);
                                                              let _ = path.close_path();
                                                              let _ = path.set_fill_and_stroke_mode(pdfium_render::prelude::PdfPathFillMode::Winding, false);
                                                              let _ = path.set_blend_mode(pdfium_render::prelude::PdfPageObjectBlendMode::Multiply);
                                                              let _ = page.objects_mut().add_path_object(path);
                                                          }
                                                    }
                                                }
                                                crate::document::AnnotationTool::Underline => {
                                                    if action.rects.is_empty() {
                                                        continue;
                                                    }
                                                    for r in &action.rects {
                                                        let x1 =
                                                            pdfium_render::prelude::PdfPoints::new(
                                                                r.min.x * page_w,
                                                            );
                                                        let x2 =
                                                            pdfium_render::prelude::PdfPoints::new(
                                                                r.max.x * page_w,
                                                            );
                                                        let y =
                                                            pdfium_render::prelude::PdfPoints::new(
                                                                (1.0 - r.max.y) * page_h,
                                                            );

                                                        if let Ok(path) = pdfium_render::prelude::PdfPagePathObject::new_line(&doc, x1, y, x2, y, pdf_color, pdfium_render::prelude::PdfPoints::new(2.0)) {
                                                              let _ = page.objects_mut().add_path_object(path);
                                                          }
                                                    }
                                                }
                                                crate::document::AnnotationTool::Strikethrough => {
                                                    if action.rects.is_empty() {
                                                        continue;
                                                    }
                                                    for r in &action.rects {
                                                        let x1 =
                                                            pdfium_render::prelude::PdfPoints::new(
                                                                r.min.x * page_w,
                                                            );
                                                        let x2 =
                                                            pdfium_render::prelude::PdfPoints::new(
                                                                r.max.x * page_w,
                                                            );
                                                        let y =
                                                            pdfium_render::prelude::PdfPoints::new(
                                                                (1.0 - (r.min.y + r.max.y) / 2.0)
                                                                    * page_h,
                                                            );
                                                        if let Ok(path) = pdfium_render::prelude::PdfPagePathObject::new_line(&doc, x1, y, x2, y, pdf_color, pdfium_render::prelude::PdfPoints::new(2.0)) {
                                                              let _ = page.objects_mut().add_path_object(path);
                                                          }
                                                    }
                                                }
                                                crate::document::AnnotationTool::Redact => {
                                                    if action.rects.is_empty() {
                                                        continue;
                                                    }
                                                    for r in &action.rects {
                                                        let x1 =
                                                            pdfium_render::prelude::PdfPoints::new(
                                                                r.min.x * page_w,
                                                            );
                                                        let y1 =
                                                            pdfium_render::prelude::PdfPoints::new(
                                                                (1.0 - r.max.y) * page_h,
                                                            );
                                                        let x2 =
                                                            pdfium_render::prelude::PdfPoints::new(
                                                                r.max.x * page_w,
                                                            );
                                                        let y2 =
                                                            pdfium_render::prelude::PdfPoints::new(
                                                                (1.0 - r.min.y) * page_h,
                                                            );

                                                        if let Ok(mut path) = pdfium_render::prelude::PdfPagePathObject::new(&doc, x1, y1, None, None, Some(pdfium_render::prelude::PdfColor::new(0, 0, 0, 255))) {
                                                              let _ = path.line_to(x2, y1);
                                                              let _ = path.line_to(x2, y2);
                                                              let _ = path.line_to(x1, y2);
                                                              let _ = path.close_path();
                                                              let _ = path.set_fill_and_stroke_mode(pdfium_render::prelude::PdfPathFillMode::Winding, false);
                                                              let _ = page.objects_mut().add_path_object(path);
                                                          }
                                                    }
                                                }
                                                crate::document::AnnotationTool::Text => {
                                                    if let Some(text) = &action.text {
                                                        if !text.is_empty() {
                                                            let pos = action
                                                                .position
                                                                .unwrap_or(egui::pos2(0.5, 0.5));
                                                            let scale =
                                                                action.scale.unwrap_or(12.0);

                                                            let font_size = scale;
                                                            let doc_fonts = doc.fonts_mut();
                                                            let font = doc_fonts.helvetica();

                                                            if let Ok(mut text_obj) = pdfium_render::prelude::PdfPageTextObject::new(&doc, text, font, pdfium_render::prelude::PdfPoints::new(font_size)) {
                                                                  let _ = text_obj.set_fill_color(pdf_color);

                                                                  let text_w = text_obj.width().unwrap_or(pdfium_render::prelude::PdfPoints::new(0.0)).value;
                                                                  let text_h = font_size; // approximate height

                                                                  let _ = text_obj.translate(
                                                                      pdfium_render::prelude::PdfPoints::new(pos.x * page_w - text_w / 2.0),
                                                                      pdfium_render::prelude::PdfPoints::new((1.0 - pos.y) * page_h - text_h / 2.0)
                                                                  );

                                                                  let _ = page.objects_mut().add_text_object(text_obj);
                                                              }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Ensure we save via a temporary file to avoid file lock issues on Windows
                                    let mut tmp_path = std::env::temp_dir();
                                    tmp_path.push(format!(
                                        "tmp_pdf_export_{}_{}.pdf",
                                        std::process::id(),
                                        std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_millis()
                                    ));
                                    if let Ok(_) = doc.save_to_file(&tmp_path) {
                                        drop(doc); // Release file lock on Windows before copy!

                                        // Retry copying up to 10 times in case of transient locks from antivirus, etc.
                                        let mut copy_success = false;
                                        for _ in 0..10 {
                                            if std::fs::copy(&tmp_path, &path).is_ok() {
                                                copy_success = true;
                                                break;
                                            }
                                            std::thread::sleep(std::time::Duration::from_millis(
                                                300,
                                            ));
                                        }

                                        if copy_success {
                                            let _ = std::fs::remove_file(&tmp_path);
                                            let _ = tx.send(PdfWorkerMessage::AnnotationsSaved {
                                                path: path.clone(),
                                            });
                                        } else {
                                            let _ = tx.send(PdfWorkerMessage::ExportComplete { success: false, message: "Failed to copy saved annotations PDF. Is it open in another program?".into() });
                                        }
                                    } else {
                                        let _ = tx.send(PdfWorkerMessage::ExportComplete {
                                            success: false,
                                            message: "Failed to save PDF annotations".into(),
                                        });
                                    }
                                }
                                Err(_) => {
                                    let _ = tx.send(PdfWorkerMessage::ExportComplete {
                                        success: false,
                                        message: "Failed to load PDF for saving".into(),
                                    });
                                }
                            }
                        }));

                        if result.is_err() {
                            let _ = msg_tx_clone.send(PdfWorkerMessage::ExportComplete {
                                success: false,
                                message: "A fatal error occurred while saving annotations.".into(),
                            });
                        }

                        ctx.request_repaint();
                    }
                    PdfWorkerTask::AiSummarize {
                        is_chatbot,
                        messages,
                        endpoint_url,
                        model,
                        api_key,
                        ctx,
                    } => {
                        let tx = msg_tx_clone.clone();
                        std::thread::spawn(move || {
                            let result =
                                summarize_with_llm(messages, &endpoint_url, &model, &api_key);
                            let (success, response_text, error) = match result {
                                Ok(t) => (true, t, None),
                                Err(e) => (false, String::new(), Some(e)),
                            };
                            let _ = tx.send(PdfWorkerMessage::AiSummaryResult {
                                is_chatbot,
                                success,
                                text: response_text,
                                error,
                            });
                            ctx.request_repaint();
                        });
                    }
                }
            } else {
                match task {
                    PdfWorkerTask::CheckUpdate { is_manual, ctx } => {
                        let tx = msg_tx_clone.clone();
                        std::thread::spawn(move || {
                            let url = "https://api.github.com/repos/borneelphukan/nixobdo-pdf/releases/latest";
                            let mut is_available = false;
                            let mut version = None;
                            if let Ok(response) =
                                ureq::get(url).header("User-Agent", "nixobdo-pdf").call()
                            {
                                use std::io::Read;
                                let mut json = String::new();
                                if response
                                    .into_body()
                                    .into_reader()
                                    .read_to_string(&mut json)
                                    .is_ok()
                                {
                                    if let Some(tag_idx) = json.find("\"tag_name\":") {
                                        let rest = &json[tag_idx + 11..];
                                        if let Some(start_quote) = rest.find('"') {
                                            let rest = &rest[start_quote + 1..];
                                            if let Some(end_quote) = rest.find('"') {
                                                let tag = &rest[..end_quote];
                                                let latest_version = tag.trim_start_matches('v');
                                                if latest_version != env!("CARGO_PKG_VERSION") {
                                                    is_available = true;
                                                    version = Some(latest_version.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            let _ = tx.send(PdfWorkerMessage::UpdateCheckResult(
                                is_available,
                                version,
                                is_manual,
                            ));
                            ctx.request_repaint();
                        });
                    }
                    PdfWorkerTask::DownloadUpdate { version, ctx } => {
                        let tx = msg_tx_clone.clone();
                        std::thread::spawn(move || {
                            let url = format!("https://github.com/borneelphukan/nixobdo-pdf/releases/download/v{}/nixobdo-pdfSetup.exe", version);
                            match ureq::get(&url).header("User-Agent", "nixobdo-pdf").call() {
                                Ok(response) => {
                                    let len: Option<u64> = response
                                        .headers()
                                        .get("Content-Length")
                                        .and_then(|h| h.to_str().ok())
                                        .and_then(|s| s.parse().ok());
                                    let mut reader = response.into_body().into_reader();
                                    let download_dir =
                                        dirs::download_dir().unwrap_or_else(|| PathBuf::from("."));
                                    let out_path = download_dir.join("nixobdo-pdfSetup.exe");
                                    if let Ok(mut file) = std::fs::File::create(&out_path) {
                                        use std::io::Read;
                                        let mut buf = [0; 8192];
                                        let mut downloaded = 0;
                                        loop {
                                            match reader.read(&mut buf) {
                                                Ok(0) => break,
                                                Ok(n) => {
                                                    use std::io::Write;
                                                    let _ = file.write_all(&buf[..n]);
                                                    downloaded += n as u64;
                                                    if let Some(total) = len {
                                                        let progress =
                                                            (downloaded as f32) / (total as f32);
                                                        let _ = tx.send(PdfWorkerMessage::UpdateDownloadProgress(progress));
                                                        ctx.request_repaint();
                                                    }
                                                }
                                                Err(_) => {
                                                    let _ = tx.send(
                                                        PdfWorkerMessage::UpdateDownloadComplete(
                                                            Err("Download failed".into()),
                                                        ),
                                                    );
                                                    ctx.request_repaint();
                                                    return;
                                                }
                                            }
                                        }
                                        let _ = tx.send(PdfWorkerMessage::UpdateDownloadComplete(
                                            Ok(out_path.to_string_lossy().into()),
                                        ));
                                    } else {
                                        let _ = tx.send(PdfWorkerMessage::UpdateDownloadComplete(
                                            Err("Failed to create file".into()),
                                        ));
                                    }
                                    ctx.request_repaint();
                                }
                                Err(_) => {
                                    let _ = tx.send(PdfWorkerMessage::UpdateDownloadComplete(Err(
                                        "Download failed".into(),
                                    )));
                                    ctx.request_repaint();
                                }
                            }
                        });
                    }
                    PdfWorkerTask::Load { path, ctx } => {
                        let _ = msg_tx_clone.send(PdfWorkerMessage::DocumentInfo {
                            path: path.clone(),
                            file_name: String::new(),
                            page_count: 0,
                            error: Some(
                                "PDFium not initialized. Please ensure libpdfium is present."
                                    .into(),
                            ),
                        });
                        ctx.request_repaint();
                    }
                    PdfWorkerTask::AiSummarize {
                        is_chatbot,
                        messages,
                        endpoint_url,
                        model,
                        api_key,
                        ctx,
                    } => {
                        let tx = msg_tx_clone.clone();
                        std::thread::spawn(move || {
                            let result =
                                summarize_with_llm(messages, &endpoint_url, &model, &api_key);
                            let (success, response_text, error) = match result {
                                Ok(t) => (true, t, None),
                                Err(e) => (false, String::new(), Some(e)),
                            };
                            let _ = tx.send(PdfWorkerMessage::AiSummaryResult {
                                is_chatbot,
                                success,
                                text: response_text,
                                error,
                            });
                            ctx.request_repaint();
                        });
                    }
                    _ => {}
                }
            }
        }
    });
}

fn summarize_with_llm(
    messages: Vec<crate::app::ChatMessage>,
    endpoint_url: &str,
    model: &str,
    api_key: &str,
) -> Result<String, String> {
    let mut base_url = endpoint_url.trim_end_matches('/');
    if !api_key.is_empty()
        && (base_url.contains("localhost") || base_url.contains("127.0.0.1") || base_url.is_empty())
    {
        base_url = "https://api.openai.com/v1";
    }
    let url = if base_url.ends_with("/chat/completions") {
        base_url.to_string()
    } else if base_url.ends_with("/v1") {
        format!("{}/chat/completions", base_url)
    } else {
        format!("{}/v1/chat/completions", base_url)
    };

    let mut json_messages = Vec::new();
    for msg in messages {
        json_messages.push(serde_json::json!({
            "role": msg.role,
            "content": msg.content
        }));
    }

    let body = serde_json::json!({
        "model": model,
        "messages": json_messages,
        "temperature": 0.3,
        "max_tokens": 1500,
        "stream": false
    });

    let json_string =
        serde_json::to_string(&body).map_err(|e| format!("Failed to serialize request: {}", e))?;

    let mut req = ureq::post(&url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "nixobdo-pdf");

    if !api_key.is_empty() {
        req = req.header("Authorization", &format!("Bearer {}", api_key));
    }

    let response = req.send(json_string.as_bytes())
        .map_err(|e| {
            let err_str = e.to_string().to_lowercase();
            if err_str.contains("refused") || err_str.contains("10061") {
                format!("Connection Refused. If using Local LLM, ensure your server is running. If using API Key, check your Endpoint URL.")
            } else {
                format!("Request failed: {}", e)
            }
        })?;

    use std::io::Read;
    let mut json = String::new();
    response
        .into_body()
        .into_reader()
        .read_to_string(&mut json)
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let parsed: serde_json::Value = serde_json::from_str(&json).map_err(|e| {
        format!(
            "Failed to parse response: {} - Response: {}",
            e,
            &json[..json.len().min(200)]
        )
    })?;

    let content = parsed["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| {
            let error_msg = parsed["error"]["message"]
                .as_str()
                .unwrap_or("Unknown error");
            format!("API error: {}", error_msg)
        })?;

    Ok(content.trim().to_string())
}
