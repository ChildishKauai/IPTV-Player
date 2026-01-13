// Football Fixtures API - integrates with Soccer-Scraper SQLite database
// Provides upcoming match data with broadcast channel information

use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a football fixture (match)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FootballFixture {
    pub id: i64,
    pub home_team: String,
    pub away_team: String,
    pub competition: String,
    pub fixture_date: String,
    pub fixture_time: Option<String>,
    pub venue: Option<String>,
    pub broadcasters: Vec<Broadcaster>,
}

#[allow(dead_code)]
impl FootballFixture {
    /// Returns match title formatted as "Home vs Away"
    pub fn match_title(&self) -> String {
        format!("{} vs {}", self.home_team, self.away_team)
    }
    
    /// Returns display time or "TBD" if not set
    pub fn display_time(&self) -> String {
        self.fixture_time.clone().unwrap_or_else(|| "TBD".to_string())
    }
    
    /// Get all channel names for searching
    pub fn all_channel_names(&self) -> Vec<String> {
        self.broadcasters.iter()
            .map(|b| b.channel.clone())
            .collect()
    }
    
    /// Get channels grouped by country
    pub fn channels_by_country(&self) -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for b in &self.broadcasters {
            map.entry(b.country.clone())
                .or_default()
                .push(b.channel.clone());
        }
        map
    }
}

/// Represents a broadcaster (channel and country)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Broadcaster {
    pub country: String,
    pub channel: String,
}

/// Client for reading football fixtures from SQLite database
pub struct FootballClient {
    db_path: PathBuf,
}

#[allow(dead_code)]
impl FootballClient {
    /// Create new client with path to fixtures database
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
    
    /// Create client with default database path (Soccer-Scraper-main/output/fixtures.db)
    pub fn with_default_path() -> Option<Self> {
        // Try to find the database relative to executable or current directory
        let possible_paths = vec![
            PathBuf::from("Soccer-Scraper-main/output/fixtures.db"),
            PathBuf::from("../Soccer-Scraper-main/output/fixtures.db"),
            PathBuf::from("./output/fixtures.db"),
        ];
        
        // Also check executable directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let db_path = exe_dir.join("Soccer-Scraper-main/output/fixtures.db");
                if db_path.exists() {
                    return Some(Self::new(db_path));
                }
            }
        }
        
        for path in possible_paths {
            if path.exists() {
                return Some(Self::new(path));
            }
        }
        
        None
    }
    
    /// Check if database exists
    pub fn database_exists(&self) -> bool {
        self.db_path.exists()
    }
    
    /// Get database path as string for display
    pub fn database_path(&self) -> String {
        self.db_path.to_string_lossy().to_string()
    }
    
    /// Query fixtures from SQLite database
    fn query_fixtures(&self, where_clause: &str, params: &[&dyn rusqlite::ToSql]) -> Result<Vec<FootballFixture>, String> {
        let conn = rusqlite::Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        let query = format!(
            "SELECT id, home_team, away_team, competition, fixture_date, fixture_time, venue 
             FROM fixtures 
             {} 
             ORDER BY fixture_date ASC, fixture_time ASC",
            where_clause
        );
        
        let mut stmt = conn.prepare(&query)
            .map_err(|e| format!("Failed to prepare query: {}", e))?;
        
        let fixture_iter = stmt.query_map(params, |row| {
            Ok(FootballFixture {
                id: row.get(0)?,
                home_team: row.get(1)?,
                away_team: row.get(2)?,
                competition: row.get(3)?,
                fixture_date: row.get(4)?,
                fixture_time: row.get(5)?,
                venue: row.get(6)?,
                broadcasters: Vec::new(), // Will be populated later
            })
        }).map_err(|e| format!("Query failed: {}", e))?;
        
        let mut fixtures: Vec<FootballFixture> = Vec::new();
        for fixture_result in fixture_iter {
            if let Ok(fixture) = fixture_result {
                fixtures.push(fixture);
            }
        }
        
        // Load broadcasters for each fixture
        for fixture in &mut fixtures {
            let broadcaster_query = 
                "SELECT country, channel FROM broadcasters WHERE fixture_id = ? ORDER BY country, channel";
            
            if let Ok(mut stmt) = conn.prepare(broadcaster_query) {
                let broadcaster_iter = stmt.query_map([fixture.id], |row| {
                    Ok(Broadcaster {
                        country: row.get(0)?,
                        channel: row.get(1)?,
                    })
                });
                
                if let Ok(iter) = broadcaster_iter {
                    for b in iter.flatten() {
                        fixture.broadcasters.push(b);
                    }
                }
            }
        }
        
        Ok(fixtures)
    }
    
    /// Get today's fixtures
    pub fn get_today(&self) -> Result<Vec<FootballFixture>, String> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        self.query_fixtures("WHERE fixture_date = ?", &[&today])
    }
    
    /// Get tomorrow's fixtures
    pub fn get_tomorrow(&self) -> Result<Vec<FootballFixture>, String> {
        let tomorrow = (chrono::Local::now() + chrono::Duration::days(1))
            .format("%Y-%m-%d").to_string();
        self.query_fixtures("WHERE fixture_date = ?", &[&tomorrow])
    }
    
    /// Get this week's fixtures (next 7 days)
    pub fn get_this_week(&self) -> Result<Vec<FootballFixture>, String> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let week_later = (chrono::Local::now() + chrono::Duration::days(7))
            .format("%Y-%m-%d").to_string();
        self.query_fixtures(
            "WHERE fixture_date >= ? AND fixture_date <= ?",
            &[&today, &week_later]
        )
    }
    
    /// Get all upcoming fixtures
    pub fn get_upcoming(&self) -> Result<Vec<FootballFixture>, String> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        self.query_fixtures("WHERE fixture_date >= ?", &[&today])
    }
    
    /// Get fixtures for a specific competition
    pub fn get_by_competition(&self, competition: &str) -> Result<Vec<FootballFixture>, String> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        self.query_fixtures(
            "WHERE fixture_date >= ? AND competition LIKE ?",
            &[&today, &format!("%{}%", competition)]
        )
    }
    
    /// Get database statistics
    pub fn get_stats(&self) -> Result<FootballStats, String> {
        let conn = rusqlite::Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        let total_fixtures: i64 = conn.query_row(
            "SELECT COUNT(*) FROM fixtures", [], |row| row.get(0)
        ).unwrap_or(0);
        
        let total_broadcasters: i64 = conn.query_row(
            "SELECT COUNT(*) FROM broadcasters", [], |row| row.get(0)
        ).unwrap_or(0);
        
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let upcoming_fixtures: i64 = conn.query_row(
            "SELECT COUNT(*) FROM fixtures WHERE fixture_date >= ?", [&today], |row| row.get(0)
        ).unwrap_or(0);
        
        // Get competitions
        let mut stmt = conn.prepare(
            "SELECT DISTINCT competition FROM fixtures WHERE fixture_date >= ? ORDER BY competition"
        ).map_err(|e| format!("Query failed: {}", e))?;
        
        let competitions: Vec<String> = stmt.query_map([&today], |row| row.get(0))
            .map_err(|e| format!("Query failed: {}", e))?
            .filter_map(|r| r.ok())
            .collect();
        
        // Get date range
        let min_date: Option<String> = conn.query_row(
            "SELECT MIN(fixture_date) FROM fixtures WHERE fixture_date >= ?", [&today], |row| row.get(0)
        ).ok();
        
        let max_date: Option<String> = conn.query_row(
            "SELECT MAX(fixture_date) FROM fixtures WHERE fixture_date >= ?", [&today], |row| row.get(0)
        ).ok();
        
        Ok(FootballStats {
            total_fixtures,
            upcoming_fixtures,
            total_broadcasters,
            competitions,
            min_date,
            max_date,
        })
    }
}

