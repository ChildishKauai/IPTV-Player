use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use std::thread;

/// TMDB API base URL
const TMDB_API_BASE: &str = "https://api.themoviedb.org/3";
/// TMDB image base URL
const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p";

/// TMDB API client for fetching popular and trending content.
#[derive(Clone)]
pub struct TmdbClient {
    api_key: String,
    client: reqwest::blocking::Client,
}

/// Movie result from TMDB API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbMovie {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub original_title: String,
    #[serde(default)]
    pub overview: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    #[serde(default)]
    pub release_date: String,
    #[serde(default)]
    pub vote_average: f64,
    #[serde(default)]
    pub vote_count: i64,
    #[serde(default)]
    pub popularity: f64,
    #[serde(default)]
    pub genre_ids: Vec<i32>,
    #[serde(default)]
    pub adult: bool,
}

/// TV show result from TMDB API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbTvShow {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub original_name: String,
    #[serde(default)]
    pub overview: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    #[serde(default)]
    pub first_air_date: String,
    #[serde(default)]
    pub vote_average: f64,
    #[serde(default)]
    pub vote_count: i64,
    #[serde(default)]
    pub popularity: f64,
    #[serde(default)]
    pub genre_ids: Vec<i32>,
}

/// Generic paginated response from TMDB.
#[derive(Debug, Clone, Deserialize)]
pub struct TmdbResponse<T> {
    pub page: i32,
    pub results: Vec<T>,
    pub total_pages: i32,
    pub total_results: i32,
}

/// Content type for TMDB items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TmdbContentType {
    Movie,
    TvShow,
}

/// Unified TMDB item that can be either a movie or TV show.
#[derive(Debug, Clone)]
pub struct TmdbItem {
    pub id: i64,
    pub title: String,
    pub overview: String,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub release_date: String,
    pub vote_average: f64,
    pub content_type: TmdbContentType,
}

impl TmdbMovie {
    /// Convert to unified TmdbItem.
    pub fn to_item(&self) -> TmdbItem {
        TmdbItem {
            id: self.id,
            title: self.title.clone(),
            overview: self.overview.clone(),
            poster_url: self.poster_path.as_ref().map(|p| format!("{}/w342{}", TMDB_IMAGE_BASE, p)),
            backdrop_url: self.backdrop_path.as_ref().map(|p| format!("{}/w780{}", TMDB_IMAGE_BASE, p)),
            release_date: self.release_date.clone(),
            vote_average: self.vote_average,
            content_type: TmdbContentType::Movie,
        }
    }
}

impl TmdbTvShow {
    /// Convert to unified TmdbItem.
    pub fn to_item(&self) -> TmdbItem {
        TmdbItem {
            id: self.id,
            title: self.name.clone(),
            overview: self.overview.clone(),
            poster_url: self.poster_path.as_ref().map(|p| format!("{}/w342{}", TMDB_IMAGE_BASE, p)),
            backdrop_url: self.backdrop_path.as_ref().map(|p| format!("{}/w780{}", TMDB_IMAGE_BASE, p)),
            release_date: self.first_air_date.clone(),
            vote_average: self.vote_average,
            content_type: TmdbContentType::TvShow,
        }
    }
}

impl TmdbItem {
    /// Get the year from the release date.
    pub fn year(&self) -> Option<String> {
        if self.release_date.len() >= 4 {
            Some(self.release_date[..4].to_string())
        } else {
            None
        }
    }
    
    /// Get a search query string for finding this content in IPTV.
    pub fn search_query(&self) -> String {
        self.title.clone()
    }
    
    /// Get content type display name.
    pub fn content_type_name(&self) -> &'static str {
        match self.content_type {
            TmdbContentType::Movie => "Movie",
            TmdbContentType::TvShow => "TV Show",
        }
    }
}

