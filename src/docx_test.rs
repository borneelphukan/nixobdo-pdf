use docx_rs::*;
use std::fs::File;

fn main() {
    let path = std::path::Path::new("test.docx");
    let file = File::create(&path).unwrap();
    Docx::new()
        .add_paragraph(Paragraph::new().add_run(Run::new().add_text("Hello")))
        .build()
        .pack(file)
        .unwrap();
}
