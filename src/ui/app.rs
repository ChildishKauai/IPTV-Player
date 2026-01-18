//! Main application state and logic for the IPTV Player.
//!
//! This module contains the core application struct that manages:
//! - User authentication and connection state
//! - Content data (channels, series, movies)
//! - UI state (current view, pagination, search)
//! - Background task communication

use eframe::egui;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::api::XtreamClient;
use crate::api::DiscoverCategory;
use crate::api::{FootballCache, FootballCategory};
use crate::api::ScraperManager;
use crate::models::*;
use super::theme::{Theme, dimensions};
use super::messages::{AppMessage, ContentType};
use super::image_cache::ImageCache;
use super::components::*;

/// Main application struct for the IPTV Player.
///
/// Implements `eframe::App` to integrate with the egui framework.
pub struct IPTVPlayerApp {
    // ─────────────────────────────────────────────────────────────────────
    // Configuration
    // ─────────────────────────────────────────────────────────────────────
    
    /// User configuration (saved credentials, favorites, etc.)
    config: Config,
    
    // ─────────────────────────────────────────────────────────────────────
    // Connection State
    // ─────────────────────────────────────────────────────────────────────
    
    /// Server URL for Xtream API
    server_url: String,
    /// Username for authentication
    username: String,
    /// Password for authentication
    password: String,
    /// Whether the user is connected
    connected: bool,
    /// Whether a connection attempt is in progress
    connecting: bool,
    /// Current error message to display
    error_message: Option<String>,
    
    // ─────────────────────────────────────────────────────────────────────
    // Content Data
    // ─────────────────────────────────────────────────────────────────────
    
    /// Live TV categories
    live_categories: Vec<Category>,
    /// Series categories
    series_categories: Vec<Category>,
    /// Movie categories
    movie_categories: Vec<Category>,
    
    /// All live channels
    all_channels: Vec<Channel>,
    /// Filtered live channels (based on search/category)
    filtered_channels: Vec<Channel>,
    
    /// All series
    all_series: Vec<Series>,
    /// Filtered series
    filtered_series: Vec<Series>,
    
    /// All movies (as raw JSON values)
    all_movies: Vec<serde_json::Value>,
    /// Filtered movies
    filtered_movies: Vec<serde_json::Value>,
    
    // ─────────────────────────────────────────────────────────────────────
    // UI State
    // ─────────────────────────────────────────────────────────────────────
    
    /// Current content type being displayed
    current_content: ContentType,
    /// Currently selected category filter
    selected_category: Option<String>,
    /// Search query for filtering content
    search_query: String,
    /// Search query for filtering categories
    category_search: String,
    /// Episode dialog state (holds cached data for fast rendering)
    episode_dialog_state: Option<episode_dialog::EpisodeDialogState>,
    /// Whether the player settings dialog is open
    show_player_settings: bool,
    /// Temporary player settings for the dialog (to allow cancel)
    temp_player_settings: Option<crate::models::PlayerSettings>,
    /// Whether dark mode is enabled
    dark_mode: bool,
    /// Whether sidebar is visible (for mobile view)
    sidebar_visible: bool,
    /// Cached screen width for responsive layout
    screen_width: f32,
    /// Cached screen height for Steam Deck detection
    screen_height: f32,
    
    // ─────────────────────────────────────────────────────────────────────
    // Pagination
    // ─────────────────────────────────────────────────────────────────────
    
    /// Number of items per page
    page_size: usize,
    /// Current page index (0-based)
    current_page: usize,
    
    // ─────────────────────────────────────────────────────────────────────
    // Background Communication
    // ─────────────────────────────────────────────────────────────────────
    
    /// Receiver for messages from background threads
    rx: Option<Receiver<AppMessage>>,
    /// Sender for messages to background threads
    tx: Option<Sender<AppMessage>>,
    
    // ─────────────────────────────────────────────────────────────────────
    // Media
    // ─────────────────────────────────────────────────────────────────────
    
    /// Current stream URL being played
    #[allow(dead_code)]
    current_stream_url: Option<String>,
    /// Image cache for channel/series/movie artwork
    image_cache: ImageCache,
    /// EPG cache for program guide data
    epg_cache: super::epg_cache::EpgCache,
    /// Discover cache for TV show discovery (TVMaze - free, no API key needed)
    discover_cache: crate::api::DiscoverCache,
    /// Currently selected discover category
    discover_category: crate::api::DiscoverCategory,
    /// Football fixtures cache
    football_cache: FootballCache,
    /// Currently selected football category
    football_category: FootballCategory,
    /// Watch history for continue watching feature
    watch_history: crate::models::WatchHistory,
    /// Football fixtures scraper manager
    scraper_manager: ScraperManager,
    /// Whether the scraper settings dialog is open
    show_scraper_settings: bool,
}

impl IPTVPlayerApp {
    /// Creates a new application instance.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load().unwrap_or_default();
        let (tx, rx) = channel();
        
        let mut app = Self {
            username: config.username.clone(),
            password: config.password.clone(),
            server_url: config.server_url.clone(),
            connected: false,
            connecting: false,
            error_message: None,
            live_categories: Vec::new(),
            series_categories: Vec::new(),
            movie_categories: Vec::new(),
            all_channels: Vec::new(),
            filtered_channels: Vec::new(),
            all_series: Vec::new(),
            filtered_series: Vec::new(),
            all_movies: Vec::new(),
            filtered_movies: Vec::new(),
            current_content: ContentType::LiveTV,
            selected_category: None,
            search_query: String::new(),
            category_search: String::new(),
            episode_dialog_state: None,
            show_player_settings: false,
            temp_player_settings: None,
            dark_mode: true,
            sidebar_visible: true,  // Visible by default on desktop
            screen_width: 1280.0,   // Default, will be updated each frame
            screen_height: 800.0,   // Default Steam Deck height, will be updated each frame
            page_size: dimensions::DEFAULT_PAGE_SIZE,
            current_page: 0,
            rx: Some(rx),
            tx: Some(tx),
            current_stream_url: None,
            config,
            image_cache: ImageCache::new(),
            epg_cache: super::epg_cache::EpgCache::new(),
            discover_cache: crate::api::DiscoverCache::new(),
            discover_category: crate::api::DiscoverCategory::NewMovies2026,
            football_cache: FootballCache::new(),
            football_category: FootballCategory::Today,
            watch_history: crate::models::WatchHistory::load(),
            scraper_manager: ScraperManager::new(),
            show_scraper_settings: false,
        };
        
        // Auto-login if credentials are saved
        if app.config.auto_login && !app.username.is_empty() && !app.password.is_empty() && !app.server_url.is_empty() {
            app.connect();
        }
        
