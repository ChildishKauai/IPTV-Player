//! EPG settings dialog - Modern, clean design
//!
//! Configure external EPG sources with a premium settings experience.

use eframe::egui;
use crate::ui::theme::{spacing, typography, radius};

/// Actions returned by the EPG settings dialog.
#[derive(Debug, Clone)]
pub enum EpgSettingsAction {
    /// Settings were saved
    Saved,
    /// Dialog was cancelled
    Cancelled,
}

/// EPG settings dialog component - Modern design
pub struct EpgSettingsDialog;

impl EpgSettingsDialog {
    /// Shows the EPG settings dialog.
    pub fn show(
        ctx: &egui::Context,
        epg_enabled: &mut bool,
        epg_url: &mut String,
    ) -> Option<EpgSettingsAction> {
        let mut action = None;

        // Colors
        let bg = egui::Color32::from_rgb(18, 18, 18);
        let card_bg = egui::Color32::from_rgb(28, 28, 28);
        let text_primary = egui::Color32::WHITE;
        let text_secondary = egui::Color32::from_rgb(170, 170, 170);
        let text_tertiary = egui::Color32::from_rgb(128, 128, 128);
        let accent = egui::Color32::from_rgb(255, 90, 95);
        let info_bg = egui::Color32::from_rgb(20, 35, 50);

        egui::Window::new("")
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .default_width(520.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(
                egui::Frame::none()
                    .fill(bg)
                    .rounding(egui::Rounding::same(radius::XL))
                    .inner_margin(egui::Margin::same(spacing::XL))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(38, 38, 38))),
            )
            .show(ctx, |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("EPG Settings")
                            .size(typography::H1)
                            .color(text_primary)
                            .strong(),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("✕")
                                        .size(20.0)
                                        .color(text_secondary),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .min_size(egui::vec2(40.0, 40.0)),
                            )
                            .clicked()
                        {
                            action = Some(EpgSettingsAction::Cancelled);
                        }
                    });
                });

                ui.add_space(spacing::XL);

                // Enable toggle section
                egui::Frame::none()
                    .fill(card_bg)
                    .rounding(egui::Rounding::same(radius::LG))
                    .inner_margin(egui::Margin::same(spacing::LG))
                    .show(ui, |ui| {
                        ui.set_min_width(460.0);

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("External EPG")
                                        .size(typography::BODY)
                                        .color(text_primary),
                                );
                                ui.label(
                                    egui::RichText::new("Use XMLTV sources like EPGShare01")
                                        .size(typography::CAPTION)
                                        .color(text_tertiary),
                                );
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.checkbox(epg_enabled, "");
                            });
                        });
                    });

                ui.add_space(spacing::MD);

                // URL input section
                egui::Frame::none()
                    .fill(card_bg)
                    .rounding(egui::Rounding::same(radius::LG))
                    .inner_margin(egui::Margin::same(spacing::LG))
                    .show(ui, |ui| {
                        ui.set_min_width(460.0);

                        ui.label(
                            egui::RichText::new("XMLTV URL")
                                .size(typography::BODY_SM)
                                .color(text_secondary),
                        );

                        ui.add_space(spacing::SM);

                        let text_edit = egui::TextEdit::singleline(epg_url)
                            .hint_text("https://epgshare01.online/...")
                            .desired_width(ui.available_width() - spacing::SM)
                            .font(egui::FontId::proportional(typography::BODY_SM));

                        ui.add_enabled(*epg_enabled, text_edit);

                        ui.add_space(spacing::SM);

                        ui.label(
                            egui::RichText::new("Supports .xml and .xml.gz formats")
                                .size(typography::LABEL)
                                .color(text_tertiary),
                        );
                    });

                ui.add_space(spacing::MD);

                // Quick select section
                egui::Frame::none()
                    .fill(card_bg)
                    .rounding(egui::Rounding::same(radius::LG))
                    .inner_margin(egui::Margin::same(spacing::LG))
                    .show(ui, |ui| {
                        ui.set_min_width(460.0);

                        ui.label(
                            egui::RichText::new("Quick Select")
                                .size(typography::BODY)
                                .color(text_primary),
                        );

                        ui.add_space(spacing::MD);

                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(spacing::SM, spacing::SM);

                            let countries = [
                                ("US", "epg_ripper_US2"),
                                ("UK", "epg_ripper_UK1"),
                                ("FR", "epg_ripper_FR1"),
                                ("DE", "epg_ripper_DE1"),
                                ("ES", "epg_ripper_ES1"),
                                ("IT", "epg_ripper_IT1"),
                                ("CA", "epg_ripper_CA2"),
                                ("AU", "epg_ripper_AU1"),
                            ];

                            for (code, file) in countries {
                                let btn = egui::Button::new(
                                    egui::RichText::new(code)
                                        .size(typography::BODY_SM)
                                        .color(if *epg_enabled {
                                            text_primary
                                        } else {
                                            text_tertiary
                                        }),
                                )
                                .fill(if *epg_enabled {
                                    egui::Color32::from_rgb(40, 40, 40)
                                } else {
                                    egui::Color32::from_rgb(30, 30, 30)
                                })
                                .min_size(egui::vec2(52.0, 36.0))
                                .rounding(egui::Rounding::same(radius::MD));

                                if ui.add_enabled(*epg_enabled, btn).clicked() {
                                    *epg_url = format!(
                                        "https://epgshare01.online/epgshare01/{}.xml.gz",
                                        file
                                    );
                                }
                            }
                        });

                        ui.add_space(spacing::SM);

                        ui.label(
                            egui::RichText::new("More regions at epgshare01.online")
                                .size(typography::LABEL)
                                .color(text_tertiary),
                        );
                    });

                ui.add_space(spacing::MD);

                // Info box
                egui::Frame::none()
                    .fill(info_bg)
                    .rounding(egui::Rounding::same(radius::MD))
                    .inner_margin(egui::Margin::same(spacing::MD))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                egui::RichText::new(
                                    "External EPG supplements Xtream API. Channels with tvg-id use external EPG first.",
                                )
                                .size(typography::CAPTION)
                                .color(egui::Color32::from_rgb(150, 180, 210)),
                            );
                        });
                    });

                ui.add_space(spacing::XL);

                // Action buttons
                ui.horizontal(|ui| {
                    // Save button
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Save")
                                    .size(typography::BODY_SM)
                                    .color(egui::Color32::WHITE)
                                    .strong(),
                            )
                            .fill(accent)
                            .rounding(egui::Rounding::same(radius::MD))
                            .min_size(egui::vec2(100.0, 44.0)),
                        )
                        .clicked()
                    {
                        action = Some(EpgSettingsAction::Saved);
                    }

                    ui.add_space(spacing::SM);

                    // Cancel button
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Cancel")
                                    .size(typography::BODY_SM)
                                    .color(text_secondary),
                            )
                            .fill(card_bg)
                            .rounding(egui::Rounding::same(radius::MD))
                            .min_size(egui::vec2(100.0, 44.0)),
                        )
                        .clicked()
                    {
                        action = Some(EpgSettingsAction::Cancelled);
                    }
                });
            });

        action
    }
}
