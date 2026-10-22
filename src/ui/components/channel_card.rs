//! Channel card component - Modern, clean design
//!
//! Displays live TV channels with EPG information in a premium card layout.
//! Features clean typography, subtle hover effects, and efficient space usage.

use eframe::egui;
use crate::models::{Channel, EpgProgram};
use crate::ui::theme::{Theme, spacing, typography, radius};
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

/// Channel card component - Modern horizontal card design
pub struct ChannelCard;

impl ChannelCard {
    /// Renders a modern channel card.
    /// Returns any action that was triggered.
    pub fn show(
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        theme: &Theme,
        channel: &Channel,
        is_favorite: bool,
        image_cache: &ImageCache,
        card_width: f32,
        epg_info: Option<&ChannelEpgInfo>,
    ) -> Option<ChannelAction> {
        let mut action: Option<ChannelAction> = None;
        let is_small = card_width < 340.0;

        // Card dimensions - width comes from the responsive grid
        let card_height = if is_small { 88.0 } else { 96.0 };
        let icon_size = if is_small { 52.0 } else { 60.0 };

        // Load image if needed
        if !channel.stream_icon.is_empty() {
            image_cache.load(ctx, channel.stream_icon.clone());
        }

        // Allocate card space
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(card_width, card_height),
            egui::Sense::click(),
        );

        // Keep gamepad/keyboard focus visible inside scrolling rows and grids
        if response.gained_focus() {
            response.scroll_to_me(None);
        }

        if response.clicked() {
            action = Some(ChannelAction::Play(channel.clone()));
        }

        let is_hovered = response.hovered();
        let has_focus = response.has_focus();
        let card_rect = rect;

        // Card background with subtle shadow on hover or focus (for gamepad navigation)
        if is_hovered || has_focus {
            let shadow_rect = card_rect.expand(2.0);
            ui.painter().rect_filled(
                shadow_rect.translate(egui::vec2(0.0, 2.0)),
                radius::LG,
                theme.card_shadow(),
            );
        }

        // Card background
        let card_bg = if is_hovered || has_focus {
            theme.card_elevated
        } else {
            theme.card_bg
        };
        ui.painter().rect_filled(card_rect, radius::LG, card_bg);

        // Border - accent color on focus for gamepad navigation visibility
        let border_color = if has_focus {
            theme.accent_blue
        } else {
            theme.border_color
        };
        let border_width = if has_focus { 2.0 } else { 1.0 };
        ui.painter().rect_stroke(
            card_rect,
            radius::LG,
            egui::Stroke::new(border_width, border_color),
        );

        // Channel icon - circular with border
        let icon_margin = (card_height - icon_size) / 2.0;
        let icon_rect = egui::Rect::from_min_size(
            card_rect.min + egui::vec2(spacing::LG, icon_margin),
            egui::vec2(icon_size, icon_size),
        );

        if !channel.stream_icon.is_empty() {
            if let Some(texture) = image_cache.get(&channel.stream_icon) {
                // Draw circular image with clipping
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                ui.painter().image(texture.id(), icon_rect, uv, egui::Color32::WHITE);
                // Circular border
                ui.painter().rect_stroke(
                    icon_rect,
                    radius::MD,
                    egui::Stroke::new(1.0, theme.border_color),
                );
            } else {
                Self::paint_placeholder(ui, theme, icon_rect);
            }
        } else {
            Self::paint_placeholder(ui, theme, icon_rect);
        }

        // Content area (right side of icon)
        let content_x = icon_rect.max.x + spacing::MD;
        let content_width = card_width - icon_size - spacing::LG * 2.0 - spacing::MD - 32.0; // Reserve space for favorite

        // Channel name - elided to the available width
        let name_galley = super::single_line(
            ui,
            &channel.name,
            egui::FontId::proportional(if is_small { typography::BODY_SM } else { typography::BODY }),
            theme.text_primary,
            content_width,
        );
        ui.painter().galley(
            egui::pos2(content_x, card_rect.min.y + spacing::MD),
            name_galley,
            theme.text_primary,
        );

        // EPG info
        let has_epg = epg_info.map(|e| e.current_program.is_some()).unwrap_or(false);

