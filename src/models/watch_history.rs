use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Positions closer than this to the start aren't worth resuming from.
const MIN_RESUME_SECONDS: f64 = 30.0;
/// Resume a few seconds early so the viewer gets their bearings.
const RESUME_REWIND_SECONDS: f64 = 5.0;

/// The episode that follows the one a [`WatchProgress`] entry is for.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct NextEpisode {
    pub episode_id: String,
    pub season: i32,
    pub episode: i32,
    pub title: String,
    pub container: String,
}

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
    /// Movie stream id, episode id or channel stream id - enough to play it
    /// again without looking it up. Missing on entries from older versions.
    #[serde(default)]
    pub stream_id: Option<String>,
    /// Container extension for movies and episodes
    #[serde(default)]
    pub container: Option<String>,
    #[serde(default)]
    pub series_id: Option<i32>,
    #[serde(default)]
    pub episode_title: Option<String>,
    /// The following episode, offered once this one is finished
    #[serde(default)]
    pub next_episode: Option<NextEpisode>,
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

    /// Whether the player reported a real position for this entry.
    pub fn has_position(&self) -> bool {
        self.duration_seconds > 0.0 && self.position_seconds > 0.0
    }

    /// Where to start playback when this item is played again.
    pub fn resume_position(&self) -> Option<f64> {
        if self.position_seconds >= MIN_RESUME_SECONDS && !self.is_nearly_finished() {
            Some((self.position_seconds - RESUME_REWIND_SECONDS).max(0.0))
        } else {
            None
        }
    }

    /// Whether Continue Watching should offer the next episode instead.
    pub fn offers_next_episode(&self) -> bool {
        self.is_nearly_finished() && self.next_episode.is_some()
    }

    /// Groups the episodes of one show together.
    fn series_key(&self) -> String {
        match self.series_id {
            Some(id) => format!("id:{}", id),
            None => format!("name:{}", self.content_name),
        }
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
        self.update_progress_unsaved(progress);
        let _ = self.save();
    }

    /// Updates an entry without writing the file (for frequent position
    /// updates during playback - call [`save`](Self::save) periodically).
    pub fn update_progress_unsaved(&mut self, progress: WatchProgress) {
        self.history.insert(progress.content_id.clone(), progress);
    }

    pub fn get_progress(&self, content_id: &str) -> Option<&WatchProgress> {
        self.history.get(content_id)
    }

    /// Where to resume `content_id` from, if it was left part-way through.
    pub fn resume_position(&self, content_id: &str) -> Option<f64> {
        self.history.get(content_id).and_then(|p| p.resume_position())
    }

    /// Movies and shows to pick up again, most recent first. Each show
    /// appears once, for its most recently watched episode; a finished
    /// episode is kept when the next one is known so it can be offered.
    pub fn get_continue_watching(&self, limit: usize) -> Vec<WatchProgress> {
        let mut latest_per_show: HashMap<String, &WatchProgress> = HashMap::new();
        let mut items: Vec<WatchProgress> = Vec::new();

        for p in self.history.values() {
            match p.content_type.as_str() {
                "movie" if !p.is_nearly_finished() => items.push(p.clone()),
                "series" => {
                    let entry = latest_per_show.entry(p.series_key()).or_insert(p);
                    if p.last_watched > entry.last_watched {
                        *entry = p;
                    }
                }
                _ => {}
            }
        }

        items.extend(
            latest_per_show
                .into_values()
                .filter(|p| !p.is_nearly_finished() || p.offers_next_episode())
                .cloned(),
        );
        items.sort_by(|a, b| b.last_watched.cmp(&a.last_watched));
        items.truncate(limit);
        items
    }

    /// Recently watched live channels, most recent first.
    pub fn recent_channels(&self, limit: usize) -> Vec<WatchProgress> {
        let mut items: Vec<_> = self.history.values()
            .filter(|p| p.content_type == "channel")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, kind: &str, pos: f64, dur: f64, when: i64) -> WatchProgress {
        WatchProgress {
            content_id: id.to_string(),
            content_name: id.to_string(),
            content_type: kind.to_string(),
            position_seconds: pos,
            duration_seconds: dur,
            last_watched: when,
            ..Default::default()
        }
    }

    #[test]
    fn resume_position_skips_start_and_end() {
        assert_eq!(entry("a", "movie", 10.0, 100.0, 0).resume_position(), None);
        assert_eq!(entry("a", "movie", 95.0, 100.0, 0).resume_position(), None);
        assert_eq!(entry("a", "movie", 60.0, 100.0, 0).resume_position(), Some(55.0));
        // Unknown duration (player didn't report one)
        assert_eq!(entry("a", "movie", 0.0, 0.0, 0).resume_position(), None);
    }

    #[test]
    fn continue_watching_groups_episodes_and_offers_next() {
        let mut history = WatchHistory::default();
        let mut ep1 = entry("episode:1", "series", 1300.0, 1400.0, 10);
        ep1.content_name = "Show".into();
        ep1.series_id = Some(7);
        ep1.next_episode = Some(NextEpisode { episode_id: "2".into(), ..Default::default() });
        let mut ep0 = ep1.clone();
        ep0.content_id = "episode:0".into();
        ep0.last_watched = 5;
        history.update_progress_unsaved(ep0);
        history.update_progress_unsaved(ep1);
        history.update_progress_unsaved(entry("m1", "movie", 50.0, 100.0, 20));
        history.update_progress_unsaved(entry("m2", "movie", 99.0, 100.0, 30));
        history.update_progress_unsaved(entry("channel:9", "channel", 0.0, 0.0, 40));

        let items = history.get_continue_watching(10);
        let ids: Vec<_> = items.iter().map(|p| p.content_id.as_str()).collect();
        assert_eq!(ids, vec!["m1", "episode:1"]);
        assert!(items[1].offers_next_episode());

        let channels = history.recent_channels(10);
        assert_eq!(channels.len(), 1);
    }
}
