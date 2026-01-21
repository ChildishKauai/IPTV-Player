//! EPG (Electronic Program Guide) cache for loading EPG data in background.

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::api::XtreamClient;
use crate::models::EpgProgram;
use crate::xmltv::XmltvParser;

/// Message type for EPG loading
struct EpgLoadResult {
    stream_id: String,
    programs: Vec<EpgProgram>,
}

/// EPG Cache for storing and fetching EPG data
pub struct EpgCache {
    /// Cached EPG data: stream_id -> programs
    cache: Arc<Mutex<HashMap<String, Vec<EpgProgram>>>>,
    /// Set of stream IDs currently being loaded
    loading: Arc<Mutex<std::collections::HashSet<String>>>,
    /// Sender for background load results
    tx: Option<Sender<EpgLoadResult>>,
    /// Receiver for background load results
    rx: Option<Receiver<EpgLoadResult>>,
    /// API credentials
    server_url: String,
    username: String,
    password: String,
    /// Last cache refresh timestamp
    last_refresh: std::time::Instant,
    /// External XMLTV EPG data: tvg_id -> programs
    xmltv_cache: Arc<Mutex<HashMap<String, Vec<EpgProgram>>>>,
    /// XMLTV EPG URL
    xmltv_url: Arc<Mutex<Option<String>>>,
    /// Whether XMLTV has been loaded
    xmltv_loaded: Arc<Mutex<bool>>,
}

#[allow(dead_code)]
impl EpgCache {
    /// Create a new EPG cache
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            loading: Arc::new(Mutex::new(std::collections::HashSet::new())),
            tx: Some(tx),
            rx: Some(rx),
            server_url: String::new(),
            username: String::new(),
            password: String::new(),
            last_refresh: std::time::Instant::now(),
            xmltv_cache: Arc::new(Mutex::new(HashMap::new())),
            xmltv_url: Arc::new(Mutex::new(None)),
            xmltv_loaded: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Set API credentials
    pub fn set_credentials(&mut self, server_url: String, username: String, password: String) {
        self.server_url = server_url;
        self.username = username;
        self.password = password;
        // Clear cache when credentials change
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }
    
    /// Check for completed background loads and update cache
    pub fn process_pending(&mut self) {
        if let Some(rx) = &self.rx {
            // Process all pending results
            while let Ok(result) = rx.try_recv() {
                if let Ok(mut cache) = self.cache.lock() {
                    cache.insert(result.stream_id.clone(), result.programs);
                }
                if let Ok(mut loading) = self.loading.lock() {
                    loading.remove(&result.stream_id);
                }
            }
        }
    }
    
    /// Request EPG for a stream (loads in background if not cached)
    pub fn request_epg(&self, stream_id: &str) {
        // Check if already cached
        if let Ok(cache) = self.cache.lock() {
            if cache.contains_key(stream_id) {
                return;
            }
        }
        
        // Check if already loading
        if let Ok(mut loading) = self.loading.lock() {
            if loading.contains(stream_id) {
                return;
            }
            loading.insert(stream_id.to_string());
        }
        
        // Skip if no credentials
        if self.server_url.is_empty() {
            return;
        }
        
        // Clone for thread
        let tx = self.tx.clone();
        let stream_id = stream_id.to_string();
        let server_url = self.server_url.clone();
        let username = self.username.clone();
        let password = self.password.clone();
        
        // Load in background thread with retry
        thread::spawn(move || {
            if let Some(tx) = tx {
                let client = XtreamClient::new(server_url, username, password);
                
                // Try up to 2 times with a small delay
                let mut programs = Vec::new();
                for attempt in 0..2 {
                    match client.get_short_epg(&stream_id) {
                        Ok(p) => {
                            programs = p;
                            if !programs.is_empty() {
                                eprintln!("[EPG] Loaded {} programs for stream {}", programs.len(), stream_id);
                            }
                            break;
                        }
                        Err(e) => {
                            if attempt == 0 {
                                // Wait a bit before retry
                                std::thread::sleep(std::time::Duration::from_millis(500));
                            } else {
                                // Only log on final failure, and silently for 503 errors
                                if !e.to_string().contains("503") {
                                    eprintln!("[EPG] Error loading stream {}: {}", stream_id, e);
                                }
                            }
                        }
                    }
                }
                let _ = tx.send(EpgLoadResult { stream_id, programs });
            }
        });
    }
    
    /// Get cached EPG for a stream (returns None if not loaded yet)
    pub fn get_epg(&self, stream_id: &str) -> Option<Vec<EpgProgram>> {
        if let Ok(cache) = self.cache.lock() {
            cache.get(stream_id).cloned()
        } else {
            None
        }
    }
    
    /// Get the current program for a stream
    pub fn get_current_program(&self, stream_id: &str) -> Option<EpgProgram> {
        self.get_epg(stream_id).and_then(|programs| {
            programs.into_iter().find(|p| p.is_now_playing())
        })
    }
    