        if let Some(epg) = epg_info {
            if let Some(current) = &epg.current_program {
                // Current program title
                let prog_galley = super::single_line(
                    ui,
                    &current.title,
                    egui::FontId::proportional(typography::CAPTION),
                    theme.text_secondary,
                    content_width,
                );
                ui.painter().galley(
                    egui::pos2(content_x, card_rect.min.y + spacing::MD + 20.0),
                    prog_galley,
                    theme.text_secondary,
                );

                // Progress bar - thin and elegant
                let progress = current.progress();
                let bar_y = card_rect.min.y + spacing::MD + 40.0;
                let bar_width = (content_width * 0.45).min(180.0);
                let bar_height = 3.0;

                let bar_bg_rect = egui::Rect::from_min_size(
                    egui::pos2(content_x, bar_y),
                    egui::vec2(bar_width, bar_height),
                );
                ui.painter().rect_filled(bar_bg_rect, radius::FULL, theme.border_color);

                let progress_rect = egui::Rect::from_min_size(
                    bar_bg_rect.min,
                    egui::vec2(bar_width * progress, bar_height),
                );
                ui.painter().rect_filled(progress_rect, radius::FULL, theme.accent_blue);

                // Next program (if space)
                if let Some(next) = &epg.next_program {
                    let time_str = next.start_time_formatted();
                    let next_text = if time_str.is_empty() {
                        format!("Next: {}", next.title)
                    } else {
                        format!("{} {}", time_str, next.title)
                    };
                    let next_galley = super::single_line(
                        ui,
                        &next_text,
                        egui::FontId::proportional(typography::LABEL),
                        theme.text_tertiary,
                        content_width - bar_width - spacing::SM,
                    );
                    ui.painter().galley(
                        egui::pos2(content_x + bar_width + spacing::SM, bar_y - 1.0),
                        next_galley,
                        theme.text_tertiary,
                    );
                }
            }
        }

        // If no EPG, show LIVE badge
        if !has_epg {
            let badge_rect = egui::Rect::from_min_size(
                egui::pos2(content_x, card_rect.min.y + spacing::MD + 24.0),
                egui::vec2(42.0, 20.0),
            );

            // Pulsing live badge
            ui.painter().rect_filled(badge_rect, radius::SM, theme.live_badge());

            let live_galley = ui.painter().layout_no_wrap(
                "LIVE".to_string(),
                egui::FontId::proportional(typography::LABEL),
                egui::Color32::WHITE,
            );
            let text_pos = egui::pos2(
                badge_rect.center().x - live_galley.size().x / 2.0,
                badge_rect.center().y - live_galley.size().y / 2.0,
            );
            ui.painter().galley(text_pos, live_galley, egui::Color32::WHITE);
        }

        // Favorite star (right side)
        let star_pos = egui::pos2(card_rect.max.x - spacing::LG - 6.0, card_rect.center().y);
        let star_rect = egui::Rect::from_center_size(star_pos, egui::vec2(28.0, 28.0));
        let star_response = ui.interact(
            star_rect,
            ui.id().with(&channel.stream_id),
            egui::Sense::click(),
        );

        if star_response.clicked() {
            action = Some(ChannelAction::ToggleFavorite(channel.stream_id.clone()));
        }

        // Star icon
        let star_color = if is_favorite {
            theme.warning_color
        } else if star_response.hovered() {
            theme.text_secondary
        } else {
            theme.text_muted
        };
        let star_text = if is_favorite { "★" } else { "☆" };
        let star_galley = ui.painter().layout_no_wrap(
            star_text.to_string(),
            egui::FontId::proportional(18.0),
            star_color,
        );
        ui.painter().galley(
            egui::pos2(
                star_pos.x - star_galley.size().x / 2.0,
                star_pos.y - star_galley.size().y / 2.0,
            ),
            star_galley,
            star_color,
        );

        action
    }

    /// Paints a placeholder for missing icons
    fn paint_placeholder(ui: &egui::Ui, theme: &Theme, rect: egui::Rect) {
        ui.painter().rect_filled(rect, radius::MD, theme.placeholder_bg());

        let center = rect.center();
        let galley = ui.painter().layout_no_wrap(
            "TV".to_string(),
            egui::FontId::proportional(rect.width() * 0.35),
            theme.placeholder_icon(),
        );
        let text_pos = egui::pos2(
            center.x - galley.size().x / 2.0,
            center.y - galley.size().y / 2.0,
        );
        ui.painter().galley(text_pos, galley, theme.placeholder_icon());
    }
}
