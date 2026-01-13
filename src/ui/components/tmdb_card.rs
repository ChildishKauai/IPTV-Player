use egui::{self, Color32, Rounding, Vec2};
use crate::api::{TmdbItem, TmdbContentType};
use crate::ui::theme::Theme;
use crate::ui::image_cache::ImageCache;

/// Actions that can be triggered from a TMDB card.
#[derive(Debug, Clone)]
pub enum TmdbAction {
    /// Search for this content in the IPTV library.
    SearchInIptv(String, TmdbContentType),
    /// Show details for this item.
    ShowDetails(TmdbItem),
}

/// A card component for displaying TMDB content.
pub struct TmdbCard;

impl TmdbCard {
    /// Show a TMDB item card.
    pub fn show(
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        theme: &Theme,
        item: &TmdbItem,
        image_cache: &ImageCache,
        screen_width: f32,
    ) -> Option<TmdbAction> {
        let mut action = None;
        
        // Responsive card dimensions
        let card_width = if screen_width < 600.0 {
            (screen_width - 48.0) / 2.0 // 2 cards on mobile
        } else if screen_width < 900.0 {
            (screen_width - 80.0) / 3.0 // 3 cards on tablet
        } else if screen_width < 1200.0 {
            (screen_width - 100.0) / 4.0 // 4 cards on small desktop
        } else {
            180.0 // Fixed width on large screens
        }.min(200.0);
        
        let card_height = card_width * 1.5; // Movie poster ratio
        let total_height = card_height + 80.0; // Extra space for text
        
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(card_width, total_height),
            egui::Sense::click(),
        );
        
        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);
            
            // Card background
            let bg_color = if response.hovered() {
                Color32::from_rgb(50, 50, 55)
            } else {
                theme.card_bg
            };
            
            painter.rect_filled(rect, Rounding::same(8.0), bg_color);
            
            // Poster area
            let poster_rect = egui::Rect::from_min_size(
                rect.min,
                Vec2::new(card_width, card_height),
            );
            
            // Draw poster image or placeholder
            if let Some(poster_url) = &item.poster_url {
                if let Some(texture) = image_cache.get(poster_url) {
                    painter.image(
                        texture.id(),
                        poster_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    // Loading placeholder
                    painter.rect_filled(poster_rect, Rounding::same(8.0), Color32::from_rgb(40, 40, 45));
                    painter.text(
                        poster_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "⏳",
                        egui::FontId::proportional(24.0),
                        theme.text_secondary,
                    );
                }
            } else {
                // No poster placeholder
                painter.rect_filled(poster_rect, Rounding::same(8.0), Color32::from_rgb(40, 40, 45));
                let icon = match item.content_type {
                    TmdbContentType::Movie => "🎬",
                    TmdbContentType::TvShow => "📺",
                };
                painter.text(
                    poster_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    icon,
                    egui::FontId::proportional(32.0),
                    theme.text_secondary,
                );
            }
            
            // Rating badge
            if item.vote_average > 0.0 {
                let badge_size = Vec2::new(36.0, 20.0);
                let badge_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.max.x - badge_size.x - 4.0, rect.min.y + 4.0),
                    badge_size,
                );
                
                let rating_color = if item.vote_average >= 7.0 {
                    Color32::from_rgb(46, 204, 113) // Green
                } else if item.vote_average >= 5.0 {
                    Color32::from_rgb(241, 196, 15) // Yellow
                } else {
                    Color32::from_rgb(231, 76, 60) // Red
                };
                
                painter.rect_filled(badge_rect, Rounding::same(4.0), rating_color);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{:.1}", item.vote_average),
                    egui::FontId::proportional(11.0),
                    Color32::WHITE,
                );
            }
            
            // Content type badge
            let type_badge_text = match item.content_type {
                TmdbContentType::Movie => "MOVIE",
                TmdbContentType::TvShow => "TV",
            };
            let type_badge_color = match item.content_type {
                TmdbContentType::Movie => Color32::from_rgb(155, 89, 182), // Purple
                TmdbContentType::TvShow => Color32::from_rgb(52, 152, 219), // Blue
            };
            
            let type_badge_size = Vec2::new(if item.content_type == TmdbContentType::Movie { 44.0 } else { 24.0 }, 16.0);
            let type_badge_rect = egui::Rect::from_min_size(
                egui::pos2(rect.min.x + 4.0, rect.min.y + 4.0),
                type_badge_size,
            );
            
            painter.rect_filled(type_badge_rect, Rounding::same(3.0), type_badge_color);
            painter.text(
                type_badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                type_badge_text,
                egui::FontId::proportional(9.0),
                Color32::WHITE,
            );
            
            // Title (below poster)
            let title_rect = egui::Rect::from_min_size(
                egui::pos2(rect.min.x + 4.0, poster_rect.max.y + 4.0),
                Vec2::new(card_width - 8.0, 36.0),
            );
            
            // Truncate title if too long
            let max_chars = (card_width / 7.0) as usize;
            let display_title = if item.title.len() > max_chars * 2 {
                format!("{}...", &item.title[..max_chars * 2 - 3])
            } else {
                item.title.clone()
            };
            
            painter.text(
                title_rect.left_top(),
                egui::Align2::LEFT_TOP,
                display_title,
                egui::FontId::proportional(12.0),
                theme.text_primary,
            );
            
            // Year
            if let Some(year) = item.year() {
                painter.text(
                    egui::pos2(rect.min.x + 4.0, poster_rect.max.y + 40.0),
                    egui::Align2::LEFT_TOP,
                    year,
                    egui::FontId::proportional(11.0),
                    theme.text_secondary,
                );
            }
            
            // Search button
            let button_rect = egui::Rect::from_min_size(
                egui::pos2(rect.min.x + 4.0, rect.max.y - 24.0),
                Vec2::new(card_width - 8.0, 20.0),
            );
            
            let button_hovered = response.hovered() && 
                ui.input(|i| i.pointer.hover_pos().map(|p| button_rect.contains(p)).unwrap_or(false));
            
            let button_color = if button_hovered {
                theme.accent_blue
            } else {
                Color32::from_rgb(60, 60, 65)
            };
            
            painter.rect_filled(button_rect, Rounding::same(4.0), button_color);
            painter.text(
                button_rect.center(),
                egui::Align2::CENTER_CENTER,
                "🔍 Find in IPTV",
                egui::FontId::proportional(10.0),
                Color32::WHITE,
            );
            
            // Handle click
            if response.clicked() {
                action = Some(TmdbAction::SearchInIptv(item.search_query(), item.content_type));
            }
        }
        
        action
    }
}

