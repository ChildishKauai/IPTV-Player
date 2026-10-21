//! UI Components module - reusable UI building blocks.
//!
//! This module contains all the visual components used in the IPTV Player:
//! - Login screen
//! - Category sidebar
//! - Content cards (channels, series, movies)
//! - Navigation and pagination
//! - Player settings for audio/subtitle configuration
//! - Discover cards for TV show discovery
//! - Football fixture cards for live sports
//! - Scraper settings for fixture scraping
//! - Common UI utilities

pub mod login;
pub mod sidebar;
pub mod top_nav;
pub mod channel_card;
pub mod series_card;
pub mod movie_card;
pub mod pagination;
pub mod episode_dialog;
pub mod player_settings;
pub mod epg_settings;
pub mod discover_card;
pub mod football_card;
pub mod scraper_settings;

pub use login::LoginScreen;
pub use sidebar::CategorySidebar;
pub use top_nav::TopNavigation;
pub use channel_card::ChannelCard;
pub use series_card::SeriesCard;
pub use movie_card::MovieCard;
pub use pagination::Pagination;
pub use episode_dialog::EpisodeDialog;
pub use football_card::{FootballCard, FootballAction};
pub use player_settings::PlayerSettingsDialog;
pub use epg_settings::EpgSettingsDialog;
pub use discover_card::{DiscoverCard, DiscoverAction};
pub use scraper_settings::ScraperSettingsDialog;

use eframe::egui;
use std::sync::Arc;

/// Lays out `text` on a single line, eliding with "…" if it is wider than
/// `max_width`. Use this instead of character counts so labels follow the
/// card width on every screen size.
pub fn single_line(
    ui: &egui::Ui,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    max_width: f32,
) -> Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::single_section(
        text.to_owned(),
        egui::TextFormat::simple(font, color),
    );
    job.wrap = egui::text::TextWrapping {
        max_width: max_width.max(1.0),
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some('…'),
    };
    ui.painter().layout_job(job)
}
