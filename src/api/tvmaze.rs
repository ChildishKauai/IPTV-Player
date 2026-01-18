use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use std::thread;

/// TVMaze API base URL (completely free, no API key needed)
const TVMAZE_API_BASE: &str = "https://api.tvmaze.com";

/// TVMaze API client for fetching TV show data.
#[derive(Clone)]
pub struct TvMazeClient {
    client: reqwest::blocking::Client,
}

/// Show result from TVMaze API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvMazeShow {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub summary: Option<String>,
    pub image: Option<TvMazeImage>,
    pub rating: Option<TvMazeRating>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub premiered: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvMazeImage {
    pub medium: Option<String>,
    pub original: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvMazeRating {
    pub average: Option<f64>,
}

/// Schedule entry from TVMaze
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvMazeScheduleEntry {
    pub id: i64,
    pub name: String,
    pub airdate: String,
    pub airtime: String,
    pub show: TvMazeShow,
}

/// Search result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvMazeSearchResult {
    pub score: f64,
    pub show: TvMazeShow,
}

/// Unified item for display
#[derive(Debug, Clone)]
pub struct DiscoverItem {
    pub id: i64,
    pub title: String,
    pub overview: String,
    pub poster_url: Option<String>,
    pub rating: Option<f64>,
    pub year: Option<String>,
    pub content_type: DiscoverContentType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoverContentType {
    TvShow,
}

impl DiscoverItem {
    pub fn from_tvmaze_show(show: &TvMazeShow) -> Self {
        // Clean HTML from summary
        let overview = show.summary.as_ref()
            .map(|s| {
                // Remove HTML tags
                let mut clean = s.replace("<p>", "").replace("</p>", "\n")
                    .replace("<b>", "").replace("</b>", "")
                    .replace("<i>", "").replace("</i>", "")
                    .replace("<br>", "\n").replace("<br/>", "\n")
                    .replace("&amp;", "&").replace("&quot;", "\"");
                // Remove any remaining HTML tags (Unicode-safe: find returns byte positions that are valid char boundaries for ASCII '<' and '>')
                while let Some(start) = clean.find('<') {
                    if let Some(relative_end) = clean[start..].find('>') {
                        let end = start + relative_end;
                        clean = format!("{}{}", &clean[..start], &clean[end + 1..]);
                    } else {
                        break;
                    }
                }
                clean.trim().to_string()
            })
            .unwrap_or_default();

        let year = show.premiered.as_ref()
            .and_then(|p| p.split('-').next())
            .map(|s| s.to_string());

        Self {
            id: show.id,
            title: show.name.clone(),
            overview,
            poster_url: show.image.as_ref().and_then(|i| i.medium.clone()),
            rating: show.rating.as_ref().and_then(|r| r.average),
            year,
            content_type: DiscoverContentType::TvShow,
        }
    }
}

/// Categories for discovery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiscoverCategory {
    AiringToday,
    Popular,
    TopRated,
    SciFi,
    Drama,
    Comedy,
    Action,
}

impl DiscoverCategory {
    pub fn all() -> &'static [DiscoverCategory] {
        &[
            DiscoverCategory::AiringToday,
            DiscoverCategory::Popular,
            DiscoverCategory::TopRated,
            DiscoverCategory::SciFi,
            DiscoverCategory::Drama,
            DiscoverCategory::Comedy,
            DiscoverCategory::Action,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            DiscoverCategory::AiringToday => "📺 Airing Today",
            DiscoverCategory::Popular => "🔥 Popular Shows",
            DiscoverCategory::TopRated => "⭐ Top Rated",
            DiscoverCategory::SciFi => "🚀 Sci-Fi",
            DiscoverCategory::Drama => "🎭 Drama",
            DiscoverCategory::Comedy => "😂 Comedy",
            DiscoverCategory::Action => "💥 Action",
        }
    }
}

