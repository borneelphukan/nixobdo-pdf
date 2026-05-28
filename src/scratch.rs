
use pdfium_render::prelude::*;
fn test(page: &PdfPage) {
    for obj in page.objects().iter() {
        if let Ok(text_obj) = obj.as_text_object() {
            let t: String = text_obj.text();
            let fs: f32 = text_obj.scaled_font_size().value;
            let bounds = obj.bounds().unwrap();
            let y = bounds.bottom().value;
            let x = bounds.left().value;
        }
    }
}

