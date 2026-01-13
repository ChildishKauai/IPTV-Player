use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedData<T> {
    pub data: T,
    pub cached_at: u64,
    pub cache_duration_secs: u64,
}

impl<T> CachedData<T> {
    pub fn new(data: T, cache_duration_secs: u64) -> Self {
        let cached_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            data,
            cached_at,
            cache_duration_secs,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.cached_at > self.cache_duration_secs
    }
}

pub struct ContentCache;

#[allow(dead_code)]
impl ContentCache {
    fn get_cache_dir() -> PathBuf {
        let mut path = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("iptv-player");
        path.push("cache");
        path
    }
    
    fn get_cache_path(key: &str) -> PathBuf {
        let mut path = Self::get_cache_dir();
        path.push(format!("{}.json", key));
        path
    }
    
    pub fn save<T: Serialize>(key: &str, data: &T, cache_duration_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
        let cached = CachedData::new(data, cache_duration_secs);
        let path = Self::get_cache_path(key);
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string(&cached)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    pub fn load<T: for<'de> Deserialize<'de>>(key: &str) -> Option<T> {
        let path = Self::get_cache_path(key);
        if !path.exists() {
            return None;
        }
        
        let content = fs::read_to_string(path).ok()?;
        let cached: CachedData<T> = serde_json::from_str(&content).ok()?;
        
        if cached.is_expired() {
            return None;
        }
        
        Some(cached.data)
    }
    
    pub fn clear_all() -> Result<(), Box<dyn std::error::Error>> {
        let cache_dir = Self::get_cache_dir();
        if cache_dir.exists() {
            fs::remove_dir_all(cache_dir)?;
        }
        Ok(())
    }
    
    pub fn remove(key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_cache_path(key);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}
