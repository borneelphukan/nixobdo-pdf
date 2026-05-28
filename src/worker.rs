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
    Export { path: PathBuf, out_path: PathBuf, format: ExportFormat, retain_layout: bool, include_images: bool },
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
                    PdfWorkerTask::Export { path, out_path, format, retain_layout, include_images } => {
                        eprintln!("[Export] Starting: path={:?} out={:?} fmt={:?} layout={} images={}", path, out_path, format, retain_layout, include_images);
                        
                        if format == ExportFormat::Png || format == ExportFormat::Jpeg {
                            export_image(pdf, &path, &out_path);
                        } else if format == ExportFormat::Docx {
                            export_docx(pdf, &path, &out_path, retain_layout, include_images);
                        } else {
                            export_doc_rtf(pdf, &path, &out_path, retain_layout, include_images);
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

fn export_image(pdf: &Pdfium, path: &PathBuf, out_path: &PathBuf) {
    if let Ok(doc) = pdf.load_pdf_from_file(path, None) {
        let pages = doc.pages();
        if let Ok(page) = pages.get(0) {
            let render_config = PdfRenderConfig::new()
                .set_target_width(2000)
                .set_clear_color(PdfColor::new(255, 255, 255, 255));
                
            if let Ok(bitmap) = page.render_with_config(&render_config) {
                let _ = bitmap.as_image().save(out_path);
                eprintln!("[Export] Image saved to {:?}", out_path);
            }
        }
    }
}

fn export_docx(pdf: &Pdfium, path: &PathBuf, out_path: &PathBuf, retain_layout: bool, include_images: bool) {
    let doc = match pdf.load_pdf_from_file(path, None) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[Export] Failed to load PDF: {:?}", e);
            return;
        }
    };
    
    let page_count = doc.pages().len();
    eprintln!("[Export] DOCX: PDF loaded, {} pages", page_count);
    
    let file = match std::fs::File::create(out_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[Export] Failed to create output file: {:?}", e);
            return;
        }
    };
    
    let mut docx = docx_rs::Docx::new();
    
    for (pi, page) in doc.pages().iter().enumerate() {
        // Extract text content
        match page.text() {
            Ok(text) => {
                let char_count = text.chars().len();
                eprintln!("[Export] Page {} — {} chars", pi, char_count);
                
                if retain_layout {
                    docx = export_docx_layout_text(docx, &text);
                } else {
                    let raw_text = text.all();
                    eprintln!("[Export] Page {} flowing text: {} bytes", pi, raw_text.len());
                    for line in raw_text.lines() {
                        let paragraph = docx_rs::Paragraph::new().add_run(
                            docx_rs::Run::new()
                                .size(24) // 12pt
                                .add_text(line.to_string())
                        );
                        docx = docx.add_paragraph(paragraph);
                    }
                }
            }
            Err(e) => {
                eprintln!("[Export] Page {} text extraction failed: {:?}", pi, e);
            }
        }
        
        // Extract images from page
        if include_images {
            docx = export_docx_images(docx, &page);
        }
    }
    
    match docx.build().pack(file) {
        Ok(_) => eprintln!("[Export] DOCX written successfully to {:?}", out_path),
        Err(e) => eprintln!("[Export] DOCX pack failed: {:?}", e),
    }
}

fn export_docx_layout_text<'a>(mut docx: docx_rs::Docx, text: &PdfPageText<'a>) -> docx_rs::Docx {
    let mut current_paragraph = docx_rs::Paragraph::new();
    let mut current_run_text = String::new();
    let mut current_font_size: f32 = 12.0;
    let mut last_y: Option<f32> = None;
    let mut last_x: Option<f32> = None;
    
    for c in text.chars().iter() {
        let char_text = c.unicode_string()
            .as_deref()
            .and_then(|s| s.chars().next())
            .unwrap_or(' ');
        
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
    
    docx
}

fn export_docx_images<'a>(mut docx: docx_rs::Docx, page: &PdfPage<'a>) -> docx_rs::Docx {
    for obj in page.objects().iter() {
        if let Some(img_obj) = obj.as_image_object() {
            if let Ok(img) = img_obj.get_raw_image() {
                let mut bytes: Vec<u8> = Vec::new();
                if img.write_to(
                    &mut std::io::Cursor::new(&mut bytes),
                    image::ImageFormat::Png,
                ).is_ok() && !bytes.is_empty() {
                    let pic = docx_rs::Pic::new(&bytes);
                    let run = docx_rs::Run::new().add_image(pic);
                    docx = docx.add_paragraph(docx_rs::Paragraph::new().add_run(run));
                    eprintln!("[Export] Added image ({} bytes PNG)", bytes.len());
                }
            }
        }
    }
    docx
}

fn export_doc_rtf(pdf: &Pdfium, path: &PathBuf, out_path: &PathBuf, retain_layout: bool, include_images: bool) {
    let doc = match pdf.load_pdf_from_file(path, None) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[Export] Failed to load PDF for DOC: {:?}", e);
            return;
        }
    };
    
    eprintln!("[Export] DOC: PDF loaded, {} pages", doc.pages().len());
    
    let mut content = String::new();
    for (pi, page) in doc.pages().iter().enumerate() {
        match page.text() {
            Ok(text) => {
                if retain_layout {
                    let mut last_y: Option<f32> = None;
                    let mut last_x: Option<f32> = None;
                    for c in text.chars().iter() {
                        let char_text = c.unicode_string()
                            .as_deref()
                            .and_then(|s| s.chars().next())
                            .unwrap_or(' ');
                        let mut char_y = last_y.unwrap_or(0.0);
                        let mut char_x = last_x.unwrap_or(0.0);
                        if let Ok(bounds) = c.loose_bounds() {
                            char_y = bounds.bottom().value;
                            char_x = bounds.left().value;
                        }
                        if let Some(ly) = last_y {
                            if (ly - char_y).abs() > 6.0 {
                                content.push('\n');
                            }
                        }
                        if let Some(lx) = last_x {
                            if char_x - lx > 4.0 {
                                content.push(' ');
                            }
                        }
                        content.push(char_text);
                        last_y = Some(char_y);
                        if let Ok(bounds) = c.loose_bounds() {
                            last_x = Some(bounds.right().value);
                        }
                    }
                } else {
                    content.push_str(&text.all());
                }
                content.push_str("\n\n");
            }
            Err(e) => {
                eprintln!("[Export] DOC page {} text extraction failed: {:?}", pi, e);
            }
        }
        
        if include_images {
            for obj in page.objects().iter() {
                if let Some(img_obj) = obj.as_image_object() {
                    if let Ok(img) = img_obj.get_raw_image() {
                        let mut bytes: Vec<u8> = Vec::new();
                        if img.write_to(
                            &mut std::io::Cursor::new(&mut bytes),
                            image::ImageFormat::Png,
                        ).is_ok() {
                            content.push_str("{\\pict\\pngblip\\picwgoal1000\\pichgoal1000\n");
                            for b in bytes {
                                use std::fmt::Write;
                                let _ = write!(&mut content, "{:02x}", b);
                            }
                            content.push_str("\n}\n");
                        }
                    }
                }
            }
        }
    }
    
    eprintln!("[Export] DOC content: {} chars", content.len());
    
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
    let _ = std::fs::write(out_path, rtf);
    eprintln!("[Export] DOC written to {:?}", out_path);
}
