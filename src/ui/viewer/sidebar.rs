use crate::app::NixobdoPdfApp;
use eframe::egui;

impl NixobdoPdfApp {
    pub(crate) fn ui_sidebar(&mut self, ctx: &egui::Context) {
        let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        if !self.sidebar_open || is_fullscreen {
            return;
        }
        egui::SidePanel::left("preview_panel")
            .resizable(false)
            .exact_width(180.0)
            .show(ctx, |ui| {
                let mut pages_empty = true;
                if let Some(active_idx) = self.active_tab_index {
                    if let Some(tab) = self.tabs.get_mut(active_idx) {
                        pages_empty = tab.pages.is_empty();
                        
                        if !pages_empty {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    for (index, texture_opt) in tab.thumbnails.iter().enumerate() {
                                        let thumb_w = (ui.available_width() - 16.0).max(40.0);
                                        let thumb_h = thumb_w * 1.414; // Default A4 ratio for placeholder
                                        let thumb_size = egui::vec2(thumb_w, thumb_h);
                                        
                                        let is_selected = tab.selected_page == index;
                                        let stroke_color = if is_selected {
                                            ui.visuals().selection.bg_fill
                                        } else {
                                            ui.visuals().widgets.noninteractive.bg_stroke.color
                                        };
                                        let bg_color = if is_selected {
                                            ui.visuals().selection.bg_fill.gamma_multiply(0.15)
                                        } else {
                                            ui.visuals().widgets.noninteractive.bg_fill
                                        };

                                        egui::Frame::NONE
                                            .stroke(egui::Stroke::new(if is_selected { 2.0 } else { 1.0 }, stroke_color))
                                            .fill(bg_color)
                                            .corner_radius(4.0)
                                            .inner_margin(egui::Margin::symmetric(4, 3))
                                            .show(ui, |ui| {
                                                if let Some(texture) = texture_opt {
                                                    let actual_aspect = texture.size_vec2().y / texture.size_vec2().x;
                                                    let actual_h = thumb_w * actual_aspect;
                                                    
                                                    let size = egui::vec2(thumb_w, actual_h);
                                                    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
                                                    if ui.is_rect_visible(rect) {
                                                        ui.painter().rect_filled(rect, 0.0, egui::Color32::WHITE);
                                                        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                                                        ui.painter().image(texture.id(), rect, uv, egui::Color32::WHITE);
                                                    }
                                                    if response.clicked() {
                                                        tab.selected_page = index;
                                                        tab.scroll_to_page = Some(index);
                                                    }
                                                } else {
                                                    // Placeholder spinner for loading pages
                                                    let (rect, response) = ui.allocate_exact_size(thumb_size, egui::Sense::click());
                                                    if ui.is_rect_visible(rect) {
                                                        ui.painter().rect_filled(rect, 2.0, egui::Color32::from_gray(40));
                                                        ui.painter().text(
                                                            rect.center(),
                                                            egui::Align2::CENTER_CENTER,
                                                            "...",
                                                            egui::FontId::proportional(14.0),
                                                            egui::Color32::GRAY,
                                                        );
                                                    }
                                                    if response.clicked() {
                                                        tab.selected_page = index;
                                                        tab.scroll_to_page = Some(index);
                                                    }
                                                }
                                                
                                                ui.vertical_centered(|ui| {
                                                    ui.add_space(4.0);
                                                    ui.label(format!("{}", index + 1));
                                                });
                                            });
                                        ui.add_space(8.0);
                                    }
                                });
                            });
                        }
                    }
                }
                
                if pages_empty {
                    ui.centered_and_justified(|ui| {
                        ui.label("No PDF loaded");
                    });
                }
            });
    }
}
