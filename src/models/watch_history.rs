use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WatchProgress {
    pub content_id: String,
    pub content_name: String,
    pub content_type: String, // "movie", "series", "channel"
    pub position_seconds: f64,
    pub duration_seconds: f64,
    pub last_watched: i64, // Unix timestamp
    pub thumbnail: Option<String>,
    // For series episodes
    pub season: Option<i32>,
    pub episode: Option<i32>,
}

impl WatchProgress {
    pub fn progress_percentage(&self) -> f32 {
        if self.duration_seconds > 0.0 {
            ((self.position_seconds / self.duration_seconds) * 100.0) as f32
        } else {
            0.0
        }
    }
    
    pub fn is_nearly_finished(&self) -> bool {
        self.progress_percentage() > 90.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WatchHistory {
    history: HashMap<String, WatchProgress>,
}

#[allow(dead_code)]
impl WatchHistory {
    pub fn load() -> Self {
        let path = Self::get_history_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(history) = serde_json::from_str(&content) {
                    return history;
                }
            }
        }
        Self::default()
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_history_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    fn get_history_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("iptv-player");
        path.push("watch_history.json");
        path
    }
    
    pub fn update_progress(&mut self, progress: WatchProgress) {
        self.history.insert(progress.content_id.clone(), progress);
        let _ = self.save();
    }
    
    pub fn get_progress(&self, content_id: &str) -> Option<&WatchProgress> {
        self.history.get(content_id)
    }
    
    pub fn get_continue_watching(&self, limit: usize) -> Vec<WatchProgress> {
        let mut items: Vec<_> = self.history.values()
            .filter(|p| !p.is_nearly_finished())
            .cloned()
            .collect();
        
        items.sort_by(|a, b| b.last_watched.cmp(&a.last_watched));
        items.truncate(limit);
        items
    }
    
    pub fn remove(&mut self, content_id: &str) {
        self.history.remove(content_id);
        let _ = self.save();
    }
    
    pub fn clear(&mut self) {
        self.history.clear();
        let _ = self.save();
    }
}