        app
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // Connection Methods
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Initiates a connection to the IPTV server.
    fn connect(&mut self) {
        self.connecting = true;
        self.error_message = None;
        
        // Set EPG cache credentials
        self.epg_cache.set_credentials(
            self.server_url.clone(),
            self.username.clone(),
            self.password.clone(),
        );
        
        let server_url = self.server_url.clone();
        let username = self.username.clone();
        let password = self.password.clone();
        let tx = self.tx.as_ref().unwrap().clone();
        
        thread::spawn(move || {
            use crate::models::ContentCache;
            
            // Validate inputs
            if server_url.is_empty() || username.is_empty() || password.is_empty() {
                let _ = tx.send(AppMessage::Error("Please fill in all fields".to_string()));
                return;
            }
            
            let cache_key = format!("channels_{}_{}", &username, &server_url);
            let cat_cache_key = format!("categories_{}_{}", &username, &server_url);
            
            // Try to load from cache first (24 hour cache)
            if let (Some(categories), Some(channels)) = (
                ContentCache::load::<Vec<Category>>(&cat_cache_key),
                ContentCache::load::<Vec<Channel>>(&cache_key)
            ) {
                let _ = tx.send(AppMessage::Connected(categories, channels));
                return;
            }
            
            let client = XtreamClient::new(server_url, username, password);
            
            match client.authenticate() {
                Ok(true) => {
                    match (client.get_live_categories(), client.get_live_streams()) {
                        (Ok(categories), Ok(channels)) => {
                            // Cache for 24 hours (86400 seconds)
                            let _ = ContentCache::save(&cat_cache_key, &categories, 86400);
                            let _ = ContentCache::save(&cache_key, &channels, 86400);
                            let _ = tx.send(AppMessage::Connected(categories, channels));
                        }
                        (Err(e), _) => {
                            let _ = tx.send(AppMessage::Error(format!("Failed to fetch categories: {}", e)));
                        }
                        (_, Err(e)) => {
                            let _ = tx.send(AppMessage::Error(format!("Failed to fetch channels: {}", e)));
                        }
                    }
                }
                Ok(false) => {
                    let _ = tx.send(AppMessage::Error("Authentication failed: Invalid credentials or server response".to_string()));
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::Error(format!("Connection error: {}. Check your server URL and network connection.", e)));
                }
            }
        });
    }
    
    /// Disconnects from the server and clears all data.
    fn disconnect(&mut self) {
        self.connected = false;
        self.live_categories.clear();
        self.series_categories.clear();
        self.movie_categories.clear();
        self.all_channels.clear();
        self.filtered_channels.clear();
        self.all_series.clear();
        self.filtered_series.clear();
        self.all_movies.clear();
        self.filtered_movies.clear();
        // Clear EPG cache on disconnect
        self.epg_cache.clear();
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // Data Loading Methods
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Loads series data in the background.
    fn load_series(&mut self) {
        let server_url = self.server_url.clone();
        let username = self.username.clone();
        let password = self.password.clone();
        let tx = self.tx.as_ref().unwrap().clone();
        
        thread::spawn(move || {
            use crate::models::ContentCache;
            
            let cache_key = format!("series_{}_{}", &username, &server_url);
            let cat_cache_key = format!("series_cat_{}_{}", &username, &server_url);
            
            // Try cache first (24 hour cache)
            if let (Some(categories), Some(series)) = (
                ContentCache::load::<Vec<Category>>(&cat_cache_key),
                ContentCache::load::<Vec<Series>>(&cache_key)
            ) {
                let _ = tx.send(AppMessage::SeriesLoaded(categories, series));
                return;
            }
            
            let client = XtreamClient::new(server_url, username, password);
            
            match (client.get_series_categories(), client.get_series()) {
                (Ok(categories), Ok(series)) => {
                    // Cache for 24 hours
                    let _ = ContentCache::save(&cat_cache_key, &categories, 86400);
                    let _ = ContentCache::save(&cache_key, &series, 86400);
                    let _ = tx.send(AppMessage::SeriesLoaded(categories, series));
                }
                (Err(e), _) => {
                    let _ = tx.send(AppMessage::Error(format!("Failed to fetch series categories: {}", e)));
                }
                (_, Err(e)) => {
                    let _ = tx.send(AppMessage::Error(format!("Failed to fetch series: {}", e)));
                }
            }
        });
    }
    
    /// Loads movies data in the background.
    fn load_movies(&mut self) {
        let server_url = self.server_url.clone();
        let username = self.username.clone();
        let password = self.password.clone();
        let tx = self.tx.as_ref().unwrap().clone();
        
        thread::spawn(move || {
            use crate::models::ContentCache;
            
            let cache_key = format!("movies_{}_{}", &username, &server_url);
            let cat_cache_key = format!("movies_cat_{}_{}", &username, &server_url);
            
            // Try cache first (24 hour cache)
            if let (Some(categories), Some(movies)) = (
                ContentCache::load::<Vec<Category>>(&cat_cache_key),
                ContentCache::load::<Vec<serde_json::Value>>(&cache_key)
            ) {
                let _ = tx.send(AppMessage::MoviesLoaded(categories, movies));
                return;
            }
            
            let client = XtreamClient::new(server_url, username, password);
            
            match (client.get_vod_categories(), client.get_vod_streams()) {
                (Ok(categories), Ok(movies)) => {
                    // Cache for 24 hours
                    let _ = ContentCache::save(&cat_cache_key, &categories, 86400);
                    let _ = ContentCache::save(&cache_key, &movies, 86400);
                    let _ = tx.send(AppMessage::MoviesLoaded(categories, movies));
                }
                (Err(e), _) => {
                    let _ = tx.send(AppMessage::Error(format!("Failed to fetch movie categories: {}", e)));
                }
                (_, Err(e)) => {
                    let _ = tx.send(AppMessage::Error(format!("Failed to fetch movies: {}", e)));
                }
            }
        });
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // Content Filtering
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Filters content based on current search query and category.
    fn filter_content(&mut self) {
        let query = self.search_query.to_lowercase();
        self.current_page = 0; // Reset to first page when filtering
        
        match self.current_content {
            ContentType::LiveTV => {
                self.filtered_channels = self.all_channels.iter()
                    .filter(|ch| {
                        let matches_search = query.is_empty() || ch.name.to_lowercase().contains(&query);
                        let matches_category = self.selected_category.is_none() 
                            || self.selected_category.as_ref() == Some(&ch.category_id);
                        matches_search && matches_category
                    })
                    .take(1000)
                    .cloned()
                    .collect();
            }
            ContentType::Series => {
                self.filtered_series = self.all_series.iter()
                    .filter(|s| {
                        let matches_search = query.is_empty() || s.name.to_lowercase().contains(&query);
                        let matches_category = self.selected_category.is_none() 
                            || self.selected_category.as_ref() == Some(&s.category_id);
                        matches_search && matches_category
                    })
                    .take(1000)
                    .cloned()
                    .collect();
            }
            ContentType::Favorites => {
                self.filtered_channels = self.all_channels.iter()
                    .filter(|ch| {
                        let is_favorite = self.config.favorites.contains(&ch.stream_id);
                        let matches_search = query.is_empty() || ch.name.to_lowercase().contains(&query);
                        is_favorite && matches_search
                    })
                    .take(1000)
                    .cloned()
                    .collect();
            }
            ContentType::Movies => {
                self.filtered_movies = self.all_movies.iter()
                    .filter(|m| {
                        let name = m.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let matches_search = query.is_empty() || name.to_lowercase().contains(&query);
                        let category_id = m.get("category_id").and_then(|v| v.as_str()).unwrap_or("");
                        let matches_category = self.selected_category.is_none() 
                            || self.selected_category.as_ref() == Some(&category_id.to_string());
                        matches_search && matches_category
                    })
                    .take(1000)
                    .cloned()
                    .collect();
            }
            ContentType::Discover => {
                // Discover doesn't use traditional filtering
            }
            ContentType::FootballFixtures => {
                // Football fixtures use their own filtering system
            }
            ContentType::ContinueWatching => {
                // Continue watching doesn't use traditional filtering
            }
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // Playback Methods
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Plays a live channel using the configured media player.
    fn play_channel(&mut self, channel: &Channel) {
        let client = XtreamClient::new(
            self.server_url.clone(),
            self.username.clone(),
            self.password.clone(),
        );
        
        let url = client.get_live_stream_url(&channel.stream_id);
        self.current_stream_url = Some(url.clone());
        
        // Launch the configured player (live stream = true)
        let _ = self.config.player_settings.launch_player(&url, &channel.name, true);
    }
    
    /// Plays a movie using the configured media player.
    fn play_movie(&mut self, stream_id: i64, name: &str, container_extension: &str, thumbnail: Option<String>) {
        let client = XtreamClient::new(
            self.server_url.clone(),
            self.username.clone(),
            self.password.clone(),
        );
        let url = client.get_stream_url(&stream_id.to_string(), container_extension);
        
        // Add to watch history
        let progress = crate::models::watch_history::WatchProgress {
            content_id: stream_id.to_string(),
            content_name: name.to_string(),
            content_type: "movie".to_string(),
            position_seconds: 0.0,
            duration_seconds: 0.0, // We don't know duration yet
            last_watched: chrono::Utc::now().timestamp(),
            thumbnail,
            season: None,
            episode: None,
        };
        self.watch_history.update_progress(progress);
        
        // Launch the configured player (not live stream)
        let _ = self.config.player_settings.launch_player(&url, name, false);
    }
    
    /// Plays an episode and adds it to watch history.
    fn play_episode(&mut self, episode_id: &str, series_name: &str, season: i32, episode: i32, title: &str, container: &str) {
        let client = XtreamClient::new(
            self.server_url.clone(),
            self.username.clone(),
            self.password.clone(),
        );
        let url = client.get_episode_url(episode_id, container);
        let window_title = format!("S{}E{}: {}", season, episode, title);
        
        // Add to watch history
        let progress = crate::models::watch_history::WatchProgress {
            content_id: format!("{}_{}_{}",  series_name, season, episode),
            content_name: series_name.to_string(),
            content_type: "series".to_string(),
            position_seconds: 0.0,
            duration_seconds: 0.0,
            last_watched: chrono::Utc::now().timestamp(),
            thumbnail: None, // We don't have episode thumbnails readily available
            season: Some(season),
            episode: Some(episode),
        };
        self.watch_history.update_progress(progress);
        
        // Launch the configured player (not live stream)
        let _ = self.config.player_settings.launch_player(&url, &window_title, false);
    }
    
    /// Resumes playback from continue watching
    fn resume_playback(&mut self, content_id: &str, content_type: &str, season: Option<i32>, episode: Option<i32>) {
        match content_type {
            "movie" => {
                // For movies, content_id is the stream_id
                if let Ok(stream_id) = content_id.parse::<i64>() {
                    // Find the movie in our cache to get details
                    let movie_data = self.all_movies.iter().find(|m| {
                        m.get("stream_id").and_then(|v| v.as_i64()) == Some(stream_id)
                    }).map(|movie| {
                        (
                            stream_id,
                            movie.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                            movie.get("container_extension").and_then(|v| v.as_str()).unwrap_or("mp4").to_string(),
                            movie.get("stream_icon").and_then(|v| v.as_str()).map(|s| s.to_string())
                        )
                    });
                    
                    if let Some((id, name, container, thumbnail)) = movie_data {
                        self.play_movie(id, &name, &container, thumbnail);
                    }
                }
            }
            "series" => {
                // For series, we need to find the series and episode
                // The content_id format is "series_name_season_episode"
                if season.is_some() && episode.is_some() {
                    // Parse the series name from content_id (remove the _season_episode suffix)
                    let parts: Vec<&str> = content_id.rsplitn(3, '_').collect();
                    if parts.len() == 3 {
                        let series_name = parts[2];
                        
                        // Find the series in our cache
                        let series_data = self.all_series.iter()
                            .find(|s| s.name == series_name)
                            .map(|s| s.series_id);
                        
                        if let Some(series_id) = series_data {
                            // Open the episode dialog for this series
                            self.episode_dialog_state = Some(episode_dialog::EpisodeDialogState::new(
                                series_id,
                                self.server_url.clone(),
                                self.username.clone(),
                                self.password.clone(),
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // Favorites Management
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Toggles favorite status for a channel.
    fn toggle_favorite(&mut self, stream_id: &str) {
        if self.config.favorites.contains(stream_id) {
            self.config.favorites.remove(stream_id);
        } else {
            self.config.favorites.insert(stream_id.to_string());
        }
        let _ = self.config.save();
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // Configuration
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Saves user credentials to config.
    fn save_credentials(&mut self) {
        self.config.server_url = self.server_url.clone();
        self.config.username = self.username.clone();
        self.config.password = self.password.clone();
        self.config.auto_login = true;
        let _ = self.config.save();
    }
    
    /// Returns whether we're in touch-friendly mode (Steam Deck or tablet).
    fn is_touch_mode(&self) -> bool {
        dimensions::is_touch_mode(self.screen_width, self.screen_height)
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // Message Processing
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Processes messages from background threads.
    fn process_messages(&mut self) {
        if let Some(rx) = &self.rx {
            if let Ok(msg) = rx.try_recv() {
                match msg {
                    AppMessage::Connected(categories, channels) => {
                        self.live_categories = categories;
                        self.all_channels = channels;
                        self.filtered_channels = self.all_channels.clone();
                        self.connected = true;
                        self.connecting = false;
                        self.save_credentials();
                        // Load series and movies in the background immediately
                        self.load_series();
                        self.load_movies();
                    }
                    AppMessage::Error(err) => {
                        self.error_message = Some(err);
                        self.connecting = false;
                    }
                    AppMessage::SeriesLoaded(categories, series) => {
                        self.series_categories = categories;
                        self.all_series = series;
                        self.filtered_series = self.all_series.clone();
                    }
                    AppMessage::MoviesLoaded(categories, movies) => {
                        self.movie_categories = categories;
                        self.all_movies = movies;
                        self.filtered_movies = self.all_movies.clone();
                    }
                }
            }
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // UI Rendering Helpers
    // ═══════════════════════════════════════════════════════════════════════
    

    
    /// Renders the main content area.
    fn render_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, theme: &Theme) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(20.0);
            
            match self.current_content {
                ContentType::LiveTV | ContentType::Favorites => {
                    self.render_channels(ui, ctx, theme);
                }
                ContentType::ContinueWatching => {
                    self.render_continue_watching(ui, ctx, theme);
                }
                ContentType::Series => {
                    self.render_series(ui, ctx, theme);
                }
                ContentType::Movies => {
                    self.render_movies(ui, ctx, theme);
                }
                ContentType::Discover => {
                    self.render_discover(ui, ctx, theme);
                }
                ContentType::FootballFixtures => {
                    self.render_football_section(ui, theme);
                }
            }
        });
    }
    
    /// Renders the continue watching section.
    fn render_continue_watching(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context, theme: &Theme) {
        ui.label(egui::RichText::new(self.current_content.title())
            .size(24.0)
            .color(theme.text_primary)
            .strong());
        
        ui.add_space(4.0);
        let items = self.watch_history.get_continue_watching(50);
        ui.label(egui::RichText::new(format!("{} items", items.len())).size(14.0).color(theme.text_secondary));
        ui.add_space(16.0);
        
        if items.is_empty() {
            self.render_empty_state(ui, theme);
            return;
        }
        
        let card_width = dimensions::card_width(self.screen_width);
        
        let mut clicked_content: Option<crate::models::watch_history::WatchProgress> = None;
        
        ui.horizontal_wrapped(|ui| {
            for progress in &items {
                // Make the entire card clickable
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(card_width, 280.0),
                    egui::Sense::click(),
                );
                
                if response.clicked() {
                    clicked_content = Some(progress.clone());
                }
                
                // Draw card with hover effect
                let bg_color = if response.hovered() {
                    theme.card_bg.linear_multiply(1.1)
                } else {
                    theme.card_bg
                };
                
                ui.painter().rect_filled(
                    rect,
                    4.0,
                    bg_color,
                );
                
                // Draw border
                ui.painter().rect_stroke(
                    rect,
                    4.0,
                    egui::Stroke::new(1.0, theme.border_color),
                );
                
                // Draw content inside the card
                let mut child_ui = ui.new_child(egui::UiBuilder::new()
                    .max_rect(rect.shrink(8.0))
                    .layout(egui::Layout::top_down(egui::Align::LEFT)));
                
                // Thumbnail
                if let Some(thumbnail_url) = &progress.thumbnail {
                    if let Some(texture) = self.image_cache.get(thumbnail_url) {
                        child_ui.add(egui::Image::new(&texture)
                            .fit_to_exact_size(egui::vec2(card_width - 32.0, 150.0))
                            .rounding(4.0));
                    } else {
                        child_ui.add_space(150.0);
                    }
                } else {
                    // Placeholder
                    let (placeholder_rect, _) = child_ui.allocate_exact_size(
                        egui::vec2(card_width - 32.0, 150.0),
                        egui::Sense::hover(),
                    );
                    child_ui.painter().rect_filled(placeholder_rect, 4.0, theme.panel_bg);
                    child_ui.painter().text(
                        placeholder_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &progress.content_type.to_uppercase(),
                        egui::FontId::proportional(14.0),
                        theme.text_secondary,
                    );
                }
                
                child_ui.add_space(8.0);
                
                // Progress bar
                let progress_pct = progress.progress_percentage() / 100.0;
                let (bar_rect, _) = child_ui.allocate_exact_size(
                    egui::vec2(card_width - 32.0, 4.0),
                    egui::Sense::hover(),
                );
                child_ui.painter().rect_filled(bar_rect, 2.0, theme.border_color);
                let progress_rect = egui::Rect::from_min_size(
                    bar_rect.min,
                    egui::vec2((card_width - 32.0) * progress_pct, 4.0),
                );
                child_ui.painter().rect_filled(progress_rect, 2.0, theme.accent_blue);
                
                child_ui.add_space(8.0);
                
                // Title
                child_ui.label(egui::RichText::new(&progress.content_name)
                    .size(14.0)
                    .color(theme.text_primary)
                    .strong());
                
                // Episode info if series
                if let (Some(season), Some(episode)) = (progress.season, progress.episode) {
                    child_ui.label(egui::RichText::new(format!("S{}:E{}", season, episode))
                        .size(12.0)
                        .color(theme.text_secondary));
                }
                
                // Progress info
                let mins_watched = (progress.position_seconds / 60.0) as i32;
                let mins_total = (progress.duration_seconds / 60.0) as i32;
                child_ui.label(egui::RichText::new(format!("{} / {} min ({:.0}%)", 
                    mins_watched, mins_total, progress.progress_percentage()))
                    .size(12.0)
                    .color(theme.text_secondary));
            }
        });
        
        // Handle resume playback
        if let Some(progress) = clicked_content {
            self.resume_playback(&progress.content_id, &progress.content_type, progress.season, progress.episode);
        }
    }
    
    /// Renders the channels grid.
    fn render_channels(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, theme: &Theme) {
        ui.label(egui::RichText::new(self.current_content.title())
            .size(24.0)
            .color(theme.text_primary)
            .strong());
        
        // Process pending EPG data
        self.epg_cache.process_pending();
        
        // Pagination calculations
        let total_items = self.filtered_channels.len();
        let total_pages = (total_items + self.page_size - 1) / self.page_size;
        let start_idx = self.current_page * self.page_size;
        let end_idx = (start_idx + self.page_size).min(total_items);
        
        Pagination::show_info(ui, theme, start_idx, end_idx, total_items, "channels", self.current_page, total_pages);
        ui.add_space(16.0);
        
        // Get current page items
        let page_channels: Vec<Channel> = self.filtered_channels.iter()
            .skip(start_idx)
            .take(self.page_size)
            .cloned()
            .collect();
        
        // Request EPG data for visible channels
        for channel in &page_channels {
            self.epg_cache.request_epg(&channel.stream_id);
        }
        
        let favorites = self.config.favorites.clone();
        let mut channel_to_play: Option<Channel> = None;
        let mut channel_to_toggle: Option<String> = None;
        
        let screen_width = self.screen_width;
        ui.horizontal_wrapped(|ui| {
            for channel in &page_channels {
                // Build EPG info for this channel
                let epg_info = {
                    let current = self.epg_cache.get_current_program(&channel.stream_id);
                    let next = self.epg_cache.get_next_program(&channel.stream_id);
                    if current.is_some() || next.is_some() {
                        Some(channel_card::ChannelEpgInfo {
                            current_program: current,
                            next_program: next,
                        })
                    } else {
                        None
                    }
                };
                
                if let Some(action) = ChannelCard::show(
                    ui,
                    ctx,
                    theme,
                    channel,
                    favorites.contains(&channel.stream_id),
                    &self.image_cache,
                    screen_width,
                    epg_info.as_ref(),
                ) {
                    match action {
                        channel_card::ChannelAction::Play(ch) => {
                            channel_to_play = Some(ch);
                        }
                        channel_card::ChannelAction::ToggleFavorite(id) => {
                            channel_to_toggle = Some(id);
                        }
                    }
                }
            }
        });
        
        // Handle pagination
        if let Some(new_page) = Pagination::show(ui, theme, self.current_page, total_pages, self.is_touch_mode()) {
            self.current_page = new_page;
        }
        
        // Process actions
        if let Some(channel) = channel_to_play {
            self.play_channel(&channel);
        }
        if let Some(stream_id) = channel_to_toggle {
            self.toggle_favorite(&stream_id);
            if matches!(self.current_content, ContentType::Favorites) {
                self.filter_content();
            }
        }
        
        // Empty state
        if self.filtered_channels.is_empty() {
            self.render_empty_state(ui, theme);
        }
    }
    
    /// Renders the series grid.
    fn render_series(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, theme: &Theme) {
        ui.label(egui::RichText::new(self.current_content.title())
            .size(24.0)
            .color(theme.text_primary)
            .strong());
        
        if self.all_series.is_empty() {
            ui.add_space(4.0);
            ui.label(egui::RichText::new("0 series").size(14.0).color(theme.text_secondary));
            ui.add_space(16.0);
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.spinner();
                ui.add_space(8.0);
                ui.label("Loading series...");
            });
            return;
        }
        
        // Pagination
        let total_items = self.filtered_series.len();
        let total_pages = (total_items + self.page_size - 1) / self.page_size;
        let start_idx = self.current_page * self.page_size;
        let end_idx = (start_idx + self.page_size).min(total_items);
        
        Pagination::show_info(ui, theme, start_idx, end_idx, total_items, "series", self.current_page, total_pages);
        ui.add_space(16.0);
        
        let page_series: Vec<Series> = self.filtered_series.iter()
            .skip(start_idx)
            .take(self.page_size)
            .cloned()
            .collect();
        
        let screen_width = self.screen_width;
        ui.horizontal_wrapped(|ui| {
            for series in &page_series {
                if let Some(action) = SeriesCard::show(ui, ctx, theme, series, &self.image_cache, screen_width) {
                    match action {
                        series_card::SeriesAction::ViewEpisodes(id) => {
                            // Create new episode dialog state - data loads in background
                            self.episode_dialog_state = Some(episode_dialog::EpisodeDialogState::new(
                                id,
                                self.server_url.clone(),
                                self.username.clone(),
                                self.password.clone(),
                            ));
                        }
                    }
                }
            }
        });
        
        if let Some(new_page) = Pagination::show(ui, theme, self.current_page, total_pages, self.is_touch_mode()) {
            self.current_page = new_page;
        }
    }
    
    /// Renders the movies grid.
    fn render_movies(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, theme: &Theme) {
        ui.label(egui::RichText::new(self.current_content.title())
            .size(24.0)
            .color(theme.text_primary)
            .strong());
        
        if self.all_movies.is_empty() {
            ui.add_space(4.0);
            ui.label(egui::RichText::new("0 movies").size(14.0).color(theme.text_secondary));
            ui.add_space(16.0);
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.spinner();
                ui.add_space(8.0);
                ui.label("Loading movies...");
            });
            return;
        }
        
        // Pagination
        let total_items = self.filtered_movies.len();
        let total_pages = (total_items + self.page_size - 1) / self.page_size;
        let start_idx = self.current_page * self.page_size;
        let end_idx = (start_idx + self.page_size).min(total_items);
        
        Pagination::show_info(ui, theme, start_idx, end_idx, total_items, "movies", self.current_page, total_pages);
        ui.add_space(16.0);
        
        let page_movies: Vec<serde_json::Value> = self.filtered_movies.iter()
            .skip(start_idx)
            .take(self.page_size)
            .cloned()
            .collect();
        
        let mut movie_to_play: Option<(i64, String, String, Option<String>)> = None;
        let screen_width = self.screen_width;
        
        ui.horizontal_wrapped(|ui| {
            for movie in &page_movies {
                if let Some(action) = MovieCard::show(ui, ctx, theme, movie, &self.image_cache, screen_width) {
                    match action {
                        movie_card::MovieAction::Play { stream_id, name, container_extension, thumbnail } => {
                            movie_to_play = Some((stream_id, name, container_extension, thumbnail));
                        }
                    }
                }
            }
        });
        
        if let Some(new_page) = Pagination::show(ui, theme, self.current_page, total_pages, self.is_touch_mode()) {
            self.current_page = new_page;
        }
        
        if let Some((stream_id, name, ext, thumbnail)) = movie_to_play {
            self.play_movie(stream_id, &name, &ext, thumbnail);
        }
        
        if self.filtered_movies.is_empty() {
            self.render_empty_state(ui, theme);
        }
    }
    
    /// Renders the empty state message.
    fn render_empty_state(&self, ui: &mut egui::Ui, theme: &Theme) {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("🔍")
                .size(48.0)
                .color(theme.text_secondary));
            ui.add_space(8.0);
            ui.label(egui::RichText::new(self.current_content.empty_message())
                .size(18.0)
                .color(theme.text_secondary));
        });
    }
    
    /// Renders the Discover section with movies and series.
    fn render_discover(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, theme: &Theme) {
        // Process pending data
        self.discover_cache.process_pending();
        
        self.render_movies_discover(ui, ctx, theme);
    }
    
    /// Renders the football fixtures section
    fn render_football_section(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        // Process pending data
        self.football_cache.process_pending();
        
        ui.label(egui::RichText::new(self.current_content.title())
            .size(24.0)
            .color(theme.text_primary)
            .strong());
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Find upcoming matches and which channels broadcast them")
            .size(11.0)
            .color(theme.text_secondary));
        ui.add_space(8.0);
        
        // Check if database exists
        if !self.football_cache.has_database() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(egui::RichText::new("⚽")
                    .size(48.0)
                    .color(theme.text_secondary));
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Football Database Not Found")
                    .size(18.0)
                    .color(theme.text_primary));
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Click the button below to fetch upcoming fixtures")
                    .size(12.0)
                    .color(theme.text_secondary));
                ui.add_space(16.0);
                if ui.add(
                    egui::Button::new(
                        egui::RichText::new("⚽ Fetch Fixtures")
                            .size(14.0)
                            .color(egui::Color32::WHITE)
                    )
                    .fill(theme.accent_blue)
                    .rounding(egui::Rounding::same(4.0))
                    .min_size(egui::vec2(140.0, 36.0))
                ).clicked() {
                    // Run the scraper in background using venv Python
                    std::thread::spawn(|| {
                        let _ = std::process::Command::new(".venv\\Scripts\\python.exe")
                            .args(["fixtures.py", "scrape"])
                            .current_dir("Soccer-Scraper-main")
                            .spawn();
                    });
                    self.football_cache.clear();
                }
            });
            return;
        }
        
        ui.add_space(8.0);
        
        // Request data for current category
        self.football_cache.request_category(self.football_category);
        
        let is_loading = self.football_cache.is_loading(self.football_category);
        let fixtures = self.football_cache.get_category(self.football_category);
        
        // Show fixtures
        if let Some(fixtures) = fixtures {
            if fixtures.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(egui::RichText::new("No fixtures found for this category")
                        .color(theme.text_secondary));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Try selecting a different category or run the scraper")
                        .size(12.0)
                        .color(theme.text_secondary));
                });
            } else {
                let mut channel_to_search: Option<String> = None;
                
                ui.label(egui::RichText::new(format!("Found {} matches", fixtures.len()))
                    .size(12.0)
                    .color(theme.text_secondary));
                ui.add_space(8.0);
                
                // Group fixtures by date
                let mut fixtures_by_date: std::collections::HashMap<String, Vec<&crate::api::FootballFixture>> = std::collections::HashMap::new();
                for fixture in fixtures.iter() {
                    fixtures_by_date
                        .entry(fixture.fixture_date.clone())
                        .or_default()
                        .push(fixture);
                }
                
                // Sort dates
                let mut dates: Vec<String> = fixtures_by_date.keys().cloned().collect();
                dates.sort();
                
                // Calculate how many cards fit per row based on window width
                let card_width = 320.0;
                let card_spacing = 12.0;
                let available_width = ui.available_width();
                let cards_per_row = ((available_width + card_spacing) / (card_width + card_spacing)).floor().max(1.0) as usize;
                let cards_per_row = cards_per_row.min(4); // Max 4 per row
                
                // Display fixtures grouped by date in a scrollable area
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for date in dates.iter() {
                        if let Some(date_fixtures) = fixtures_by_date.get(date) {
                            // Date header
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new(date)
                                .size(16.0)
                                .color(theme.text_primary)
                                .strong());
                            ui.add_space(8.0);
                            
                            // Display fixtures for this date in responsive rows
                            let fixtures_list: Vec<_> = date_fixtures.iter().take(30).collect();
                            for chunk in fixtures_list.chunks(cards_per_row) {
                                ui.horizontal(|ui| {
                                    for fixture in chunk {
                                        if let Some(action) = FootballCard::show(
                                            ui,
                                            theme,
                                            fixture,
                                            self.screen_width,
                                        ) {
                                            match action {
                                                FootballAction::SearchChannel(channel) => {
                                                    channel_to_search = Some(channel);
                                                }
                                                FootballAction::SearchTeam(team) => {
                                                    channel_to_search = Some(team);
                                                }
                                            }
                                        }
                                    }
                                });
                                ui.add_space(8.0);
                            }
                        }
                    }
                });
                
                // Handle search action - switch to Live TV and search
                if let Some(query) = channel_to_search {
                    self.search_query = query;
                    self.current_content = ContentType::LiveTV;
                    self.current_page = 0;
                    self.filter_content();
                }
            }
        } else if is_loading {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.spinner();
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Loading fixtures...")
                    .color(theme.text_secondary));
            });
        } else if let Some(error) = self.football_cache.last_error.clone() {
            let mut should_retry = false;
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(egui::RichText::new("❌")
                    .size(48.0)
                    .color(egui::Color32::from_rgb(231, 76, 60)));
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Failed to load fixtures")
                    .size(18.0)
                    .color(theme.text_primary));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(&error)
                    .size(12.0)
                    .color(theme.text_secondary));
                ui.add_space(16.0);
                if ui.button("🔄 Retry").clicked() {
                    should_retry = true;
                }
            });
            if should_retry {
                self.football_cache.clear();
            }
        }
    }
    
    /// Renders the movies/series discover section
    fn render_movies_discover(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, theme: &Theme) {
        ui.label(egui::RichText::new("🎬 Discover Trending & Popular")
            .size(24.0)
            .color(theme.text_primary)
            .strong());
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Powered by OMDb - Curated trending and popular titles")
            .size(11.0)
            .color(theme.text_secondary));
        ui.add_space(8.0);
        
        // Category selector
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Category:").color(theme.text_primary));
            egui::ComboBox::from_id_salt("discover_category")
                .selected_text(self.discover_category.display_name())
                .show_ui(ui, |ui| {
                    for category in crate::api::DiscoverCategory::all() {
                        if ui.selectable_value(&mut self.discover_category, *category, category.display_name()).clicked() {
                            // Will trigger a new request
                        }
                    }
                });
            
            // Refresh button
            if ui.button("🔄 Refresh").clicked() {
                self.discover_cache.clear();
            }
        });
        
        ui.add_space(16.0);
        
        // Request data for current category
        self.discover_cache.request_category(self.discover_category);
        
        let is_loading = self.discover_cache.is_loading(self.discover_category);
        let items = self.discover_cache.get_category(self.discover_category);
        
        // Show content - limit to 20 items for performance
        if let Some(items) = items {
            let mut search_query: Option<String> = None;
            let display_items: Vec<_> = items.iter().take(20).collect();
            
            // Display items in a grid
            ui.horizontal_wrapped(|ui| {
                for item in display_items.iter() {
                    // Only load images for items being displayed
                    if let Some(poster_url) = &item.poster_url {
                        self.image_cache.load(ctx, poster_url.clone());
                    }
                    
                    if let Some(action) = DiscoverCard::show(
                        ui,
                        theme,
                        item,
                        &self.image_cache,
                        self.screen_width,
                    ) {
                        match action {
                            DiscoverAction::SearchInIptv(query) => {
                                search_query = Some(query);
                            }
                        }
                    }
                }
            });
            
            // Handle search action - switch to Series and search
            if let Some(query) = search_query {
                self.search_query = query;
                self.current_content = ContentType::Series;
                self.current_page = 0;
                self.filter_content();
            }
        } else if is_loading {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.spinner();
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Loading content...")
                    .color(theme.text_secondary));
            });
        } else if let Some(error) = self.discover_cache.last_error.clone() {
            // Show error message (clone to avoid borrow issues)
            let mut should_retry = false;
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(egui::RichText::new("❌")
                    .size(48.0)
                    .color(egui::Color32::from_rgb(231, 76, 60)));
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Failed to load content")
                    .size(18.0)
                    .color(theme.text_primary));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(&error)
                    .size(12.0)
                    .color(theme.text_secondary));
                ui.add_space(16.0);
                if ui.button("🔄 Retry").clicked() {
                    should_retry = true;
                }
            });
            if should_retry {
                self.discover_cache.clear();
            }
        } else {
            let mut should_retry = false;
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(egui::RichText::new("No content available")
                    .color(theme.text_secondary));
                ui.add_space(8.0);
                if ui.button("🔄 Retry").clicked() {
                    should_retry = true;
                }
            });
            if should_retry {
                self.discover_cache.clear();
            }
        }
    }
}

