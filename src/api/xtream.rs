use crate::models::*;
use serde_json::Value;
use std::time::Duration;

pub struct XtreamClient {
    base_url: String,
    username: String,
    password: String,
    client: reqwest::blocking::Client,
}

impl XtreamClient {
    pub fn new(server_url: String, username: String, password: String) -> Self {
        // Trim whitespace and remove trailing slash from server URL
        let base_url = server_url.trim().trim_end_matches('/').to_string();
        let username = username.trim().to_string();
        let password = password.trim().to_string();
        
        // Create client with timeout and redirect settings
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))  // Increased for large responses (13MB+)
            .connect_timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        
        Self {
            base_url,
            username,
            password,
            client,
        }
    }

    fn api_url(&self, action: &str) -> String {
        format!(
            "{}/player_api.php?username={}&password={}&action={}",
            self.base_url, self.username, self.password, action
        )
    }

    pub fn authenticate(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/player_api.php?username={}&password={}",
            self.base_url, self.username, self.password
        );
        
        let response = self.client.get(&url).send()?;
        
        if !response.status().is_success() {
            return Ok(false);
        }
        
        // Try to parse the response as JSON to verify it's valid
        let json: Result<Value, _> = response.json();
        match json {
            Ok(val) => {
                // Check if the response contains user_info (valid authentication)
                if val.get("user_info").is_some() {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false),
        }
    }

    pub fn get_live_categories(&self) -> Result<Vec<Category>, Box<dyn std::error::Error>> {
        let url = self.api_url("get_live_categories");
        let response = self.client.get(&url).send()?;
        
        if !response.status().is_success() {
            return Err(format!("API returned status: {}", response.status()).into());
        }
        
        // Read raw bytes to handle encoding issues
        let bytes = response.bytes()?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        
        let categories: Vec<Category> = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse categories: {}. Response: {}", e, &text[..text.len().min(200)]))?;
        Ok(categories)
    }

    pub fn get_live_streams(&self) -> Result<Vec<Channel>, Box<dyn std::error::Error>> {
        let url = self.api_url("get_live_streams");
        eprintln!("[DEBUG] Fetching channels from: {}", url);
        
        let response = self.client.get(&url).send()?;
        eprintln!("[DEBUG] Response status: {}", response.status());
        eprintln!("[DEBUG] Response headers: {:?}", response.headers());
        
        if !response.status().is_success() {
            return Err(format!("API returned status: {}", response.status()).into());
        }
        
        // Read raw bytes first to handle encoding issues
        let bytes = response.bytes().map_err(|e| {
            eprintln!("[DEBUG] Error reading response bytes: {}", e);
            format!("Error reading response bytes: {}", e)
        })?;
        
        eprintln!("[DEBUG] Response bytes length: {}", bytes.len());
        
        // Try to convert to UTF-8, handling invalid sequences
        let text = String::from_utf8_lossy(&bytes).into_owned();
        
        eprintln!("[DEBUG] Response text length: {} chars", text.len());
        eprintln!("[DEBUG] Response first 500 chars: {}", &text[..text.len().min(500)]);
        
        // First, let's check if it's a valid JSON array
        let json_value: Value = serde_json::from_str(&text)
            .map_err(|e| format!("Invalid JSON response: {}. Response start: {}", e, &text[..text.len().min(200)]))?;
        
        // Check if it's an array
        if !json_value.is_array() {
            return Err(format!("Expected array, got: {}", &text[..text.len().min(200)]).into());
        }
        
        // Try to deserialize with better error messages
        let mut channels: Vec<Channel> = serde_json::from_str(&text)
            .map_err(|e| {
                // Try to find which field is causing the issue
                let array = json_value.as_array().unwrap();
                if let Some(first_item) = array.first() {
                    let error_msg = format!("Failed to parse channels: {}. First item: {}", e, 
                        serde_json::to_string_pretty(first_item).unwrap_or_default());
                    eprintln!("\n=== DESERIALIZATION ERROR ===");
                    eprintln!("{}", error_msg);
                    eprintln!("=========================\n");
                    error_msg
                } else {
                    format!("Failed to parse channels: {}. Empty array", e)
                }
            })?;
        
        // Ensure num field is populated
        for (idx, channel) in channels.iter_mut().enumerate() {
            if channel.num.is_empty() {
                channel.num = (idx + 1).to_string();
            }
        }
        
        Ok(channels)
    }

    #[allow(dead_code)]
    pub fn get_vod_categories(&self) -> Result<Vec<Category>, Box<dyn std::error::Error>> {
        let url = self.api_url("get_vod_categories");
        let response = self.client.get(&url).send()?;
        
        if !response.status().is_success() {
            return Err(format!("API returned status: {}", response.status()).into());
        }
        
        // Read raw bytes to handle encoding issues
        let bytes = response.bytes()?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        
        let categories: Vec<Category> = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse VOD categories: {}. Response: {}", e, &text[..text.len().min(200)]))?;
        Ok(categories)
    }

    #[allow(dead_code)]
    pub fn get_vod_streams(&self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let url = self.api_url("get_vod_streams");
        eprintln!("[DEBUG] Fetching VOD streams from: {}", url);
        
        let response = self.client.get(&url).send()?;
        eprintln!("[DEBUG] VOD Response status: {}", response.status());
        
        if !response.status().is_success() {
            return Err(format!("API returned status: {}", response.status()).into());
        }
        
        // Read raw bytes to handle encoding issues
        let bytes = response.bytes()?;
        eprintln!("[DEBUG] VOD Response bytes length: {}", bytes.len());
        
        // Check if response is too small (likely empty or error)
        if bytes.len() < 10 {
            let text = String::from_utf8_lossy(&bytes);
            eprintln!("[DEBUG] VOD Response too small: {}", text);
            // Return empty array if response is just "[]"
            if text.trim() == "[]" {
                return Ok(Vec::new());
            }
        }
        
        let text = String::from_utf8_lossy(&bytes).into_owned();
        eprintln!("[DEBUG] VOD First 200 chars: {}", &text[..text.len().min(200)]);
        
        let streams: Vec<Value> = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse VOD streams: {}. Response: {}", e, &text[..text.len().min(200)]))?;
        
        eprintln!("[DEBUG] VOD Parsed {} movies", streams.len());
        Ok(streams)
    }

    pub fn get_series_categories(&self) -> Result<Vec<Category>, Box<dyn std::error::Error>> {
        let url = self.api_url("get_series_categories");
        let response = self.client.get(&url).send()?;
        
        if !response.status().is_success() {
            return Err(format!("API returned status: {}", response.status()).into());
        }
        
        // Read raw bytes to handle encoding issues
        let bytes = response.bytes()?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        
        let categories: Vec<Category> = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse series categories: {}. Response: {}", e, &text[..text.len().min(200)]))?;
        Ok(categories)
    }

    pub fn get_series(&self) -> Result<Vec<Series>, Box<dyn std::error::Error>> {
        let url = self.api_url("get_series");
        eprintln!("[DEBUG] Fetching series from: {}", url);
        
        let response = self.client.get(&url).send()?;
        eprintln!("[DEBUG] Series Response status: {}", response.status());
        
        if !response.status().is_success() {
            return Err(format!("API returned status: {}", response.status()).into());
        }
        
        // Read raw bytes to handle encoding issues
        let bytes = response.bytes()?;
        eprintln!("[DEBUG] Series Response bytes length: {}", bytes.len());
        
        let text = String::from_utf8_lossy(&bytes).into_owned();
        
        let mut series: Vec<Series> = serde_json::from_str(&text)
            .map_err(|e| {
                eprintln!("[DEBUG] Series parse error: {}", e);
                eprintln!("[DEBUG] First 500 chars: {}", &text[..text.len().min(500)]);
                format!("Failed to parse series: {}. Response: {}", e, &text[..text.len().min(200)])
            })?;
        
        // Ensure num field is populated
        for (idx, s) in series.iter_mut().enumerate() {
            if s.num == 0 {
                s.num = (idx + 1) as i32;
            }
        }
        
        Ok(series)
    }

    pub fn get_series_info(&self, series_id: i32) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_series_info&series_id={}",
            self.base_url, self.username, self.password, series_id
        );
        let response = self.client.get(&url).send()?;
        
        // Read raw bytes to handle encoding issues
        let bytes = response.bytes()?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        
        let info: Value = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse series info: {}", e))?;
        Ok(info)
    }

    pub fn get_stream_url(&self, stream_id: &str, extension: &str) -> String {
        format!(
            "{}/movie/{}/{}/{}.{}",
            self.base_url, self.username, self.password, stream_id, extension
        )
    }

    pub fn get_live_stream_url(&self, stream_id: &str) -> String {
        format!(
            "{}/live/{}/{}/{}.ts",
            self.base_url, self.username, self.password, stream_id
        )
    }

    pub fn get_episode_url(&self, episode_id: &str, extension: &str) -> String {
        format!(
            "{}/series/{}/{}/{}.{}",
            self.base_url, self.username, self.password, episode_id, extension
        )
    }
    
    /// Get short EPG for a specific stream (current and next few programs)
    pub fn get_short_epg(&self, stream_id: &str) -> Result<Vec<EpgProgram>, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_short_epg&stream_id={}&limit=4",
            self.base_url, self.username, self.password, stream_id
        );
        
        let response = self.client.get(&url).send()?;
        
        if !response.status().is_success() {
            return Err(format!("EPG API returned status: {}", response.status()).into());
        }
        
        let text = response.text()?;
        
        // Parse the response - EPG data is usually in epg_listings array
        let json: Value = serde_json::from_str(&text)?;
        
        if let Some(listings) = json.get("epg_listings").and_then(|v| v.as_array()) {
            let programs: Vec<EpgProgram> = listings.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
            Ok(programs)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Get EPG for all streams for a specific date range
    #[allow(dead_code)]
    pub fn get_simple_data_table(&self, stream_id: &str) -> Result<Vec<EpgProgram>, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_simple_data_table&stream_id={}",
            self.base_url, self.username, self.password, stream_id
        );
        
        let response = self.client.get(&url).send()?;
        
        if !response.status().is_success() {
            return Err(format!("EPG API returned status: {}", response.status()).into());
        }
        
        let text = response.text()?;
        let json: Value = serde_json::from_str(&text)?;
        
        if let Some(listings) = json.get("epg_listings").and_then(|v| v.as_array()) {
            let programs: Vec<EpgProgram> = listings.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
            Ok(programs)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Get XMLTV EPG URL (for full EPG data)
    #[allow(dead_code)]
    pub fn get_xmltv_url(&self) -> String {
        format!(
            "{}/xmltv.php?username={}&password={}",
            self.base_url, self.username, self.password
        )
    }
}
