//! Series card component for displaying TV series (Netflix-style).

use eframe::egui;
use crate::models::Series;
use crate::ui::theme::{Theme, dimensions};
use crate::ui::image_cache::ImageCache;

/// Actions that can be triggered from a series card.
#[derive(Debug, Clone)]
pub enum SeriesAction {
    /// View episodes for this series
    ViewEpisodes(i32),
}

/// Series card component (Netflix poster style).
pub struct SeriesCard;

impl SeriesCard {
    /// Renders a Netflix-style series card (poster with hover overlay).
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
        
        // Netflix-style poster dimensions
        let card_width = dimensions::card_width(screen_width);
        let card_height = dimensions::poster_height(card_width);
        
        // Load poster image if available
        if let Some(cover) = &series.cover {
            if !cover.is_empty() {
                image_cache.load(ctx, cover.clone());
            }
        }
        
        // Allocate space for card with some padding
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(card_width + 8.0, card_height + 50.0),
            egui::Sense::click(),
        );
        
        if response.clicked() {
            action = Some(SeriesAction::ViewEpisodes(series.series_id));
        }
        
        let is_hovered = response.hovered();
        let poster_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(4.0, 0.0),
            egui::vec2(card_width, card_height),
        );
        
        // Draw shadow on hover
        if is_hovered {
            let shadow_rect = poster_rect.expand(3.0);
            ui.painter().rect_filled(shadow_rect, 6.0, theme.card_shadow());
        }
        
        // Draw poster image or placeholder
        if let Some(cover_url) = &series.cover {
            if let Some(texture) = image_cache.get(cover_url) {
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                ui.painter().image(
                    texture.id(),
                    poster_rect,
                    uv,
                    egui::Color32::WHITE,
                );
                ui.painter().rect_stroke(poster_rect, 6.0, egui::Stroke::new(1.0, theme.bg_color));
            } else {
                Self::paint_placeholder(ui, theme, poster_rect);
            }
        } else {
            Self::paint_placeholder(ui, theme, poster_rect);
        }
        
        // Hover overlay with gradient and play button
        if is_hovered {
            // Dark gradient at bottom
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
            
            // Genre pill at bottom of poster
            if let Some(genre) = &series.genre {
                let genre_text = truncate_text(genre, 15);
                let galley = ui.painter().layout_no_wrap(
                    genre_text,
                    egui::FontId::proportional(10.0),
                    egui::Color32::WHITE,
                );
                let pill_width = galley.size().x + 12.0;
                let pill_rect = egui::Rect::from_min_size(
                    egui::pos2(poster_rect.min.x + 8.0, poster_rect.max.y - 30.0),
                    egui::vec2(pill_width, 20.0),
                );
                ui.painter().rect_filled(pill_rect, 10.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40));
                ui.painter().galley(
                    egui::pos2(pill_rect.min.x + 6.0, pill_rect.min.y + 3.0),
                    galley,
                    egui::Color32::WHITE,
                );
            }
        }
        
        // Title below poster
        let title_rect = egui::Rect::from_min_size(
            egui::pos2(poster_rect.min.x, poster_rect.max.y + 6.0),
            egui::vec2(card_width, 40.0),
        );
        
        let display_name = truncate_text(&series.name, if is_mobile { 18 } else { 22 });
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
        if let Some(rating) = &series.rating {
            if !rating.is_empty() {
                let rating_text = format!("★ {}", rating);
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