/// Database statistics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FootballStats {
    pub total_fixtures: i64,
    pub upcoming_fixtures: i64,
    pub total_broadcasters: i64,
    pub competitions: Vec<String>,
    pub min_date: Option<String>,
    pub max_date: Option<String>,
}

/// Categories for filtering fixtures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FootballCategory {
    Today,
    Tomorrow,
    ThisWeek,
    PremierLeague,
    LaLiga,
    SerieA,
    Bundesliga,
    Ligue1,
    ChampionsLeague,
}

impl FootballCategory {
    pub fn all() -> &'static [FootballCategory] {
        &[
            FootballCategory::Today,
            FootballCategory::Tomorrow,
            FootballCategory::ThisWeek,
            FootballCategory::PremierLeague,
            FootballCategory::LaLiga,
            FootballCategory::SerieA,
            FootballCategory::Bundesliga,
            FootballCategory::Ligue1,
            FootballCategory::ChampionsLeague,
        ]
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            FootballCategory::Today => "⚽ Today's Matches",
            FootballCategory::Tomorrow => "📅 Tomorrow",
            FootballCategory::ThisWeek => "📆 This Week",
            FootballCategory::PremierLeague => "🏴󠁧󠁢󠁥󠁮󠁧󠁿 Premier League",
            FootballCategory::LaLiga => "🇪🇸 La Liga",
            FootballCategory::SerieA => "🇮🇹 Serie A",
            FootballCategory::Bundesliga => "🇩🇪 Bundesliga",
            FootballCategory::Ligue1 => "🇫🇷 Ligue 1",
            FootballCategory::ChampionsLeague => "🏆 Champions League",
        }
    }
    
    /// Get the competition filter string (if any)
    pub fn competition_filter(&self) -> Option<&'static str> {
        match self {
            FootballCategory::PremierLeague => Some("Premier League"),
            FootballCategory::LaLiga => Some("La Liga"),
            FootballCategory::SerieA => Some("Serie A"),
            FootballCategory::Bundesliga => Some("Bundesliga"),
            FootballCategory::Ligue1 => Some("Ligue 1"),
            FootballCategory::ChampionsLeague => Some("Champions League"),
            _ => None,
        }
    }
}

