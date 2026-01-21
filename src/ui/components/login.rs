//! Login screen component - Modern, premium design
//!
//! Inspired by Apple TV+ and Airbnb's login experiences.
//! Features clean typography, generous spacing, and subtle interactions.

use eframe::egui;
use crate::ui::theme::{Theme, dimensions, spacing, typography, radius};

/// Login screen component for initial authentication.
pub struct LoginScreen;

impl LoginScreen {
    /// Renders a modern, premium login screen.
    /// Returns true if connect was clicked.
    pub fn show(
        ui: &mut egui::Ui,
        theme: &Theme,
        server_url: &mut String,
        username: &mut String,
        password: &mut String,
        connecting: bool,
        error_message: &Option<String>,
        is_touch_mode: bool,
    ) -> bool {
        let mut should_connect = false;
        let screen_width = ui.available_width();
        let is_mobile = dimensions::is_mobile(screen_width);

        // Responsive sizing
        let input_height = if is_touch_mode { 56.0 } else { 52.0 };
        let button_height = if is_touch_mode { 56.0 } else { 52.0 };
        let font_size = if is_touch_mode { 16.0 } else { 15.0 };
        let input_width = if is_mobile {
            (screen_width - 48.0).max(280.0)
        } else if is_touch_mode {
            420.0
        } else {
            380.0
        };
        let card_max_width = if is_mobile {
            screen_width - 32.0
        } else if is_touch_mode {
            500.0
        } else {
            460.0
        };

        // Center everything vertically
        ui.vertical_centered(|ui| {
            // Adaptive top spacing
            let top_space = if is_mobile { 40.0 } else { 60.0 };
            ui.add_space(top_space);

            // Logo/Brand area - minimal and elegant
            ui.label(
                egui::RichText::new("IPTV")
                    .size(if is_mobile { 36.0 } else { 44.0 })
                    .color(theme.text_primary)
                    .strong(),
            );

            ui.add_space(spacing::SM);

            ui.label(
                egui::RichText::new("Stream your world")
                    .size(typography::BODY)
                    .color(theme.text_tertiary),
            );

            ui.add_space(spacing::XXXL);

            // Main card container
            egui::Frame::none()
                .fill(theme.card_bg)
                .rounding(egui::Rounding::same(radius::XL))
                .inner_margin(egui::Margin::same(if is_mobile { 24.0 } else { 40.0 }))
                .stroke(egui::Stroke::new(1.0, theme.border_color))
                .show(ui, |ui| {
                    ui.set_max_width(card_max_width);

                    ui.vertical(|ui| {
                        // Header
                        ui.label(
                            egui::RichText::new("Sign in")
                                .size(typography::H1)
                                .color(theme.text_primary)
                                .strong(),
                        );

                        ui.add_space(spacing::SM);

                        ui.label(
                            egui::RichText::new("Enter your Xtream Codes credentials")
                                .size(typography::BODY_SM)
                                .color(theme.text_secondary),
                        );

                        ui.add_space(spacing::XL);

                        // Server URL field
                        Self::input_field(
                            ui,
                            theme,
                            server_url,
                            "Server URL",
                            "http://server:port",
                            input_width,
                            input_height,
                            font_size,
                            false,
                        );

                        ui.add_space(spacing::MD);

                        // Username field
                        Self::input_field(
                            ui,
                            theme,
                            username,
                            "Username",
                            "Enter your username",
                            input_width,
                            input_height,
                            font_size,
                            false,
                        );

                        ui.add_space(spacing::MD);

                        // Password field
                        Self::input_field(
                            ui,
                            theme,
                            password,
                            "Password",
                            "Enter your password",
                            input_width,
                            input_height,
                            font_size,
                            true,
                        );

                        ui.add_space(spacing::XL);

                        // Connect button or loading state
                        if connecting {
                            ui.vertical_centered(|ui| {
                                ui.add_space(spacing::SM);
                                ui.horizontal(|ui| {
                                    ui.add_space((input_width - 120.0) / 2.0);
                                    ui.spinner();
                                    ui.add_space(spacing::SM);
                                    ui.label(
                                        egui::RichText::new("Connecting...")
                                            .size(typography::BODY_SM)
                                            .color(theme.text_secondary),
                                    );
                                });
                                ui.add_space(spacing::SM);
                            });
                        } else {
                            // Modern sign-in button
                            let button = egui::Button::new(
                                egui::RichText::new("Sign in")
                                    .size(font_size)
                                    .color(egui::Color32::WHITE)
                                    .strong(),
                            )
                            .fill(theme.accent_blue)
                            .min_size(egui::vec2(input_width, button_height))
                            .rounding(egui::Rounding::same(radius::MD));

                            if ui.add(button).clicked() {
                                should_connect = true;
                            }
                        }
                    });
                });

            // Error message - outside the card for better visibility
            if let Some(error) = error_message {
                ui.add_space(spacing::LG);

                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(255, 69, 58, 15))
                    .rounding(egui::Rounding::same(radius::MD))
                    .inner_margin(egui::Margin::symmetric(spacing::LG, spacing::MD))
                    .show(ui, |ui| {
                        ui.set_max_width(card_max_width);
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                egui::RichText::new(error)
                                    .size(typography::BODY_SM)
                                    .color(theme.error_color),
                            );
                        });
                    });
            }

            // Footer
            ui.add_space(spacing::XXL);
            ui.label(
                egui::RichText::new("Compatible with Xtream Codes API")
                    .size(typography::CAPTION)
                    .color(theme.text_muted),
            );
        });

        should_connect
    }

    /// Renders a modern input field with floating label effect
    fn input_field(
        ui: &mut egui::Ui,
        theme: &Theme,
        value: &mut String,
        label: &str,
        placeholder: &str,
        width: f32,
        height: f32,
        font_size: f32,
        is_password: bool,
    ) {
        // Label above the field
        ui.label(
            egui::RichText::new(label)
                .size(typography::BODY_SM)
                .color(theme.text_secondary),
        );

        ui.add_space(spacing::XS);

        // Input container
        egui::Frame::none()
            .fill(theme.inactive_bg())
            .rounding(egui::Rounding::same(radius::MD))
            .stroke(egui::Stroke::new(1.0, theme.border_color))
            .inner_margin(egui::Margin::symmetric(spacing::LG, (height - font_size - 8.0) / 2.0))
            .show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height - 16.0);

                let mut edit = egui::TextEdit::singleline(value)
                    .hint_text(
                        egui::RichText::new(placeholder)
                            .size(font_size)
                            .color(theme.text_muted),
                    )
                    .desired_width(width - 32.0)
                    .text_color(theme.text_primary)
                    .font(egui::FontId::proportional(font_size))
                    .frame(false);

                if is_password {
                    edit = edit.password(true);
                }

                ui.add(edit);
            });
    }
}
