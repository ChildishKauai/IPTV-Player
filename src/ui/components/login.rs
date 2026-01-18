//! Login screen component (Netflix-style).

use eframe::egui;
use crate::ui::theme::{Theme, dimensions};

/// Login screen component for initial authentication.
pub struct LoginScreen;

impl LoginScreen {
    /// Renders the Netflix-style login screen and returns true if connect was clicked.
    /// Supports Steam Deck touch mode with larger input targets.
    pub fn show(
        ui: &mut egui::Ui,
        theme: &Theme,
        server_url: &mut String,
        username: &mut String,
        password: &mut String,
        connecting: bool,
        error_message: &Option<String>,
        is_touch_mode: bool, // Steam Deck or tablet mode
    ) -> bool {
        let mut should_connect = false;
        let screen_width = ui.available_width();
        let is_mobile = dimensions::is_mobile(screen_width);
        
        // Touch mode adjustments (Steam Deck friendly)
        let input_height = if is_touch_mode { 56.0 } else { 48.0 };
        let button_height = if is_touch_mode { 60.0 } else if is_mobile { 48.0 } else { 44.0 };
        let font_size_large = if is_touch_mode { 18.0 } else if is_mobile { 15.0 } else { 16.0 };
        let spacing = if is_touch_mode { 20.0 } else if is_mobile { 12.0 } else { 16.0 };
        
        // Responsive sizing
        let input_width = if is_mobile { 
            (screen_width - 64.0).max(260.0) 
        } else if is_touch_mode {
            400.0  // Wider for Steam Deck
        } else { 
            350.0 
        };
        let card_max_width = if is_mobile { 
            screen_width - 32.0 
        } else if is_touch_mode {
            460.0  // Wider for Steam Deck
        } else { 
            400.0 
        };
        let top_space = if is_mobile { 60.0 } else if is_touch_mode { 40.0 } else { 80.0 };
        
        ui.vertical_centered(|ui| {
            ui.add_space(top_space);
            
            // Netflix-style logo area
            ui.label(egui::RichText::new("IPTV")
                .size(if is_mobile { 42.0 } else if is_touch_mode { 48.0 } else { 56.0 })
                .color(theme.accent_blue)
                .strong());
            ui.add_space(8.0);
            
            ui.label(egui::RichText::new("Stream Anywhere")
                .size(if is_mobile { 14.0 } else if is_touch_mode { 18.0 } else { 16.0 })
                .color(theme.text_secondary));
            
            ui.add_space(if is_mobile { 40.0 } else if is_touch_mode { 30.0 } else { 50.0 });
            
            // Netflix-style dark card
            egui::Frame::none()
                .fill(if theme.dark_mode {
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200)
                } else {
                    theme.card_bg
                })
                .rounding(egui::Rounding::same(if is_touch_mode { 8.0 } else { 4.0 }))
                .inner_margin(egui::Margin::same(if is_mobile { 24.0 } else if is_touch_mode { 32.0 } else { 40.0 }))
                .show(ui, |ui| {
                    ui.set_max_width(card_max_width);
                    
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Sign In")
                            .size(if is_mobile { 24.0 } else if is_touch_mode { 28.0 } else { 28.0 })
                            .color(theme.text_primary)
                            .strong());
                        
                        ui.add_space(if is_mobile { 20.0 } else if is_touch_mode { 24.0 } else { 28.0 });
                        
                        // Server URL input
                        Self::input_field(ui, theme, server_url, "Server URL", "http://server:port", input_width, input_height, font_size_large, false);
                        ui.add_space(spacing);
                        
                        // Username input
                        Self::input_field(ui, theme, username, "Username", "Enter username", input_width, input_height, font_size_large, false);
                        ui.add_space(spacing);
                        
                        // Password input
                        Self::input_field(ui, theme, password, "Password", "Enter password", input_width, input_height, font_size_large, true);
                        
                        ui.add_space(if is_mobile { 24.0 } else if is_touch_mode { 28.0 } else { 32.0 });
                        
                        if connecting {
                            ui.horizontal(|ui| {
                                ui.add_space((input_width - 100.0) / 2.0);
                                ui.spinner();
                                ui.label(egui::RichText::new("Connecting...")
                                    .color(theme.text_secondary));
                            });
                        } else {
                            // Netflix-style red button - larger for touch
                            let connect_btn = egui::Button::new(
                                egui::RichText::new("Sign In")
                                    .size(font_size_large)
                                    .color(egui::Color32::WHITE)
                                    .strong()
                            )
                            .fill(theme.accent_blue)
                            .min_size(egui::vec2(input_width, button_height))
                            .rounding(egui::Rounding::same(if is_touch_mode { 8.0 } else { 4.0 }));
                            
                            if ui.add(connect_btn).clicked() {
                                should_connect = true;
                            }
                        }
                        
                        ui.add_space(16.0);
                    });
                });
            
            // Error message
            if let Some(error) = error_message {
                ui.add_space(16.0);
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(50, 20, 20))
                    .rounding(egui::Rounding::same(if is_touch_mode { 8.0 } else { 4.0 }))
                    .inner_margin(egui::Margin::same(if is_mobile { 12.0 } else { 16.0 }))
                    .show(ui, |ui| {
                        ui.set_max_width(card_max_width);
                        ui.horizontal_wrapped(|ui| {
                            ui.label(egui::RichText::new("⚠")
                                .color(theme.warning_color));
                            ui.label(egui::RichText::new(error)
                                .color(theme.warning_color)
                                .size(13.0));
                        });
                    });
            }
            
            // Footer
            ui.add_space(40.0);
            ui.label(egui::RichText::new("Connect to your Xtream Codes IPTV service")
                .size(if is_touch_mode { 14.0 } else { 12.0 })
                .color(theme.text_secondary));
        });
        
        should_connect
    }
    
    /// Renders a Netflix-style input field with configurable dimensions.
    fn input_field(
        ui: &mut egui::Ui,
        theme: &Theme,
        value: &mut String,
        _label: &str,
        placeholder: &str,
        width: f32,
        height: f32,
        font_size: f32,
        is_password: bool,
    ) {
        egui::Frame::none()
            .fill(theme.inactive_bg())
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::symmetric(16.0, (height - font_size) / 2.0))
            .show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                let mut edit = egui::TextEdit::singleline(value)
                    .hint_text(egui::RichText::new(placeholder).color(theme.text_secondary).size(font_size))
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
