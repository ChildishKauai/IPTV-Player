//! Resume card - a poster with watch progress, for Continue Watching.
//!
//! Shows how far through a movie or episode the viewer is, or offers the
//! next episode once the current one is finished.

use eframe::egui;
use crate::models::WatchProgress;
use crate::ui::image_cache::ImageCache;
use crate::ui::theme::{Theme, dimensions, spacing, typography, radius};

pub struct ResumeCard;

impl ResumeCard {
    /// Renders the card. Returns true when it was clicked.
    pub fn show(
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        theme: &Theme,
        progress: &WatchProgress,
        image_cache: &ImageCache,
        card_width: f32,
    ) -> bool {
        let poster_height = dimensions::poster_height(card_width);
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(card_width, poster_height + 48.0),
            egui::Sense::click(),
        );
        if response.gained_focus() {
            response.scroll_to_me(None);
        }
        let highlighted = response.hovered() || response.has_focus();
        let poster_rect = egui::Rect::from_min_size(rect.min, egui::vec2(card_width, poster_height));
        let up_next = progress.offers_next_episode();

        if highlighted {
            ui.painter().rect_filled(
                poster_rect.expand(3.0).translate(egui::vec2(0.0, 4.0)),
                radius::LG,
                theme.elevated_shadow(),
            );
        }

        // Poster or placeholder
        let texture = progress.thumbnail.as_ref().filter(|u| !u.is_empty()).and_then(|url| {
            image_cache.load(ctx, url.clone());
            image_cache.get(url)
        });
        if let Some(texture) = texture {
            egui::Image::new(&texture)
                .rounding(radius::LG)
                .paint_at(ui, poster_rect);
        } else {
            ui.painter().rect_filled(poster_rect, radius::LG, theme.placeholder_bg());
            let label = if progress.content_type == "series" { "Series" } else { "Film" };
            ui.painter().text(
                poster_rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(card_width * 0.18),
                theme.placeholder_icon(),
            );
        }
        let (border, width) = if response.has_focus() { (theme.accent_blue, 2.0) } else { (theme.border_color, 1.0) };
        ui.painter().rect_stroke(poster_rect, radius::LG, egui::Stroke::new(width, border));

        // "Up next" badge
        if up_next {
            let text = ui.painter().layout_no_wrap(
                "UP NEXT".to_string(),
                egui::FontId::proportional(typography::LABEL),
                egui::Color32::WHITE,
            );
            let badge = egui::Rect::from_min_size(
                poster_rect.min + egui::vec2(spacing::SM, spacing::SM),
                text.size() + egui::vec2(spacing::MD, spacing::XS * 1.5),
            );
            ui.painter().rect_filled(badge, radius::SM, theme.accent_blue);
            ui.painter().galley(badge.center() - text.size() / 2.0, text, egui::Color32::WHITE);
        }

        // Play button on hover or focus
        if highlighted {
            let center = poster_rect.center();
            let r = (card_width * 0.16).clamp(22.0, 36.0);
            ui.painter().circle_filled(center, r, egui::Color32::from_black_alpha(180));
            ui.painter().circle_stroke(center, r, egui::Stroke::new(2.0, egui::Color32::WHITE));
            let t = r * 0.45;
            ui.painter().add(egui::Shape::convex_polygon(
                vec![
                    egui::pos2(center.x - t * 0.35, center.y - t * 0.6),
                    egui::pos2(center.x - t * 0.35, center.y + t * 0.6),
                    egui::pos2(center.x + t * 0.65, center.y),
                ],
                egui::Color32::WHITE,
                egui::Stroke::NONE,
            ));
        }

        // Progress bar along the bottom edge of the poster
        if progress.has_position() && !up_next {
            let bar = egui::Rect::from_min_max(
                egui::pos2(poster_rect.min.x + spacing::SM, poster_rect.max.y - spacing::SM - 4.0),
                egui::pos2(poster_rect.max.x - spacing::SM, poster_rect.max.y - spacing::SM),
            );
            let pct = (progress.progress_percentage() / 100.0).clamp(0.0, 1.0);
            ui.painter().rect_filled(bar, 2.0, egui::Color32::from_white_alpha(60));
            ui.painter().rect_filled(
                egui::Rect::from_min_size(bar.min, egui::vec2(bar.width() * pct, bar.height())),
                2.0,
                theme.accent_blue,
            );
        }

        // Title and details
        let title = super::single_line(
            ui,
            &progress.content_name,
            egui::FontId::proportional(typography::BODY_SM),
            theme.text_primary,
            card_width,
        );
        let title_y = poster_rect.max.y + spacing::SM;
        ui.painter().galley(egui::pos2(rect.min.x, title_y), title, theme.text_primary);

        let detail = super::single_line(
            ui,
            &Self::detail(progress),
            egui::FontId::proportional(typography::CAPTION),
            theme.text_secondary,
            card_width,
        );
        ui.painter().galley(egui::pos2(rect.min.x, title_y + 20.0), detail, theme.text_secondary);

        response.clicked()
    }

    /// Second line: next episode, time left, or when it was watched.
    fn detail(p: &WatchProgress) -> String {
        if let (true, Some(next)) = (p.offers_next_episode(), &p.next_episode) {
            return format!("S{}:E{} · {}", next.season, next.episode, next.title).trim_end_matches([' ', '·']).to_string();
        }
        let episode = match (p.season, p.episode) {
            (Some(s), Some(e)) => Some(format!("S{}:E{}", s, e)),
            _ => None,
        };
        let when = if p.has_position() {
            let mins_left = ((p.duration_seconds - p.position_seconds) / 60.0).ceil().max(1.0) as i64;
            format!("{} min left", mins_left)
        } else {
            watched_ago(p.last_watched)
        };
        match episode {
            Some(ep) => format!("{} · {}", ep, when),
            None => when,
        }
    }
}

/// "Watched today", "Watched yesterday", "Watched 3 days ago".
fn watched_ago(timestamp: i64) -> String {
    let days = (chrono::Utc::now().timestamp() - timestamp).max(0) / 86_400;
    match days {
        0 => "Watched today".to_string(),
        1 => "Watched yesterday".to_string(),
        d => format!("Watched {} days ago", d),
    }
}