    /// Get the next program for a stream
    pub fn get_next_program(&self, stream_id: &str) -> Option<EpgProgram> {
        if let Some(programs) = self.get_epg(stream_id) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            
            // Find the first program that starts after now
            programs.into_iter()
                .find(|p| p.start_timestamp_unix() > now)
        } else {
            None
        }
    }
    
    /// Clear the EPG cache (e.g., on disconnect)
    pub fn clear(&mut self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
        if let Ok(mut loading) = self.loading.lock() {
            loading.clear();
        }
        self.last_refresh = std::time::Instant::now();
    }
    
    /// Bulk request EPG for multiple streams
    pub fn request_epg_batch(&self, stream_ids: &[String]) {
        for stream_id in stream_ids {
            self.request_epg(stream_id);
        }
    }
    
    /// Check if cache needs refresh (older than 5 minutes)
    pub fn needs_refresh(&self) -> bool {
        self.last_refresh.elapsed().as_secs() > 300
    }
    
    /// Refresh all cached entries
    pub fn refresh_all(&mut self) {
        if let Ok(cache) = self.cache.lock() {
            let stream_ids: Vec<String> = cache.keys().cloned().collect();
            drop(cache); // Release lock before requesting
            for stream_id in stream_ids {
                // Force re-request by removing from cache
                if let Ok(mut cache) = self.cache.lock() {
                    cache.remove(&stream_id);
                }
                self.request_epg(&stream_id);
            }
        }
        self.last_refresh = std::time::Instant::now();
    }

    /// Set external XMLTV EPG URL
    pub fn set_xmltv_url(&self, url: Option<String>) {
        if let Ok(mut xmltv_url) = self.xmltv_url.lock() {
            *xmltv_url = url;
        }
        // Reset loaded flag when URL changes
        if let Ok(mut loaded) = self.xmltv_loaded.lock() {
            *loaded = false;
        }
        // Clear XMLTV cache
        if let Ok(mut cache) = self.xmltv_cache.lock() {
            cache.clear();
        }
    }

    /// Load XMLTV EPG data from configured URL
    pub fn load_xmltv(&self) {
        // Check if already loaded or loading
        if let Ok(loaded) = self.xmltv_loaded.lock() {
            if *loaded {
                return; // Already loaded, don't reload
            }
        }

        let url = {
            if let Ok(xmltv_url) = self.xmltv_url.lock() {
                xmltv_url.clone()
            } else {
                None
            }
        };

        if let Some(url) = url {
            // Mark as loaded immediately to prevent duplicate loads
            if let Ok(mut loaded) = self.xmltv_loaded.lock() {
                *loaded = true;
            }

            let xmltv_cache = self.xmltv_cache.clone();
            let xmltv_loaded = self.xmltv_loaded.clone();

            thread::spawn(move || {
                eprintln!("[EPG] Loading XMLTV from: {}", url);
                match XmltvParser::parse_url(&url) {
                    Ok(programs_map) => {
                        eprintln!("[EPG] Successfully loaded XMLTV data for {} channels", programs_map.len());
                        if let Ok(mut cache) = xmltv_cache.lock() {
                            *cache = programs_map;
                        }
                    }
                    Err(e) => {
                        eprintln!("[EPG] Error loading XMLTV: {}", e);
                        // Reset loaded flag on error so we can retry later
                        if let Ok(mut loaded) = xmltv_loaded.lock() {
                            *loaded = false;
                        }
                    }
                }
            });
        }
    }

    /// Get EPG from XMLTV cache by tvg-id
    pub fn get_xmltv_epg(&self, tvg_id: &str) -> Option<Vec<EpgProgram>> {
        if let Ok(cache) = self.xmltv_cache.lock() {
            cache.get(tvg_id).cloned()
        } else {
            None
        }
    }

    /// Request EPG for a channel with tvg-id (tries XMLTV first, then Xtream API)
    pub fn request_epg_with_tvg(&self, stream_id: &str, tvg_id: Option<&str>) {
        // First, try to load XMLTV if we have a URL and haven't loaded yet
        if let Ok(loaded) = self.xmltv_loaded.lock() {
            if !*loaded {
                drop(loaded);
                if let Ok(url) = self.xmltv_url.lock() {
                    if url.is_some() {
                        drop(url);
                        self.load_xmltv();
                    }
                }
            }
        }

        // If we have a tvg-id, try to get EPG from XMLTV cache
        if let Some(tvg_id) = tvg_id {
            if !tvg_id.is_empty() {
                if let Some(programs) = self.get_xmltv_epg(tvg_id) {
                    // Store in main cache using stream_id
                    if let Ok(mut cache) = self.cache.lock() {
                        cache.insert(stream_id.to_string(), programs);
                    }
                    return;
                }
            }
        }

        // Fallback to Xtream API
        self.request_epg(stream_id);
    }
}

impl Default for EpgCache {
    fn default() -> Self {
        Self::new()
    }
}