/// Category of trending/popular content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum TmdbCategory {
    #[default]
    TrendingAll,
    TrendingMovies,
    TrendingTv,
    PopularMovies,
    PopularTv,
    TopRatedMovies,
    TopRatedTv,
    NowPlayingMovies,
    AiringTodayTv,
}

impl TmdbCategory {
    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            TmdbCategory::TrendingAll => "🔥 Trending",
            TmdbCategory::TrendingMovies => "🔥 Trending Movies",
            TmdbCategory::TrendingTv => "🔥 Trending TV",
            TmdbCategory::PopularMovies => "⭐ Popular Movies",
            TmdbCategory::PopularTv => "⭐ Popular TV",
            TmdbCategory::TopRatedMovies => "🏆 Top Rated Movies",
            TmdbCategory::TopRatedTv => "🏆 Top Rated TV",
            TmdbCategory::NowPlayingMovies => "🎬 Now Playing",
            TmdbCategory::AiringTodayTv => "📺 Airing Today",
        }
    }
    
    /// Get all categories.
    pub fn all() -> &'static [TmdbCategory] {
        &[
            TmdbCategory::TrendingAll,
            TmdbCategory::TrendingMovies,
            TmdbCategory::TrendingTv,
            TmdbCategory::PopularMovies,
            TmdbCategory::PopularTv,
            TmdbCategory::TopRatedMovies,
            TmdbCategory::TopRatedTv,
            TmdbCategory::NowPlayingMovies,
            TmdbCategory::AiringTodayTv,
        ]
    }
}

