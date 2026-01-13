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
        
        // DAZN mappings
        if channel_lower.contains("dazn") {
            if channel_lower.contains("ital") {
                return "|IT| DAZN".to_string();
            } else if channel_lower.contains("german") || channel_lower.contains("deutschland") {
                return "|DE| DAZN".to_string();
            }
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

    /// Show a Netflix-style football fixture card
    pub fn show(
        ui: &mut egui::Ui,
        theme: &Theme,
        fixture: &FootballFixture,
        _screen_width: f32,
    ) -> Option<FootballAction> {
        let mut action = None;
        
        // Card dimensions
        let card_width = 280.0;
        let card_height = 200.0;
        
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
            
            // Content area
            let content_rect = rect.shrink2(egui::vec2(16.0, 8.0));
            let mut y_pos = content_rect.min.y + 12.0;
            
            // Competition name
            ui.painter().text(
                egui::pos2(content_rect.center().x, y_pos),
                egui::Align2::CENTER_TOP,
                &fixture.competition,
                egui::FontId::proportional(11.0),
                badge_color,
            );
            y_pos += 20.0;
            
            // Time and date
            let time_text = format!("{}  •  {}", fixture.display_time(), fixture.fixture_date);
            ui.painter().text(
                egui::pos2(content_rect.center().x, y_pos),
                egui::Align2::CENTER_TOP,
                &time_text,
                egui::FontId::proportional(10.0),
                theme.text_secondary,
            );
            y_pos += 24.0;
            
            // Home team
            ui.painter().text(
                egui::pos2(content_rect.center().x, y_pos),
                egui::Align2::CENTER_TOP,
                &fixture.home_team,
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );
            y_pos += 18.0;
            
            // vs
            ui.painter().text(
                egui::pos2(content_rect.center().x, y_pos),
                egui::Align2::CENTER_TOP,
                "vs",
                egui::FontId::proportional(10.0),
                theme.text_secondary,
            );
            y_pos += 14.0;
            
            // Away team
            ui.painter().text(
                egui::pos2(content_rect.center().x, y_pos),
                egui::Align2::CENTER_TOP,
                &fixture.away_team,
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );
            y_pos += 24.0;
            
            // Channel buttons - only show on hover or always show first one
            let filtered_channels = Self::get_filtered_channels(fixture);
            
            if !filtered_channels.is_empty() {
                let channels_to_show = if is_hovered { 3 } else { 2 };
                let button_spacing = 6.0;
                let button_height = 24.0;
                
                // Calculate total width needed
                let max_button_width = 80.0;
                let total_buttons = filtered_channels.len().min(channels_to_show);
                let total_width = (max_button_width * total_buttons as f32) + (button_spacing * (total_buttons - 1) as f32);
                let start_x = content_rect.center().x - total_width / 2.0;
                
                for (i, channel) in filtered_channels.iter().take(channels_to_show).enumerate() {
                    let search_query = Self::map_channel_to_search_query(channel);
                    
                    let btn_x = start_x + (i as f32 * (max_button_width + button_spacing));
                    let btn_rect = egui::Rect::from_min_size(
                        egui::pos2(btn_x, y_pos),
                        egui::vec2(max_button_width, button_height),
                    );
                    
                    let btn_response = ui.allocate_rect(btn_rect, egui::Sense::click());
                    let btn_hovered = btn_response.hovered();
                    
                    // Button background
                    let btn_color = if btn_hovered {
                        badge_color
                    } else {
                        badge_color.linear_multiply(0.7)
                    };
                    
                    ui.painter().rect_filled(btn_rect, Rounding::same(4.0), btn_color);
                    
                    // Truncate channel name
                    let display_name = if channel.chars().count() > 10 {
                        format!("{}...", channel.chars().take(8).collect::<String>())
                    } else {
                        channel.clone()
                    };
                    
                    ui.painter().text(
                        btn_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &display_name,
                        egui::FontId::proportional(9.0),
                        Color32::WHITE,
                    );
                    
                    if btn_response.on_hover_text(format!("Search '{}' in Live TV", search_query)).clicked() {
                        action = Some(FootballAction::SearchChannel(search_query));
                    }
                }
                
                // Show +N more indicator
                if filtered_channels.len() > channels_to_show {
                    let more_text = format!("+{}", filtered_channels.len() - channels_to_show);
                    let more_x = start_x + (total_buttons as f32 * (max_button_width + button_spacing));
                    ui.painter().text(
                        egui::pos2(more_x + 10.0, y_pos + button_height / 2.0),
                        egui::Align2::LEFT_CENTER,
                        &more_text,
                        egui::FontId::proportional(10.0),
                        theme.text_secondary,
                    );
                }
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
