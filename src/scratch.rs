use pdfium_render::prelude::*;

fn test_rotation(page: &mut PdfPage) {
    let _rot = page.rotation();
    page.set_rotation(PdfPageRotation::Degrees90);
}