/// Message for async loading
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FootballMessage {
    FixturesLoaded(FootballCategory, Vec<FootballFixture>),
    FixturesError(FootballCategory, String),
    StatsLoaded(FootballStats),
}

/// Cache for football fixtures with async loading
pub struct FootballCache {
    cache: HashMap<FootballCategory, Vec<FootballFixture>>,
    pending_requests: std::collections::HashSet<FootballCategory>,
    sender: std::sync::mpsc::Sender<FootballMessage>,
    receiver: std::sync::mpsc::Receiver<FootballMessage>,
    last_fetch: HashMap<FootballCategory, std::time::Instant>,
    pub last_error: Option<String>,
    pub stats: Option<FootballStats>,
    db_path: Option<PathBuf>,
}

#[allow(dead_code)]
impl FootballCache {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        
        // Try to find database
        let db_path = Self::find_database();
        
        Self {
            cache: HashMap::new(),
            pending_requests: std::collections::HashSet::new(),
            sender,
            receiver,
            last_fetch: HashMap::new(),
            last_error: None,
            stats: None,
            db_path,
        }
    }
    
    fn find_database() -> Option<PathBuf> {
        let possible_paths = vec![
            PathBuf::from("Soccer-Scraper-main/output/fixtures.db"),
            PathBuf::from("../Soccer-Scraper-main/output/fixtures.db"),
            PathBuf::from("./output/fixtures.db"),
        ];
        
        // Check executable directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let db_path = exe_dir.join("Soccer-Scraper-main/output/fixtures.db");
                if db_path.exists() {
                    return Some(db_path);
                }
            }
        }
        
        for path in possible_paths {
            if path.exists() {
                return Some(path);
            }
        }
        
        None
    }
    
    /// Check if database is available
    pub fn has_database(&self) -> bool {
        self.db_path.is_some()
    }
    
    /// Get database path for display
    pub fn database_path(&self) -> Option<String> {
        self.db_path.as_ref().map(|p| p.to_string_lossy().to_string())
    }
    
    /// Request fixtures for a category
    pub fn request_category(&mut self, category: FootballCategory) {
        // Check cache validity (5 minutes)
        if let Some(last_fetch) = self.last_fetch.get(&category) {
            if last_fetch.elapsed() < std::time::Duration::from_secs(300) {
                return;
            }
        }
        
        if self.pending_requests.contains(&category) {
            return;
        }
        
        let db_path = match &self.db_path {
            Some(p) => p.clone(),
            None => {
                self.last_error = Some("Database not found. Run Soccer-Scraper to fetch fixtures.".to_string());
                return;
            }
        };
        
        self.pending_requests.insert(category);
        let sender = self.sender.clone();
        
        std::thread::spawn(move || {
            let client = FootballClient::new(db_path);
            
            let result = match category {
                FootballCategory::Today => client.get_today(),
                FootballCategory::Tomorrow => client.get_tomorrow(),
                FootballCategory::ThisWeek => client.get_this_week(),
                _ => {
                    if let Some(comp) = category.competition_filter() {
                        client.get_by_competition(comp)
                    } else {
                        client.get_upcoming()
                    }
                }
            };
            
            match result {
                Ok(fixtures) => {
                    let _ = sender.send(FootballMessage::FixturesLoaded(category, fixtures));
                }
                Err(e) => {
                    let _ = sender.send(FootballMessage::FixturesError(category, e));
                }
            }
        });
    }
    
    /// Request database stats
    pub fn request_stats(&mut self) {
        let db_path = match &self.db_path {
            Some(p) => p.clone(),
            None => return,
        };
        
        let sender = self.sender.clone();
        
        std::thread::spawn(move || {
            let client = FootballClient::new(db_path);
            if let Ok(stats) = client.get_stats() {
                let _ = sender.send(FootballMessage::StatsLoaded(stats));
            }
        });
    }
    
    /// Process pending results
    pub fn process_pending(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                FootballMessage::FixturesLoaded(category, fixtures) => {
                    self.pending_requests.remove(&category);
                    self.cache.insert(category, fixtures);
                    self.last_fetch.insert(category, std::time::Instant::now());
                    self.last_error = None;
                }
                FootballMessage::FixturesError(category, e) => {
                    self.pending_requests.remove(&category);
                    self.last_error = Some(e);
                }
                FootballMessage::StatsLoaded(stats) => {
                    self.stats = Some(stats);
                }
            }
        }
    }
    
    /// Get cached fixtures for a category
    pub fn get_category(&self, category: FootballCategory) -> Option<&Vec<FootballFixture>> {
        self.cache.get(&category)
    }
    
    /// Check if a category is loading
    pub fn is_loading(&self, category: FootballCategory) -> bool {
        self.pending_requests.contains(&category)
    }
    
    /// Clear cache to force refresh
    pub fn clear(&mut self) {
        self.cache.clear();
        self.last_fetch.clear();
        self.last_error = None;
        // Re-check for database
        self.db_path = Self::find_database();
    }
}
