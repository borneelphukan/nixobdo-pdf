use eframe::egui;
use pdfium_render::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;

use crate::document::PdfWorkerMessage;

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

/// Export all pages as separate image files inside a ZIP. Returns Ok(message) or Err(message).
pub fn export_image(
    pdf: &Pdfium,
    path: &PathBuf,
    out_path: &PathBuf,
    format: ExportFormat,
    msg_tx: &Sender<PdfWorkerMessage>,
    ctx: &egui::Context,
    cancel_flag: &AtomicBool,
) -> Result<String, String> {
    use std::io::Write;

    let doc = pdf
        .load_pdf_from_file(path, None)
        .map_err(|e| format!("Failed to load PDF: {:?}", e))?;

    let page_count = doc.pages().len();
    let stem = out_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("export");
    let ext = format.extension();

    let zip_path = out_path.with_extension("zip");
    let file = std::fs::File::create(&zip_path)
        .map_err(|e| format!("Failed to create ZIP file: {:?}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let mut exported = 0;
    for (i, page) in doc.pages().iter().enumerate() {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("Export cancelled by user.".to_string());
        }

        let progress = (i as f32) / (page_count as f32);
        let _ = msg_tx.send(PdfWorkerMessage::ExportProgress { progress });
        ctx.request_repaint();

        let render_config = PdfRenderConfig::new()
            .set_target_width(2000)
            .set_clear_color(PdfColor::new(255, 255, 255, 255));

        if let Ok(bitmap) = page.render_with_config(&render_config) {
            let file_name = format!("{}_page{}.{}", stem, i + 1, ext);

            let img = bitmap.as_image();
            let mut buf = std::io::Cursor::new(Vec::new());
            let format_enum = if format == ExportFormat::Png {
                image::ImageFormat::Png
            } else {
                image::ImageFormat::Jpeg
            };

            if img.write_to(&mut buf, format_enum).is_ok() {
                if zip.start_file(file_name, options).is_ok() {
                    if zip.write_all(&buf.into_inner()).is_ok() {
                        exported += 1;
                    }
                }
            }
        }
    }

    zip.finish()
        .map_err(|e| format!("Failed to finalize ZIP: {:?}", e))?;

    let _ = msg_tx.send(PdfWorkerMessage::ExportProgress { progress: 1.0 });
    ctx.request_repaint();

    if exported > 0 {
        Ok(format!(
            "Exported {} page(s) as {}.zip successfully.",
            exported,
            ext.to_uppercase()
        ))
    } else {
        Err("Failed to export any pages.".to_string())
    }
}

/// Export as DOCX. Returns Ok(message) or Err(message).
pub fn export_docx(
    pdf: &Pdfium,
    path: &PathBuf,
    out_path: &PathBuf,
    retain_layout: bool,
    include_images: bool,
    msg_tx: &Sender<PdfWorkerMessage>,
    ctx: &egui::Context,
    cancel_flag: &AtomicBool,
) -> Result<String, String> {
    let doc = match pdf.load_pdf_from_file(path, None) {
        Ok(d) => d,
        Err(e) => {
            return Err(format!("Failed to load PDF: {:?}", e));
        }
    };

    let page_count = doc.pages().len();

    let file = match std::fs::File::create(out_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!("Failed to create output file: {:?}", e));
        }
    };

    let mut docx = docx_rs::Docx::new();

    for (pi, page) in doc.pages().iter().enumerate() {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("Export cancelled by user.".to_string());
        }

        let progress = (pi as f32) / (page_count as f32);
        let _ = msg_tx.send(PdfWorkerMessage::ExportProgress { progress });
        ctx.request_repaint();

        // Extract text content
        let mut has_text = false;
        if let Ok(text) = page.text() {
            let char_count = text.chars().len();

            if char_count > 0 {
                has_text = true;
                if retain_layout {
                    docx = export_docx_layout_text(docx, &text);
                } else {
                    let raw_text = text.all();
                    for line in raw_text.lines() {
                        let sanitized_line: String = line
                            .chars()
                            .filter(|&c| {
                                let u = c as u32;
                                u == 0x09
                                    || u == 0x0A
                                    || u == 0x0D
                                    || (u >= 0x20 && u <= 0xD7FF)
                                    || (u >= 0xE000 && u <= 0xFFFD)
                                    || (u >= 0x10000 && u <= 0x10FFFF)
                            })
                            .collect();

                        let paragraph = docx_rs::Paragraph::new().add_run(
                            docx_rs::Run::new()
                                .size(24) // 12pt
                                .add_text(sanitized_line),
                        );
                        docx = docx.add_paragraph(paragraph);
                    }
                }
            }
        }

        // Extract images from page
        if include_images {
            docx = export_docx_images(docx, &page, !has_text);
        }
    }

    let _ = msg_tx.send(PdfWorkerMessage::ExportProgress { progress: 1.0 });
    ctx.request_repaint();

    match docx.build().pack(file) {
        Ok(_) => {}
        Err(e) => return Err(format!("Failed to write DOCX: {:?}", e)),
    }

    Ok("Exported as DOCX successfully.".to_string())
}

