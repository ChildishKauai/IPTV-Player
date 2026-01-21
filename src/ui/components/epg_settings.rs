//! EPG settings dialog for configuring external EPG sources.

use eframe::egui;

/// Actions returned by the EPG settings dialog.
#[derive(Debug, Clone)]
pub enum EpgSettingsAction {
    /// Settings were saved
    Saved,
    /// Dialog was cancelled
    Cancelled,
}

/// EPG settings dialog component.
pub struct EpgSettingsDialog;

impl EpgSettingsDialog {
    /// Shows the EPG settings dialog.
    /// Returns an action if the dialog was closed.
    pub fn show(
        ctx: &egui::Context,
        epg_enabled: &mut bool,
        epg_url: &mut String,
    ) -> Option<EpgSettingsAction> {
        let mut action = None;

        egui::Window::new("EPG Settings")
            .resizable(false)
            .collapsible(false)
            .default_width(500.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(24, 24, 24))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(24.0)))
            .show(ctx, |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📺 EPG Configuration")
                        .size(22.0)
                        .color(egui::Color32::WHITE)
                        .strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(
                            egui::RichText::new("✕").size(16.0).color(egui::Color32::WHITE)
                        ).fill(egui::Color32::TRANSPARENT)).clicked() {
                            action = Some(EpgSettingsAction::Cancelled);
                        }
                    });
                });

                ui.add_space(20.0);

                // EPG Enable Toggle
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(35, 35, 35))
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::same(16.0))
                    .show(ui, |ui| {
                        ui.set_min_width(440.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Enable External EPG:")
                                .color(egui::Color32::from_rgb(180, 180, 180)));
                            ui.add_space(10.0);
                            ui.checkbox(epg_enabled, "");
                        });

                        ui.add_space(8.0);

                        ui.label(egui::RichText::new("Enable this to use external XMLTV EPG sources (like EPGShare01)")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(120, 120, 120)));
                    });

                ui.add_space(12.0);

                // EPG URL Input
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(35, 35, 35))
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::same(16.0))
                    .show(ui, |ui| {
                        ui.set_min_width(440.0);

                        ui.label(egui::RichText::new("EPG URL:")
                            .color(egui::Color32::from_rgb(180, 180, 180)));

                        ui.add_space(8.0);

                        let text_edit = egui::TextEdit::singleline(epg_url)
                            .hint_text("https://epgshare01.online/epgshare01/epg_ripper_US2.xml.gz")
                            .desired_width(420.0);

                        ui.add_enabled(*epg_enabled, text_edit);

                        ui.add_space(8.0);

                        ui.label(egui::RichText::new("Enter XMLTV EPG URL. Supports .xml and .xml.gz formats.")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(120, 120, 120)));
                    });

                ui.add_space(12.0);

                // Quick Links
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(35, 35, 35))
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::same(16.0))
                    .show(ui, |ui| {
                        ui.set_min_width(440.0);

                        ui.label(egui::RichText::new("📌 Popular EPG Sources (EPGShare01):")
                            .color(egui::Color32::from_rgb(180, 180, 180))
                            .strong());

                        ui.add_space(8.0);

                        // Quick select buttons - EPGShare01 sources
                        ui.horizontal(|ui| {
                            if ui.button("🇺🇸 US").clicked() && *epg_enabled {
                                *epg_url = "https://epgshare01.online/epgshare01/epg_ripper_US2.xml.gz".to_string();
                            }
                            if ui.button("🇬🇧 UK").clicked() && *epg_enabled {
                                *epg_url = "https://epgshare01.online/epgshare01/epg_ripper_UK1.xml.gz".to_string();
                            }
                            if ui.button("🇫🇷 France").clicked() && *epg_enabled {
                                *epg_url = "https://epgshare01.online/epgshare01/epg_ripper_FR1.xml.gz".to_string();
                            }
                            if ui.button("🇩🇪 Germany").clicked() && *epg_enabled {
                                *epg_url = "https://epgshare01.online/epgshare01/epg_ripper_DE1.xml.gz".to_string();
                            }
                        });

                        ui.horizontal(|ui| {
                            if ui.button("🇪🇸 Spain").clicked() && *epg_enabled {
                                *epg_url = "https://epgshare01.online/epgshare01/epg_ripper_ES1.xml.gz".to_string();
                            }
                            if ui.button("🇮🇹 Italy").clicked() && *epg_enabled {
                                *epg_url = "https://epgshare01.online/epgshare01/epg_ripper_IT1.xml.gz".to_string();
                            }
                            if ui.button("🇨🇦 Canada").clicked() && *epg_enabled {
                                *epg_url = "https://epgshare01.online/epgshare01/epg_ripper_CA2.xml.gz".to_string();
                            }
                            if ui.button("🇦🇺 Australia").clicked() && *epg_enabled {
                                *epg_url = "https://epgshare01.online/epgshare01/epg_ripper_AU1.xml.gz".to_string();
                            }
                        });

                        ui.add_space(8.0);

                        ui.label(egui::RichText::new("Browse 60+ countries: epgshare01.online")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(100, 100, 100)));
                    });

                ui.add_space(16.0);

                // Info box
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(20, 40, 60))
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.set_min_width(440.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.label(egui::RichText::new("ℹ️").size(14.0));
                            ui.label(egui::RichText::new(
                                "External EPG supplements Xtream API. Channels with tvg-id will use external EPG first. M3U playlists with x-tvg-url auto-detect EPG."
                            ).size(11.0).color(egui::Color32::from_rgb(180, 200, 220)));
                        });
                    });

                ui.add_space(16.0);

                // Action buttons
                ui.horizontal(|ui| {
                    // Save button
                    if ui.add(egui::Button::new(
                        egui::RichText::new("Save")
                            .color(egui::Color32::WHITE)
                            .strong()
                    ).fill(egui::Color32::from_rgb(0, 122, 255))
                        .rounding(egui::Rounding::same(4.0))
                        .min_size(egui::vec2(80.0, 36.0)))
                        .clicked() {
                        action = Some(EpgSettingsAction::Saved);
                    }

                    ui.add_space(8.0);

                    // Cancel button
                    if ui.add(egui::Button::new(
                        egui::RichText::new("Cancel")
                            .color(egui::Color32::WHITE)
                    ).fill(egui::Color32::from_rgb(60, 60, 60))
                        .rounding(egui::Rounding::same(4.0))
                        .min_size(egui::vec2(80.0, 36.0)))
                        .clicked() {
                        action = Some(EpgSettingsAction::Cancelled);
                    }
                });
            });

        action
    }
}
