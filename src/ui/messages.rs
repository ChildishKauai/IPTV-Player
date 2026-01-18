//! Application messages for async communication between threads.
//!
//! This module defines the message types used for communication between
//! the UI thread and background worker threads.

use crate::models::{Category, Channel, Series};

/// Messages sent from background threads to the main UI thread.
///
/// These messages represent the results of async operations like
/// API calls and data fetching.
#[derive(Debug)]
pub enum AppMessage {
    /// Successfully connected and loaded live TV data
    Connected(Vec<Category>, Vec<Channel>),
    
    /// An error occurred during an operation
    Error(String),
    
    /// Series data loaded successfully
    SeriesLoaded(Vec<Category>, Vec<Series>),
    
    /// Movies data loaded successfully
    MoviesLoaded(Vec<Category>, Vec<serde_json::Value>),
    
    /// Scraper started
    ScraperStarted,
    
    /// Scraper completed successfully
    ScraperCompleted(String),
    
    /// Scraper failed with error
    ScraperFailed(String),
}

/// Content type currently being displayed in the main view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContentType {
    #[default]
    LiveTV,
    Series,
    Movies,
    ContinueWatching,
    Favorites,
    Discover,
    FootballFixtures,
}

impl ContentType {
    /// Returns the display title for this content type
    pub fn title(&self) -> &'static str {
        match self {
            ContentType::LiveTV => "📺 Live Channels",
            ContentType::Series => "📚 Series",
            ContentType::Movies => "🎬 Movies",
            ContentType::ContinueWatching => "▶️ Continue Watching",
            ContentType::Favorites => "⭐ Favorite Channels",
            ContentType::Discover => "🔥 Discover",
            ContentType::FootballFixtures => "⚽ Football Fixtures",
        }
    }
    
    /// Returns the empty state message
    pub fn empty_message(&self) -> &'static str {
        match self {
            ContentType::LiveTV => "No channels found",
            ContentType::Series => "No series found",
            ContentType::Movies => "No movies found",
            ContentType::ContinueWatching => "No recent viewing history",
            ContentType::Favorites => "No favorites yet",
            ContentType::Discover => "Configure TMDB API key in settings",
            ContentType::FootballFixtures => "No upcoming fixtures found",
        }
    }
}
