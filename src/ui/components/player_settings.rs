//! Player settings dialog for audio and subtitle configuration (Netflix-style).

use eframe::egui;
use crate::models::{PlayerSettings, PlayerType};

/// Common audio language options.
const AUDIO_LANGUAGES: &[(&str, &str)] = &[
    ("", "Auto/Default"),
    ("eng", "English"),
    ("spa", "Spanish"),
    ("fra", "French"),
    ("deu", "German"),
    ("ita", "Italian"),
    ("por", "Portuguese"),
    ("rus", "Russian"),
    ("jpn", "Japanese"),
    ("kor", "Korean"),
    ("zho", "Chinese"),
    ("ara", "Arabic"),
    ("hin", "Hindi"),
    ("tur", "Turkish"),
    ("pol", "Polish"),
    ("nld", "Dutch"),
];

/// Common subtitle language options.
const SUBTITLE_LANGUAGES: &[(&str, &str)] = &[
    ("", "Auto/Default"),
    ("eng", "English"),
    ("spa", "Spanish"),
    ("fra", "French"),
    ("deu", "German"),
    ("ita", "Italian"),
    ("por", "Portuguese"),
    ("rus", "Russian"),
    ("jpn", "Japanese"),
    ("kor", "Korean"),
    ("zho", "Chinese"),
    ("ara", "Arabic"),
    ("hin", "Hindi"),
    ("tur", "Turkish"),
    ("pol", "Polish"),
    ("nld", "Dutch"),
    ("off", "Disabled"),
];

/// Actions returned by the player settings dialog.
#[derive(Debug, Clone)]
pub enum PlayerSettingsAction {
    /// Settings were saved
    Saved,
    /// Dialog was cancelled
    Cancelled,
    /// Reset to defaults
    Reset,
}

/// Player settings dialog component (Netflix-style).
pub struct PlayerSettingsDialog;

impl PlayerSettingsDialog {
    /// Renders a section header in Netflix style.
    fn section_header(ui: &mut egui::Ui, text: &str) {
        ui.label(egui::RichText::new(text)
            .size(14.0)
            .color(egui::Color32::from_rgb(180, 180, 180))
            .strong());
        ui.add_space(4.0);
    }
    