fn export_docx_layout_text<'a>(mut docx: docx_rs::Docx, text: &PdfPageText<'a>) -> docx_rs::Docx {
    let mut current_paragraph = docx_rs::Paragraph::new();
    let mut current_run_text = String::new();
    let mut current_font_size: f32 = 12.0;
    let mut last_y: Option<f32> = None;
    let mut last_x: Option<f32> = None;

    for c in text.chars().iter() {
        let char_text = c
            .unicode_string()
            .as_deref()
            .and_then(|s| s.chars().next())
            .unwrap_or(' ');

        let u = char_text as u32;
        let is_valid = u == 0x09
            || u == 0x0A
            || u == 0x0D
            || (u >= 0x20 && u <= 0xD7FF)
            || (u >= 0xE000 && u <= 0xFFFD)
            || (u >= 0x10000 && u <= 0x10FFFF);
        let char_text = if is_valid { char_text } else { ' ' };

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
                        .add_text(current_run_text.clone()),
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
                .add_text(current_run_text),
        );
    }
    docx = docx.add_paragraph(current_paragraph);

    docx
}

fn export_docx_images<'a>(
    mut docx: docx_rs::Docx,
    page: &PdfPage<'a>,
    fallback_rasterize: bool,
) -> docx_rs::Docx {
    let mut added_images = false;
    for obj in page.objects().iter() {
        if let Some(img_obj) = obj.as_image_object() {
            if let Ok(img) = img_obj.get_raw_image() {
                let mut bytes: Vec<u8> = Vec::new();
                if img
                    .write_to(
                        &mut std::io::Cursor::new(&mut bytes),
                        image::ImageFormat::Jpeg,
                    )
                    .is_ok()
                    && !bytes.is_empty()
                {
                    let pic = docx_rs::Pic::new(&bytes);
                    let run = docx_rs::Run::new().add_image(pic);
                    docx = docx.add_paragraph(docx_rs::Paragraph::new().add_run(run));
                    added_images = true;
                }
            }
        }
    }

    // Fallback if the page has no text, but we failed to extract any raw images
    if !added_images && fallback_rasterize {
        let render_config = PdfRenderConfig::new()
            .set_target_width(1200)
            .set_clear_color(PdfColor::new(255, 255, 255, 255));
        if let Ok(bitmap) = page.render_with_config(&render_config) {
            let mut bytes: Vec<u8> = Vec::new();
            if bitmap
                .as_image()
                .write_to(
                    &mut std::io::Cursor::new(&mut bytes),
                    image::ImageFormat::Jpeg,
                )
                .is_ok()
            {
                let pic = docx_rs::Pic::new(&bytes);
                let run = docx_rs::Run::new().add_image(pic);
                docx = docx.add_paragraph(docx_rs::Paragraph::new().add_run(run));
            }
        }
    }

    docx
}

