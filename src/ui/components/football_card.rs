// Football Fixture Card Component (Netflix-style)
// Displays a football match with broadcast channels and allows searching IPTV channels

use egui::{self, Color32, Rounding, Vec2};
use crate::api::FootballFixture;
use crate::ui::theme::Theme;

/// Channels to ignore (betting sites, not actual TV channels)
const IGNORED_CHANNELS: &[&str] = &["Bet365", "bet365", "Bet 365"];

/// Actions that can be triggered from a football card
#[derive(Debug, Clone)]
pub enum FootballAction {
    /// Search for a channel in live TV
    SearchChannel(String),
    /// Search for team name in live TV
    SearchTeam(String),
}

/// A card component for displaying football fixtures (Netflix-style)
pub struct FootballCard;

impl FootballCard {
    /// Check if a channel should be ignored
    fn should_ignore_channel(channel: &str) -> bool {
        IGNORED_CHANNELS.iter().any(|&ignored| 
            channel.to_lowercase().contains(&ignored.to_lowercase())
        )
    }
    
    /// Map channel name to IPTV search query
    fn map_channel_to_search_query(channel: &str) -> String {
        let channel_lower = channel.to_lowercase();

        // DAZN mappings with country codes
        if channel_lower.contains("dazn") {
            // Extract country from channel name
            if channel_lower.contains("italy") || channel_lower.contains("italia") {
                return "|IT| DAZN".to_string();
            } else if channel_lower.contains("germany") || channel_lower.contains("deutschland") {
                return "|DE| DAZN".to_string();
            } else if channel_lower.contains("spain") || channel_lower.contains("españa") || channel_lower.contains("espana") {
                return "|ES| DAZN".to_string();
            } else if channel_lower.contains("portugal") {
                return "|PT| DAZN".to_string();
            } else if channel_lower.contains("france") {
                return "|FR| DAZN".to_string();
            } else if channel_lower.contains("uk") || channel_lower.contains("united kingdom") {
                return "|UK| DAZN".to_string();
            } else if channel_lower.contains("belgium") {
                return "|BE| DAZN".to_string();
            } else if channel_lower.contains("austria") {
                return "|AT| DAZN".to_string();
            } else if channel_lower.contains("switzerland") || channel_lower.contains("swiss") {
                return "|CH| DAZN".to_string();
            } else if channel_lower.contains("japan") {
                return "|JP| DAZN".to_string();
            } else if channel_lower.contains("canada") {
                return "|CA| DAZN".to_string();
            } else if channel_lower.contains("usa") || channel_lower.contains("us ") {
                return "|US| DAZN".to_string();
            }
            // If no country detected, just return "DAZN"
            return "DAZN".to_string();
        }

        // Tring mappings
        if channel_lower.contains("tring") {
            return "Tring Sport".to_string();
        }

        // Default: return original channel name
        channel.to_string()
    }
    
    /// Get filtered channels (excluding betting sites)
    fn get_filtered_channels(fixture: &FootballFixture) -> Vec<String> {
        fixture.broadcasters.iter()
            .filter(|b| !Self::should_ignore_channel(&b.channel))
            .map(|b| b.channel.clone())
            .collect()
    }

