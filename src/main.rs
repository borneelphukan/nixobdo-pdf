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
            .with_transparent(false),
        ..Default::default()
    };
    eframe::run_native(
        "PDF Viewer",
        options,
        Box::new(|cc| {
            let mut style = egui::Style::default();
            style.visuals = egui::Visuals::dark();
            
            // Light gray accent color for selection
            let accent = egui::Color32::from_rgb(180, 180, 180);
            style.visuals.selection.bg_fill = accent;
            // Opaque panel and window backgrounds
            style.visuals.panel_fill = egui::Color32::from_rgb(20, 20, 25);
            style.visuals.window_fill = egui::Color32::from_rgb(25, 25, 30);
            
            // Generous rounding for a softer, premium feel
            style.visuals.window_corner_radius = egui::CornerRadius::same(12);
            style.visuals.menu_corner_radius = egui::CornerRadius::same(8);
            
            style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(8);
            style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
            style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
            style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);
            
            // Opaque widgets
            style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 35);
            style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(45, 45, 55);
            style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(65, 65, 80);
            
            cc.egui_ctx.set_style(style);
            
            egui_extras::install_image_loaders(&cc.egui_ctx);
            
            let mut app = NixobdoPdfApp::default();
            
            if let Some(file_path) = std::env::args().nth(1) {
                app.load_pdf(&cc.egui_ctx, std::path::PathBuf::from(file_path));
            }
            
            Ok(Box::new(app))
        }),
    )
}
