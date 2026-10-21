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
use super::theme::{Theme, dimensions, spacing, typography};
use super::layout::Layout;
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
    /// Whether the EPG settings dialog is open
    show_epg_settings: bool,
    /// Temporary EPG settings for the dialog (to allow cancel)
    temp_epg_enabled: bool,
    temp_epg_url: String,
    /// Whether dark mode is enabled
    dark_mode: bool,
    /// Whether the docked sidebar is shown (wide layouts)
    sidebar_visible: bool,
    /// Whether the slide-over category drawer is open (narrow layouts)
    drawer_open: bool,
    /// Responsive layout metrics, recomputed every frame
    layout: Layout,
    /// Whether running in Steam Deck Game Mode environment
    is_steam_deck_mode: bool,
    
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
    /// Whether scraper is currently running
    scraping_in_progress: bool,
    /// Last scraper result message
    scraper_message: Option<String>,
}

impl IPTVPlayerApp {
    /// Creates a new application instance.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load().unwrap_or_default();
        let (tx, rx) = channel();

        // Detect Steam Deck environment for scaling
        let is_steam_deck_mode = Self::detect_steam_deck_environment();
        
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
            show_epg_settings: false,
            temp_epg_enabled: false,
            temp_epg_url: String::new(),
            dark_mode: true,
            sidebar_visible: true,
            drawer_open: false,
            layout: Layout::new(1280.0, is_steam_deck_mode),
            is_steam_deck_mode,
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
            scraping_in_progress: false,
            scraper_message: None,
        };
        
        // Auto-login if credentials are saved
        if app.config.auto_login && !app.username.is_empty() && !app.password.is_empty() && !app.server_url.is_empty() {
            app.connect();
        }
        
        app
    }

    /// Detects if running on Steam Deck based on environment variables.
    fn detect_steam_deck_environment() -> bool {
        // Check for Steam Deck specific environment variable
        if std::env::var("SteamDeck").is_ok() || std::env::var("STEAM_DECK").is_ok() {
            return true;
        }

        // Check for gamescope (Steam's compositor used in Game Mode)
        if std::env::var("GAMESCOPE_WAYLAND_DISPLAY").is_ok() {
            return true;
        }

        // Check for Steam runtime
        if let Ok(runtime) = std::env::var("SteamAppId") {
            if !runtime.is_empty() {
                return true;
            }
        }

        // Check /etc/os-release for SteamOS (Linux only)
        #[cfg(target_os = "linux")]
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            if content.contains("SteamOS") || content.contains("steamos") {
                return true;
            }
        }

        false
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

        // Set external EPG URL if enabled
        if self.config.epg_enabled {
            self.epg_cache.set_xmltv_url(self.config.epg_url.clone());
        } else {
            self.epg_cache.set_xmltv_url(None);
        }

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
    
    /// Changes the page size while keeping the first visible item on screen.
    fn set_page_size(&mut self, page_size: usize) {
        if page_size != self.page_size && page_size > 0 {
            let first_item = self.current_page * self.page_size;
            self.page_size = page_size;
            self.current_page = first_item / page_size;
        }
    }

    /// Toggles the category sidebar (docked) or drawer (overlay).
    fn toggle_sidebar(&mut self) {
        if self.layout.docked_sidebar() {
            self.sidebar_visible = !self.sidebar_visible;
        } else {
            self.drawer_open = !self.drawer_open;
        }
    }

    /// Whether the current view has anything to show in the sidebar.
    fn has_sidebar(&self) -> bool {
        !matches!(self.current_content, ContentType::ContinueWatching)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Steam Deck / Gamepad Input
    // ═══════════════════════════════════════════════════════════════════════

    /// Handles gamepad and keyboard input for Steam Deck Game Mode.
    /// This enables navigation via controller (D-pad maps to arrow keys,
    /// A button maps to Enter, B button maps to Escape).
    fn handle_gamepad_input(&mut self, ctx: &egui::Context) {
        let is_touch_mode = self.layout.touch;
        // Configure egui for better gamepad/keyboard navigation
        let mut style = (*ctx.style()).clone();

        // Enable keyboard navigation (Tab to move between widgets)
        // This is crucial for Steam Deck controller support as Steam maps
        // D-pad to arrow keys and face buttons to keyboard keys
        style.interaction.selectable_labels = true;

        // Larger focus rectangles for better visibility on Steam Deck
        if is_touch_mode {
            style.visuals.widgets.hovered.expansion = 4.0;
            style.visuals.widgets.active.expansion = 2.0;
            style.visuals.selection.stroke.width = 2.0;
        }

        ctx.set_style(style);

        // Handle keyboard shortcuts for navigation
        ctx.input(|i| {
            // Escape key - close dialogs or go back
            if i.key_pressed(egui::Key::Escape) {
                if self.episode_dialog_state.is_some() {
                    self.episode_dialog_state = None;
                } else if self.show_player_settings {
                    self.show_player_settings = false;
                    self.temp_player_settings = None;
                } else if self.show_epg_settings {
                    self.show_epg_settings = false;
                } else if self.show_scraper_settings {
                    self.show_scraper_settings = false;
                } else if self.drawer_open {
                    self.drawer_open = false;
                }
            }

            // Tab navigation between content types (LB/RB on controller)
            // Steam typically maps LB/RB to Q/E or Page Up/Down
            if i.key_pressed(egui::Key::PageDown) || (i.modifiers.ctrl && i.key_pressed(egui::Key::Tab)) {
                self.cycle_content_type(true);
            }
            if i.key_pressed(egui::Key::PageUp) || (i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::Tab)) {
                self.cycle_content_type(false);
            }

            // Quick menu toggle (Start button often maps to Enter or F10)
            if i.key_pressed(egui::Key::F10) {
                self.toggle_sidebar();
            }

            // Home key to scroll to top / reset view
            if i.key_pressed(egui::Key::Home) {
                self.current_page = 0;
            }

            // Page navigation with arrow keys when not in text input (no widget has focus)
            if !i.focused {
                if i.key_pressed(egui::Key::ArrowLeft) && i.modifiers.alt {
                    if self.current_page > 0 {
                        self.current_page -= 1;
                    }
                }
                if i.key_pressed(egui::Key::ArrowRight) && i.modifiers.alt {
                    let total_pages = self.calculate_total_pages();
                    if self.current_page < total_pages.saturating_sub(1) {
                        self.current_page += 1;
                    }
                }
            }
        });
    }

    /// Cycles through content types (for bumper button navigation)
    fn cycle_content_type(&mut self, forward: bool) {
        let content_order = [
            ContentType::LiveTV,
            ContentType::ContinueWatching,
            ContentType::Series,
            ContentType::Movies,
            ContentType::Favorites,
            ContentType::Discover,
            ContentType::FootballFixtures,
        ];

        let current_idx = content_order.iter()
            .position(|&c| c == self.current_content)
            .unwrap_or(0);

        let new_idx = if forward {
            (current_idx + 1) % content_order.len()
        } else {
            if current_idx == 0 { content_order.len() - 1 } else { current_idx - 1 }
        };

        let new_content = content_order[new_idx];

        // Load data if needed
        match new_content {
            ContentType::Series if self.all_series.is_empty() => {
                self.load_series();
            }
            ContentType::Movies if self.all_movies.is_empty() => {
                self.load_movies();
            }
            _ => {}
        }

        self.current_content = new_content;
        self.selected_category = None;
        self.filter_content();
    }

    /// Calculates total pages for current content
    fn calculate_total_pages(&self) -> usize {
        let items_count = match self.current_content {
            ContentType::LiveTV | ContentType::Favorites => self.filtered_channels.len(),
            ContentType::Series => self.filtered_series.len(),
            ContentType::Movies => self.filtered_movies.len(),
            _ => 0,
        };

        if items_count == 0 || self.page_size == 0 {
            1
        } else {
            (items_count + self.page_size - 1) / self.page_size
        }
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
                    AppMessage::ScraperStarted => {
                        self.scraping_in_progress = true;
                        self.scraper_message = Some("Scraping fixtures...".to_string());
                    }
                    AppMessage::ScraperCompleted(msg) => {
                        self.scraping_in_progress = false;
                        self.scraper_message = Some(msg);
                        self.football_cache.clear(); // Refresh the cache
                    }
                    AppMessage::ScraperFailed(err) => {
                        self.scraping_in_progress = false;
                        self.scraper_message = Some(format!("Error: {}", err));
                    }
                }
            }
        }
    }
    
    /// Triggers the football fixtures scraper in a background thread
    fn trigger_scraper(&mut self) {
        if self.scraping_in_progress {
            return; // Already running
        }
        
        // Need a valid sender to receive results
        let tx = match &self.tx {
            Some(tx) => tx.clone(),
            None => return, // No sender available
        };
        
        self.scraping_in_progress = true;
        self.scraper_message = Some("Starting scraper...".to_string());
        
        thread::spawn(move || {
            // Determine Python executable path (cross-platform)
            let python_path = if cfg!(windows) {
                "Soccer-Scraper-main\\.venv\\Scripts\\python.exe"
            } else {
                "Soccer-Scraper-main/.venv/bin/python"
            };
            
            // Try venv Python first, fall back to system Python
            let python_cmd = if std::path::Path::new(python_path).exists() {
                python_path.to_string()
            } else if cfg!(windows) {
                "python".to_string()
            } else {
                "python3".to_string()
            };
            
            let result = std::process::Command::new(&python_cmd)
                .args(["fixtures.py", "scrape"])
                .current_dir("Soccer-Scraper-main")
                .output();
            
            match result {
                Ok(output) => {
                    if output.status.success() {
                        let _ = tx.send(AppMessage::ScraperCompleted(
                            "Fixtures updated successfully!".to_string()
                        ));
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let _ = tx.send(AppMessage::ScraperFailed(
                            format!("Scraper failed: {}", stderr.lines().next().unwrap_or("Unknown error"))
                        ));
                    }
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::ScraperFailed(
                        format!("Failed to run scraper: {}. Ensure Python is installed.", e)
                    ));
                }
            }
        });
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // UI Rendering Helpers
    // ═══════════════════════════════════════════════════════════════════════
    

    
    /// Renders the main content area.
    fn render_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, theme: &Theme) {
        let padding = self.layout.content_padding();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::none()
                    .inner_margin(egui::Margin {
                        left: padding,
                        right: padding,
                        top: spacing::XL,
                        bottom: spacing::XXL,
                    })
                    .show(ui, |ui| {
                        // Keep pages made of complete rows for the current width
                        let width = ui.available_width();
                        let columns = match self.current_content {
                            ContentType::LiveTV | ContentType::Favorites => {
                                self.layout.wide_grid(width).columns
                            }
                            ContentType::Series | ContentType::Movies => {
                                self.layout.poster_grid(width).columns
                            }
                            _ => 1,
                        };
                        self.set_page_size(self.layout.page_size(columns));

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
            });
    }

    /// Renders a page heading.
    fn render_heading(&self, ui: &mut egui::Ui, theme: &Theme, title: &str) {
        ui.label(egui::RichText::new(title)
            .size(self.layout.heading_size())
            .color(theme.text_primary)
            .strong());
        ui.add_space(spacing::XS);
    }

    /// Renders a centered loading spinner with a message.
    fn render_loading(&self, ui: &mut egui::Ui, theme: &Theme, message: &str) {
        ui.vertical_centered(|ui| {
            ui.add_space(spacing::XXXL);
            ui.spinner();
            ui.add_space(spacing::SM);
            ui.label(egui::RichText::new(message).color(theme.text_secondary));
        });
    }

    /// Renders the continue watching section.
    fn render_continue_watching(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, theme: &Theme) {
        self.render_heading(ui, theme, self.current_content.title());
        let items = self.watch_history.get_continue_watching(50);
        ui.label(egui::RichText::new(format!("{} items", items.len()))
            .size(typography::CAPTION)
            .color(theme.text_tertiary));
        ui.add_space(spacing::LG);

        if items.is_empty() {
            self.render_empty_state(ui, theme);
            return;
        }

        let grid = self.layout.poster_grid(ui.available_width());
        let mut clicked_content: Option<crate::models::watch_history::WatchProgress> = None;

        grid.show(ui, &items, |ui, progress| {
            let card_width = grid.item_width;
            let poster_height = dimensions::poster_height(card_width);
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(card_width, poster_height + 64.0),
                egui::Sense::click(),
            );
            if response.clicked() {
                clicked_content = Some(progress.clone());
            }
            let highlighted = response.hovered() || response.has_focus();
            let poster_rect = egui::Rect::from_min_size(rect.min, egui::vec2(card_width, poster_height));
            let rounding = super::theme::radius::LG;

            if highlighted {
                ui.painter().rect_filled(
                    poster_rect.expand(3.0).translate(egui::vec2(0.0, 4.0)),
                    rounding,
                    theme.elevated_shadow(),
                );
            }

            // Poster or placeholder
            let texture = progress.thumbnail.as_ref().and_then(|url| {
                self.image_cache.load(ctx, url.clone());
                self.image_cache.get(url)
            });
            if let Some(texture) = texture {
                egui::Image::new(&texture)
                    .rounding(rounding)
                    .paint_at(ui, poster_rect);
            } else {
                ui.painter().rect_filled(poster_rect, rounding, theme.placeholder_bg());
                ui.painter().text(
                    poster_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    progress.content_type.to_uppercase(),
                    egui::FontId::proportional(typography::BODY_SM),
                    theme.placeholder_icon(),
                );
            }
            let border = if response.has_focus() { theme.accent_blue } else { theme.border_color };
            ui.painter().rect_stroke(poster_rect, rounding, egui::Stroke::new(1.0, border));

            // Resume button on hover
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
            let bar = egui::Rect::from_min_max(
                egui::pos2(poster_rect.min.x + spacing::SM, poster_rect.max.y - spacing::SM - 4.0),
                egui::pos2(poster_rect.max.x - spacing::SM, poster_rect.max.y - spacing::SM),
            );
            let pct = (progress.progress_percentage() / 100.0).clamp(0.0, 1.0) as f32;
            ui.painter().rect_filled(bar, 2.0, egui::Color32::from_white_alpha(60));
            ui.painter().rect_filled(
                egui::Rect::from_min_size(bar.min, egui::vec2(bar.width() * pct, bar.height())),
                2.0,
                theme.accent_blue,
            );

            // Title and details
            let title = super::components::single_line(
                ui,
                &progress.content_name,
                egui::FontId::proportional(typography::BODY_SM),
                theme.text_primary,
                card_width,
            );
            let title_y = poster_rect.max.y + spacing::SM;
            ui.painter().galley(egui::pos2(rect.min.x, title_y), title, theme.text_primary);

            let mins_left = ((progress.duration_seconds - progress.position_seconds) / 60.0).max(0.0) as i32;
            let detail = match (progress.season, progress.episode) {
                (Some(season), Some(episode)) => format!("S{}:E{} · {} min left", season, episode, mins_left),
                _ => format!("{} min left", mins_left),
            };
            let detail = super::components::single_line(
                ui,
                &detail,
                egui::FontId::proportional(typography::CAPTION),
                theme.text_tertiary,
                card_width,
            );
            ui.painter().galley(egui::pos2(rect.min.x, title_y + 20.0), detail, theme.text_tertiary);
        });

        // Handle resume playback
        if let Some(progress) = clicked_content {
            self.resume_playback(&progress.content_id, &progress.content_type, progress.season, progress.episode);
        }
    }

    /// Returns (start, end, total_pages) for the current page of `total_items`.
    fn page_bounds(&mut self, total_items: usize) -> (usize, usize, usize) {
        let total_pages = ((total_items + self.page_size - 1) / self.page_size).max(1);
        // The page size can change on resize - never point past the last page
        self.current_page = self.current_page.min(total_pages - 1);
        let start_idx = self.current_page * self.page_size;
        let end_idx = (start_idx + self.page_size).min(total_items);
        (start_idx, end_idx, total_pages)
    }

    /// Renders the channels grid.
    fn render_channels(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, theme: &Theme) {
        self.render_heading(ui, theme, self.current_content.title());

        // Process pending EPG data
        self.epg_cache.process_pending();

        let (start_idx, end_idx, total_pages) = self.page_bounds(self.filtered_channels.len());
        Pagination::show_info(ui, theme, start_idx, end_idx, self.filtered_channels.len(), "channels", self.current_page, total_pages);
        ui.add_space(spacing::LG);

        // Get current page items
        let page_channels: Vec<Channel> = self.filtered_channels[start_idx..end_idx].to_vec();

        // Request EPG data for visible channels
        for channel in &page_channels {
            self.epg_cache.request_epg_with_tvg(&channel.stream_id, channel.epg_channel_id.as_deref());
        }

        let favorites = self.config.favorites.clone();
        let mut channel_to_play: Option<Channel> = None;
        let mut channel_to_toggle: Option<String> = None;

        let grid = self.layout.wide_grid(ui.available_width());
        grid.show(ui, &page_channels, |ui, channel| {
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
                grid.item_width,
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
        });

        // Handle pagination
        if let Some(new_page) = Pagination::show(ui, theme, self.current_page, total_pages, &self.layout) {
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
        self.render_heading(ui, theme, self.current_content.title());

        if self.all_series.is_empty() {
            self.render_loading(ui, theme, "Loading series…");
            return;
        }

        let (start_idx, end_idx, total_pages) = self.page_bounds(self.filtered_series.len());
        Pagination::show_info(ui, theme, start_idx, end_idx, self.filtered_series.len(), "series", self.current_page, total_pages);
        ui.add_space(spacing::LG);

        let page_series: Vec<Series> = self.filtered_series[start_idx..end_idx].to_vec();
        let mut series_to_open: Option<i32> = None;

        let grid = self.layout.poster_grid(ui.available_width());
        grid.show(ui, &page_series, |ui, series| {
            if let Some(series_card::SeriesAction::ViewEpisodes(id)) =
                SeriesCard::show(ui, ctx, theme, series, &self.image_cache, grid.item_width)
            {
                series_to_open = Some(id);
            }
        });

        if let Some(id) = series_to_open {
            // Create new episode dialog state - data loads in background
            self.episode_dialog_state = Some(episode_dialog::EpisodeDialogState::new(
                id,
                self.server_url.clone(),
                self.username.clone(),
                self.password.clone(),
            ));
        }

        if let Some(new_page) = Pagination::show(ui, theme, self.current_page, total_pages, &self.layout) {
            self.current_page = new_page;
        }

        if self.filtered_series.is_empty() {
            self.render_empty_state(ui, theme);
        }
    }

    /// Renders the movies grid.
    fn render_movies(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, theme: &Theme) {
        self.render_heading(ui, theme, self.current_content.title());

        if self.all_movies.is_empty() {
            self.render_loading(ui, theme, "Loading movies…");
            return;
        }

        let (start_idx, end_idx, total_pages) = self.page_bounds(self.filtered_movies.len());
        Pagination::show_info(ui, theme, start_idx, end_idx, self.filtered_movies.len(), "movies", self.current_page, total_pages);
        ui.add_space(spacing::LG);

        let page_movies: Vec<serde_json::Value> = self.filtered_movies[start_idx..end_idx].to_vec();
        let mut movie_to_play: Option<(i64, String, String, Option<String>)> = None;

        let grid = self.layout.poster_grid(ui.available_width());
        grid.show(ui, &page_movies, |ui, movie| {
            if let Some(movie_card::MovieAction::Play { stream_id, name, container_extension, thumbnail }) =
                MovieCard::show(ui, ctx, theme, movie, &self.image_cache, grid.item_width)
            {
                movie_to_play = Some((stream_id, name, container_extension, thumbnail));
            }
        });

        if let Some(new_page) = Pagination::show(ui, theme, self.current_page, total_pages, &self.layout) {
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
        
        self.render_heading(ui, theme, self.current_content.title());
        ui.label(egui::RichText::new("Find upcoming matches and which channels broadcast them")
            .size(typography::CAPTION)
            .color(theme.text_tertiary));
        ui.add_space(spacing::MD);

        // Refresh button and status - always visible at the top
        ui.horizontal_wrapped(|ui| {
            let button_text = if self.scraping_in_progress {
                "⏳ Scraping..."
            } else {
                "🔄 Refresh Fixtures"
            };
            
            let button = egui::Button::new(
                egui::RichText::new(button_text)
                    .size(12.0)
                    .color(egui::Color32::WHITE)
            )
            .fill(if self.scraping_in_progress { theme.text_secondary } else { theme.accent_blue })
            .rounding(egui::Rounding::same(super::theme::radius::MD))
            .min_size(egui::vec2(130.0, self.layout.touch_target()));
            
            if ui.add_enabled(!self.scraping_in_progress, button).clicked() {
                self.trigger_scraper();
            }
            
            // Show status message
            if let Some(msg) = &self.scraper_message {
                ui.add_space(8.0);
                let color = if msg.starts_with("Error") {
                    theme.error_color
                } else if msg.contains("successfully") {
                    theme.success_color
                } else {
                    theme.text_secondary
                };
                ui.label(egui::RichText::new(msg).size(11.0).color(color));
            }
        });
        
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
                ui.label(egui::RichText::new("Click 'Refresh Fixtures' above to fetch upcoming matches")
                    .size(12.0)
                    .color(theme.text_secondary));
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
                
                // Display fixtures grouped by date (the page itself scrolls)
                {
                    for date in dates.iter() {
                        if let Some(date_fixtures) = fixtures_by_date.get(date) {
                            // Date header with better styling
                            ui.add_space(16.0);

                            egui::Frame::none()
                                .fill(theme.card_bg)
                                .stroke(egui::Stroke::new(1.0, theme.border_color))
                                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                                .rounding(egui::Rounding::same(super::theme::radius::MD))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("📅")
                                            .size(16.0)
                                            .color(theme.accent_blue));
                                        ui.add_space(6.0);
                                        ui.label(egui::RichText::new(date)
                                            .size(16.0)
                                            .color(theme.text_primary)
                                            .strong());
                                    });
                                });

                            ui.add_space(12.0);

                            // Display fixtures as a simple list
                            for fixture in date_fixtures.iter().take(30) {
                                if let Some(action) = FootballCard::show(
                                    ui,
                                    theme,
                                    fixture,
                                    self.layout.width,
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
                        }
                    }

                }
                
                // Handle search action - switch to Live TV and search
                if let Some(query) = channel_to_search {
                    self.search_query = query;
                    self.current_content = ContentType::LiveTV;
                    self.current_page = 0;
                    self.filter_content();
                }
            }
        } else if is_loading {
            self.render_loading(ui, theme, "Loading fixtures…");
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
        self.render_heading(ui, theme, "🔥 Discover");
        ui.label(egui::RichText::new("Curated trending and popular titles")
            .size(typography::CAPTION)
            .color(theme.text_tertiary));
        ui.add_space(spacing::MD);

        // Category selector (also available in the sidebar on wide screens)
        ui.horizontal_wrapped(|ui| {
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
            let grid = self.layout.poster_grid(ui.available_width());
            grid.show(ui, &display_items, |ui, item| {
                // Only load images for items being displayed
                if let Some(poster_url) = &item.poster_url {
                    self.image_cache.load(ctx, poster_url.clone());
                }

                if let Some(DiscoverAction::SearchInIptv(query)) =
                    DiscoverCard::show(ui, theme, item, &self.image_cache, grid.item_width)
                {
                    search_query = Some(query);
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
            self.render_loading(ui, theme, "Loading content…");
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

impl IPTVPlayerApp {
    /// Renders the sidebar contents: categories, or filters for Discover/Sports.
    fn render_sidebar(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        let item_height = if self.layout.touch { 48.0 } else { 40.0 };
        let mut selected = false;

        match self.current_content {
            ContentType::LiveTV | ContentType::Favorites | ContentType::Series | ContentType::Movies => {
                let categories = match self.current_content {
                    ContentType::Series => &self.series_categories,
                    ContentType::Movies => &self.movie_categories,
                    _ => &self.live_categories,
                };
                if let Some(selection) = CategorySidebar::show(
                    ui,
                    theme,
                    item_height,
                    categories,
                    &self.selected_category,
                    &mut self.category_search,
                ) {
                    self.selected_category = selection;
                    self.filter_content();
                    selected = true;
                }
            }
            ContentType::FootballFixtures => {
                ui.add_space(spacing::SM);
                CategorySidebar::header(ui, theme, "Fixtures");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = spacing::XS;
                    for category in FootballCategory::all() {
                        let is_selected = self.football_category == *category;
                        if CategorySidebar::category_button(ui, theme, category.display_name(), is_selected, item_height, typography::BODY_SM).clicked() {
                            self.football_category = *category;
                            selected = true;
                        }
                    }
                    ui.add_space(spacing::LG);

                    // Refresh button - runs the scraper
                    let btn_text = if self.scraping_in_progress { "⏳ Scraping…" } else { "🔄 Refresh fixtures" };
                    let button = egui::Button::new(
                        egui::RichText::new(btn_text)
                            .size(typography::BODY_SM)
                            .color(egui::Color32::WHITE)
                    )
                    .fill(if self.scraping_in_progress { theme.text_tertiary } else { theme.accent_blue })
                    .rounding(egui::Rounding::same(super::theme::radius::MD))
                    .min_size(egui::vec2(ui.available_width(), item_height));

                    if ui.add_enabled(!self.scraping_in_progress, button).clicked() {
                        self.trigger_scraper();
                    }
                });
            }
            ContentType::Discover => {
                ui.add_space(spacing::SM);
                CategorySidebar::header(ui, theme, "Discover");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = spacing::XS;
                    for category in DiscoverCategory::all() {
                        let is_selected = self.discover_category == *category;
                        if CategorySidebar::category_button(ui, theme, category.display_name(), is_selected, item_height, typography::BODY_SM).clicked() {
                            self.discover_category = *category;
                            selected = true;
                        }
                    }
                });
            }
            ContentType::ContinueWatching => {}
        }

        // Selecting something in the drawer dismisses it
        if selected {
            self.drawer_open = false;
        }
    }

    /// Renders the slide-over category drawer used on narrow layouts.
    fn render_drawer(&mut self, ctx: &egui::Context, theme: &Theme) {
        let screen = ctx.screen_rect();

        // Dimmed scrim - tapping it closes the drawer
        egui::Area::new(egui::Id::new("drawer_scrim"))
            .order(egui::Order::Middle)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let (rect, response) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
                ui.painter().rect_filled(rect, 0.0, egui::Color32::from_black_alpha(150));
                if response.clicked() {
                    self.drawer_open = false;
                }
            });

        let drawer_rect = egui::Rect::from_min_size(
            screen.min,
            egui::vec2(self.layout.sidebar_width(), screen.height()),
        );
        egui::Area::new(egui::Id::new("drawer"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                ui.painter().rect_filled(drawer_rect, 0.0, theme.panel_bg);
                ui.painter().vline(
                    drawer_rect.max.x,
                    drawer_rect.y_range(),
                    egui::Stroke::new(1.0, theme.border_color),
                );
                let inner = drawer_rect.shrink(spacing::LG);
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
                    ui.set_min_size(inner.size());
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(self.current_content.title())
                            .size(typography::H3)
                            .color(theme.text_primary)
                            .strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let size = self.layout.touch_target();
                            let close = egui::Button::new(egui::RichText::new("🗙").size(18.0).color(theme.text_secondary))
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE)
                                .min_size(egui::vec2(size, size));
                            if ui.add(close).on_hover_text("Close").clicked() {
                                self.drawer_open = false;
                            }
                        });
                    });
                    ui.add_space(spacing::SM);
                    self.render_sidebar(ui, theme);
                });
            });
    }

    /// Applies a navigation bar action.
    fn handle_nav_action(&mut self, action: top_nav::NavAction) {
        match action {
            top_nav::NavAction::SwitchContent(content_type) => {
                self.current_content = content_type;
                self.selected_category = None;
                self.drawer_open = false;
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
            top_nav::NavAction::OpenEpgSettings => {
                self.temp_epg_enabled = self.config.epg_enabled;
                self.temp_epg_url = self.config.epg_url.clone().unwrap_or_default();
                self.show_epg_settings = true;
            }
            top_nav::NavAction::OpenScraperSettings => {
                self.show_scraper_settings = true;
            }
            top_nav::NavAction::ToggleSidebar => {
                self.toggle_sidebar();
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

        // Ensure Steam Deck scaling is maintained (Gamescope may try to reset it)
        if self.is_steam_deck_mode {
            if (ctx.pixels_per_point() - 1.0).abs() > 0.001 {
                ctx.set_pixels_per_point(1.0);
            }
        }

        // Process background messages
        self.process_messages();

        // Update responsive layout for the current window size
        let screen = ctx.screen_rect();
        self.layout = Layout::new(screen.width(), self.is_steam_deck_mode);
        if self.layout.docked_sidebar() {
            self.drawer_open = false;
        }

        // Steam Deck / Gamepad input handling
        // Enable keyboard navigation for better controller support
        self.handle_gamepad_input(ctx);

        if !self.connected {
            // Show login screen
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if LoginScreen::show(
                        ui,
                        &theme,
                        &mut self.server_url,
                        &mut self.username,
                        &mut self.password,
                        self.connecting,
                        &self.error_message,
                        self.layout.touch,
                    ) {
                        self.connect();
                    }
                });
            });
        } else {
            // Show main application
            let layout = self.layout;
            let has_sidebar = self.has_sidebar();

            // Phones: primary navigation lives in a bottom tab bar
            if layout.is_compact() {
                egui::TopBottomPanel::bottom("bottom_nav")
                    .frame(egui::Frame::none()
                        .fill(theme.panel_bg)
                        .stroke(egui::Stroke::new(1.0, theme.border_color))
                        .inner_margin(egui::Margin::symmetric(0.0, spacing::XS)))
                    .show(ctx, |ui| {
                        if let Some(action) = TopNavigation::bottom_bar(ui, &theme, self.current_content) {
                            self.handle_nav_action(action);
                        }
                    });
            }

            // Wide layouts dock the sidebar beside the content
            if layout.docked_sidebar() && self.sidebar_visible && has_sidebar {
                egui::SidePanel::left("categories")
                    .resizable(false)
                    .exact_width(layout.sidebar_width())
                    .frame(egui::Frame::none()
                        .fill(theme.panel_bg)
                        .stroke(egui::Stroke::new(1.0, theme.border_color))
                        .inner_margin(egui::Margin::same(spacing::LG)))
                    .show(ctx, |ui| {
                        self.render_sidebar(ui, &theme);
                    });
            }

            // Main content area
            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(theme.bg_color))
                .show(ctx, |ui| {
                    let padding = layout.content_padding();
                    egui::TopBottomPanel::top("top_panel")
                        .frame(egui::Frame::none()
                            .fill(theme.panel_bg)
                            .stroke(egui::Stroke::new(1.0, theme.border_color))
                            .inner_margin(egui::Margin::symmetric(padding, if layout.touch { spacing::MD } else { spacing::LG })))
                        .show_inside(ui, |ui| {
                            if let Some(action) = TopNavigation::show(
                                ui,
                                &theme,
                                self.current_content,
                                &mut self.search_query,
                                &layout,
                                has_sidebar,
                            ) {
                                self.handle_nav_action(action);
                            }
                        });

                    // Content area
                    self.render_content(ui, ctx, &theme);
                });

            // Narrow layouts show the sidebar as a slide-over drawer
            if self.drawer_open && has_sidebar {
                self.render_drawer(ctx, &theme);
            }

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

            // EPG settings dialog
            if self.show_epg_settings {
                if let Some(action) = EpgSettingsDialog::show(
                    ctx,
                    &mut self.temp_epg_enabled,
                    &mut self.temp_epg_url,
                ) {
                    match action {
                        epg_settings::EpgSettingsAction::Saved => {
                            self.config.epg_enabled = self.temp_epg_enabled;
                            self.config.epg_url = if self.temp_epg_url.is_empty() {
                                None
                            } else {
                                Some(self.temp_epg_url.clone())
                            };
                            let _ = self.config.save();

                            // Update EPG cache with new settings
                            if self.config.epg_enabled {
                                self.epg_cache.set_xmltv_url(self.config.epg_url.clone());
                            } else {
                                self.epg_cache.set_xmltv_url(None);
                            }

                            self.show_epg_settings = false;
                        }
                        epg_settings::EpgSettingsAction::Cancelled => {
                            self.show_epg_settings = false;
                        }
                    }
                }
            }

            // Scraper settings dialog
            if self.show_scraper_settings {
                let mut should_close = false;
                let mut should_trigger = false;
                egui::Window::new("Football Fixtures Scraper")
                    .open(&mut self.show_scraper_settings)
                    .resizable(true)
                    .collapsible(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .default_width((400.0f32).min(self.layout.width - 48.0))
                    .max_width(self.layout.width - 48.0)
                    .show(ctx, |ui| {
                        // Show current scraping status
                        ui.heading("📊 Football Fixtures Scraper");
                        ui.separator();
                        
                        if !self.scraper_manager.is_available() {
                            ui.label(egui::RichText::new("⚠ Soccer Scraper not found")
                                .color(theme.warning_color));
                            ui.label("Ensure Soccer-Scraper-main directory exists in the app folder");
                        } else if self.scraping_in_progress {
                            ui.label(egui::RichText::new("⏳ Scraping in progress...")
                                .color(theme.accent_blue));
                            ui.label("Opening Chrome browser to bypass Cloudflare...");
                            ui.label("Please wait, this may take 2 minutes");
                        } else {
                            ui.label("✅ Scraper ready");
                            ui.label("Click below to download today's fixtures from LiveSoccerTV");
                            
                            ui.add_space(8.0);
                            if ui.button("🔄 Scrape Fixtures Now").clicked() {
                                should_trigger = true;
                            }
                            ui.label("⏱ Takes approximately 2 minutes");
                        }
                        
                        // Show last message
                        if let Some(msg) = &self.scraper_message {
                            ui.add_space(8.0);
                            let color = if msg.starts_with("Error") {
                                theme.error_color
                            } else if msg.contains("successfully") {
                                theme.success_color
                            } else {
                                theme.text_secondary
                            };
                            ui.label(egui::RichText::new(msg).color(color));
                        }
                        
                        ui.separator();
                        if ui.button("Close").clicked() {
                            should_close = true;
                        }
                    });
                
                if should_trigger {
                    self.trigger_scraper();
                }
                if should_close {
                    self.show_scraper_settings = false;
                }
            }
        }
    }
}