impl TvMazeClient {
    /// Create a new TVMaze client.
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                // Accept invalid certs to work around network proxies
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap_or_default(),
        }
    }

    /// Make a GET request to the TVMaze API.
    fn get<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T, String> {
        let url = format!("{}/{}", TVMAZE_API_BASE, endpoint);

        let response = self.client.get(&url)
            .send()
            .map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("certificate") {
                    "Network proxy blocking connection. Try using a VPN.".to_string()
                } else if err_str.contains("Connect") || err_str.contains("timeout") {
                    "Unable to connect. Check your internet connection.".to_string()
                } else {
                    format!("Connection error: {}", e)
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            if body.contains("<!DOCTYPE") || body.contains("<html") {
                return Err("API blocked by network. Try using a VPN.".to_string());
            }
            return Err(format!("API error: {}", status));
        }

        response.json::<T>()
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Get shows airing today
    pub fn get_airing_today(&self) -> Result<Vec<DiscoverItem>, String> {
        let entries: Vec<TvMazeScheduleEntry> = self.get("schedule")?;
        
        // Deduplicate by show ID and take first 20
        let mut seen = std::collections::HashSet::new();
        let items: Vec<DiscoverItem> = entries.into_iter()
            .filter(|e| seen.insert(e.show.id))
            .take(20)
            .map(|e| DiscoverItem::from_tvmaze_show(&e.show))
            .collect();
        
        Ok(items)
    }

    /// Get popular shows (using TVMaze's index which is roughly by popularity)
    pub fn get_popular(&self) -> Result<Vec<DiscoverItem>, String> {
        // TVMaze doesn't have a direct "popular" endpoint, but we can use
        // shows with IDs in certain ranges that tend to be popular
        // Or search for common terms
        let results: Vec<TvMazeSearchResult> = self.get("search/shows?q=the")?;
        
        let mut items: Vec<DiscoverItem> = results.into_iter()
            .take(20)
            .map(|r| DiscoverItem::from_tvmaze_show(&r.show))
            .collect();
        
        // Sort by rating
        items.sort_by(|a, b| {
            b.rating.unwrap_or(0.0).partial_cmp(&a.rating.unwrap_or(0.0)).unwrap()
        });
        
        Ok(items)
    }

    /// Get top rated shows
    pub fn get_top_rated(&self) -> Result<Vec<DiscoverItem>, String> {
        // Search for popular shows and sort by rating
        let results: Vec<TvMazeSearchResult> = self.get("search/shows?q=best")?;
        
        let mut items: Vec<DiscoverItem> = results.into_iter()
            .filter(|r| r.show.rating.as_ref().and_then(|r| r.average).unwrap_or(0.0) >= 7.0)
            .map(|r| DiscoverItem::from_tvmaze_show(&r.show))
            .collect();
        
        items.sort_by(|a, b| {
            b.rating.unwrap_or(0.0).partial_cmp(&a.rating.unwrap_or(0.0)).unwrap()
        });
        
        items.truncate(20);
        Ok(items)
    }

    /// Get shows by genre
    pub fn get_by_genre(&self, genre: &str) -> Result<Vec<DiscoverItem>, String> {
        let results: Vec<TvMazeSearchResult> = self.get(&format!("search/shows?q={}", genre))?;
        
        let mut items: Vec<DiscoverItem> = results.into_iter()
            .filter(|r| {
                r.show.genres.iter()
                    .any(|g| g.to_lowercase().contains(&genre.to_lowercase()))
            })
            .take(20)
            .map(|r| DiscoverItem::from_tvmaze_show(&r.show))
            .collect();
        
        // If genre filter didn't find enough, just take top results
        if items.len() < 10 {
            let results: Vec<TvMazeSearchResult> = self.get(&format!("search/shows?q={}", genre))?;
            items = results.into_iter()
                .take(20)
                .map(|r| DiscoverItem::from_tvmaze_show(&r.show))
                .collect();
        }
        
        items.sort_by(|a, b| {
            b.rating.unwrap_or(0.0).partial_cmp(&a.rating.unwrap_or(0.0)).unwrap()
        });
        
        Ok(items)
    }

    /// Get content by category
    pub fn get_by_category(&self, category: DiscoverCategory) -> Result<Vec<DiscoverItem>, String> {
        match category {
            DiscoverCategory::AiringToday => self.get_airing_today(),
            DiscoverCategory::Popular => self.get_popular(),
            DiscoverCategory::TopRated => self.get_top_rated(),
            DiscoverCategory::SciFi => self.get_by_genre("science-fiction"),
            DiscoverCategory::Drama => self.get_by_genre("drama"),
            DiscoverCategory::Comedy => self.get_by_genre("comedy"),
            DiscoverCategory::Action => self.get_by_genre("action"),
        }
    }

    /// Search for shows
    pub fn search(&self, query: &str) -> Result<Vec<DiscoverItem>, String> {
        let encoded = urlencoding::encode(query);
        let results: Vec<TvMazeSearchResult> = self.get(&format!("search/shows?q={}", encoded))?;
        
        Ok(results.into_iter()
            .take(20)
            .map(|r| DiscoverItem::from_tvmaze_show(&r.show))
            .collect())
    }
}

impl Default for TvMazeClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Message types for async communication
pub enum DiscoverMessage {
    CategoryLoaded(DiscoverCategory, Vec<DiscoverItem>),
    CategoryError(DiscoverCategory, String),
    SearchResults(String, Vec<DiscoverItem>),
    SearchError(String, String),
}

/// Cache for discover data with background loading.
pub struct DiscoverCache {
    cache: std::collections::HashMap<DiscoverCategory, Vec<DiscoverItem>>,
    search_cache: std::collections::HashMap<String, Vec<DiscoverItem>>,
    pending_requests: std::collections::HashSet<DiscoverCategory>,
    pending_searches: std::collections::HashSet<String>,
    failed_requests: std::collections::HashMap<DiscoverCategory, std::time::Instant>,
    sender: mpsc::Sender<DiscoverMessage>,
    receiver: mpsc::Receiver<DiscoverMessage>,
    last_fetch: std::collections::HashMap<DiscoverCategory, std::time::Instant>,
    pub last_error: Option<String>,
}

impl DiscoverCache {
    /// Create a new discover cache.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            cache: std::collections::HashMap::new(),
            search_cache: std::collections::HashMap::new(),
            pending_requests: std::collections::HashSet::new(),
            pending_searches: std::collections::HashSet::new(),
            failed_requests: std::collections::HashMap::new(),
            sender,
            receiver,
            last_fetch: std::collections::HashMap::new(),
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

        thread::spawn(move || {
            let client = TvMazeClient::new();
            match client.get_by_category(category) {
                Ok(items) => {
                    let _ = sender.send(DiscoverMessage::CategoryLoaded(category, items));
                }
                Err(e) => {
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
                DiscoverMessage::SearchResults(query, items) => {
                    self.pending_searches.remove(&query);
                    self.search_cache.insert(query, items);
                }
                DiscoverMessage::SearchError(query, e) => {
                    self.pending_searches.remove(&query);
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

impl Default for DiscoverCache {
    fn default() -> Self {
        Self::new()
    }
}