/// A horizontal scrollable row of TMDB items.
pub struct TmdbRow;

impl TmdbRow {
    /// Show a horizontal row of TMDB items with a category header.
    pub fn show(
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        theme: &Theme,
        title: &str,
        items: &[TmdbItem],
        image_cache: &ImageCache,
        is_loading: bool,
    ) -> Option<TmdbAction> {
        let mut action = None;
        
        // Category header
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(title)
                .size(18.0)
                .color(theme.text_primary)
                .strong());
            
            if is_loading {
                ui.spinner();
            }
        });
        
        ui.add_space(8.0);
        
        // Horizontal scroll area for items
        egui::ScrollArea::horizontal()
            .id_salt(format!("tmdb_row_{}", title))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    
                    for item in items.iter().take(20) {
                        // Request image loading
                        if let Some(poster_url) = &item.poster_url {
                            image_cache.load(ctx, poster_url.clone());
                        }
                        
                        if let Some(a) = TmdbCard::show(ui, ctx, theme, item, image_cache, 1200.0) {
                            action = Some(a);
                        }
                    }
                    
                    if items.is_empty() && !is_loading {
                        ui.label(egui::RichText::new("No content available")
                            .color(theme.text_secondary));
                    }
                });
            });
        
        ui.add_space(16.0);
        
        action
    }
}
