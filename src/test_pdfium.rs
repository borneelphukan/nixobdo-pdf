use pdfium_render::prelude::*;
fn main() {
    let bindings = Pdfium::bind_to_system_library().unwrap();
    let pdfium = Pdfium::new(bindings);
    let mut doc = pdfium.create_new_pdf().unwrap();
    let mut page = doc.pages_mut().create_page_at_end(PdfPagePaperSize::a4()).unwrap();
    page.set_rotation(PdfBitmapRotation::Degrees90);
    doc.save_to_file("out.pdf").unwrap();
}
