//! Channel card component for displaying live TV channels (Netflix-style).

use eframe::egui;
use crate::models::{Channel, EpgProgram};
use crate::ui::theme::{Theme, dimensions};
use crate::ui::image_cache::ImageCache;

/// Actions that can be triggered from a channel card.
#[derive(Debug, Clone)]
pub enum ChannelAction {
    /// Play the channel
    Play(Channel),
    /// Toggle favorite status
    ToggleFavorite(String),
}

/// EPG info to display on a channel card
pub struct ChannelEpgInfo {
    pub current_program: Option<EpgProgram>,
    pub next_program: Option<EpgProgram>,
}

#[allow(dead_code)]
impl ChannelEpgInfo {
    pub fn empty() -> Self {
        Self {
            current_program: None,
            next_program: None,
        }
    }
}

/// Channel card component (Netflix-style tile).
pub struct ChannelCard;

impl ChannelCard {
    /// Renders a Netflix-style channel card.
    /// Returns any action that was triggered.
    pub fn show(
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        theme: &Theme,
        channel: &Channel,
        is_favorite: bool,
        image_cache: &ImageCache,
        screen_width: f32,
        epg_info: Option<&ChannelEpgInfo>,
    ) -> Option<ChannelAction> {
        let mut action: Option<ChannelAction> = None;
        let is_mobile = dimensions::is_mobile(screen_width);
        
        // Netflix-style wide tile for channels
        let card_width = if is_mobile {
            (screen_width - 48.0).max(280.0)
        } else {
            320.0
        };
        let card_height = if is_mobile { 90.0 } else { 100.0 };
        
        // Load image if needed
        if !channel.stream_icon.is_empty() {
            image_cache.load(ctx, channel.stream_icon.clone());
        }
        
        // Allocate space for card
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(card_width + 8.0, card_height + 8.0),
            egui::Sense::click(),
        );
        
        if response.clicked() {
            action = Some(ChannelAction::Play(channel.clone()));
        }
        
