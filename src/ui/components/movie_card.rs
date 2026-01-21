//! Movie card component - Modern poster design
//!
//! Displays movies with elegant poster cards and subtle hover effects.
//! Consistent with series card styling for a unified experience.

use eframe::egui;
use crate::ui::theme::{Theme, dimensions, spacing, typography, radius};
use crate::ui::image_cache::ImageCache;

/// Actions that can be triggered from a movie card.
#[derive(Debug, Clone)]
pub enum MovieAction {
    /// Play the movie
    Play {
        stream_id: i64,
        name: String,
        container_extension: String,
        thumbnail: Option<String>,
    },
}

/// Movie card component - Modern poster style
pub struct MovieCard;

impl MovieCard {
    /// Renders a modern movie card with poster image.
    /// Returns any action that was triggered.
    pub fn show(
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        theme: &Theme,
        movie: &serde_json::Value,
        image_cache: &ImageCache,
        screen_width: f32,
    ) -> Option<MovieAction> {
        let mut action: Option<MovieAction> = None;
        let is_mobile = dimensions::is_mobile(screen_width);

        // Card dimensions
        let card_width = dimensions::card_width(screen_width);
        let poster_height = dimensions::poster_height(card_width);
        let total_height = poster_height + 48.0;

        // Extract movie data
        let movie_name = movie.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
        let stream_id = movie.get("stream_id").and_then(|v| v.as_i64()).unwrap_or(0);
        let container_extension = movie
            .get("container_extension")
            .and_then(|v| v.as_str())
            .unwrap_or("mp4");
        let cover = movie.get("stream_icon").and_then(|v| v.as_str());
        let rating = movie.get("rating").and_then(|v| v.as_f64());

        // Load cover image
        if let Some(cover_url) = cover {
            if !cover_url.is_empty() {
                image_cache.load(ctx, cover_url.to_string());
            }
        }

        // Allocate card space
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(card_width + spacing::SM, total_height + spacing::SM),
            egui::Sense::click(),
        );

        if response.clicked() {
            action = Some(MovieAction::Play {
                stream_id,
                name: movie_name.to_string(),
                container_extension: container_extension.to_string(),
                thumbnail: cover.map(|s| s.to_string()),
            });
        }

        let is_hovered = response.hovered();
        let has_focus = response.has_focus();
        let poster_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(spacing::XS, 0.0),
            egui::vec2(card_width, poster_height),
        );

        // Shadow on hover or focus (for gamepad navigation) - subtle lift effect
        if is_hovered || has_focus {
            let shadow_rect = poster_rect.expand(3.0);
            ui.painter().rect_filled(
                shadow_rect.translate(egui::vec2(0.0, 4.0)),
                radius::LG,
                theme.elevated_shadow(),
            );
        }

        // Draw poster or placeholder
        if let Some(cover_url) = cover {
            if let Some(texture) = image_cache.get(cover_url) {
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                ui.painter().image(texture.id(), poster_rect, uv, egui::Color32::WHITE);

                // Border - accent color on focus for gamepad navigation
                let border_color = if has_focus { theme.accent_blue } else { theme.border_color };
                let border_width = if has_focus { 2.0 } else { 1.0 };
                ui.painter().rect_stroke(
                    poster_rect,
                    radius::LG,
                    egui::Stroke::new(border_width, border_color),
                );
            } else {
                Self::paint_placeholder(ui, theme, poster_rect);
            }
        } else {
            Self::paint_placeholder(ui, theme, poster_rect);
        }

        // Hover/focus overlay with play button (for gamepad navigation)
        if is_hovered || has_focus {
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
        }

        // Title below poster
        let title_y = poster_rect.max.y + spacing::SM;
        let display_name = truncate_text(movie_name, if is_mobile { 18 } else { 22 });
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
        if let Some(rating_val) = rating {
            if rating_val > 0.0 {
                let rating_text = format!("★ {:.1}", rating_val);
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
            "Film".to_string(),
            egui::FontId::proportional(rect.width() * 0.18),
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
