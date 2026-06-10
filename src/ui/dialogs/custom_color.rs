use crate::app::NixobdoPdfApp;
use eframe::egui;

fn contrast_color(color: impl Into<egui::Rgba>) -> egui::Color32 {
    if color.into().intensity() < 0.5 {
        egui::Color32::WHITE
    } else {
        egui::Color32::BLACK
    }
}

const N: u32 = 6 * 6;

fn color_slider_1d(
    ui: &mut egui::Ui,
    value: &mut f32,
    color_at: impl Fn(f32) -> egui::Color32,
) -> egui::Response {
    let desired_size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

    if let Some(mpos) = response.interact_pointer_pos() {
        *value = egui::remap_clamp(mpos.x, rect.left()..=rect.right(), 0.0..=1.0);
    }

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        {
            let mut mesh = egui::Mesh::default();
            for i in 0..=N {
                let t = i as f32 / (N as f32);
                let color = color_at(t);
                let x = egui::lerp(rect.left()..=rect.right(), t);
                mesh.colored_vertex(egui::pos2(x, rect.top()), color);
                mesh.colored_vertex(egui::pos2(x, rect.bottom()), color);
                if i < N {
                    mesh.add_triangle(2 * i + 0, 2 * i + 1, 2 * i + 2);
                    mesh.add_triangle(2 * i + 1, 2 * i + 2, 2 * i + 3);
                }
            }
            ui.painter().add(egui::Shape::mesh(mesh));
        }

        ui.painter()
            .rect_stroke(rect, 0.0, visuals.bg_stroke, egui::StrokeKind::Inside);

        let x = egui::lerp(rect.left()..=rect.right(), *value);
        let r = rect.height() / 4.0;
        let picked_color = color_at(*value);
        ui.painter().add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(x, rect.center().y),
                egui::pos2(x + r, rect.bottom()),
                egui::pos2(x - r, rect.bottom()),
            ],
            picked_color,
            egui::Stroke::new(visuals.fg_stroke.width, contrast_color(picked_color)),
        ));
    }
    response
}

fn color_slider_2d(
    ui: &mut egui::Ui,
    x_value: &mut f32,
    y_value: &mut f32,
    color_at: impl Fn(f32, f32) -> egui::Color32,
) -> egui::Response {
    let desired_size = egui::vec2(ui.available_width(), 200.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

    if let Some(mpos) = response.interact_pointer_pos() {
        *x_value = egui::remap_clamp(mpos.x, rect.left()..=rect.right(), 0.0..=1.0);
        *y_value = egui::remap_clamp(mpos.y, rect.bottom()..=rect.top(), 0.0..=1.0);
    }

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let mut mesh = egui::Mesh::default();

        for xi in 0..=N {
            for yi in 0..=N {
                let xt = xi as f32 / (N as f32);
                let yt = yi as f32 / (N as f32);
                let color = color_at(xt, yt);
                let x = egui::lerp(rect.left()..=rect.right(), xt);
                let y = egui::lerp(rect.bottom()..=rect.top(), yt);
                mesh.colored_vertex(egui::pos2(x, y), color);

                if xi < N && yi < N {
                    let x_offset = 1;
                    let y_offset = N + 1;
                    let tl = yi * y_offset + xi;
                    mesh.add_triangle(tl, tl + x_offset, tl + y_offset);
                    mesh.add_triangle(tl + x_offset, tl + y_offset, tl + y_offset + x_offset);
                }
            }
        }
        ui.painter().add(egui::Shape::mesh(mesh));

        ui.painter()
            .rect_stroke(rect, 0.0, visuals.bg_stroke, egui::StrokeKind::Inside);

        let x = egui::lerp(rect.left()..=rect.right(), *x_value);
        let y = egui::lerp(rect.bottom()..=rect.top(), *y_value);
        let picked_color = color_at(*x_value, *y_value);
        ui.painter().add(egui::Shape::circle_stroke(
            egui::pos2(x, y),
            rect.width() / 24.0,
            egui::Stroke::new(2.0, contrast_color(picked_color)),
        ));
    }
    response
}

impl NixobdoPdfApp {
    pub(crate) fn ui_custom_color_dialog(&mut self, ui: &mut egui::Ui) {
        if self.is_custom_text_color_open {
            let mut is_open = true;

            egui::Window::new("Custom Color")
                .open(&mut is_open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .min_width(280.0)
                .show(ui.ctx(), |ui| {
                    let mut hsva = egui::ecolor::Hsva::from(self.custom_text_color_temp);

                    // 2D Gradient
                    color_slider_2d(ui, &mut hsva.s, &mut hsva.v, |s, v| {
                        egui::ecolor::Hsva {
                            h: hsva.h,
                            s,
                            v,
                            a: 1.0,
                        }
                        .into()
                    });

                    ui.add_space(4.0);

                    // 1D Hue Slider
                    color_slider_1d(ui, &mut hsva.h, |h| {
                        egui::ecolor::Hsva {
                            h,
                            s: 1.0,
                            v: 1.0,
                            a: 1.0,
                        }
                        .into()
                    });

                    ui.add_space(8.0);

                    self.custom_text_color_temp = hsva.into();

                    // Hex code
                    let mut hex = format!(
                        "#{:02X}{:02X}{:02X}",
                        self.custom_text_color_temp.r(),
                        self.custom_text_color_temp.g(),
                        self.custom_text_color_temp.b()
                    );
                    let response = ui.add_sized(
                        [ui.available_width(), 30.0],
                        egui::TextEdit::singleline(&mut hex).hint_text("#000000"),
                    );
                    if response.changed() {
                        if let Ok(color) = egui::Color32::from_hex(&hex) {
                            self.custom_text_color_temp = color;
                        }
                    }

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Add").clicked() {
                                if self.active_annotation_tool
                                    == Some(crate::document::AnnotationTool::Highlight)
                                {
                                    self.annotation_color = self.custom_text_color_temp;
                                    // Highlight colors shouldn't update retroactively
                                } else {
                                    self.text_annotation_color = self.custom_text_color_temp;
                                    if let Some(last_text) =
                                        self.pending_annotations.iter_mut().rev().find(|a| {
                                            a.tool == crate::document::AnnotationTool::Text
                                        })
                                    {
                                        last_text.color = self.text_annotation_color;
                                    }
                                }
                                self.is_custom_text_color_open = false;
                            }

                            if ui.button("Cancel").clicked() {
                                self.is_custom_text_color_open = false;
                            }
                        });
                    });
                });

            if !is_open {
                self.is_custom_text_color_open = false;
            }
        }
    }
}
