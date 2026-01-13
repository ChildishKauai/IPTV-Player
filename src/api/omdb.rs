// OMDb API client for movie/TV show discovery
// API Documentation: https://www.omdbapi.com/
// Uses search functionality to get dynamic content

use std::collections::HashMap;
use std::sync::mpsc;

const OMDB_API_URL: &str = "https://www.omdbapi.com/";
const DEFAULT_API_KEY: &str = "46b22508";

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct OmdbSearchResult {
    #[serde(rename = "Search")]
    pub search: Option<Vec<OmdbSearchItem>>,
    #[serde(rename = "totalResults")]
    pub total_results: Option<String>,
    #[serde(rename = "Response")]
    pub response: String,
    #[serde(rename = "Error")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OmdbSearchItem {
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Year")]
    pub year: String,
    #[serde(rename = "imdbID")]
    pub imdb_id: String,
    #[serde(rename = "Type")]
    pub content_type: String,
    #[serde(rename = "Poster")]
    pub poster: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct OmdbDetailResponse {
    #[serde(rename = "Title")]
    pub title: Option<String>,
    #[serde(rename = "Year")]
    pub year: Option<String>,
    #[serde(rename = "Rated")]
    pub rated: Option<String>,
    #[serde(rename = "Runtime")]
    pub runtime: Option<String>,
    #[serde(rename = "Genre")]
    pub genre: Option<String>,
    #[serde(rename = "Director")]
    pub director: Option<String>,
    #[serde(rename = "Actors")]
    pub actors: Option<String>,
    #[serde(rename = "Plot")]
    pub plot: Option<String>,
    #[serde(rename = "Poster")]
    pub poster: Option<String>,
    #[serde(rename = "imdbRating")]
    pub imdb_rating: Option<String>,
    #[serde(rename = "imdbVotes")]
    pub imdb_votes: Option<String>,
    #[serde(rename = "imdbID")]
    pub imdb_id: Option<String>,
    #[serde(rename = "Type")]
    pub content_type: Option<String>,
    #[serde(rename = "Response")]
    pub response: Option<String>,
    #[serde(rename = "Error")]
    pub error: Option<String>,
}

// Unified item for display
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DiscoverItem {
    pub id: String,
    pub title: String,
    pub year: Option<String>,
    pub overview: String,
    pub rating: Option<f64>,
    pub votes: Option<String>,
    pub poster_url: Option<String>,
    pub content_type: DiscoverContentType,
    pub genres: Vec<String>,
    pub imdb_id: String,
    pub runtime: Option<String>,
    pub director: Option<String>,
    pub actors: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoverContentType {
    TvShow,
    Movie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiscoverCategory {
    NewMovies2026,
    Movies2025,
    Series2025,
    ActionMovies,
    ComedyMovies,
    HorrorMovies,
    SciFiMovies,
    DramaSeries,
    CrimeSeries,
    MarvelContent,
    StarWarsContent,
}

impl DiscoverCategory {
    pub fn all() -> &'static [DiscoverCategory] {
        &[
            DiscoverCategory::NewMovies2026,
            DiscoverCategory::Movies2025,
            DiscoverCategory::Series2025,
            DiscoverCategory::ActionMovies,
            DiscoverCategory::ComedyMovies,
            DiscoverCategory::HorrorMovies,
            DiscoverCategory::SciFiMovies,
            DiscoverCategory::DramaSeries,
            DiscoverCategory::CrimeSeries,
            DiscoverCategory::MarvelContent,
            DiscoverCategory::StarWarsContent,
        ]
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            DiscoverCategory::NewMovies2026 => "🔥 New Movies 2026",
            DiscoverCategory::Movies2025 => "🎬 Movies 2025",
            DiscoverCategory::Series2025 => "📺 Series 2025",
            DiscoverCategory::ActionMovies => "💥 Action Movies",
            DiscoverCategory::ComedyMovies => "😂 Comedy Movies",
            DiscoverCategory::HorrorMovies => "👻 Horror Movies",
            DiscoverCategory::SciFiMovies => "🚀 Sci-Fi Movies",
            DiscoverCategory::DramaSeries => "🎭 Drama Series",
            DiscoverCategory::CrimeSeries => "🔍 Crime Series",
            DiscoverCategory::MarvelContent => "🦸 Marvel",
            DiscoverCategory::StarWarsContent => "⭐ Star Wars",
        }
    }
    
    /// Get search query and type filter for this category
    pub fn search_params(&self) -> (&'static str, Option<&'static str>, Option<&'static str>) {
        // Returns: (search_query, type_filter, year_filter)
        match self {
            DiscoverCategory::NewMovies2026 => ("2026", Some("movie"), Some("2026")),
            DiscoverCategory::Movies2025 => ("2025", Some("movie"), Some("2025")),
            DiscoverCategory::Series2025 => ("2025", Some("series"), Some("2025")),
            DiscoverCategory::ActionMovies => ("action", Some("movie"), None),
            DiscoverCategory::ComedyMovies => ("comedy", Some("movie"), None),
            DiscoverCategory::HorrorMovies => ("horror", Some("movie"), None),
            DiscoverCategory::SciFiMovies => ("sci-fi", Some("movie"), None),
            DiscoverCategory::DramaSeries => ("drama", Some("series"), None),
            DiscoverCategory::CrimeSeries => ("crime", Some("series"), None),
            DiscoverCategory::MarvelContent => ("marvel", Some("movie"), None),
            DiscoverCategory::StarWarsContent => ("star wars", None, None),
        }
    }
}

pub struct OmdbClient {
    client: reqwest::blocking::Client,
    api_key: String,
}

impl OmdbClient {
    pub fn new() -> Self {
        Self::with_api_key(DEFAULT_API_KEY.to_string())
    }
    
    pub fn with_api_key(api_key: String) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Failed to create HTTP client");
            
        Self { client, api_key }
    }
    
    /// Search for content using OMDb's search API
    pub fn search(&self, query: &str, content_type: Option<&str>, year: Option<&str>, page: u32) -> Result<Vec<DiscoverItem>, String> {
        let mut url = format!("{}?apikey={}&s={}&page={}", 
            OMDB_API_URL, self.api_key, urlencoding::encode(query), page);
        
        if let Some(t) = content_type {
            url.push_str(&format!("&type={}", t));
        }
        
        if let Some(y) = year {
            url.push_str(&format!("&y={}", y));
        }
        
        eprintln!("[OMDb] Searching: {}", url);
        
        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;
            
        let status = response.status();
        if !status.is_success() {
            return Err(format!("API error: {}", status));
        }
        
        let text = response.text().map_err(|e| format!("Failed to read response: {}", e))?;
        
        // Check if it's HTML (blocked by proxy)
        if text.trim().starts_with("<!") || text.trim().starts_with("<html") {
            return Err("API blocked by network proxy".to_string());
        }
        
        let result: OmdbSearchResult = serde_json::from_str(&text)
            .map_err(|e| format!("Parse error: {} - Response: {}", e, &text[..text.len().min(200)]))?;
        
        if result.response == "False" {
            return Err(result.error.unwrap_or_else(|| "No results found".to_string()));
        }
        
        let items = result.search.unwrap_or_default();
        eprintln!("[OMDb] Found {} results for '{}'", items.len(), query);
        
        // Convert search items to DiscoverItems
        let discover_items: Vec<DiscoverItem> = items.into_iter().map(|item| {
            let content_type = if item.content_type == "series" {
                DiscoverContentType::TvShow
            } else {
                DiscoverContentType::Movie
            };
            
            let poster_url = item.poster.filter(|p| p != "N/A" && !p.is_empty());
            
            DiscoverItem {
                id: item.imdb_id.clone(),
                title: item.title,
                year: Some(item.year).filter(|y| y != "N/A"),
                overview: String::new(), // Not available in search results
                rating: None, // Not available in search results
                votes: None,
                poster_url,
                content_type,
                genres: Vec::new(),
                imdb_id: item.imdb_id,
                runtime: None,
                director: None,
                actors: None,
            }
        }).collect();
        
        Ok(discover_items)
    }
    
    /// Get detailed info for a single item
    #[allow(dead_code)]
    pub fn get_details(&self, imdb_id: &str) -> Result<DiscoverItem, String> {
        let url = format!("{}?apikey={}&i={}&plot=short", OMDB_API_URL, self.api_key, imdb_id);
        
        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;
            
        let status = response.status();
        if !status.is_success() {
            return Err(format!("API error: {}", status));
        }
        
        let text = response.text().map_err(|e| format!("Failed to read response: {}", e))?;
        
        if text.trim().starts_with("<!") || text.trim().starts_with("<html") {
            return Err("API blocked by network proxy".to_string());
        }
        
        let data: OmdbDetailResponse = serde_json::from_str(&text)
            .map_err(|e| format!("Parse error: {}", e))?;
            
        if data.response.as_deref() == Some("False") {
            return Err(data.error.unwrap_or_else(|| "Unknown error".to_string()));
        }
        
        let rating = data.imdb_rating
            .and_then(|r| if r == "N/A" { None } else { r.parse::<f64>().ok() });
            
        let content_type = match data.content_type.as_deref() {
            Some("series") => DiscoverContentType::TvShow,
            _ => DiscoverContentType::Movie,
        };
        
        let genres = data.genre
            .map(|g| g.split(", ").map(|s| s.to_string()).collect())
            .unwrap_or_default();
            
        let poster_url = data.poster.filter(|p| p != "N/A" && !p.is_empty());
        
        Ok(DiscoverItem {
            id: imdb_id.to_string(),
            title: data.title.unwrap_or_else(|| "Unknown".to_string()),
            year: data.year.filter(|y| y != "N/A"),
            overview: data.plot.unwrap_or_default(),
            rating,
            votes: data.imdb_votes.filter(|v| v != "N/A"),
            poster_url,
            content_type,
            genres,
            imdb_id: imdb_id.to_string(),
            runtime: data.runtime.filter(|r| r != "N/A"),
            director: data.director.filter(|d| d != "N/A"),
            actors: data.actors.filter(|a| a != "N/A"),
        })
    }
    
    /// Get content for a category using search
    pub fn get_category(&self, category: DiscoverCategory) -> Result<Vec<DiscoverItem>, String> {
        let (query, content_type, year) = category.search_params();
        
        // Fetch 2 pages for more results
        let mut all_items = Vec::new();
        
        for page in 1..=2 {
            match self.search(query, content_type, year, page) {
                Ok(items) => all_items.extend(items),
                Err(e) => {
                    if page == 1 {
                        return Err(e);
                    }
                    // Ignore errors on page 2
                }
            }
        }
        
        // Remove duplicates by imdb_id
        let mut seen = std::collections::HashSet::new();
        all_items.retain(|item| seen.insert(item.imdb_id.clone()));
        
        Ok(all_items)
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
        // Check if we already have data (cache for 30 minutes)
        if let Some(last_fetch) = self.last_fetch.get(&category) {
            if last_fetch.elapsed() < std::time::Duration::from_secs(1800) {
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
            let client = OmdbClient::new();
            
            match client.get_category(category) {
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
