//! Movie card component for displaying movies (Netflix-style).

use eframe::egui;
use crate::ui::theme::{Theme, dimensions};
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

/// Movie card component (Netflix poster style).
pub struct MovieCard;

impl MovieCard {
    /// Renders a Netflix-style movie card (poster with hover overlay).
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
        
        // Netflix-style poster dimensions
        let card_width = dimensions::card_width(screen_width);
        let card_height = dimensions::poster_height(card_width);
        
        let movie_name = movie.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
        let stream_id = movie.get("stream_id").and_then(|v| v.as_i64()).unwrap_or(0);
        let container_extension = movie.get("container_extension").and_then(|v| v.as_str()).unwrap_or("mp4");
        let cover = movie.get("stream_icon").and_then(|v| v.as_str());
        let rating = movie.get("rating").and_then(|v| v.as_f64());
        
        // Load cover image if available
        if let Some(cover_url) = cover {
            if !cover_url.is_empty() {
                image_cache.load(ctx, cover_url.to_string());
            }
        }
        
        // Allocate space for card with some padding
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(card_width + 8.0, card_height + 50.0),
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
        let poster_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(4.0, 0.0),
            egui::vec2(card_width, card_height),
        );
        
        // Draw poster with shadow on hover
        if is_hovered {
            // Glow/shadow effect on hover
            let shadow_rect = poster_rect.expand(3.0);
            ui.painter().rect_filled(shadow_rect, 6.0, theme.card_shadow());
        }
        
        // Draw poster image or placeholder
        if let Some(cover_url) = cover {
            if let Some(texture) = image_cache.get(cover_url) {
                // Draw poster image
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                ui.painter().image(
                    texture.id(),
                    poster_rect,
                    uv,
                    egui::Color32::WHITE,
                );
                // Rounded corners overlay
                ui.painter().rect_stroke(poster_rect, 6.0, egui::Stroke::new(1.0, theme.bg_color));
            } else {
                Self::paint_placeholder(ui, theme, poster_rect);
            }
        } else {
            Self::paint_placeholder(ui, theme, poster_rect);
        }
        
        // Hover overlay with gradient
        if is_hovered {
            // Dark gradient at bottom for text
            let gradient_rect = egui::Rect::from_min_max(
                egui::pos2(poster_rect.min.x, poster_rect.max.y - 80.0),
                poster_rect.max,
            );
            ui.painter().rect_filled(
                gradient_rect,
                egui::Rounding { nw: 0.0, ne: 0.0, sw: 6.0, se: 6.0 },
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180),
            );
            
            // Play button circle in center
            let center = poster_rect.center();
            ui.painter().circle_filled(center, 28.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 160));
            ui.painter().circle_stroke(center, 28.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
            
            // Play triangle
            let play_size = 14.0;
            let play_points = vec![
                egui::pos2(center.x - play_size * 0.4, center.y - play_size * 0.6),
                egui::pos2(center.x - play_size * 0.4, center.y + play_size * 0.6),
                egui::pos2(center.x + play_size * 0.6, center.y),
            ];
            ui.painter().add(egui::Shape::convex_polygon(
                play_points,
                egui::Color32::WHITE,
                egui::Stroke::NONE,
            ));
        }
        
        // Title below poster
        let title_rect = egui::Rect::from_min_size(
            egui::pos2(poster_rect.min.x, poster_rect.max.y + 6.0),
            egui::vec2(card_width, 40.0),
        );
        
        let display_name = truncate_text(movie_name, if is_mobile { 18 } else { 22 });
        let galley = ui.painter().layout_no_wrap(
            display_name,
            egui::FontId::proportional(if is_mobile { 12.0 } else { 13.0 }),
            theme.text_primary,
        );
        ui.painter().galley(
            egui::pos2(title_rect.min.x, title_rect.min.y),
            galley,
            theme.text_primary,
        );
        
        // Rating badge
        if let Some(rating_val) = rating {
            if rating_val > 0.0 {
                let rating_text = format!("★ {:.1}", rating_val);
                let galley = ui.painter().layout_no_wrap(
                    rating_text,
                    egui::FontId::proportional(if is_mobile { 10.0 } else { 11.0 }),
                    theme.warning_color,
                );
                ui.painter().galley(
                    egui::pos2(title_rect.min.x, title_rect.min.y + 18.0),
                    galley,
                    theme.warning_color,
                );
            }
        }
        
        action
    }
    
    /// Paints a placeholder for missing posters.
    fn paint_placeholder(ui: &egui::Ui, theme: &Theme, rect: egui::Rect) {
        ui.painter().rect_filled(rect, 6.0, theme.placeholder_bg());
        
        // Movie icon in center
        let center = rect.center();
        let galley = ui.painter().layout_no_wrap(
            "🎬".to_string(),
            egui::FontId::proportional(rect.width() * 0.25),
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
        format!("{}...", text.chars().take(max_len.saturating_sub(3)).collect::<String>())
    } else {
        text.to_string()
    }
}
