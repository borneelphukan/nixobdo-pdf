use eframe::egui;
use pdfium_render::prelude::*;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::document::{PdfDocumentState, PdfWorkerMessage};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ExportFormat {
    Doc,
    Docx,
    Png,
    Jpeg,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Doc => "doc",
            ExportFormat::Docx => "docx",
            ExportFormat::Png => "png",
            ExportFormat::Jpeg => "jpeg",
        }
    }
}

pub enum PdfWorkerTask {
    Load { path: PathBuf, ctx: egui::Context },
    Export { path: PathBuf, out_path: PathBuf, format: ExportFormat },
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
                    PdfWorkerTask::Export { path, out_path, format } => {
                        if format == ExportFormat::Png || format == ExportFormat::Jpeg {
                            if let Ok(doc) = pdf.load_pdf_from_file(&path, None) {
                                let pages = doc.pages();
                                if let Ok(page) = pages.get(0) {
                                    let render_config = PdfRenderConfig::new()
                                        .set_target_width(2000)
                                        .set_clear_color(PdfColor::new(255, 255, 255, 255));
                                        
                                    if let Ok(bitmap) = page.render_with_config(&render_config) {
                                        let _ = bitmap.as_image().save(&out_path);
                                    }
                                }
                            }
                        } else {
                            if let Ok(doc) = pdf.load_pdf_from_file(&path, None) {
                                if format == ExportFormat::Docx {
                                    if let Ok(file) = std::fs::File::create(&out_path) {
                                        let mut docx = docx_rs::Docx::new();
                                        for page in doc.pages().iter() {
                                            if let Ok(text) = page.text() {
                                                let mut current_paragraph = docx_rs::Paragraph::new();
                                                let mut current_run_text = String::new();
                                                let mut current_font_size: f32 = 12.0;
                                                let mut last_y: Option<f32> = None;
                                                let mut last_x: Option<f32> = None;
                                                
                                                for c in text.chars().iter() {
                                                    let char_text = c.unicode_string().as_deref().and_then(|s| s.chars().next()).unwrap_or(' ');
                                                    
                                                    let char_size = if let Ok(text_obj) = c.text_object() {
                                                        text_obj.scaled_font_size().value
                                                    } else {
                                                        c.scaled_font_size().value
                                                    };
                                                    
                                                    let mut char_y = last_y.unwrap_or(0.0);
                                                    let mut char_x = last_x.unwrap_or(0.0);
                                                    if let Ok(bounds) = c.loose_bounds() {
                                                        char_y = bounds.bottom().value;
                                                        char_x = bounds.left().value;
                                                    }
                                                    
                                                    let is_new_line = if let Some(ly) = last_y {
                                                        (ly - char_y).abs() > (char_size * 0.5)
                                                    } else {
                                                        false
                                                    };
                                                    
                                                    if is_new_line || (char_size - current_font_size).abs() > 1.0 {
                                                        if !current_run_text.is_empty() {
                                                            current_paragraph = current_paragraph.add_run(
                                                                docx_rs::Run::new()
                                                                    .size((current_font_size * 2.0) as usize)
                                                                    .add_text(current_run_text.clone())
                                                            );
                                                            current_run_text.clear();
                                                        }
                                                        
                                                        if is_new_line {
                                                            docx = docx.add_paragraph(current_paragraph);
                                                            current_paragraph = docx_rs::Paragraph::new();
                                                        }
                                                        
                                                        current_font_size = char_size;
                                                    } else if let Some(lx) = last_x {
                                                        if char_x - lx > char_size * 0.3 {
                                                            if !current_run_text.ends_with(' ') {
                                                                current_run_text.push(' ');
                                                            }
                                                        }
                                                    }
                                                    
                                                    current_run_text.push(char_text);
                                                    last_y = Some(char_y);
                                                    if let Ok(bounds) = c.loose_bounds() {
                                                        last_x = Some(bounds.right().value);
                                                    }
                                                }
                                                
                                                if !current_run_text.is_empty() {
                                                    current_paragraph = current_paragraph.add_run(
                                                        docx_rs::Run::new()
                                                            .size((current_font_size * 2.0) as usize)
                                                            .add_text(current_run_text)
                                                    );
                                                }
                                                docx = docx.add_paragraph(current_paragraph);
                                            }
                                        }
                                        let _ = docx.build().pack(file);
                                    }
                                } else {
                                    let mut content = String::new();
                                    for page in doc.pages().iter() {
                                        if let Ok(text) = page.text() {
                                            content.push_str(&text.all());
                                            content.push_str("\n\n");
                                        }
                                    }
                                    
                                    // Write valid RTF for .doc format
                                    let mut rtf = String::from(r"{\rtf1\ansi\ansicpg1252\deff0\nouicompat{\fonttbl{\f0\fnil\fcharset0 Calibri;}}\viewkind4\uc1\pard\f0\fs22 ");
                                    for c in content.chars() {
                                        match c {
                                            '\\' => rtf.push_str(r"\\"),
                                            '{' => rtf.push_str(r"\{"),
                                            '}' => rtf.push_str(r"\}"),
                                            '\n' => rtf.push_str("\\par\n"),
                                            _ => rtf.push(c),
                                        }
                                    }
                                    rtf.push('}');
                                    let _ = std::fs::write(&out_path, rtf);
                                }
                            }
                        }
                    }
                }
            } else {
                if let PdfWorkerTask::Load { path, ctx } = task {
                    let _ = msg_tx_clone.send(PdfWorkerMessage::DocumentInfo {
                        path: path.clone(),
                        file_name: String::new(),
                        page_count: 0,
                        error: Some("PDFium not initialized. Please ensure libpdfium is present.".into()),
                    });
                    ctx.request_repaint();
                }
            }
        }
    });
}
