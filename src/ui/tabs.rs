use crate::app::NixobdoPdfApp;
use eframe::egui;

impl NixobdoPdfApp {
    pub(crate) fn ui_tabs(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("tab_bar_panel").show_inside(ui, |ui| {
            ui.scope(|ui| {
                let is_dark_mode = ui.visuals().dark_mode;

                // Adjust colors based on user request
                if !is_dark_mode {
                    // Light mode: use same color as Reset button (standard inactive button bg)
                    let reset_btn_color = ui.visuals().widgets.inactive.weak_bg_fill;
                    ui.style_mut().visuals.selection.bg_fill = reset_btn_color;
                    ui.style_mut().visuals.widgets.hovered.bg_fill =
                        ui.visuals().widgets.hovered.weak_bg_fill;
                } else {
                    // Dark mode: darker than default tab, slightly lighter than universal background (rgb 27)
                    let custom_dark_bg = egui::Color32::from_rgb(38, 38, 38);
                    let custom_hover_bg = egui::Color32::from_rgb(45, 45, 45);
                    ui.style_mut().visuals.selection.bg_fill = custom_dark_bg;
                    ui.style_mut().visuals.widgets.hovered.bg_fill = custom_hover_bg;
                }

                // Add a little bit of roundness (4px)
                ui.style_mut().visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);
                ui.style_mut().visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
                ui.style_mut().visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
                ui.style_mut().visuals.widgets.noninteractive.corner_radius =
                    egui::CornerRadius::same(4);

                if !self.tabs.is_empty() {
                    ui.horizontal(|ui| {
                        let mut tab_to_close = None;
                        for (index, tab) in self.tabs.iter().enumerate() {
                            let is_active = Some(index) == self.active_tab_index;

                            // Remove logo and add space for the close button within the tab
                            let text = format!("{}      ", tab.file_name);

                            let mut text_color = ui.visuals().text_color();
                            if is_active {
                                if !is_dark_mode {
                                    text_color = ui.visuals().text_color();
                                } else {
                                    text_color = egui::Color32::from_rgb(220, 220, 220);
                                }
                            }

                            let text_style = if is_active {
                                egui::RichText::new(text).color(text_color).strong()
                            } else {
                                egui::RichText::new(text).color(text_color)
                            };

                            let tab_resp = ui.selectable_label(is_active, text_style);
                            if tab_resp.clicked() {
                                self.active_tab_index = Some(index);
                            }

                            // Overlay the close button inside the tab's rect, aligned to text
                            let rect = tab_resp.rect;
                            let close_rect = egui::Rect::from_min_size(
                                egui::pos2(
                                    rect.right() - 22.0,
                                    rect.top() + (rect.height() - 16.0) / 2.0 - 2.0,
                                ),
                                egui::vec2(16.0, 16.0),
                            );

                            let close_btn =
                                egui::Button::new(egui::RichText::new("×").size(14.0)).frame(false);
                            if ui.put(close_rect, close_btn).clicked() {
                                tab_to_close = Some(index);
                            }

                            ui.add_space(4.0);
                        }

                        if let Some(close_idx) = tab_to_close {
                            self.close_tab(close_idx);
                        }
                    });
                } else {
                    ui.add_space(16.0);
                }
            });
        });
    }
}
