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
pub mod discover_card;
pub mod football_card;

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
pub use discover_card::{DiscoverCard, DiscoverAction};