impl TmdbClient {
    /// Create a new TMDB client with the given API key.
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                // WARNING: Disabling certificate verification to work around
                // network proxies/firewalls that intercept HTTPS traffic.
                // This is NOT recommended for production use.
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap_or_default(),
        }
    }
    
    /// Make a GET request to the TMDB API.
    fn get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T, String> {
        let url = if endpoint.contains('?') {
            format!("{}/{}&api_key={}", TMDB_API_BASE, endpoint, self.api_key)
        } else {
            format!("{}/{}?api_key={}", TMDB_API_BASE, endpoint, self.api_key)
        };
        
        let response = self.client.get(&url)
            .send()
            .map_err(|e| {
                // Check for common error types
                let err_str = e.to_string();
                if err_str.contains("certificate") {
                    "Network proxy blocking connection. Try using a VPN.".to_string()
                } else if err_str.contains("Connect") || err_str.contains("timeout") {
                    "Unable to connect to TMDB. Check your internet connection.".to_string()
                } else {
                    format!("Connection error: {}", e)
                }
            })?;
        
        let status = response.status();
        if !status.is_success() {
            // Try to get the response body for debugging
            let body = response.text().unwrap_or_default();
            
            // Check if this is a proxy block (HTML response instead of JSON)
            if body.contains("<!DOCTYPE") || body.contains("<html") {
                return Err("TMDB API blocked by network. Try using a VPN or mobile hotspot.".to_string());
            }
            
            // Try to parse TMDB error response
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(msg) = error_json["status_message"].as_str() {
                    return Err(format!("TMDB: {}", msg));
                }
            }
            
            return Err(format!("API error: {}", status));
        }
        
        response.json::<T>()
            .map_err(|e| format!("Failed to parse response: {}", e))
    }
    
    /// Get trending content (movies and TV shows).
    pub fn get_trending_all(&self, page: i32) -> Result<Vec<TmdbItem>, String> {
        // Trending endpoint returns mixed results, we need to handle both types
        let url = format!("trending/all/day?page={}", page);
        let response: serde_json::Value = self.get(&url)?;
        
        let results = response["results"].as_array()
            .ok_or("Invalid response format")?;
        
        let items: Vec<TmdbItem> = results.iter().take(20).filter_map(|item| {
            let media_type = item["media_type"].as_str()?;
            match media_type {
                "movie" => {
                    let movie: TmdbMovie = serde_json::from_value(item.clone()).ok()?;
                    Some(movie.to_item())
                }
                "tv" => {
                    let tv: TmdbTvShow = serde_json::from_value(item.clone()).ok()?;
                    Some(tv.to_item())
                }
                _ => None,
            }
        }).collect();
        
        Ok(items)
    }
    
    /// Get trending movies.
    pub fn get_trending_movies(&self, page: i32) -> Result<Vec<TmdbItem>, String> {
        let url = format!("trending/movie/day?page={}", page);
        let response: TmdbResponse<TmdbMovie> = self.get(&url)?;
        Ok(response.results.into_iter().take(20).map(|m| m.to_item()).collect())
    }
    
    /// Get trending TV shows.
    pub fn get_trending_tv(&self, page: i32) -> Result<Vec<TmdbItem>, String> {
        let url = format!("trending/tv/day?page={}", page);
        let response: TmdbResponse<TmdbTvShow> = self.get(&url)?;
        Ok(response.results.into_iter().take(20).map(|t| t.to_item()).collect())
    }
    
    /// Get popular movies.
    pub fn get_popular_movies(&self, page: i32) -> Result<Vec<TmdbItem>, String> {
        let url = format!("movie/popular?page={}", page);
        let response: TmdbResponse<TmdbMovie> = self.get(&url)?;
        Ok(response.results.into_iter().take(20).map(|m| m.to_item()).collect())
    }
    
    /// Get popular TV shows.
    pub fn get_popular_tv(&self, page: i32) -> Result<Vec<TmdbItem>, String> {
        let url = format!("tv/popular?page={}", page);
        let response: TmdbResponse<TmdbTvShow> = self.get(&url)?;
        Ok(response.results.into_iter().take(20).map(|t| t.to_item()).collect())
    }
    
    /// Get top rated movies.
    pub fn get_top_rated_movies(&self, page: i32) -> Result<Vec<TmdbItem>, String> {
        let url = format!("movie/top_rated?page={}", page);
        let response: TmdbResponse<TmdbMovie> = self.get(&url)?;
        Ok(response.results.into_iter().take(20).map(|m| m.to_item()).collect())
    }
    
    /// Get top rated TV shows.
    pub fn get_top_rated_tv(&self, page: i32) -> Result<Vec<TmdbItem>, String> {
        let url = format!("tv/top_rated?page={}", page);
        let response: TmdbResponse<TmdbTvShow> = self.get(&url)?;
        Ok(response.results.into_iter().take(20).map(|t| t.to_item()).collect())
    }
    
    /// Get now playing movies.
    pub fn get_now_playing_movies(&self, page: i32) -> Result<Vec<TmdbItem>, String> {
        let url = format!("movie/now_playing?page={}", page);
        let response: TmdbResponse<TmdbMovie> = self.get(&url)?;
        Ok(response.results.into_iter().take(20).map(|m| m.to_item()).collect())
    }
    
    /// Get TV shows airing today.
    pub fn get_airing_today_tv(&self, page: i32) -> Result<Vec<TmdbItem>, String> {
        let url = format!("tv/airing_today?page={}", page);
        let response: TmdbResponse<TmdbTvShow> = self.get(&url)?;
        Ok(response.results.into_iter().take(20).map(|t| t.to_item()).collect())
    }
    
    /// Get content by category.
    pub fn get_by_category(&self, category: TmdbCategory, page: i32) -> Result<Vec<TmdbItem>, String> {
        match category {
            TmdbCategory::TrendingAll => self.get_trending_all(page),
            TmdbCategory::TrendingMovies => self.get_trending_movies(page),
            TmdbCategory::TrendingTv => self.get_trending_tv(page),
            TmdbCategory::PopularMovies => self.get_popular_movies(page),
            TmdbCategory::PopularTv => self.get_popular_tv(page),
            TmdbCategory::TopRatedMovies => self.get_top_rated_movies(page),
            TmdbCategory::TopRatedTv => self.get_top_rated_tv(page),
            TmdbCategory::NowPlayingMovies => self.get_now_playing_movies(page),
            TmdbCategory::AiringTodayTv => self.get_airing_today_tv(page),
        }
    }
    
    /// Search for movies.
    pub fn search_movies(&self, query: &str, page: i32) -> Result<Vec<TmdbItem>, String> {
        let encoded_query = urlencoding::encode(query);
        let url = format!("search/movie?query={}&page={}", encoded_query, page);
        let response: TmdbResponse<TmdbMovie> = self.get(&url)?;
        Ok(response.results.into_iter().map(|m| m.to_item()).collect())
    }
    
    /// Search for TV shows.
    pub fn search_tv(&self, query: &str, page: i32) -> Result<Vec<TmdbItem>, String> {
        let encoded_query = urlencoding::encode(query);
        let url = format!("search/tv?query={}&page={}", encoded_query, page);
        let response: TmdbResponse<TmdbTvShow> = self.get(&url)?;
        Ok(response.results.into_iter().map(|t| t.to_item()).collect())
    }
    
    /// Search for both movies and TV shows.
    pub fn search_multi(&self, query: &str, page: i32) -> Result<Vec<TmdbItem>, String> {
        let encoded_query = urlencoding::encode(query);
        let url = format!("search/multi?query={}&page={}", encoded_query, page);
        let response: serde_json::Value = self.get(&url)?;
        
        let results = response["results"].as_array()
            .ok_or("Invalid response format")?;
        
        let items: Vec<TmdbItem> = results.iter().filter_map(|item| {
            let media_type = item["media_type"].as_str()?;
            match media_type {
                "movie" => {
                    let movie: TmdbMovie = serde_json::from_value(item.clone()).ok()?;
                    Some(movie.to_item())
                }
                "tv" => {
                    let tv: TmdbTvShow = serde_json::from_value(item.clone()).ok()?;
                    Some(tv.to_item())
                }
                _ => None,
            }
        }).collect();
        
        Ok(items)
    }
}