    /// Show a simple row-based football fixture card
    pub fn show(
        ui: &mut egui::Ui,
        theme: &Theme,
        fixture: &FootballFixture,
        _screen_width: f32,
    ) -> Option<FootballAction> {
        let mut action = None;

        // Simple row layout - full width
        let row_height = 80.0;

        egui::Frame::none()
            .fill(Color32::from_rgb(30, 30, 30))
            .rounding(Rounding::same(6.0))
            .inner_margin(egui::Margin::symmetric(16.0, 12.0))
            .show(ui, |ui| {
                ui.set_min_height(row_height);

                ui.horizontal(|ui| {
                    // Left: Time and teams (40%)
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width() * 0.4);

                        // Competition badge
                        let badge_color = Self::competition_color(&fixture.competition);
                        ui.label(egui::RichText::new(&fixture.competition)
                            .size(10.0)
                            .color(badge_color));

                        ui.add_space(4.0);

                        // Time
                        ui.label(egui::RichText::new(&fixture.display_time())
                            .size(14.0)
                            .color(badge_color)
                            .strong());

                        ui.add_space(6.0);

                        // Teams
                        ui.label(egui::RichText::new(format!("{} vs {}",
                            fixture.home_team, fixture.away_team))
                            .size(13.0)
                            .color(Color32::WHITE));
                    });

                    ui.add_space(16.0);

                    // Right: Channels (60%)
                    ui.vertical(|ui| {
                        let filtered_channels = Self::get_filtered_channels(fixture);

                        if !filtered_channels.is_empty() {
                            ui.label(egui::RichText::new("📡 Watch on:")
                                .size(9.0)
                                .color(theme.text_secondary));

                            ui.add_space(4.0);

                            // Show channels as wrapped pills
                            ui.horizontal_wrapped(|ui| {
                                let badge_color = Self::competition_color(&fixture.competition);

                                for (i, channel) in filtered_channels.iter().take(6).enumerate() {
                                    if i >= 6 { break; }

                                    let search_query = Self::map_channel_to_search_query(channel);
                                    let display_name = if channel.len() > 15 {
                                        format!("{}...", &channel[..12])
                                    } else {
                                        channel.clone()
                                    };

                                    let btn = egui::Button::new(
                                        egui::RichText::new(&display_name)
                                            .size(9.5)
                                            .color(Color32::from_rgb(220, 220, 220))
                                    )
                                    .fill(Color32::from_rgb(45, 45, 45))
                                    .rounding(Rounding::same(12.0))
                                    .min_size(egui::vec2(0.0, 22.0));

                                    if ui.add(btn)
                                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                                        .on_hover_text(format!("🔍 Search '{}' in Live TV", search_query))
                                        .clicked()
                                    {
                                        action = Some(FootballAction::SearchChannel(search_query));
                                    }
                                }

                                if filtered_channels.len() > 6 {
                                    ui.label(egui::RichText::new(format!("+{}", filtered_channels.len() - 6))
                                        .size(9.0)
                                        .color(theme.text_secondary));
                                }
                            });
                        } else {
                            ui.label(egui::RichText::new("No broadcast info")
                                .size(10.0)
                                .color(theme.text_secondary));
                        }
                    });
                });
            });

        ui.add_space(8.0);

        action
    }

    /// Show a Netflix-style football fixture card (OLD - unused)
    #[allow(dead_code)]
    pub fn show_old(
        ui: &mut egui::Ui,
        theme: &Theme,
        fixture: &FootballFixture,
        _screen_width: f32,
    ) -> Option<FootballAction> {
        let mut action = None;

        // Use full available width - parent handles the grid layout
        let card_width = ui.available_width();
        let card_height = 160.0; // Compact height

        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(card_width, card_height),
            egui::Sense::hover(),
        );

        if ui.is_rect_visible(rect) {
            let is_hovered = response.hovered();
            
            // Card background - Netflix dark with hover effect
            let bg_color = if is_hovered {
                Color32::from_rgb(50, 50, 50)
            } else {
                Color32::from_rgb(35, 35, 35)
            };
            
            // Draw card shadow
            ui.painter().rect_filled(
                rect.translate(egui::vec2(2.0, 4.0)),
                Rounding::same(8.0),
                Color32::from_rgba_unmultiplied(0, 0, 0, 80),
            );
            
            // Draw card background
            ui.painter().rect_filled(rect, Rounding::same(8.0), bg_color);
            
            // Competition color stripe at top
            let badge_color = Self::competition_color(&fixture.competition);
            let stripe_rect = egui::Rect::from_min_size(
                rect.min,
                egui::vec2(card_width, 4.0),
            );
            ui.painter().rect_filled(
                stripe_rect,
                Rounding { nw: 8.0, ne: 8.0, sw: 0.0, se: 0.0 },
                badge_color,
            );
            
            // Horizontal layout: left side = match info, right side = channels
            let content_rect = rect.shrink2(egui::vec2(20.0, 16.0));

            // Left column - Match info (50% width)
            let left_width = (content_rect.width() * 0.5).min(300.0);
            let left_rect = egui::Rect::from_min_max(
                content_rect.min,
                egui::pos2(content_rect.min.x + left_width, content_rect.max.y),
            );

            let mut y_pos = left_rect.min.y;

            // Competition badge
            ui.painter().text(
                egui::pos2(left_rect.min.x, y_pos),
                egui::Align2::LEFT_TOP,
                &fixture.competition,
                egui::FontId::proportional(10.0),
                badge_color,
            );
            y_pos += 20.0;

            // Time - larger and prominent
            ui.painter().text(
                egui::pos2(left_rect.min.x, y_pos),
                egui::Align2::LEFT_TOP,
                &fixture.display_time(),
                egui::FontId::proportional(16.0),
                badge_color.linear_multiply(1.2),
            );
            y_pos += 28.0;

            // Home team
            ui.painter().text(
                egui::pos2(left_rect.min.x, y_pos),
                egui::Align2::LEFT_TOP,
                &fixture.home_team,
                egui::FontId::proportional(15.0),
                Color32::WHITE,
            );
            y_pos += 22.0;

            // VS
            ui.painter().text(
                egui::pos2(left_rect.min.x, y_pos),
                egui::Align2::LEFT_TOP,
                "vs",
                egui::FontId::proportional(11.0),
                theme.text_secondary,
            );
            y_pos += 20.0;

            // Away team
            ui.painter().text(
                egui::pos2(left_rect.min.x, y_pos),
                egui::Align2::LEFT_TOP,
                &fixture.away_team,
                egui::FontId::proportional(15.0),
                Color32::WHITE,
            );

            // Vertical divider
            let divider_x = content_rect.min.x + left_width + 16.0;
            ui.painter().vline(
                divider_x,
                content_rect.min.y..=content_rect.max.y,
                egui::Stroke::new(1.0, Color32::from_rgb(50, 50, 50)),
            );

            // Right column - Channels
            let right_x = divider_x + 16.0;
            let filtered_channels = Self::get_filtered_channels(fixture);

            if !filtered_channels.is_empty() {
                let mut channel_y = content_rect.min.y;

                // "Broadcast on:" label
                ui.painter().text(
                    egui::pos2(right_x, channel_y),
                    egui::Align2::LEFT_TOP,
                    "📡 Available on:",
                    egui::FontId::proportional(10.0),
                    theme.text_secondary,
                );
                channel_y += 22.0;

                // Show channels as pills (up to 4)
                let channels_to_show = filtered_channels.len().min(4);
                let button_height = 28.0;
                let button_spacing = 8.0;

                for (i, channel) in filtered_channels.iter().take(channels_to_show).enumerate() {
                    let search_query = Self::map_channel_to_search_query(channel);

                    // Calculate button width
                    let display_name = if channel.chars().count() > 22 {
                        format!("{}...", channel.chars().take(20).collect::<String>())
                    } else {
                        channel.clone()
                    };

                    let text_width = display_name.len() as f32 * 6.0 + 20.0;
                    let max_right_width = content_rect.max.x - right_x - 8.0;
                    let button_width = text_width.min(max_right_width);

                    let btn_y = channel_y + (i as f32 * (button_height + button_spacing));
                    let btn_rect = egui::Rect::from_min_size(
                        egui::pos2(right_x, btn_y),
                        egui::vec2(button_width, button_height),
                    );

                    let btn_response = ui.allocate_rect(btn_rect, egui::Sense::click());
                    let btn_hovered = btn_response.hovered();

                    // Button styling
                    let btn_color = if btn_hovered {
                        badge_color
                    } else {
                        Color32::from_rgb(45, 45, 45)
                    };

                    ui.painter().rect_filled(btn_rect, Rounding::same(14.0), btn_color);

                    ui.painter().text(
                        egui::pos2(btn_rect.min.x + 10.0, btn_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &display_name,
                        egui::FontId::proportional(10.5),
                        if btn_hovered { Color32::WHITE } else { Color32::from_rgb(200, 200, 200) },
                    );

                    if btn_response.on_hover_text(format!("🔍 Search '{}' in Live TV", search_query)).clicked() {
                        action = Some(FootballAction::SearchChannel(search_query));
                    }
                }

                // "+N more" indicator
                if filtered_channels.len() > channels_to_show {
                    let remaining = filtered_channels.len() - channels_to_show;
                    let more_y = channel_y + (channels_to_show as f32 * (button_height + button_spacing)) + 4.0;
                    ui.painter().text(
                        egui::pos2(right_x + 10.0, more_y),
                        egui::Align2::LEFT_TOP,
                        &format!("+{} more", remaining),
                        egui::FontId::proportional(9.0),
                        theme.text_secondary.linear_multiply(0.7),
                    );
                }
            } else {
                ui.painter().text(
                    egui::pos2(right_x, content_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    "No broadcast info",
                    egui::FontId::proportional(10.0),
                    theme.text_secondary.linear_multiply(0.6),
                );
            }
        }
        
        ui.add_space(12.0); // Gap between cards
        
        action
    }
    
    /// Get color for competition badge
    fn competition_color(competition: &str) -> Color32 {
        match competition.to_lowercase().as_str() {
            s if s.contains("premier") => Color32::from_rgb(130, 40, 180), // Purple
            s if s.contains("la liga") => Color32::from_rgb(255, 87, 34), // Orange
            s if s.contains("serie a") => Color32::from_rgb(0, 130, 180), // Blue
            s if s.contains("bundesliga") => Color32::from_rgb(220, 20, 60), // Red
            s if s.contains("ligue 1") => Color32::from_rgb(30, 80, 150), // Dark Blue
            s if s.contains("champions") => Color32::from_rgb(0, 80, 150), // UEFA Blue
            s if s.contains("europa") => Color32::from_rgb(255, 140, 0), // Orange
            _ => Color32::from_rgb(70, 130, 80), // Default green
        }
    }

    /// Show a compact fixture row (for lists)
    #[allow(dead_code)]
    pub fn show_row(
        ui: &mut egui::Ui,
        theme: &Theme,
        fixture: &FootballFixture,
    ) -> Option<FootballAction> {
        let mut action = None;
        let filtered_channels = Self::get_filtered_channels(fixture);
        
        ui.horizontal(|ui| {
            // Time
            ui.label(
                egui::RichText::new(&fixture.display_time())
                    .size(12.0)
                    .color(theme.accent_blue)
                    .strong()
            );
            
            ui.separator();
            
            // Competition badge
            let badge_color = Self::competition_color(&fixture.competition);
            let badge_text = egui::RichText::new(&fixture.competition)
                .size(10.0)
                .color(Color32::WHITE);
            
            let badge_response = ui.add(
                egui::Button::new(badge_text)
                    .fill(badge_color)
                    .rounding(Rounding::same(4.0))
                    .sense(egui::Sense::hover())
            );
            badge_response.on_hover_text(&fixture.competition);
            
            ui.separator();
            
            // Teams
            let match_text = format!("{} vs {}", fixture.home_team, fixture.away_team);
            if ui.link(
                egui::RichText::new(&match_text)
                    .size(13.0)
                    .color(theme.text_primary)
            ).on_hover_text("Search for this match").clicked() {
                action = Some(FootballAction::SearchTeam(fixture.home_team.clone()));
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Show first 2 channels as buttons
                for channel in filtered_channels.iter().take(2) {
                    let search_query = Self::map_channel_to_search_query(channel);
                    if ui.small_button(channel).on_hover_text(format!("Search '{}' in Live TV", search_query)).clicked() {
                        action = Some(FootballAction::SearchChannel(search_query));
                    }
                }
                
                if filtered_channels.len() > 2 {
                    ui.label(
                        egui::RichText::new(format!("+{}", filtered_channels.len() - 2))
                            .size(10.0)
                            .color(theme.text_secondary)
                    );
                }
            });
        });
        
        action
    }
}
