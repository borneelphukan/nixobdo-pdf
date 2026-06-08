use eframe::egui;

fn test(ui: &mut egui::Ui) {
    let mut hsva = egui::ecolor::Hsva::default();
    egui::color_picker::color_picker_hsva_2d(ui, &mut hsva, egui::color_picker::Alpha::Opaque);
    egui::color_picker::color_picker_1d(ui, &mut hsva.h);
}