    /// Shows the Netflix-style player settings dialog.
    /// Returns an action if the dialog was closed.
    pub fn show(
        ctx: &egui::Context,
        _theme: &crate::ui::theme::Theme,
        settings: &mut PlayerSettings,
    ) -> Option<PlayerSettingsAction> {
        let mut action = None;
        
        egui::Window::new("")
            .resizable(true)
            .collapsible(false)
            .title_bar(false)
            .default_width(520.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(24, 24, 24))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(24.0)))
            .show(ctx, |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Settings")
                        .size(22.0)
                        .color(egui::Color32::WHITE)
                        .strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(
                            egui::RichText::new("✕").size(16.0).color(egui::Color32::WHITE)
                        ).fill(egui::Color32::TRANSPARENT)).clicked() {
                            action = Some(PlayerSettingsAction::Cancelled);
                        }
                    });
                });
                
                ui.add_space(20.0);
                
                egui::ScrollArea::vertical().max_height(450.0).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 12.0;
                    
                    // Player Selection Section
                    Self::section_header(ui, "🎬 Media Player");
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(35, 35, 35))
                        .rounding(egui::Rounding::same(6.0))
                        .inner_margin(egui::Margin::same(16.0))
                        .show(ui, |ui| {
                            ui.set_min_width(460.0);
                            
                            // Player type selection
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Player:")
                                    .color(egui::Color32::from_rgb(180, 180, 180)));
                                ui.add_space(10.0);
                                
                                egui::ComboBox::from_id_salt("player_type")
                                    .selected_text(settings.player_type.display_name())
                                    .show_ui(ui, |ui| {
                                        for player in PlayerType::all() {
                                            ui.selectable_value(
                                                &mut settings.player_type,
                                                *player,
                                                player.display_name()
                                            );
                                        }
                                    });
                            });
                            
                            // Show player-specific info
                            ui.add_space(4.0);
                            let info_text = match settings.player_type {
                                PlayerType::FFplay => "Part of FFmpeg. Lightweight and fast.",
                                PlayerType::VLC => "Feature-rich player with wide format support.",
                                PlayerType::MPV => "Minimalist player with excellent performance.",
                                PlayerType::MpcHc => "Classic Windows media player.",
                                PlayerType::PotPlayer => "Advanced player with many features.",
                                PlayerType::Custom => "Use your own player executable.",
                            };
                            ui.label(egui::RichText::new(info_text)
                                .size(11.0)
                                .color(egui::Color32::from_rgb(120, 120, 120)));
                            
                            // Custom player settings
                            if settings.player_type == PlayerType::Custom {
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Executable Path:")
                                        .color(egui::Color32::from_rgb(180, 180, 180)));
                                    ui.add_space(10.0);
                                    ui.add(egui::TextEdit::singleline(&mut settings.custom_player_path)
                                        .hint_text("e.g., C:\\Program Files\\Player\\player.exe")
                                        .desired_width(260.0));
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Arguments:")
                                        .color(egui::Color32::from_rgb(180, 180, 180)));
                                    ui.add_space(10.0);
                                    ui.add(egui::TextEdit::singleline(&mut settings.custom_player_args)
                                        .hint_text("{url} {title} {volume}")
                                        .desired_width(260.0));
                                });
                                
                                ui.label(egui::RichText::new("Placeholders: {url}, {title}, {volume}, {audio_track}, {subtitle_track}")
                                    .size(10.0)
                                    .color(egui::Color32::from_rgb(100, 100, 100)));
                            }
                        });
                    
                    ui.add_space(8.0);
                    
                    // Audio Settings Section
                    Self::section_header(ui, "🔊 Audio Settings");
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(35, 35, 35))
                        .rounding(egui::Rounding::same(6.0))
                        .inner_margin(egui::Margin::same(16.0))
                        .show(ui, |ui| {
                            ui.set_min_width(460.0);
                            
                            // Audio track selection
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Audio Track:")
                                    .color(egui::Color32::from_rgb(180, 180, 180)));
                                ui.add_space(10.0);
                                
                                egui::ComboBox::from_id_salt("audio_track")
                                    .selected_text(if settings.audio_track < 0 {
                                        "Disabled".to_string()
                                    } else if settings.audio_track == 0 {
                                        "Default (Auto)".to_string()
                                } else {
                                    format!("Track {}", settings.audio_track)
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut settings.audio_track, 0, "Default (Auto)");
                                    for i in 1..=10 {
                                        ui.selectable_value(&mut settings.audio_track, i, format!("Track {}", i));
                                    }
                                    ui.selectable_value(&mut settings.audio_track, -1, "Disabled");
                                });
                        });
                        
                        // Preferred audio language
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Preferred Language:")
                                .color(egui::Color32::from_rgb(180, 180, 180)));
                            ui.add_space(10.0);
                            
                            let current_lang = AUDIO_LANGUAGES.iter()
                                .find(|(code, _)| *code == settings.preferred_audio_language)
                                .map(|(_, name)| *name)
                                .unwrap_or("Auto/Default");
                            
                            egui::ComboBox::from_id_salt("audio_language")
                                .selected_text(current_lang)
                                .show_ui(ui, |ui| {
                                    for (code, name) in AUDIO_LANGUAGES {
                                        if ui.selectable_label(
                                            settings.preferred_audio_language == *code,
                                            *name
                                        ).clicked() {
                                            settings.preferred_audio_language = code.to_string();
                                        }
                                    }
                                });
                        });
                        
                        // Volume slider
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Volume:")
                                .color(egui::Color32::from_rgb(180, 180, 180)));
                            ui.add_space(10.0);
                            
                            let mut volume_f32 = settings.volume as f32;
                            if ui.add(egui::Slider::new(&mut volume_f32, 0.0..=150.0)
                                .suffix("%"))
                                .changed() {
                                settings.volume = volume_f32.clamp(0.0, 150.0) as i32;
                            }
                        });
                        
                        // Audio sync offset
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Audio Sync:")
                                .color(egui::Color32::from_rgb(180, 180, 180)));
                            ui.add_space(10.0);
                            
                            ui.add(egui::Slider::new(&mut settings.audio_sync_offset, -5.0..=5.0)
                                .suffix("s")
                                .fixed_decimals(2));
                            
                            if ui.small_button("Reset").clicked() {
                                settings.audio_sync_offset = 0.0;
                            }
                        });
                    });
                    
                    ui.add_space(8.0);
                    
                    // Subtitle Settings Section
                    Self::section_header(ui, "📝 Subtitle Settings");
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(35, 35, 35))
                        .rounding(egui::Rounding::same(6.0))
                        .inner_margin(egui::Margin::same(16.0))
                        .show(ui, |ui| {
                            ui.set_min_width(460.0);
                        
                        // Enable subtitles toggle
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut settings.subtitles_enabled, "");
                            ui.label(egui::RichText::new("Enable Subtitles")
                                .color(if settings.subtitles_enabled { 
                                    egui::Color32::WHITE 
                                } else { 
                                    egui::Color32::from_rgb(120, 120, 120) 
                                }));
                        });
                        
                        // Subtitle track selection (only if enabled)
                        ui.add_enabled_ui(settings.subtitles_enabled, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Subtitle Track:")
                                    .color(egui::Color32::from_rgb(180, 180, 180)));
                                ui.add_space(10.0);
                                
                                egui::ComboBox::from_id_salt("subtitle_track")
                                    .selected_text(if settings.subtitle_track < 0 {
                                        "Auto (First Available)".to_string()
                                    } else {
                                        format!("Track {}", settings.subtitle_track)
                                    })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut settings.subtitle_track, -1, "Auto (First Available)");
                                        for i in 0..=10 {
                                            ui.selectable_value(&mut settings.subtitle_track, i, format!("Track {}", i));
                                        }
                                    });
                            });
                            
                            // Preferred subtitle language
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Preferred Language:")
                                    .color(egui::Color32::from_rgb(180, 180, 180)));
                                ui.add_space(10.0);
                                
                                let current_lang = SUBTITLE_LANGUAGES.iter()
                                    .find(|(code, _)| *code == settings.preferred_subtitle_language)
                                    .map(|(_, name)| *name)
                                    .unwrap_or("Auto/Default");
                                
                                egui::ComboBox::from_id_salt("subtitle_language")
                                    .selected_text(current_lang)
                                    .show_ui(ui, |ui| {
                                        for (code, name) in SUBTITLE_LANGUAGES {
                                            if ui.selectable_label(
                                                settings.preferred_subtitle_language == *code,
                                                *name
                                            ).clicked() {
                                                settings.preferred_subtitle_language = code.to_string();
                                            }
                                        }
                                    });
                            });
                            
                            // Subtitle sync offset
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Subtitle Sync:")
                                    .color(egui::Color32::from_rgb(180, 180, 180)));
                                ui.add_space(10.0);
                                
                                ui.add(egui::Slider::new(&mut settings.subtitle_sync_offset, -5.0..=5.0)
                                    .suffix("s")
                                    .fixed_decimals(2));
                                
                                if ui.small_button("Reset").clicked() {
                                    settings.subtitle_sync_offset = 0.0;
                                }
                            });
                        });
                    });
                    
                    ui.add_space(8.0);
                    
                    // Performance Settings Section
                    Self::section_header(ui, "⚡ Performance Settings");
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(35, 35, 35))
                        .rounding(egui::Rounding::same(6.0))
                        .inner_margin(egui::Margin::same(16.0))
                        .show(ui, |ui| {
                            ui.set_min_width(460.0);
                        
                        // Hardware acceleration
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut settings.hardware_acceleration, "");
                            ui.label(egui::RichText::new("Hardware Acceleration")
                                .color(egui::Color32::WHITE));
                            ui.label(egui::RichText::new("(Uses GPU for decoding)")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(120, 120, 120)));
                        });
                        
                        // Low latency mode
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut settings.low_latency_mode, "");
                            ui.label(egui::RichText::new("Low Latency Mode")
                                .color(egui::Color32::WHITE));
                            ui.label(egui::RichText::new("(Best for live streams)")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(120, 120, 120)));
                        });
                        
                        // Buffer size
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Buffer Size:")
                                .color(egui::Color32::from_rgb(180, 180, 180)));
                            ui.add_space(10.0);
                            
                            let mut buffer_f32 = settings.buffer_size_kb as f32;
                            if ui.add(egui::Slider::new(&mut buffer_f32, 0.0..=8192.0)
                                .suffix(" KB")
                                .custom_formatter(|v, _| {
                                    if v == 0.0 { "Auto".to_string() }
                                    else { format!("{:.0}", v) }
                                }))
                                .changed() {
                                settings.buffer_size_kb = buffer_f32.clamp(0.0, 8192.0) as u32;
                            }
                        });
                    });
                    
                    ui.add_space(16.0);
                    
                    // Action buttons
                    ui.horizontal(|ui| {
                        // Save button - Netflix red
                        if ui.add(egui::Button::new(
                            egui::RichText::new("Save")
                                .color(egui::Color32::WHITE)
                                .strong()
                        ).fill(egui::Color32::from_rgb(229, 9, 20))
                            .rounding(egui::Rounding::same(4.0))
                            .min_size(egui::vec2(80.0, 36.0)))
                            .clicked() {
                            action = Some(PlayerSettingsAction::Saved);
                        }
                        
                        ui.add_space(8.0);
                        
                        // Cancel button - dark
                        if ui.add(egui::Button::new(
                            egui::RichText::new("Cancel")
                                .color(egui::Color32::WHITE)
                        ).fill(egui::Color32::from_rgb(60, 60, 60))
                            .rounding(egui::Rounding::same(4.0))
                            .min_size(egui::vec2(80.0, 36.0)))
                            .clicked() {
                            action = Some(PlayerSettingsAction::Cancelled);
                        }
                        
                        ui.add_space(8.0);
                        
                        // Reset button - transparent
                        if ui.add(egui::Button::new(
                            egui::RichText::new("Reset to Defaults")
                                .color(egui::Color32::from_rgb(180, 180, 180))
                        ).fill(egui::Color32::TRANSPARENT)
                            .rounding(egui::Rounding::same(4.0)))
                            .clicked() {
                            action = Some(PlayerSettingsAction::Reset);
                        }
                    });
                }); // End ScrollArea
            });
        
        action
    }
}