/// Export as DOC (RTF). Returns Ok(message) or Err(message).
pub fn export_doc_rtf(
    pdf: &Pdfium,
    path: &PathBuf,
    out_path: &PathBuf,
    retain_layout: bool,
    include_images: bool,
    msg_tx: &Sender<PdfWorkerMessage>,
    ctx: &egui::Context,
    cancel_flag: &AtomicBool,
) -> Result<String, String> {
    let doc = pdf
        .load_pdf_from_file(path, None)
        .map_err(|e| format!("Failed to load PDF for DOC: {:?}", e))?;

    let page_count = doc.pages().len();

    let mut content = String::new();
    for (pi, page) in doc.pages().iter().enumerate() {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("Export cancelled by user.".to_string());
        }

        let progress = (pi as f32) / (page_count as f32);
        let _ = msg_tx.send(PdfWorkerMessage::ExportProgress { progress });
        ctx.request_repaint();

        let mut has_text = false;
        if let Ok(text) = page.text() {
            let char_count = text.chars().len();
            if char_count > 0 {
                has_text = true;
                if retain_layout {
                    let mut last_y: Option<f32> = None;
                    let mut last_x: Option<f32> = None;
                    for c in text.chars().iter() {
                        let char_text = c
                            .unicode_string()
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

                        let u = char_text as u32;
                        if u == 0x09
                            || u == 0x0A
                            || u == 0x0D
                            || (u >= 0x20 && u <= 0xD7FF)
                            || (u >= 0xE000 && u <= 0xFFFD)
                            || (u >= 0x10000 && u <= 0x10FFFF)
                        {
                            content.push(char_text);
                        } else {
                            content.push(' ');
                        }

                        last_y = Some(char_y);
                        if let Ok(bounds) = c.loose_bounds() {
                            last_x = Some(bounds.right().value);
                        }
                    }
                } else {
                    let raw_text = text.all();
                    let sanitized_text: String = raw_text
                        .chars()
                        .filter(|&c| {
                            let u = c as u32;
                            u == 0x09
                                || u == 0x0A
                                || u == 0x0D
                                || (u >= 0x20 && u <= 0xD7FF)
                                || (u >= 0xE000 && u <= 0xFFFD)
                                || (u >= 0x10000 && u <= 0x10FFFF)
                        })
                        .collect();
                    content.push_str(&sanitized_text);
                }
                content.push_str("\n\n");
            }
        }

        if include_images {
            let mut added_images = false;
            for obj in page.objects().iter() {
                if let Some(img_obj) = obj.as_image_object() {
                    if let Ok(img) = img_obj.get_raw_image() {
                        let mut bytes: Vec<u8> = Vec::new();
                        if img
                            .write_to(
                                &mut std::io::Cursor::new(&mut bytes),
                                image::ImageFormat::Jpeg,
                            )
                            .is_ok()
                        {
                            content.push_str("{\\pict\\jpegblip\\picwgoal1000\\pichgoal1000\n");
                            for b in bytes {
                                use std::fmt::Write;
                                let _ = write!(&mut content, "{:02x}", b);
                            }
                            content.push_str("\n}\n");
                            added_images = true;
                        }
                    }
                }
            }

            if !added_images && !has_text {
                let render_config = PdfRenderConfig::new()
                    .set_target_width(1000) // Lower resolution for RTF to keep file size reasonable
                    .set_clear_color(PdfColor::new(255, 255, 255, 255));
                if let Ok(bitmap) = page.render_with_config(&render_config) {
                    let mut bytes: Vec<u8> = Vec::new();
                    if bitmap
                        .as_image()
                        .write_to(
                            &mut std::io::Cursor::new(&mut bytes),
                            image::ImageFormat::Jpeg,
                        )
                        .is_ok()
                    {
                        content.push_str("{\\pict\\jpegblip\\picwgoal1000\\pichgoal1000\n");
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

    // Write valid RTF for .doc format
    let mut rtf = String::from(
        r"{\rtf1\ansi\ansicpg1252\deff0\nouicompat{\fonttbl{\f0\fnil\fcharset0 Calibri;}}\viewkind4\uc1\pard\f0\fs22 ",
    );
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

    let _ = msg_tx.send(PdfWorkerMessage::ExportProgress { progress: 1.0 });
    ctx.request_repaint();

    std::fs::write(out_path, rtf).map_err(|e| format!("Failed to write DOC file: {:?}", e))?;

    Ok("Exported as DOC successfully.".to_string())
}
