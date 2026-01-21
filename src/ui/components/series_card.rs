//! Series card component - Modern poster design
//!
//! Displays TV series with elegant poster cards and subtle hover effects.
//! Inspired by Apple TV+ and Netflix's content presentation.

use eframe::egui;
use crate::models::Series;
use crate::ui::theme::{Theme, dimensions, spacing, typography, radius};
use crate::ui::image_cache::ImageCache;

/// Actions that can be triggered from a series card.
#[derive(Debug, Clone)]
pub enum SeriesAction {
    /// View episodes for this series
    ViewEpisodes(i32),
}

/// Series card component - Modern poster style
pub struct SeriesCard;

impl SeriesCard {
    /// Renders a modern series card with poster image.
    /// Returns any action that was triggered.
    pub fn show(
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        theme: &Theme,
        series: &Series,
        image_cache: &ImageCache,
        screen_width: f32,
    ) -> Option<SeriesAction> {
        let mut action: Option<SeriesAction> = None;
        let is_mobile = dimensions::is_mobile(screen_width);

        // Card dimensions
        let card_width = dimensions::card_width(screen_width);
        let poster_height = dimensions::poster_height(card_width);
        let total_height = poster_height + 48.0; // Space for title and metadata

        // Load poster image
        if let Some(cover) = &series.cover {
            if !cover.is_empty() {
                image_cache.load(ctx, cover.clone());
            }
        }

        // Allocate card space
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(card_width + spacing::SM, total_height + spacing::SM),
            egui::Sense::click(),
        );

        if response.clicked() {
            action = Some(SeriesAction::ViewEpisodes(series.series_id));
        }

        let is_hovered = response.hovered();
        let poster_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(spacing::XS, 0.0),
            egui::vec2(card_width, poster_height),
        );

        // Shadow on hover - subtle lift effect
        if is_hovered {
            let shadow_rect = poster_rect.expand(3.0);
            ui.painter().rect_filled(
                shadow_rect.translate(egui::vec2(0.0, 4.0)),
                radius::LG,
                theme.elevated_shadow(),
            );
        }

        // Draw poster or placeholder
        if let Some(cover_url) = &series.cover {
            if let Some(texture) = image_cache.get(cover_url) {
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                ui.painter().image(texture.id(), poster_rect, uv, egui::Color32::WHITE);

                // Subtle border
                ui.painter().rect_stroke(
                    poster_rect,
                    radius::LG,
                    egui::Stroke::new(1.0, theme.border_color),
                );
            } else {
                Self::paint_placeholder(ui, theme, poster_rect);
            }
        } else {
            Self::paint_placeholder(ui, theme, poster_rect);
        }

        // Hover overlay with play button
        if is_hovered {
            // Gradient overlay at bottom
            let gradient_height = 100.0;
            let gradient_rect = egui::Rect::from_min_max(
                egui::pos2(poster_rect.min.x, poster_rect.max.y - gradient_height),
                poster_rect.max,
            );
            ui.painter().rect_filled(
                gradient_rect,
                egui::Rounding {
                    nw: 0.0,
                    ne: 0.0,
                    sw: radius::LG,
                    se: radius::LG,
                },
                theme.gradient_overlay(),
            );

            // Play button - centered circle
            let center = poster_rect.center();
            let play_radius = 32.0;

            // Play button background
            ui.painter().circle_filled(
                center,
                play_radius,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180),
            );
            ui.painter().circle_stroke(
                center,
                play_radius,
                egui::Stroke::new(2.0, egui::Color32::WHITE),
            );

            // Play triangle
            let tri_size = 14.0;
            let play_points = vec![
                egui::pos2(center.x - tri_size * 0.35, center.y - tri_size * 0.6),
                egui::pos2(center.x - tri_size * 0.35, center.y + tri_size * 0.6),
                egui::pos2(center.x + tri_size * 0.65, center.y),
            ];
            ui.painter().add(egui::Shape::convex_polygon(
                play_points,
                egui::Color32::WHITE,
                egui::Stroke::NONE,
            ));

            // Genre badge at bottom of poster
            if let Some(genre) = &series.genre {
                let genre_text = truncate_text(genre, 18);
                let galley = ui.painter().layout_no_wrap(
                    genre_text,
                    egui::FontId::proportional(typography::LABEL),
                    egui::Color32::WHITE,
                );
                let badge_width = galley.size().x + spacing::MD * 2.0;
                let badge_rect = egui::Rect::from_min_size(
                    egui::pos2(poster_rect.min.x + spacing::SM, poster_rect.max.y - 32.0),
                    egui::vec2(badge_width, 22.0),
                );
                ui.painter().rect_filled(badge_rect, radius::FULL, theme.badge_bg());
                ui.painter().galley(
                    egui::pos2(badge_rect.min.x + spacing::MD, badge_rect.min.y + 4.0),
                    galley,
                    egui::Color32::WHITE,
                );
            }
        }

        // Title below poster
        let title_y = poster_rect.max.y + spacing::SM;
        let display_name = truncate_text(&series.name, if is_mobile { 18 } else { 22 });
        let title_galley = ui.painter().layout_no_wrap(
            display_name,
            egui::FontId::proportional(if is_mobile { typography::BODY_SM } else { typography::BODY }),
            theme.text_primary,
        );
        ui.painter().galley(
            egui::pos2(poster_rect.min.x, title_y),
            title_galley,
            theme.text_primary,
        );

        // Rating below title
        if let Some(rating) = &series.rating {
            if !rating.is_empty() {
                let rating_text = format!("★ {}", rating);
                let rating_galley = ui.painter().layout_no_wrap(
                    rating_text,
                    egui::FontId::proportional(typography::CAPTION),
                    theme.warning_color,
                );
                ui.painter().galley(
                    egui::pos2(poster_rect.min.x, title_y + 20.0),
                    rating_galley,
                    theme.warning_color,
                );
            }
        }

        action
    }

    /// Paints a placeholder for missing posters
    fn paint_placeholder(ui: &egui::Ui, theme: &Theme, rect: egui::Rect) {
        ui.painter().rect_filled(rect, radius::LG, theme.placeholder_bg());

        let center = rect.center();
        let galley = ui.painter().layout_no_wrap(
            "TV".to_string(),
            egui::FontId::proportional(rect.width() * 0.2),
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
        format!(
            "{}...",
            text.chars().take(max_len.saturating_sub(3)).collect::<String>()
        )
    } else {
        text.to_string()
    }
}
