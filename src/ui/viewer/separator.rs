use crate::app::NixobdoPdfApp;
use eframe::egui;

impl NixobdoPdfApp {
    pub(crate) fn ui_separator(&mut self, ui: &mut egui::Ui) {
        let is_fullscreen = ui.ctx().input(|i| i.viewport().fullscreen.unwrap_or(false));
        if is_fullscreen {
            return;
        }

        // Draggable vertical separator panel
        egui::Panel::left("separator_panel")
            .resizable(false)
            .exact_size(1.0)
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                let rect = ui.max_rect();
                let (response, painter) =
                    ui.allocate_painter(rect.size(), egui::Sense::click_and_drag());

                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }

                if response.clicked() {
                    self.sidebar_open = !self.sidebar_open;
                } else if response.dragged() {
                    let delta_x = response.drag_delta().x;
                    if self.sidebar_open && delta_x < -2.0 {
                        self.sidebar_open = false;
                    } else if !self.sidebar_open && delta_x > 2.0 {
                        self.sidebar_open = true;
                    }
                }

                let is_active = response.hovered() || response.dragged();
                let color = if is_active {
                    ui.visuals().widgets.active.bg_fill
                } else {
                    ui.visuals().widgets.noninteractive.bg_stroke.color
                };

                let stroke_width = if is_active { 2.0 } else { 0.5 };
                let line_x = rect.center().x;
                painter.line_segment(
                    [
                        egui::pos2(line_x, rect.min.y),
                        egui::pos2(line_x, rect.max.y),
                    ],
                    egui::Stroke::new(stroke_width, color),
                );
            });
    }
}