impl eframe::App for IPTVPlayerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Manage repaint frequency based on loading state
        if !self.image_cache.is_loading() {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }
        
        // Create and apply theme
        let theme = Theme::new(self.dark_mode);
        theme.apply(ctx);
        
        // Process background messages
        self.process_messages();
        
        // Update screen dimensions for responsive layout
        self.screen_width = ctx.screen_rect().width();
        self.screen_height = ctx.screen_rect().height();
        let is_mobile = dimensions::is_mobile(self.screen_width);
        let is_touch_mode = dimensions::is_touch_mode(self.screen_width, self.screen_height);
        
        if !self.connected {
            // Show login screen
            egui::CentralPanel::default().show(ctx, |ui| {
                if LoginScreen::show(
                    ui,
                    &theme,
                    &mut self.server_url,
                    &mut self.username,
                    &mut self.password,
                    self.connecting,
                    &self.error_message,
                    is_touch_mode,
                ) {
                    self.connect();
                }
            });
        } else {
            // Show main application
            
            // Sidebar - on mobile/touch show as overlay when sidebar_visible is true
            // On desktop, always show
            let show_sidebar = if is_mobile || is_touch_mode { self.sidebar_visible } else { true };
            let sidebar_width = dimensions::sidebar_width_touch(self.screen_width, self.screen_height);
            
            if show_sidebar {
                egui::SidePanel::left("categories")
                    .min_width(sidebar_width)
                    .max_width(sidebar_width)
                    .frame(egui::Frame::none()
                        .fill(if self.dark_mode { 
                            egui::Color32::from_rgb(22, 22, 22) 
                        } else { 
                            egui::Color32::from_rgb(250, 250, 250) 
                        })
                        .inner_margin(egui::Margin::same(if is_mobile { 12.0 } else if is_touch_mode { 16.0 } else { 16.0 })))
                    .show(ctx, |ui| {
                        // Close button on mobile/touch
                        if is_mobile || is_touch_mode {
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let close_btn = egui::Button::new(egui::RichText::new("✕").size(if is_touch_mode { 24.0 } else { 20.0 }))
                                        .min_size(egui::vec2(if is_touch_mode { 48.0 } else { 36.0 }, if is_touch_mode { 48.0 } else { 36.0 }));
                                    if ui.add(close_btn).clicked() {
                                        self.sidebar_visible = false;
                                    }
                                });
                            });
                            ui.add_space(8.0);
                        }
                        
                        // Get categories before mutable borrow (Discover has no categories)
                        let categories: Vec<Category> = match self.current_content {
                            ContentType::LiveTV | ContentType::Favorites => self.live_categories.clone(),
                            ContentType::Series => self.series_categories.clone(),
                            ContentType::Movies => self.movie_categories.clone(),
                            ContentType::Discover => Vec::new(), // Discover doesn't use categories
                            ContentType::FootballFixtures => Vec::new(), // Football fixtures don't use categories
                            ContentType::ContinueWatching => Vec::new(), // Continue watching doesn't use categories
                        };
                        
                        // Only show category sidebar if not in Discover, Football, or Continue Watching mode
                        if !matches!(self.current_content, ContentType::Discover | ContentType::FootballFixtures | ContentType::ContinueWatching) {
                            if let Some(selection) = CategorySidebar::show(
                                ui,
                                &theme,
                                self.current_content,
                                &categories,
                                &self.selected_category,
                                &mut self.category_search,
                            ) {
                                self.selected_category = selection;
                                self.filter_content();
                                // Auto-close sidebar on mobile after selection
                                if is_mobile {
                                    self.sidebar_visible = false;
                                }
                            }
                        } else if matches!(self.current_content, ContentType::FootballFixtures) {
                            // Show Football category selector in sidebar
                            ui.label(egui::RichText::new("Filters")
                                .size(16.0)
                                .color(theme.text_primary)
                                .strong());
                            ui.add_space(12.0);
                            
                            // Thin separator
                            let separator_rect = ui.available_rect_before_wrap();
                            ui.painter().hline(
                                separator_rect.min.x..=separator_rect.min.x + 180.0,
                                separator_rect.min.y,
                                egui::Stroke::new(1.0, theme.border_color),
                            );
                            ui.add_space(8.0);
                            
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for category in FootballCategory::all() {
                                    let is_selected = self.football_category == *category;
                                    
                                    // Netflix-style button with underline on hover
                                    let text_color = if is_selected {
                                        theme.accent_blue
                                    } else {
                                        theme.text_secondary
                                    };
                                    
                                    // Touch-friendly button sizing
                                    let btn_height = if is_touch_mode { 48.0 } else { 28.0 };
                                    let font_size = if is_touch_mode { 15.0 } else { 13.0 };
                                    
                                    let response = ui.add(
                                        egui::Button::new(
                                            egui::RichText::new(category.display_name())
                                                .size(font_size)
                                                .color(text_color)
                                        )
                                        .fill(egui::Color32::TRANSPARENT)
                                        .frame(false)
                                        .min_size(egui::vec2(if is_touch_mode { 200.0 } else { 180.0 }, btn_height))
                                    );
                                    
                                    // Draw underline on hover or selected
                                    if response.hovered() || is_selected {
                                        let rect = response.rect;
                                        ui.painter().hline(
                                            rect.min.x..=rect.min.x + 40.0,
                                            rect.max.y,
                                            egui::Stroke::new(2.0, if is_selected { theme.accent_blue } else { theme.text_secondary }),
                                        );
                                    }
                                    
                                    if response.clicked() {
                                        self.football_category = *category;
                                        if is_mobile || is_touch_mode {
                                            self.sidebar_visible = false;
                                        }
                                    }
                                    ui.add_space(2.0);
                                }
                            });
                            
                            ui.add_space(16.0);
                            
                            // Refresh button - runs the scraper
                            if ui.add(
                                egui::Button::new(
                                    egui::RichText::new("🔄 Refresh")
                                        .size(12.0)
                                        .color(egui::Color32::WHITE)
                                )
                                .fill(theme.accent_blue)
                                .rounding(egui::Rounding::same(4.0))
                                .min_size(egui::vec2(100.0, 32.0))
                            ).clicked() {
                                // Run the scraper in background using venv Python
                                std::thread::spawn(|| {
                                    let _ = std::process::Command::new(".venv\\Scripts\\python.exe")
                                        .args(["fixtures.py", "scrape"])
                                        .current_dir("Soccer-Scraper-main")
                                        .spawn();
                                });
                                self.football_cache.clear();
                            }
                        } else if matches!(self.current_content, ContentType::Discover) {
                            // Show TMDB category selector in sidebar for Discover mode
                            ui.label(egui::RichText::new("Categories")
                                .size(16.0)
                                .color(theme.text_primary)
                                .strong());
                            ui.add_space(12.0);
                            
                            // Thin separator
                            let separator_rect = ui.available_rect_before_wrap();
                            ui.painter().hline(
                                separator_rect.min.x..=separator_rect.min.x + 180.0,
                                separator_rect.min.y,
                                egui::Stroke::new(1.0, theme.border_color),
                            );
                            ui.add_space(8.0);
                            
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for category in DiscoverCategory::all() {
                                    let is_selected = self.discover_category == *category;
                                    
                                    // Netflix-style button with underline on hover
                                    let text_color = if is_selected {
                                        theme.accent_blue
                                    } else {
                                        theme.text_secondary
                                    };
                                    
                                    // Touch-friendly button sizing
                                    let btn_height = if is_touch_mode { 48.0 } else { 28.0 };
                                    let font_size = if is_touch_mode { 15.0 } else { 13.0 };
                                    
                                    let response = ui.add(
                                        egui::Button::new(
                                            egui::RichText::new(category.display_name())
                                                .size(font_size)
                                                .color(text_color)
                                        )
                                        .fill(egui::Color32::TRANSPARENT)
                                        .frame(false)
                                        .min_size(egui::vec2(if is_touch_mode { 200.0 } else { 180.0 }, btn_height))
                                    );
                                    
                                    // Draw underline on hover or selected
                                    if response.hovered() || is_selected {
                                        let rect = response.rect;
                                        ui.painter().hline(
                                            rect.min.x..=rect.min.x + 40.0,
                                            rect.max.y,
                                            egui::Stroke::new(2.0, if is_selected { theme.accent_blue } else { theme.text_secondary }),
                                        );
                                    }
                                    
                                    if response.clicked() {
                                        self.discover_category = *category;
                                        if is_mobile || is_touch_mode {
                                            self.sidebar_visible = false;
                                        }
                                    }
                                    ui.add_space(2.0);
                                }
                            });
                        }
                    });
            }
            
            // Main content area
            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(theme.bg_color))
                .show(ctx, |ui| {
                    // Top navigation - larger margins for touch mode
                    let nav_margin = if is_mobile { 
                        egui::Margin::symmetric(12.0, 10.0) 
                    } else if is_touch_mode {
                        egui::Margin::symmetric(20.0, 14.0) 
                    } else { 
                        egui::Margin::symmetric(20.0, 16.0) 
                    };
                    
                    egui::TopBottomPanel::top("top_panel")
                        .frame(egui::Frame::none()
                            .fill(theme.panel_bg)
                            .inner_margin(nav_margin))
                        .show_inside(ui, |ui| {
                            if let Some(action) = TopNavigation::show(
                                ui,
                                &theme,
                                self.current_content,
                                &mut self.search_query,
                                is_mobile,
                                is_touch_mode,
                            ) {
                                match action {
                                    top_nav::NavAction::SwitchContent(content_type) => {
                                        self.current_content = content_type;
                                        self.selected_category = None;
                                        match content_type {
                                            ContentType::Series if self.all_series.is_empty() => {
                                                self.load_series();
                                            }
                                            ContentType::Movies if self.all_movies.is_empty() => {
                                                self.load_movies();
                                            }
                                            _ => {}
                                        }
                                        self.filter_content();
                                    }
                                    top_nav::NavAction::SearchChanged => {
                                        self.filter_content();
                                    }
                                    top_nav::NavAction::ToggleTheme => {
                                        self.dark_mode = !self.dark_mode;
                                    }
                                    top_nav::NavAction::Disconnect => {
                                        self.disconnect();
                                    }
                                    top_nav::NavAction::OpenPlayerSettings => {
                                        self.temp_player_settings = Some(self.config.player_settings.clone());
                                        self.show_player_settings = true;
                                    }
                                    top_nav::NavAction::OpenScraperSettings => {
                                        self.show_scraper_settings = true;
                                    }
                                    top_nav::NavAction::ToggleSidebar => {
                                        self.sidebar_visible = !self.sidebar_visible;
                                    }
                                }
                            }
                        });
                    
                    // Content area
                    self.render_content(ui, ctx, &theme);
                });
            
            // Episode dialog - use cached state for fast rendering
            if let Some(ref mut state) = self.episode_dialog_state {
                if let Some(action) = EpisodeDialog::show(ctx, state, &self.config.player_settings) {
                    match action {
                        episode_dialog::EpisodeAction::PlayEpisode { episode_id, series_name, season, episode, title, container } => {
                            self.play_episode(&episode_id, &series_name, season, episode, &title, &container);
                        }
                        episode_dialog::EpisodeAction::Close => {
                            self.episode_dialog_state = None;
                        }
                    }
                }
            }
            
            // Player settings dialog
            if self.show_player_settings {
                if let Some(ref mut temp_settings) = self.temp_player_settings {
                    if let Some(action) = PlayerSettingsDialog::show(ctx, &theme, temp_settings) {
                        match action {
                            player_settings::PlayerSettingsAction::Saved => {
                                self.config.player_settings = temp_settings.clone();
                                let _ = self.config.save();
                                self.show_player_settings = false;
                                self.temp_player_settings = None;
                            }
                            player_settings::PlayerSettingsAction::Cancelled => {
                                self.show_player_settings = false;
                                self.temp_player_settings = None;
                            }
                            player_settings::PlayerSettingsAction::Reset => {
                                *temp_settings = crate::models::PlayerSettings::default();
                            }
                        }
                    }
                }
            }
            
            // Scraper settings dialog
            if self.show_scraper_settings {
                let mut should_close = false;
                egui::Window::new("Football Fixtures Scraper")
                    .open(&mut self.show_scraper_settings)
                    .resizable(true)
                    .default_width(400.0)
                    .show(ctx, |ui| {
                        let scraper_status = self.scraper_manager.status();
                        if let Some(action) = ScraperSettingsDialog::show(
                            ui,
                            &theme,
                            self.scraper_manager.is_available(),
                            &scraper_status,
                        ) {
                            match action {
                                scraper_settings::ScraperAction::TriggerScrape => {
                                    // Spawn background thread for scraping
                                    let mut mgr = ScraperManager::new();
                                    thread::spawn(move || {
                                        let _ = mgr.scrape_blocking();
                                    });
                                }
                                scraper_settings::ScraperAction::Close => {
                                    should_close = true;
                                }
                            }
                        }
                    });
                
                if should_close {
                    self.show_scraper_settings = false;
                }
            }
        }
    }
}
