use serde::{Deserialize, Deserializer, Serialize};

// Default value for stream_type field
fn default_stream_type() -> String {
    "live".to_string()
}

// Custom deserializer for fields that can be either string or integer
fn deserialize_string_or_int<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;
    
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        Value::Null => Ok(String::new()),
        _ => Err(Error::custom("expected string or number")),
    }
}

// Custom deserializer for integer fields that can be either int or string
fn deserialize_int_or_string<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;
    
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Number(n) => n.as_i64().map(|i| Some(i as i32)).ok_or_else(|| Error::custom("invalid number")),
        Value::String(s) => {
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<i32>().map(Some).map_err(|_| Error::custom("expected integer or numeric string"))
            }
        }
        Value::Null => Ok(None),
        _ => Err(Error::custom("expected integer or string")),
    }
}

// Custom deserializer for float fields that can be either float or string
fn deserialize_float_or_string<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;
    
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Number(n) => n.as_f64().map(|f| Some(f as f32)).ok_or_else(|| Error::custom("invalid number")),
        Value::String(s) => {
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<f32>().map(Some).map_err(|_| Error::custom("expected float or numeric string"))
            }
        }
        Value::Null => Ok(None),
        _ => Err(Error::custom("expected float or string")),
    }
}

// Custom deserializer for array of strings that might contain nulls
fn deserialize_string_array_with_nulls<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Array(arr) => {
            let strings: Vec<String> = arr.into_iter()
                .filter_map(|v| match v {
                    Value::String(s) => Some(s),
                    _ => None, // Skip null and other non-string values
                })
                .collect();
            Ok(strings)
        }
        Value::Null => Ok(Vec::new()),
        _ => Ok(Vec::new()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    #[serde(deserialize_with = "deserialize_string_or_int", default)]
    pub num: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_stream_type")]
    pub stream_type: String,
    #[serde(deserialize_with = "deserialize_string_or_int", alias = "stream_id", default)]
    pub stream_id: String,
    #[serde(default)]
    pub stream_icon: String,
    #[serde(default)]
    pub epg_channel_id: Option<String>,
    #[serde(default)]
    pub added: Option<String>,
    #[serde(deserialize_with = "deserialize_string_or_int", default)]
    pub category_id: String,
    #[serde(default)]
    pub category_name: Option<String>,
    #[serde(default)]
    pub category_ids: Option<Vec<i32>>,
    #[serde(default)]
    pub custom_sid: Option<String>,
    #[serde(deserialize_with = "deserialize_int_or_string", default)]
    pub tv_archive: Option<i32>,
    #[serde(default)]
    pub direct_source: Option<String>,
    #[serde(deserialize_with = "deserialize_int_or_string", default)]
    pub tv_archive_duration: Option<i32>,
    #[serde(deserialize_with = "deserialize_int_or_string", default)]
    pub is_adult: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    #[serde(deserialize_with = "deserialize_string_or_int")]
    pub category_id: String,
    pub category_name: String,
    #[serde(default)]
    pub parent_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    #[serde(default)]
    pub num: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub series_id: i32,
    #[serde(default)]
    pub cover: Option<String>,
    #[serde(default)]
    pub plot: Option<String>,
    #[serde(default)]
    pub cast: Option<String>,
    #[serde(default)]
    pub director: Option<String>,
    #[serde(default)]
    pub genre: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub last_modified: Option<String>,
    #[serde(default)]
    pub rating: Option<String>,
    #[serde(deserialize_with = "deserialize_float_or_string", default)]
    pub rating_5based: Option<f32>,
    #[serde(deserialize_with = "deserialize_string_array_with_nulls", default)]
    pub backdrop_path: Vec<String>,
    #[serde(default)]
    pub youtube_trailer: Option<String>,
    #[serde(default)]
    pub episode_run_time: Option<String>,
    #[serde(default)]
    pub tmdb: Option<String>,
    #[serde(deserialize_with = "deserialize_string_or_int", default)]
    pub category_id: String,
    #[serde(default)]
    pub category_ids: Option<Vec<i32>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: String,
    pub episode_num: i32,
    pub title: String,
    pub container_extension: String,
    pub info: EpisodeInfo,
    pub custom_sid: Option<String>,
    pub added: String,
    pub season: i32,
    pub direct_source: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeInfo {
    pub name: Option<String>,
    pub overview: Option<String>,
    pub movie_image: Option<String>,
    pub rating: Option<String>,
    pub release_date: Option<String>,
    pub duration_secs: Option<i32>,
    pub duration: Option<String>,
}

/// EPG (Electronic Program Guide) entry for a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpgProgram {
    /// Unique EPG ID
    #[serde(default)]
    pub id: String,
    /// Channel EPG ID
    #[serde(default, alias = "channel_id")]
    pub epg_id: String,
    /// Program title
    #[serde(default)]
    pub title: String,
    /// Program language (e.g., "en")
    #[serde(default, alias = "lang")]
    pub language: String,
    /// Start time (Unix timestamp as string)
    #[serde(default)]
    pub start: String,
    /// End time (Unix timestamp as string) 
    #[serde(default, alias = "stop")]
    pub end: String,
    /// Start timestamp as readable date
    #[serde(default)]
    pub start_timestamp: String,
    /// Stop timestamp as readable date
    #[serde(default)]
    pub stop_timestamp: String,
    /// Program description
    #[serde(default)]
    pub description: String,
    /// Has archive available
    #[serde(default)]
    pub has_archive: i32,
    /// Now playing indicator
    #[serde(default)]
    pub now_playing: i32,
}

#[allow(dead_code)]
impl EpgProgram {
    /// Get start time as Unix timestamp
    pub fn start_timestamp_unix(&self) -> i64 {
        self.start.parse().unwrap_or(0)
    }
    
    /// Get end time as Unix timestamp
    pub fn end_timestamp_unix(&self) -> i64 {
        self.end.parse().unwrap_or(0)
    }
    
    /// Check if the program is currently playing
    pub fn is_now_playing(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let start = self.start_timestamp_unix();
        let end = self.end_timestamp_unix();
        now >= start && now < end
    }
    
    /// Get progress percentage (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let start = self.start_timestamp_unix();
        let end = self.end_timestamp_unix();
        
        if end <= start {
            return 0.0;
        }
        
        let elapsed = now - start;
        let duration = end - start;
        
        (elapsed as f32 / duration as f32).clamp(0.0, 1.0)
    }
    
    /// Format start time as HH:MM
    pub fn start_time_formatted(&self) -> String {
        let ts = self.start_timestamp_unix();
        if ts == 0 {
            return String::new();
        }
        
        // Simple time formatting (UTC - would need timezone handling for production)
        let secs_in_day = ts % 86400;
        let hours = secs_in_day / 3600;
        let minutes = (secs_in_day % 3600) / 60;
        format!("{:02}:{:02}", hours, minutes)
    }
    
    /// Format end time as HH:MM
    pub fn end_time_formatted(&self) -> String {
        let ts = self.end_timestamp_unix();
        if ts == 0 {
            return String::new();
        }
        
        let secs_in_day = ts % 86400;
        let hours = secs_in_day / 3600;
        let minutes = (secs_in_day % 3600) / 60;
        format!("{:02}:{:02}", hours, minutes)
    }
}
