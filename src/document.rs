use eframe::egui;
use pdfium_render::prelude::*;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::hash::{Hash, Hasher};

pub fn clean_cache() {
    let cache_root = dirs::cache_dir()
        .unwrap_or_else(|| std::env::temp_dir())
        .join("nixobdo-pdf-cache");
        
    if !cache_root.exists() {
        return;
    }
    
    let now = std::time::SystemTime::now();
    let max_age = std::time::Duration::from_secs(7 * 24 * 60 * 60); // 7 days
    
    let mut dirs_with_access = Vec::new();
    let mut total_size: u64 = 0;
    
    if let Ok(entries) = std::fs::read_dir(&cache_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let accessed_file = path.join("accessed.txt");
                let mut last_access = std::time::UNIX_EPOCH;
                
                if let Ok(meta) = std::fs::metadata(&accessed_file) {
                    last_access = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
                } else if let Ok(meta) = std::fs::metadata(&path) {
                    last_access = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
                }
                
                if let Ok(age) = now.duration_since(last_access) {
                    if age > max_age {
                        let _ = std::fs::remove_dir_all(&path);
                        continue;
                    }
                }
                
                let mut dir_size = 0;
                if let Ok(files) = std::fs::read_dir(&path) {
                    for f in files.flatten() {
                        if let Ok(meta) = f.metadata() {
                            dir_size += meta.len();
                        }
                    }
                }
                
                total_size += dir_size;
                dirs_with_access.push((path, last_access, dir_size));
            }
        }
    }
    
    // Sort by oldest access time first
    dirs_with_access.sort_by_key(|k| k.1);
    
    let max_size: u64 = 400 * 1024 * 1024; // 400MB
    while total_size > max_size && !dirs_with_access.is_empty() {
        let (path, _, size) = dirs_with_access.remove(0);
        if std::fs::remove_dir_all(&path).is_ok() {
            total_size = total_size.saturating_sub(size);
        }
    }
}

