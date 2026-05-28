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
    Load { path: PathBuf, ctx: egui::Context },
    Export { path: PathBuf, out_path: PathBuf, format: ExportFormat, retain_layout: bool, include_images: bool, ctx: egui::Context, cancel_flag: Arc<AtomicBool> },
    CheckUpdate { ctx: egui::Context },
    DownloadUpdate { ctx: egui::Context },
}

pub fn spawn_worker_thread(
    task_rx: Receiver<PdfWorkerTask>,
    msg_tx: Sender<PdfWorkerMessage>,
) {
    let msg_tx_clone = msg_tx.clone();
    
    thread::spawn(move || {
        let exe_path = std::env::current_exe().ok().unwrap_or_default();
        let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new(""));
        
        let pdfium_result = Pdfium::bind_to_library(exe_dir.join("libpdfium.dylib").to_str().unwrap_or_default())
            .or_else(|_| Pdfium::bind_to_library(exe_dir.join("pdfium.dll").to_str().unwrap_or_default()))
            .or_else(|_| Pdfium::bind_to_library("./lib/libpdfium.dylib"))
            .or_else(|_| Pdfium::bind_to_library("libpdfium.dylib"))
            .or_else(|_| Pdfium::bind_to_library("./lib/pdfium.dll"))
            .or_else(|_| Pdfium::bind_to_library("pdfium.dll"))
            .or_else(|_| Pdfium::bind_to_system_library());

        let pdfium = match pdfium_result {
            Ok(bindings) => Some(Pdfium::new(bindings)),
            Err(_) => None,
        };

        while let Ok(task) = task_rx.recv() {
            if let Some(pdf) = &pdfium {
                match task {
                    PdfWorkerTask::Load { path, ctx } => {
                        PdfDocumentState::background_load_with_pdfium(path, pdf, msg_tx_clone.clone(), ctx);
                    }
                    PdfWorkerTask::Export { path, out_path, format, retain_layout, include_images, ctx, cancel_flag } => {
                        let tx = msg_tx_clone.clone();
                        // Wrap in catch_unwind so a panic in export doesn't kill the worker thread
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            if format == ExportFormat::Png || format == ExportFormat::Jpeg {
                                export::export_image(pdf, &path, &out_path, format, &tx, &ctx, &cancel_flag)
                            } else if format == ExportFormat::Docx {
                                export::export_docx(pdf, &path, &out_path, retain_layout, include_images, &tx, &ctx, &cancel_flag)
                            } else {
                                export::export_doc_rtf(pdf, &path, &out_path, retain_layout, include_images, &tx, &ctx, &cancel_flag)
                            }
                        }));
                        
                        let (success, message) = match result {
                            Ok(Ok(msg)) => (true, msg),
                            Ok(Err(msg)) => (false, msg),
                            Err(_) => (false, "Export failed unexpectedly (internal error).".to_string()),
                        };
                        
                        let _ = msg_tx_clone.send(PdfWorkerMessage::ExportComplete { success, message });
                        ctx.request_repaint();
                    }
                    PdfWorkerTask::CheckUpdate { ctx } => {
                        let tx = msg_tx_clone.clone();
                        std::thread::spawn(move || {
                            let url = "https://nightly.link/borneelphukan/pdf-viewer/workflows/windows-build/main/PDFViewer-Installer.zip";
                            let res = ureq::head(url).call();
                            let is_available = res.is_ok();
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            let _ = tx.send(PdfWorkerMessage::UpdateCheckResult(is_available));
                            ctx.request_repaint();
                        });
                    }
                    PdfWorkerTask::DownloadUpdate { ctx } => {
                        let tx = msg_tx_clone.clone();
                        std::thread::spawn(move || {
                            let url = "https://nightly.link/borneelphukan/pdf-viewer/workflows/windows-build/main/PDFViewer-Installer.zip";
                            match ureq::get(url).call() {
                                Ok(response) => {
                                    let len: Option<u64> = response.headers().get("Content-Length").and_then(|h| h.to_str().ok()).and_then(|s| s.parse().ok());
                                    let mut reader = response.into_body().into_reader();
                                    let download_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("."));
                                    let out_path = download_dir.join("PDFViewerSetup.zip");
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
                                                        let progress = (downloaded as f32) / (total as f32);
                                                        let _ = tx.send(PdfWorkerMessage::UpdateDownloadProgress(progress));
                                                        ctx.request_repaint();
                                                    }
                                                }
                                                Err(_) => {
                                                    let _ = tx.send(PdfWorkerMessage::UpdateDownloadComplete(Err("Download failed".into())));
                                                    ctx.request_repaint();
                                                    return;
                                                }
                                            }
                                        }
                                        let _ = tx.send(PdfWorkerMessage::UpdateDownloadComplete(Ok(out_path.to_string_lossy().into())));
                                    } else {
                                        let _ = tx.send(PdfWorkerMessage::UpdateDownloadComplete(Err("Failed to create file".into())));
                                    }
                                    ctx.request_repaint();
                                }
                                Err(_) => {
                                    let _ = tx.send(PdfWorkerMessage::UpdateDownloadComplete(Err("Download failed".into())));
                                    ctx.request_repaint();
                                }
                            }
                        });
                    }
                }
            } else {
                match task {
                    PdfWorkerTask::CheckUpdate { ctx } => {
                        let tx = msg_tx_clone.clone();
                        std::thread::spawn(move || {
                            let url = "https://nightly.link/borneelphukan/pdf-viewer/workflows/windows-build/main/PDFViewer-Installer.zip";
                            let res = ureq::head(url).call();
                            let is_available = res.is_ok();
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            let _ = tx.send(PdfWorkerMessage::UpdateCheckResult(is_available));
                            ctx.request_repaint();
                        });
                    }
                    PdfWorkerTask::DownloadUpdate { ctx } => {
                        let tx = msg_tx_clone.clone();
                        std::thread::spawn(move || {
                            let url = "https://nightly.link/borneelphukan/pdf-viewer/workflows/windows-build/main/PDFViewer-Installer.zip";
                            match ureq::get(url).call() {
                                Ok(response) => {
                                    let len: Option<u64> = response.headers().get("Content-Length").and_then(|h| h.to_str().ok()).and_then(|s| s.parse().ok());
                                    let mut reader = response.into_body().into_reader();
                                    let download_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("."));
                                    let out_path = download_dir.join("PDFViewerSetup.zip");
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
                                                        let progress = (downloaded as f32) / (total as f32);
                                                        let _ = tx.send(PdfWorkerMessage::UpdateDownloadProgress(progress));
                                                        ctx.request_repaint();
                                                    }
                                                }
                                                Err(_) => {
                                                    let _ = tx.send(PdfWorkerMessage::UpdateDownloadComplete(Err("Download failed".into())));
                                                    ctx.request_repaint();
                                                    return;
                                                }
                                            }
                                        }
                                        let _ = tx.send(PdfWorkerMessage::UpdateDownloadComplete(Ok(out_path.to_string_lossy().into())));
                                    } else {
                                        let _ = tx.send(PdfWorkerMessage::UpdateDownloadComplete(Err("Failed to create file".into())));
                                    }
                                    ctx.request_repaint();
                                }
                                Err(_) => {
                                    let _ = tx.send(PdfWorkerMessage::UpdateDownloadComplete(Err("Download failed".into())));
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
                            error: Some("PDFium not initialized. Please ensure libpdfium is present.".into()),
                        });
                        ctx.request_repaint();
                    }
                    _ => {}
                }
            }
        }
    });
}
