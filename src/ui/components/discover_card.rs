use egui::{self, Color32, Vec2};
use crate::api::{DiscoverItem, DiscoverContentType};
use crate::ui::theme::{Theme, dimensions};
use crate::ui::image_cache::ImageCache;

/// Actions that can be triggered from a discover card.
#[derive(Debug, Clone)]
pub enum DiscoverAction {
    /// Search for this content in the IPTV library.
    SearchInIptv(String),
}

/// A card component for displaying discover content (Netflix-style).
pub struct DiscoverCard;

impl DiscoverCard {
    /// Show a Netflix-style discover item card.
    pub fn show(
        ui: &mut egui::Ui,
        theme: &Theme,
        item: &DiscoverItem,
        image_cache: &ImageCache,
        screen_width: f32,
    ) -> Option<DiscoverAction> {
        let mut action = None;
        
        // Netflix-style poster dimensions
        let card_width = dimensions::card_width(screen_width);
        let card_height = dimensions::poster_height(card_width);
        let total_height = card_height + 55.0; // Space for text below
        
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(card_width + 8.0, total_height),
            egui::Sense::click(),
        );
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);
            
            // Poster area
            let poster_rect = egui::Rect::from_min_size(
                rect.min + Vec2::new(4.0, 0.0),
                Vec2::new(card_width, card_height),
            );
            
            let is_hovered = response.hovered();
            
            // Shadow on hover
            if is_hovered {
                let shadow_rect = poster_rect.expand(3.0);
                painter.rect_filled(shadow_rect, 6.0, theme.card_shadow());
            }
            
            // Draw poster image or placeholder
            if let Some(poster_url) = &item.poster_url {
                if let Some(texture) = image_cache.get(poster_url) {
                    painter.image(
                        texture.id(),
                        poster_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    painter.rect_stroke(poster_rect, 6.0, egui::Stroke::new(1.0, theme.bg_color));
                } else {
                    // Loading placeholder
                    painter.rect_filled(poster_rect, 6.0, theme.placeholder_bg());
                    painter.text(
                        poster_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "⏳",
                        egui::FontId::proportional(card_width * 0.2),
                        theme.placeholder_icon(),
                    );
                }
            } else {
                // No poster placeholder
                painter.rect_filled(poster_rect, 6.0, theme.placeholder_bg());
                let icon = match item.content_type {
                    DiscoverContentType::TvShow => "📺",
                    DiscoverContentType::Movie => "🎬",
                };
                painter.text(
                    poster_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    icon,
                    egui::FontId::proportional(card_width * 0.25),
                    theme.placeholder_icon(),
                );
            }
            
            // Hover overlay with play button
            if is_hovered {
                // Dark gradient at bottom
                let gradient_rect = egui::Rect::from_min_max(
                    egui::pos2(poster_rect.min.x, poster_rect.max.y - 70.0),
                    poster_rect.max,
                );
                painter.rect_filled(
                    gradient_rect,
                    egui::Rounding { nw: 0.0, ne: 0.0, sw: 6.0, se: 6.0 },
                    Color32::from_rgba_unmultiplied(0, 0, 0, 180),
                );
                
                // Search button inside poster on hover
                let button_rect = egui::Rect::from_min_size(
                    egui::pos2(poster_rect.min.x + 10.0, poster_rect.max.y - 32.0),
                    Vec2::new(card_width - 20.0, 24.0),
                );
                painter.rect_filled(button_rect, 4.0, theme.accent_blue);
                painter.text(
                    button_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🔍 Find in IPTV",
                    egui::FontId::proportional(11.0),
                    Color32::WHITE,
                );
                
                // Handle click on button
                if response.clicked() {
                    action = Some(DiscoverAction::SearchInIptv(item.title.clone()));
                }
            }
            
            // Content type badge (top left)
            let (type_badge_text, type_badge_color) = match item.content_type {
                DiscoverContentType::TvShow => ("TV", Color32::from_rgb(52, 152, 219)),
                DiscoverContentType::Movie => ("FILM", Color32::from_rgb(155, 89, 182)),
            };
            
            let type_badge_size = Vec2::new(32.0, 18.0);
            let type_badge_rect = egui::Rect::from_min_size(
                egui::pos2(poster_rect.min.x + 6.0, poster_rect.min.y + 6.0),
                type_badge_size,
            );
            
            painter.rect_filled(type_badge_rect, 3.0, type_badge_color);
            painter.text(
                type_badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                type_badge_text,
                egui::FontId::proportional(10.0),
                Color32::WHITE,
            );
            
            // Rating badge (top right)
            if let Some(rating) = item.rating {
                if rating > 0.0 {
                    let badge_size = Vec2::new(34.0, 18.0);
                    let badge_rect = egui::Rect::from_min_size(
                        egui::pos2(poster_rect.max.x - badge_size.x - 6.0, poster_rect.min.y + 6.0),
                        badge_size,
                    );
                    
                    let rating_color = if rating >= 7.0 {
                        Color32::from_rgb(46, 204, 113)
                    } else if rating >= 5.0 {
                        Color32::from_rgb(241, 196, 15)
                    } else {
                        Color32::from_rgb(231, 76, 60)
                    };
                    
                    painter.rect_filled(badge_rect, 3.0, rating_color);
                    painter.text(
                        badge_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("{:.1}", rating),
                        egui::FontId::proportional(10.0),
                        Color32::WHITE,
                    );
                }
            }
            
            // Title below poster
            let title_y = poster_rect.max.y + 8.0;
            let is_mobile = dimensions::is_mobile(screen_width);
            let max_chars = if is_mobile { 18 } else { 22 };
            let display_title = if item.title.chars().count() > max_chars {
                format!("{}...", item.title.chars().take(max_chars - 3).collect::<String>())
            } else {
                item.title.clone()
            };
            
            painter.text(
                egui::pos2(poster_rect.min.x, title_y),
                egui::Align2::LEFT_TOP,
                display_title,
                egui::FontId::proportional(if is_mobile { 12.0 } else { 13.0 }),
                theme.text_primary,
            );
            
            // Year
            if let Some(year) = &item.year {
                painter.text(
                    egui::pos2(poster_rect.min.x, title_y + 20.0),
                    egui::Align2::LEFT_TOP,
                    year,
                    egui::FontId::proportional(11.0),
                    theme.text_secondary,
                );
            }
        }
        
        action
    }
}
