// Trakt.tv API client for trending/popular content discovery
// API Documentation: https://trakt.docs.apiary.io/

use std::collections::HashMap;
use std::sync::mpsc;

const TRAKT_API_URL: &str = "https://api.trakt.tv";
const TRAKT_CLIENT_ID: &str = "0e1b952f5bf29f4cfb9fb54a86f498bfc0e2f7ef7e4fd9c0dca5f4f8c0c4d5e6"; // Public demo key

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TraktIds {
    pub trakt: Option<i64>,
    pub slug: Option<String>,
    pub imdb: Option<String>,
    pub tmdb: Option<i64>,
    pub tvdb: Option<i64>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TraktShow {
    pub title: String,
    pub year: Option<i32>,
    pub ids: TraktIds,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub runtime: Option<i32>,
    #[serde(default)]
    pub certification: Option<String>,
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub trailer: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(default)]
    pub votes: Option<i64>,
    #[serde(default)]
    pub genres: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TraktMovie {
    pub title: String,
    pub year: Option<i32>,
    pub ids: TraktIds,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub runtime: Option<i32>,
    #[serde(default)]
    pub certification: Option<String>,
    #[serde(default)]
    pub trailer: Option<String>,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(default)]
    pub votes: Option<i64>,
    #[serde(default)]
    pub genres: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TraktTrendingShow {
    pub watchers: i64,
    pub show: TraktShow,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TraktTrendingMovie {
    pub watchers: i64,
    pub movie: TraktMovie,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TraktPopularShow {
    #[serde(flatten)]
    pub show: TraktShow,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TraktPopularMovie {
    #[serde(flatten)]
    pub movie: TraktMovie,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TraktAnticipatedShow {
    pub list_count: i64,
    pub show: TraktShow,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TraktAnticipatedMovie {
    pub list_count: i64,
    pub movie: TraktMovie,
}

// Unified item for display
#[derive(Debug, Clone)]
pub struct DiscoverItem {
    pub id: i64,
    pub title: String,
    pub year: Option<i32>,
    pub overview: String,
    pub rating: Option<f64>,
    pub votes: Option<i64>,
    pub poster_url: Option<String>,
    pub content_type: DiscoverContentType,
    pub watchers: Option<i64>, // For trending
    pub genres: Vec<String>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoverContentType {
    TvShow,
    Movie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiscoverCategory {
    TrendingShows,
    TrendingMovies,
    PopularShows,
    PopularMovies,
    AnticipatedShows,
    AnticipatedMovies,
}

impl DiscoverCategory {
    pub fn all() -> &'static [DiscoverCategory] {
        &[
            DiscoverCategory::TrendingShows,
            DiscoverCategory::TrendingMovies,
            DiscoverCategory::PopularShows,
            DiscoverCategory::PopularMovies,
            DiscoverCategory::AnticipatedShows,
            DiscoverCategory::AnticipatedMovies,
        ]
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            DiscoverCategory::TrendingShows => "🔥 Trending Shows",
            DiscoverCategory::TrendingMovies => "🔥 Trending Movies",
            DiscoverCategory::PopularShows => "⭐ Popular Shows",
            DiscoverCategory::PopularMovies => "⭐ Popular Movies",
            DiscoverCategory::AnticipatedShows => "📅 Anticipated Shows",
            DiscoverCategory::AnticipatedMovies => "📅 Anticipated Movies",
        }
    }
    
    pub fn endpoint(&self) -> &'static str {
        match self {
            DiscoverCategory::TrendingShows => "/shows/trending",
            DiscoverCategory::TrendingMovies => "/movies/trending",
            DiscoverCategory::PopularShows => "/shows/popular",
            DiscoverCategory::PopularMovies => "/movies/popular",
            DiscoverCategory::AnticipatedShows => "/shows/anticipated",
            DiscoverCategory::AnticipatedMovies => "/movies/anticipated",
        }
    }
    
    pub fn is_movie(&self) -> bool {
        matches!(self, 
            DiscoverCategory::TrendingMovies | 
            DiscoverCategory::PopularMovies | 
            DiscoverCategory::AnticipatedMovies
        )
    }
}

pub struct TraktClient {
    client: reqwest::blocking::Client,
    client_id: String,
}

impl TraktClient {
    pub fn new() -> Self {
        Self::with_client_id(TRAKT_CLIENT_ID.to_string())
    }
    
    pub fn with_client_id(client_id: String) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .danger_accept_invalid_certs(true) // For networks with proxy issues
            .build()
            .expect("Failed to create HTTP client");
            
        Self { client, client_id }
    }
    
    fn get(&self, endpoint: &str) -> Result<String, String> {
        let url = format!("{}{}", TRAKT_API_URL, endpoint);
        
        eprintln!("[Trakt] Fetching: {}", url);
        
        let response = self.client
            .get(&url)
            .header("Content-Type", "application/json")
            .header("trakt-api-version", "2")
            .header("trakt-api-key", &self.client_id)
            .send()
            .map_err(|e| {
                eprintln!("[Trakt] Request error: {:?}", e);
                format!("Request failed: {}", e)
            })?;
            
        let status = response.status();
        eprintln!("[Trakt] Response status: {}", status);
        
        if !status.is_success() {
            let error_text = response.text().unwrap_or_default();
            eprintln!("[Trakt] Error response: {}", error_text);
            return Err(format!("API error {}: {}", status, error_text));
        }
        
        let text = response.text().map_err(|e| format!("Failed to read response: {}", e))?;
        Ok(text)
    }
    
    pub fn get_trending_shows(&self, limit: usize) -> Result<Vec<DiscoverItem>, String> {
        let endpoint = format!("/shows/trending?extended=full&limit={}", limit);
        let text = self.get(&endpoint)?;
        
        let items: Vec<TraktTrendingShow> = serde_json::from_str(&text)
            .map_err(|e| format!("Parse error: {}", e))?;
            
        Ok(items.into_iter().map(|t| DiscoverItem {
            id: t.show.ids.trakt.unwrap_or(0),
            title: t.show.title,
            year: t.show.year,
            overview: t.show.overview.unwrap_or_default(),
            rating: t.show.rating,
            votes: t.show.votes,
            poster_url: None, // Trakt doesn't provide images directly
            content_type: DiscoverContentType::TvShow,
            watchers: Some(t.watchers),
            genres: t.show.genres.unwrap_or_default(),
            imdb_id: t.show.ids.imdb,
            tmdb_id: t.show.ids.tmdb,
        }).collect())
    }
    
    pub fn get_trending_movies(&self, limit: usize) -> Result<Vec<DiscoverItem>, String> {
        let endpoint = format!("/movies/trending?extended=full&limit={}", limit);
        let text = self.get(&endpoint)?;
        
        let items: Vec<TraktTrendingMovie> = serde_json::from_str(&text)
            .map_err(|e| format!("Parse error: {}", e))?;
            
        Ok(items.into_iter().map(|t| DiscoverItem {
            id: t.movie.ids.trakt.unwrap_or(0),
            title: t.movie.title,
            year: t.movie.year,
            overview: t.movie.overview.unwrap_or_default(),
            rating: t.movie.rating,
            votes: t.movie.votes,
            poster_url: None,
            content_type: DiscoverContentType::Movie,
            watchers: Some(t.watchers),
            genres: t.movie.genres.unwrap_or_default(),
            imdb_id: t.movie.ids.imdb,
            tmdb_id: t.movie.ids.tmdb,
        }).collect())
    }
    
    pub fn get_popular_shows(&self, limit: usize) -> Result<Vec<DiscoverItem>, String> {
        let endpoint = format!("/shows/popular?extended=full&limit={}", limit);
        let text = self.get(&endpoint)?;
        
        let items: Vec<TraktShow> = serde_json::from_str(&text)
            .map_err(|e| format!("Parse error: {}", e))?;
            
        Ok(items.into_iter().map(|s| DiscoverItem {
            id: s.ids.trakt.unwrap_or(0),
            title: s.title,
            year: s.year,
            overview: s.overview.unwrap_or_default(),
            rating: s.rating,
            votes: s.votes,
            poster_url: None,
            content_type: DiscoverContentType::TvShow,
            watchers: None,
            genres: s.genres.unwrap_or_default(),
            imdb_id: s.ids.imdb,
            tmdb_id: s.ids.tmdb,
        }).collect())
    }
    
    pub fn get_popular_movies(&self, limit: usize) -> Result<Vec<DiscoverItem>, String> {
        let endpoint = format!("/movies/popular?extended=full&limit={}", limit);
        let text = self.get(&endpoint)?;
        
        let items: Vec<TraktMovie> = serde_json::from_str(&text)
            .map_err(|e| format!("Parse error: {}", e))?;
            
        Ok(items.into_iter().map(|m| DiscoverItem {
            id: m.ids.trakt.unwrap_or(0),
            title: m.title,
            year: m.year,
            overview: m.overview.unwrap_or_default(),
            rating: m.rating,
            votes: m.votes,
            poster_url: None,
            content_type: DiscoverContentType::Movie,
            watchers: None,
            genres: m.genres.unwrap_or_default(),
            imdb_id: m.ids.imdb,
            tmdb_id: m.ids.tmdb,
        }).collect())
    }
    
    pub fn get_anticipated_shows(&self, limit: usize) -> Result<Vec<DiscoverItem>, String> {
        let endpoint = format!("/shows/anticipated?extended=full&limit={}", limit);
        let text = self.get(&endpoint)?;
        
        let items: Vec<TraktAnticipatedShow> = serde_json::from_str(&text)
            .map_err(|e| format!("Parse error: {}", e))?;
            
        Ok(items.into_iter().map(|a| DiscoverItem {
            id: a.show.ids.trakt.unwrap_or(0),
            title: a.show.title,
            year: a.show.year,
            overview: a.show.overview.unwrap_or_default(),
            rating: a.show.rating,
            votes: a.show.votes,
            poster_url: None,
            content_type: DiscoverContentType::TvShow,
            watchers: None,
            genres: a.show.genres.unwrap_or_default(),
            imdb_id: a.show.ids.imdb,
            tmdb_id: a.show.ids.tmdb,
        }).collect())
    }
    
    pub fn get_anticipated_movies(&self, limit: usize) -> Result<Vec<DiscoverItem>, String> {
        let endpoint = format!("/movies/anticipated?extended=full&limit={}", limit);
        let text = self.get(&endpoint)?;
        
        let items: Vec<TraktAnticipatedMovie> = serde_json::from_str(&text)
            .map_err(|e| format!("Parse error: {}", e))?;
            
        Ok(items.into_iter().map(|a| DiscoverItem {
            id: a.movie.ids.trakt.unwrap_or(0),
            title: a.movie.title,
            year: a.movie.year,
            overview: a.movie.overview.unwrap_or_default(),
            rating: a.movie.rating,
            votes: a.movie.votes,
            poster_url: None,
            content_type: DiscoverContentType::Movie,
            watchers: None,
            genres: a.movie.genres.unwrap_or_default(),
            imdb_id: a.movie.ids.imdb,
            tmdb_id: a.movie.ids.tmdb,
        }).collect())
    }
    
    pub fn get_category(&self, category: DiscoverCategory, limit: usize) -> Result<Vec<DiscoverItem>, String> {
        match category {
            DiscoverCategory::TrendingShows => self.get_trending_shows(limit),
            DiscoverCategory::TrendingMovies => self.get_trending_movies(limit),
            DiscoverCategory::PopularShows => self.get_popular_shows(limit),
            DiscoverCategory::PopularMovies => self.get_popular_movies(limit),
            DiscoverCategory::AnticipatedShows => self.get_anticipated_shows(limit),
            DiscoverCategory::AnticipatedMovies => self.get_anticipated_movies(limit),
        }
    }
}

// Message for async loading
#[derive(Debug, Clone)]
pub enum DiscoverMessage {
    CategoryLoaded(DiscoverCategory, Vec<DiscoverItem>),
    CategoryError(DiscoverCategory, String),
}

// Cache for loaded content
pub struct DiscoverCache {
    cache: HashMap<DiscoverCategory, Vec<DiscoverItem>>,
    pending_requests: std::collections::HashSet<DiscoverCategory>,
    failed_requests: HashMap<DiscoverCategory, std::time::Instant>,
    sender: mpsc::Sender<DiscoverMessage>,
    receiver: mpsc::Receiver<DiscoverMessage>,
    last_fetch: HashMap<DiscoverCategory, std::time::Instant>,
    pub last_error: Option<String>,
}

impl DiscoverCache {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            cache: HashMap::new(),
            pending_requests: std::collections::HashSet::new(),
            failed_requests: HashMap::new(),
            sender,
            receiver,
            last_fetch: HashMap::new(),
            last_error: None,
        }
    }
    
    /// Request content for a category.
    pub fn request_category(&mut self, category: DiscoverCategory) {
        // Check if we already have fresh data (less than 10 minutes old)
        if let Some(last_fetch) = self.last_fetch.get(&category) {
            if last_fetch.elapsed() < std::time::Duration::from_secs(600) {
                return;
            }
        }

        // Check if this category recently failed (30 second cooldown)
        if let Some(failed_time) = self.failed_requests.get(&category) {
            if failed_time.elapsed() < std::time::Duration::from_secs(30) {
                return;
            }
        }

        // Check if already pending
        if self.pending_requests.contains(&category) {
            return;
        }

        self.pending_requests.insert(category);
        let sender = self.sender.clone();

        std::thread::spawn(move || {
            eprintln!("[DiscoverCache] Loading {:?}", category);
            let client = TraktClient::new();
            
            match client.get_category(category, 20) {
                Ok(items) => {
                    eprintln!("[DiscoverCache] Loaded {} items for {:?}", items.len(), category);
                    let _ = sender.send(DiscoverMessage::CategoryLoaded(category, items));
                }
                Err(e) => {
                    eprintln!("[DiscoverCache] Error loading {:?}: {}", category, e);
                    let _ = sender.send(DiscoverMessage::CategoryError(category, e));
                }
            }
        });
    }
    
    /// Process pending results.
    pub fn process_pending(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                DiscoverMessage::CategoryLoaded(category, items) => {
                    self.pending_requests.remove(&category);
                    self.cache.insert(category, items);
                    self.last_fetch.insert(category, std::time::Instant::now());
                    self.failed_requests.remove(&category);
                    self.last_error = None;
                }
                DiscoverMessage::CategoryError(category, e) => {
                    self.pending_requests.remove(&category);
                    self.failed_requests.insert(category, std::time::Instant::now());
                    self.last_error = Some(e);
                }
            }
        }
    }
    
    /// Get cached content for a category.
    pub fn get_category(&self, category: DiscoverCategory) -> Option<&Vec<DiscoverItem>> {
        self.cache.get(&category)
    }
    
    /// Check if a category is loading.
    pub fn is_loading(&self, category: DiscoverCategory) -> bool {
        self.pending_requests.contains(&category)
    }
    
    /// Clear all cached data to force refresh.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.last_fetch.clear();
        self.failed_requests.clear();
        self.last_error = None;
    }
}