        let is_hovered = response.hovered();
        let card_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(4.0, 4.0),
            egui::vec2(card_width, card_height),
        );
        
        // Draw card background with shadow on hover
        if is_hovered {
            let shadow_rect = card_rect.expand(2.0);
            ui.painter().rect_filled(shadow_rect, 8.0, theme.card_shadow());
        }
        
        // Card background
        let card_bg = if is_hovered { theme.hover_bg() } else { theme.card_bg };
        ui.painter().rect_filled(card_rect, 8.0, card_bg);
        
        // Channel icon (left side)
        let icon_size = if is_mobile { 60.0 } else { 70.0 };
        let icon_rect = egui::Rect::from_min_size(
            card_rect.min + egui::vec2(12.0, (card_height - icon_size) / 2.0),
            egui::vec2(icon_size, icon_size),
        );
        
        if !channel.stream_icon.is_empty() {
            if let Some(texture) = image_cache.get(&channel.stream_icon) {
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                ui.painter().image(texture.id(), icon_rect, uv, egui::Color32::WHITE);
                ui.painter().rect_stroke(icon_rect, 6.0, egui::Stroke::new(1.0, theme.border_color));
            } else {
                Self::paint_placeholder(ui, theme, icon_rect);
            }
        } else {
            Self::paint_placeholder(ui, theme, icon_rect);
        }
        
        // Text content (right side)
        let text_x = icon_rect.max.x + 16.0;
        let text_width = card_width - icon_size - 60.0;
        
        // Channel name
        let name_text = truncate_text(&channel.name, if is_mobile { 28 } else { 35 });
        let name_galley = ui.painter().layout_no_wrap(
            name_text,
            egui::FontId::proportional(if is_mobile { 14.0 } else { 15.0 }),
            theme.text_primary,
        );
        ui.painter().galley(
            egui::pos2(text_x, card_rect.min.y + 14.0),
            name_galley,
            theme.text_primary,
        );
        
        // EPG info
        let has_epg = epg_info.map(|e| e.current_program.is_some()).unwrap_or(false);
        if let Some(epg) = epg_info {
            if let Some(current) = &epg.current_program {
                // Current program
                let prog_text = format!("▶ {}", truncate_text(&current.title, if is_mobile { 25 } else { 30 }));
                let prog_galley = ui.painter().layout_no_wrap(
                    prog_text,
                    egui::FontId::proportional(if is_mobile { 11.0 } else { 12.0 }),
                    theme.text_secondary,
                );
                ui.painter().galley(
                    egui::pos2(text_x, card_rect.min.y + 36.0),
                    prog_galley,
                    theme.text_secondary,
                );
                
                // Progress bar
                let progress = current.progress();
                let bar_y = card_rect.min.y + 55.0;
                let bar_width = text_width.min(180.0);
                let bar_rect = egui::Rect::from_min_size(
                    egui::pos2(text_x, bar_y),
                    egui::vec2(bar_width, 3.0),
                );
                ui.painter().rect_filled(bar_rect, 2.0, theme.border_color);
                let progress_rect = egui::Rect::from_min_size(
                    bar_rect.min,
                    egui::vec2(bar_width * progress, 3.0),
                );
                ui.painter().rect_filled(progress_rect, 2.0, theme.accent_blue);
                
                // Next program
                if let Some(next) = &epg.next_program {
                    let time_str = next.start_time_formatted();
                    let next_text = if time_str.is_empty() {
                        format!("⏭ {}", truncate_text(&next.title, 20))
                    } else {
                        format!("⏭ {} {}", time_str, truncate_text(&next.title, 15))
                    };
                    let next_galley = ui.painter().layout_no_wrap(
                        next_text,
                        egui::FontId::proportional(10.0),
                        theme.text_secondary,
                    );
                    ui.painter().galley(
                        egui::pos2(text_x, card_rect.min.y + 70.0),
                        next_galley,
                        theme.text_secondary,
                    );
                }
            }
        }
        
        // If no EPG, show "Live" badge
        if !has_epg {
            let live_rect = egui::Rect::from_min_size(
                egui::pos2(text_x, card_rect.min.y + 40.0),
                egui::vec2(45.0, 20.0),
            );
            ui.painter().rect_filled(live_rect, 4.0, theme.accent_blue);
            let live_galley = ui.painter().layout_no_wrap(
                "LIVE".to_string(),
                egui::FontId::proportional(10.0),
                egui::Color32::WHITE,
            );
            ui.painter().galley(
                egui::pos2(live_rect.min.x + 10.0, live_rect.min.y + 4.0),
                live_galley,
                egui::Color32::WHITE,
            );
        }
        
        // Favorite star (top right)
        let star_pos = egui::pos2(card_rect.max.x - 28.0, card_rect.min.y + 12.0);
        let star_rect = egui::Rect::from_center_size(star_pos, egui::vec2(24.0, 24.0));
        let star_response = ui.interact(star_rect, ui.id().with(&channel.stream_id), egui::Sense::click());
        
        if star_response.clicked() {
            action = Some(ChannelAction::ToggleFavorite(channel.stream_id.clone()));
        }
        
        let star_color = if is_favorite { theme.warning_color } else { theme.text_secondary };
        let star_text = if is_favorite { "★" } else { "☆" };
        let star_galley = ui.painter().layout_no_wrap(
            star_text.to_string(),
            egui::FontId::proportional(18.0),
            star_color,
        );
        ui.painter().galley(
            egui::pos2(star_pos.x - star_galley.size().x / 2.0, star_pos.y - star_galley.size().y / 2.0),
            star_galley,
            star_color,
        );
        
        action
    }
    
    /// Paints a placeholder for missing icons.
    fn paint_placeholder(ui: &egui::Ui, theme: &Theme, rect: egui::Rect) {
        ui.painter().rect_filled(rect, 6.0, theme.placeholder_bg());
        
        let center = rect.center();
        let galley = ui.painter().layout_no_wrap(
            "📺".to_string(),
            egui::FontId::proportional(rect.width() * 0.4),
            theme.placeholder_icon(),
        );
        let text_pos = egui::pos2(
            center.x - galley.size().x / 2.0,
            center.y - galley.size().y / 2.0,
        );
        ui.painter().galley(text_pos, galley, theme.placeholder_icon());
    }
}

/// Truncates text to a maximum length with ellipsis.
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.chars().count() > max_len {
        let truncated: String = text.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    } else {
        text.to_string()
    }
}
