#![allow(deprecated)]
use crate::app::NixobdoPdfApp;
use eframe::egui;

impl NixobdoPdfApp {
    pub(crate) fn ui_about_dialog(&mut self, ui: &mut egui::Ui) {
        let mut about_open = self.about_window_open;
        if about_open {
            let branch = option_env!("GIT_BRANCH").unwrap_or("unknown");
            let stability = if branch == "main" || branch == "master" { "Stable" } else { "Unstable" };
            
            ui.ctx().show_viewport_immediate(
                egui::ViewportId::from_hash_of("about_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("About Nixobdo PDF Reader")
                    .with_inner_size([750.0, 400.0])
                    .with_resizable(false)
                    .with_maximize_button(false)
                    .with_minimize_button(false),
                |ctx, _class| {
                    if ui.ctx().input(|i| i.viewport().close_requested()) {
                        about_open = false;
                    }

                    // Dynamically support light mode in the popup based on system preference
                    let is_light = ui.ctx().system_theme() == Some(egui::Theme::Light);
                    let mut style = (*ui.ctx().global_style()).clone();
                    if is_light {
                        style.visuals = egui::Visuals::light();
                    }
                    
                    let bg_fill = if is_light { egui::Color32::from_rgb(245, 245, 245) } else { ui.ctx().global_style().visuals.window_fill };
                    
                    egui::Panel::bottom("about_bottom_panel")
                        .frame(egui::Frame::default().inner_margin(egui::Margin { left: 16, right: 16, top: 8, bottom: 16 }).fill(bg_fill))
                        .show(ctx, |ui| {
                            ui.set_style(style.clone());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(egui::RichText::new("Close").size(14.0)).clicked() {
                                    about_open = false;
                                }
                            });
                        });
                        
                    egui::CentralPanel::default()
                        .frame(egui::Frame::default().inner_margin(16).fill(bg_fill))
                        .show(ctx, |ui| {
                            ui.set_style(style);
                            ui.horizontal(|ui| {
                                // Left image
                                ui.vertical(|ui| {
                                    let img = egui::Image::new(egui::include_image!("../../../assets/logo.png"))
                                        .max_width(320.0)
                                        .max_height(320.0);
                                    ui.add(img);
                                });
                                
                                ui.add_space(20.0);
                                
                                // Right info
                                ui.vertical(|ui| {
                                    ui.heading(egui::RichText::new("Nixobdo PDF Reader").size(24.0).strong());
                                    
                                    ui.add_space(12.0);
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(egui::RichText::new("Nixobdo PDF Reader is a modern, easy-to-use, open source productivity suite for reading, annotating, and managing PDF documents.").size(13.0));
                                    });
                                    
                                    ui.add_space(10.0);
                                    ui.label(egui::RichText::new("This release was provided by Borneel B. Phukan.").size(12.0));
                                    ui.label(egui::RichText::new("Copyright © 2026-Present Borneel B. Phukan.").size(12.0));
                                    
                                    ui.add_space(10.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(40.0);
                                        ui.hyperlink_to(egui::RichText::new("Credits").size(13.0), "https://github.com/borneelphukan/nixobdo-pdf/graphs/contributors");
                                        ui.add_space(8.0);
                                        ui.hyperlink_to(egui::RichText::new("Website").size(13.0), "https://borneelphukan.github.io/nixobdo-pdf/");
                                        ui.add_space(8.0);
                                        ui.hyperlink_to(egui::RichText::new("Release Notes").size(13.0), "https://github.com/borneelphukan/nixobdo-pdf/releases");
                                    });
                                    
                                    ui.add_space(20.0);
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("Version Information").size(14.0).strong());
                                        if ui.button("📋").on_hover_text("Copy Version Info").clicked() {
                                            let version_info = format!(
                                                "Version: {} ({})\nEnvironment: OS: {} ({}); Arch: {}", 
                                                env!("CARGO_PKG_VERSION"), stability, std::env::consts::OS, std::env::consts::FAMILY, std::env::consts::ARCH
                                            );
                                            ui.ctx().copy_text(version_info);
                                        }
                                    });
                                    
                                    ui.add_space(4.0);
                                    egui::Grid::new("about_version_grid").num_columns(2).spacing([12.0, 4.0]).show(ui, |ui| {
                                        ui.label(egui::RichText::new("Version:").size(12.0));
                                        ui.label(egui::RichText::new(format!("{} ({})", env!("CARGO_PKG_VERSION"), stability)).size(12.0));
                                        ui.end_row();
                                        
                                        ui.label(egui::RichText::new("Environment:").size(12.0));
                                        ui.label(egui::RichText::new(format!("OS: {} ({}); Arch: {}", std::env::consts::OS, std::env::consts::FAMILY, std::env::consts::ARCH)).size(12.0));
                                        ui.end_row();
                                        
                                        ui.label(egui::RichText::new("Developer:").size(12.0));
                                        ui.label(egui::RichText::new("Borneel B. Phukan").size(12.0));
                                        ui.end_row();
                                    });
                                });
                            });
                        });
                }
            );
            self.about_window_open = about_open;
        }
    }
}