use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PdfCharInfo {
    pub c: char,
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PdfLinkTarget {
    Url(String),
    Page(usize),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PdfLinkInfo {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub target: PdfLinkTarget,
}

pub fn find_closest_char(page_rect: egui::Rect, mouse_pos: egui::Pos2, chars: &[PdfCharInfo]) -> Option<usize> {
    if chars.is_empty() {
        return None;
    }
    let rx = ((mouse_pos.x - page_rect.min.x) / page_rect.width()) as f32;
    let ry = ((mouse_pos.y - page_rect.min.y) / page_rect.height()) as f32;

    let mut closest_idx = None;
    let mut min_dist = f32::MAX;

    for (idx, char_info) in chars.iter().enumerate() {
        let cx = (char_info.left + char_info.right) / 2.0;
        let cy = (char_info.top + char_info.bottom) / 2.0;

        let vertical_overlap = ry >= char_info.top && ry <= char_info.bottom;
        let horizontal_overlap = rx >= char_info.left && rx <= char_info.right;

        let dist = if vertical_overlap && horizontal_overlap {
            0.0
        } else if vertical_overlap {
            (rx - cx).abs()
        } else {
            let dx = rx - cx;
            let dy = ry - cy;
            (dx * dx + dy * dy).sqrt()
        };

        if dist < min_dist {
            min_dist = dist;
            closest_idx = Some(idx);
        }
    }

    closest_idx
}

pub fn is_char_selected(
    page_idx: usize,
    char_idx: usize,
    start: (usize, usize),
    end: (usize, usize),
) -> bool {
    let (p_min, c_min, p_max, c_max) = if start.0 < end.0 {
        (start.0, start.1, end.0, end.1)
    } else if start.0 > end.0 {
        (end.0, end.1, start.0, start.1)
    } else {
        let (c_min, c_max) = if start.1 <= end.1 {
            (start.1, end.1)
        } else {
            (end.1, start.1)
        };
        return page_idx == start.0 && char_idx >= c_min && char_idx <= c_max;
    };

    if page_idx < p_min || page_idx > p_max {
        false
    } else if page_idx > p_min && page_idx < p_max {
        true
    } else if page_idx == p_min {
        char_idx >= c_min
    } else {
        char_idx <= c_max
    }
}

pub enum PdfWorkerMessage {
    DocumentInfo {
        path: PathBuf,
        file_name: String,
        page_count: usize,
        error: Option<String>,
    },
    PageData {
        path: PathBuf,
        index: usize,
        image: egui::ColorImage,
        thumbnail_image: egui::ColorImage,
        text: String,
        chars: Vec<PdfCharInfo>,
        links: Vec<PdfLinkInfo>,
    },
    Finished {
        #[allow(dead_code)]
        path: PathBuf,
    },
    ExportProgress {
        progress: f32,
    },
    ExportComplete {
        success: bool,
        message: String,
    },
    UpdateCheckResult(bool, Option<String>, bool),
    UpdateDownloadProgress(f32),
    UpdateDownloadComplete(Result<String, String>),
    SignatureSaved {
        path: PathBuf,
    },
    RotationSaved {
        path: PathBuf,
    },
    AnnotationsSaved {
        path: PathBuf,
    },
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AnnotationTool {

    Highlight,
    Underline,
    Strikethrough,
    Redact,
}

#[derive(Clone, Debug)]
pub struct AnnotationAction {
    pub tool: AnnotationTool,
    pub page_index: usize,
    // Used for Highlight/Underline/Strikethrough/Redact
    pub rects: Vec<egui::Rect>, 

    pub position: Option<egui::Pos2>,
    pub text: Option<String>,
    pub color: egui::Color32,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PageLayoutMode {
    ContinuousScroll,
    SinglePage,
    TwoPage,
}

impl Default for PageLayoutMode {
    fn default() -> Self {
        PageLayoutMode::ContinuousScroll
    }
}

pub struct PdfDocumentState {
    pub file_name: String,
    pub path: PathBuf,
    pub pages: Vec<Option<egui::TextureHandle>>,
    pub thumbnails: Vec<Option<egui::TextureHandle>>,
    #[allow(dead_code)]
    pub page_texts: Vec<String>,
    pub page_chars: Vec<Vec<PdfCharInfo>>,
    pub page_links: Vec<Vec<PdfLinkInfo>>,
    pub page_rotations: Vec<i32>,
    pub zoom: f32,
    pub selected_page: usize,
    pub scroll_to_page: Option<usize>,
    pub layout_mode: PageLayoutMode,
    pub error: Option<String>,
    pub is_loading: bool,
    pub last_page_change_time: f64,
}

impl PdfDocumentState {
    pub fn empty(path: PathBuf) -> Self {
        let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Untitled".to_string());
        Self {
            file_name,
            path,
            pages: Vec::new(),
            thumbnails: Vec::new(),
            page_texts: Vec::new(),
            page_chars: Vec::new(),
            page_links: Vec::new(),
            page_rotations: Vec::new(),
            zoom: 0.0,
            selected_page: 0,
            scroll_to_page: None,
            layout_mode: PageLayoutMode::ContinuousScroll,
            error: None,
            is_loading: true,
            last_page_change_time: 0.0,
        }
    }

    pub fn background_load_with_pdfium(path: PathBuf, pdfium: &Pdfium, tx: Sender<PdfWorkerMessage>, ctx: egui::Context) {
        let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Untitled".to_string());
        
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        path.hash(&mut hasher);
        if let Ok(meta) = std::fs::metadata(&path) {
            meta.len().hash(&mut hasher);
            if let Ok(time) = meta.modified() {
                if let Ok(duration) = time.duration_since(std::time::UNIX_EPOCH) {
                    duration.as_secs().hash(&mut hasher);
                }
            }
        }
        let cache_key = format!("{:x}", hasher.finish());
        
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::env::temp_dir())
            .join("nixobdo-pdf-cache")
            .join(&cache_key);
            
        if cache_dir.exists() {
            if let Ok(meta_bytes) = std::fs::read(cache_dir.join("metadata.bin")) {
                if let Ok((page_count, page_texts, page_chars, page_links)) = bincode::deserialize::<(usize, Vec<String>, Vec<Vec<PdfCharInfo>>, Vec<Vec<PdfLinkInfo>>)>(&meta_bytes) {
                    let _ = tx.send(PdfWorkerMessage::DocumentInfo {
                        path: path.clone(), file_name: file_name.clone(), page_count, error: None,
                    });
                    ctx.request_repaint();
                    
                    let mut success = true;
                    for index in 0..page_count {
                        if let (Ok(img), Ok(thumb)) = (
                            image::open(cache_dir.join(format!("page_{}.png", index))),
                            image::open(cache_dir.join(format!("thumb_{}.png", index)))
                        ) {
                            let img_rgba = img.to_rgba8();
                            let thumb_rgba = thumb.to_rgba8();
                            let image = egui::ColorImage::from_rgba_unmultiplied(
                                [img_rgba.width() as usize, img_rgba.height() as usize],
                                img_rgba.as_flat_samples().as_slice(),
                            );
                            let thumbnail_image = egui::ColorImage::from_rgba_unmultiplied(
                                [thumb_rgba.width() as usize, thumb_rgba.height() as usize],
                                thumb_rgba.as_flat_samples().as_slice(),
                            );
                            let _ = tx.send(PdfWorkerMessage::PageData {
                                path: path.clone(), index, image, thumbnail_image, text: page_texts[index].clone(), chars: page_chars[index].clone(), links: page_links[index].clone(),
                            });
                            ctx.request_repaint();
                        } else { success = false; break; }
                    }
                    if success {
                        let _ = tx.send(PdfWorkerMessage::Finished { path: path.clone() });
                        ctx.request_repaint();
                        let _ = std::fs::File::create(cache_dir.join("accessed.txt"));
                        return;
                    }
                }
            }
        }
        
        match pdfium.load_pdf_from_file(path.to_str().unwrap_or_default(), None) {
            Ok(doc) => {
                let page_count = doc.pages().len() as usize;
                
                // Immediately send document info so UI can render the empty scrollable tab
                let _ = tx.send(PdfWorkerMessage::DocumentInfo {
                    path: path.clone(),
                    file_name: file_name.clone(),
                    page_count,
                    error: None,
                });
                ctx.request_repaint();

                let render_config = PdfRenderConfig::new()
                    .set_target_width(2400) // Optimal resolution for fast loading and crisp visuals
                    .set_clear_color(PdfColor::new(255, 255, 255, 255));

                let mut all_texts = Vec::new();
                let mut all_chars = Vec::new();
                let mut all_links = Vec::new();
                
                let _ = std::fs::create_dir_all(&cache_dir);
                let _ = std::fs::File::create(cache_dir.join("accessed.txt"));

                for (index, page) in doc.pages().iter().enumerate() {
                    let page_text = page.text().map(|t| t.all()).unwrap_or_default();
                    all_texts.push(page_text.clone());
                    
                    let page_w = page.width().value;
                    let page_h = page.height().value;
                    let mut chars = Vec::new();
                    if let Ok(text) = page.text() {
                        for c in text.chars().iter() {
                            if let Ok(bounds) = c.loose_bounds() {
                                chars.push(PdfCharInfo {
                                    c: c.unicode_string().as_deref().and_then(|s| s.chars().next()).unwrap_or(' '),
                                    left: bounds.left().value / page_w,
                                    right: bounds.right().value / page_w,
                                    top: 1.0 - (bounds.top().value / page_h),
                                    bottom: 1.0 - (bounds.bottom().value / page_h),
                                });
                            }
                        }
                    }

                    let mut links = Vec::new();
                    for link in page.links().iter() {
                        if let Ok(rect) = link.rect() {
                            let left = rect.left().value / page_w;
                            let right = rect.right().value / page_w;
                            let top = 1.0 - (rect.top().value / page_h);
                            let bottom = 1.0 - (rect.bottom().value / page_h);
                            
                            let mut target = None;
                            if let Some(action) = link.action() {
                                if let Some(uri) = action.as_uri_action() {
                                    if let Ok(uri_str) = uri.uri() {
                                        target = Some(PdfLinkTarget::Url(uri_str));
                                    }
                                } else if let Some(local_dest) = action.as_local_destination_action() {
                                    if let Ok(dest) = local_dest.destination() {
                                        if let Ok(page_idx) = dest.page_index() {
                                            target = Some(PdfLinkTarget::Page(page_idx as usize));
                                        }
                                    }
                                }
                            } else if let Some(dest) = link.destination() {
                                if let Ok(page_idx) = dest.page_index() {
                                    target = Some(PdfLinkTarget::Page(page_idx as usize));
                                }
                            }
                            
                            if let Some(t) = target {
                                links.push(PdfLinkInfo { left, right, top, bottom, target: t });
                            }
                        }
                    }
                    
                    all_chars.push(chars.clone());
                    all_links.push(links.clone());

                    let (image, thumbnail_image) = if let Ok(bitmap) = page.render_with_config(&render_config) {
                        let img = bitmap.as_image();
                        let mut rgba = img.to_rgba8();
                        
                        // Generate a high-quality downscaled thumbnail BEFORE removing the white background
                        let thumb_w = 300;
                        let thumb_h = (300.0 * (rgba.height() as f32 / rgba.width() as f32)) as u32;
                        let thumb_rgba = image::imageops::resize(&rgba, thumb_w, thumb_h, image::imageops::FilterType::Lanczos3);
                        let thumb_pixels = thumb_rgba.as_flat_samples();
                        let thumbnail_image = egui::ColorImage::from_rgba_unmultiplied(
                            [thumb_rgba.width() as usize, thumb_rgba.height() as usize],
                            thumb_pixels.as_slice(),
                        );
                        
                        // Mathematically remove the white background to make it transparent
                        // so highlights can be drawn underneath the text.
                        for pixel in rgba.pixels_mut() {
                            let r = pixel[0] as f32;
                            let g = pixel[1] as f32;
                            let b = pixel[2] as f32;
                            
                            let max_diff = (255.0 - r).max(255.0 - g).max(255.0 - b);
                            let alpha = max_diff as u8;
                            
                            if alpha < 255 {
                                pixel[3] = alpha;
                                if alpha > 0 {
                                    let a_f32 = alpha as f32 / 255.0;
                                    pixel[0] = ((r - 255.0 * (1.0 - a_f32)) / a_f32).clamp(0.0, 255.0) as u8;
                                    pixel[1] = ((g - 255.0 * (1.0 - a_f32)) / a_f32).clamp(0.0, 255.0) as u8;
                                    pixel[2] = ((b - 255.0 * (1.0 - a_f32)) / a_f32).clamp(0.0, 255.0) as u8;
                                } else {
                                    pixel[0] = 0;
                                    pixel[1] = 0;
                                    pixel[2] = 0;
                                }
                            }
                        }
                        
                        let _ = rgba.save(cache_dir.join(format!("page_{}.png", index)));
                        let _ = thumb_rgba.save(cache_dir.join(format!("thumb_{}.png", index)));
                        
                        let pixels = rgba.as_flat_samples();
                        let image = egui::ColorImage::from_rgba_unmultiplied(
                            [rgba.width() as usize, rgba.height() as usize],
                            pixels.as_slice(),
                        );
                        (image, thumbnail_image)
                    } else {
                        (egui::ColorImage::example(), egui::ColorImage::example()) // Fallback empty image
                    };

                    let _ = tx.send(PdfWorkerMessage::PageData {
                        path: path.clone(),
                        index,
                        image,
                        thumbnail_image,
                        text: page_text,
                        chars,
                        links,
                    });
                    ctx.request_repaint();
                }

                if let Ok(meta_bytes) = bincode::serialize(&(page_count, all_texts, all_chars, all_links)) {
                    let _ = std::fs::write(cache_dir.join("metadata.bin"), meta_bytes);
                }

                let _ = tx.send(PdfWorkerMessage::Finished { path: path.clone() });
                ctx.request_repaint();
            }
            Err(e) => {
                let _ = tx.send(PdfWorkerMessage::DocumentInfo {
                    path: path.clone(),
                    file_name,
                    page_count: 0,
                    error: Some(format!("Failed to load PDF: {}", e)),
                });
                ctx.request_repaint();
            }
        };
    }
}
