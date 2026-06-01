#![windows_subsystem = "windows"]
#![allow(non_snake_case)]
use eframe::egui;

mod app;
pub mod document;
pub mod worker;
pub mod ui;

use app::NixobdoPdfApp;

fn main() -> eframe::Result<()> {
    let icon_bytes = include_bytes!("../assets/logo.png");
    let image = image::load_from_memory(icon_bytes).expect("Failed to load icon").into_rgba8();
    let (width, height) = image.dimensions();
    let icon_data = egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 1000.0])
            .with_title("PDF Viewer")
            .with_icon(icon_data)
            .with_transparent(true),
        ..Default::default()
    };
    eframe::run_native(
        "PDF Viewer",
        options,
        Box::new(|cc| {
            let mut style = egui::Style::default();
            style.visuals = egui::Visuals::dark();
            
            // Vibrant cyan accent color for a shiny look
            let accent = egui::Color32::from_rgb(0, 220, 255);
            style.visuals.selection.bg_fill = accent;
            
            // Semi-transparent panel backgrounds for glass effect
            style.visuals.panel_fill = egui::Color32::from_rgba_premultiplied(20, 20, 25, 210);
            style.visuals.window_fill = egui::Color32::from_rgba_premultiplied(20, 20, 25, 230);
            
            // Generous rounding for a softer, premium feel
            style.visuals.window_corner_radius = egui::CornerRadius::same(12);
            style.visuals.menu_corner_radius = egui::CornerRadius::same(8);
            
            style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(8);
            style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
            style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
            style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);
            
            // Slightly translucent widgets to blend with the glass theme
            style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgba_premultiplied(30, 30, 35, 150);
            style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgba_premultiplied(45, 45, 55, 150);
            style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgba_premultiplied(65, 65, 80, 180);
            
            cc.egui_ctx.set_style(style);
            
            let mut app = NixobdoPdfApp::default();
            
            if let Some(file_path) = std::env::args().nth(1) {
                app.load_pdf(&cc.egui_ctx, std::path::PathBuf::from(file_path));
            }
            
            Ok(Box::new(app))
        }),
    )
}