/// Message for async TMDB data loading.
pub enum TmdbMessage {
    CategoryLoaded(TmdbCategory, Vec<TmdbItem>),
    CategoryError(TmdbCategory, String),
    SearchResults(String, Vec<TmdbItem>),
    #[allow(dead_code)]
    Error(String),
}

/// Cache for TMDB data with background loading.
pub struct TmdbCache {
    api_key: Option<String>,
    cache: std::collections::HashMap<TmdbCategory, Vec<TmdbItem>>,
    search_cache: std::collections::HashMap<String, Vec<TmdbItem>>,
    pending_requests: std::collections::HashSet<TmdbCategory>,
    pending_searches: std::collections::HashSet<String>,
    sender: mpsc::Sender<TmdbMessage>,
    receiver: mpsc::Receiver<TmdbMessage>,
    last_fetch: std::collections::HashMap<TmdbCategory, std::time::Instant>,
    /// Track failed requests with cooldown to prevent infinite retries
    failed_requests: std::collections::HashMap<TmdbCategory, std::time::Instant>,
    /// Last error message for display
    pub last_error: Option<String>,
}

impl TmdbCache {
    /// Create a new TMDB cache.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            api_key: None,
            cache: std::collections::HashMap::new(),
            search_cache: std::collections::HashMap::new(),
            pending_requests: std::collections::HashSet::new(),
            pending_searches: std::collections::HashSet::new(),
            sender,
            receiver,
            last_fetch: std::collections::HashMap::new(),
            failed_requests: std::collections::HashMap::new(),
            last_error: None,
        }
    }
    
    /// Set the API key.
    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = Some(api_key);
    }
    
    /// Check if API key is configured.
    pub fn has_api_key(&self) -> bool {
        self.api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false)
    }
    
    /// Request content for a category.
    pub fn request_category(&mut self, category: TmdbCategory) {
        if !self.has_api_key() {
            return;
        }
        
        // Check if we already have fresh data (less than 5 minutes old)
        if let Some(last_fetch) = self.last_fetch.get(&category) {
            if last_fetch.elapsed() < std::time::Duration::from_secs(300) {
                return;
            }
        }
        
        // Check if this category recently failed (30 second cooldown)
        if let Some(failed_time) = self.failed_requests.get(&category) {
            if failed_time.elapsed() < std::time::Duration::from_secs(30) {
                return; // Don't retry failed requests too quickly
            }
        }
        
        // Check if already pending
        if self.pending_requests.contains(&category) {
            return;
        }
        
        self.pending_requests.insert(category);
        
        let api_key = self.api_key.clone().unwrap();
        let sender = self.sender.clone();
        
        thread::spawn(move || {
            let client = TmdbClient::new(api_key);
            match client.get_by_category(category, 1) {
                Ok(items) => {
                    let _ = sender.send(TmdbMessage::CategoryLoaded(category, items));
                }
                Err(e) => {
                    let _ = sender.send(TmdbMessage::CategoryError(category, e));
                }
            }
        });
    }
    
    /// Request search results.
    pub fn request_search(&mut self, query: String) {
        if !self.has_api_key() || query.is_empty() {
            return;
        }
        
        // Check if already cached
        if self.search_cache.contains_key(&query) {
            return;
        }
        
        // Check if already pending
        if self.pending_searches.contains(&query) {
            return;
        }
        
        self.pending_searches.insert(query.clone());
        
        let api_key = self.api_key.clone().unwrap();
        let sender = self.sender.clone();
        let search_query = query.clone();
        
        thread::spawn(move || {
            let client = TmdbClient::new(api_key);
            match client.search_multi(&search_query, 1) {
                Ok(items) => {
                    let _ = sender.send(TmdbMessage::SearchResults(search_query, items));
                }
                Err(e) => {
                    let _ = sender.send(TmdbMessage::Error(e));
                }
            }
        });
    }
    
    /// Process pending results.
    pub fn process_pending(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                TmdbMessage::CategoryLoaded(category, items) => {
                    self.pending_requests.remove(&category);
                    self.cache.insert(category, items);
                    self.last_fetch.insert(category, std::time::Instant::now());
                    self.failed_requests.remove(&category); // Clear failure on success
                    self.last_error = None; // Clear error on success
                }
                TmdbMessage::CategoryError(category, e) => {
                    self.pending_requests.remove(&category);
                    self.failed_requests.insert(category, std::time::Instant::now());
                    self.last_error = Some(e);
                }
                TmdbMessage::SearchResults(query, items) => {
                    self.pending_searches.remove(&query);
                    self.search_cache.insert(query, items);
                }
                TmdbMessage::Error(e) => {
                    self.last_error = Some(e);
                }
            }
        }
    }
    
    /// Get cached content for a category.
    pub fn get_category(&self, category: TmdbCategory) -> Option<&Vec<TmdbItem>> {
        self.cache.get(&category)
    }
    
    /// Get cached search results.
    pub fn get_search_results(&self, query: &str) -> Option<&Vec<TmdbItem>> {
        self.search_cache.get(query)
    }
    
    /// Check if a category is loading.
    pub fn is_loading(&self, category: TmdbCategory) -> bool {
        self.pending_requests.contains(&category)
    }
    
    /// Check if a search is loading.
    pub fn is_search_loading(&self, query: &str) -> bool {
        self.pending_searches.contains(query)
    }
    
    /// Clear all cached data.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.search_cache.clear();
        self.last_fetch.clear();
        self.pending_requests.clear();
        self.failed_requests.clear();
        self.last_error = None;
    }
}

impl Default for TmdbCache {
    fn default() -> Self {
        Self::new()
    }
}
