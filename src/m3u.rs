use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::models::Channel;

#[allow(dead_code)]
pub struct M3UParser;

#[allow(dead_code)]
impl M3UParser {
    pub fn parse_file(path: &str) -> Result<Vec<Channel>, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut channels = Vec::new();
        let mut current_name = String::new();
        let mut current_logo = String::new();
        let mut current_group = String::new();
        let mut current_tvg_id = String::new();
        let mut channel_num = 1;
        
        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            let line = line.trim();
            
            if line.starts_with("#EXTINF:") {
                // Parse channel info
                current_name = Self::extract_name(&line);
                current_logo = Self::extract_attribute(&line, "tvg-logo");
                current_group = Self::extract_attribute(&line, "group-title");
                current_tvg_id = Self::extract_attribute(&line, "tvg-id");
            } else if !line.is_empty() && !line.starts_with("#") {
                // This is the stream URL
                if !current_name.is_empty() {
                    channels.push(Channel {
                        num: channel_num.to_string(),
                        name: current_name.clone(),
                        stream_id: format!("m3u_{}", channels.len()),
                        stream_type: "live".to_string(),
                        stream_icon: current_logo.clone(),
                        epg_channel_id: if current_tvg_id.is_empty() { None } else { Some(current_tvg_id.clone()) },
                        added: None,
                        category_id: if current_group.is_empty() { "Uncategorized".to_string() } else { current_group.clone() },
                        category_name: if current_group.is_empty() { None } else { Some(current_group.clone()) },
                        category_ids: None,
                        custom_sid: None,
                        tv_archive: None,
                        direct_source: Some(line.to_string()),
                        tv_archive_duration: None,
                        is_adult: None,
                    });
                    channel_num += 1;
                    current_name.clear();
                    current_logo.clear();
                    current_tvg_id.clear();
                }
            }
        }
        
        Ok(channels)
    }
    
    pub fn parse_url(url: &str) -> Result<Vec<Channel>, String> {
        let response = reqwest::blocking::get(url)
            .map_err(|e| e.to_string())?
            .text()
            .map_err(|e| e.to_string())?;
        
        Self::parse_content(&response)
    }
    
    pub fn parse_content(content: &str) -> Result<Vec<Channel>, String> {
        let mut channels = Vec::new();
        let mut current_name = String::new();
        let mut current_logo = String::new();
        let mut current_group = String::new();
        let mut current_tvg_id = String::new();
        let mut channel_num = 1;
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.starts_with("#EXTINF:") {
                current_name = Self::extract_name(&line);
                current_logo = Self::extract_attribute(&line, "tvg-logo");
                current_group = Self::extract_attribute(&line, "group-title");
                current_tvg_id = Self::extract_attribute(&line, "tvg-id");
            } else if !line.is_empty() && !line.starts_with("#") {
                if !current_name.is_empty() {
                    channels.push(Channel {
                        num: channel_num.to_string(),
                        name: current_name.clone(),
                        stream_id: format!("m3u_{}", channels.len()),
                        stream_type: "live".to_string(),
                        stream_icon: current_logo.clone(),
                        epg_channel_id: if current_tvg_id.is_empty() { None } else { Some(current_tvg_id.clone()) },
                        added: None,
                        category_id: if current_group.is_empty() { "Uncategorized".to_string() } else { current_group.clone() },
                        category_name: if current_group.is_empty() { None } else { Some(current_group.clone()) },
                        category_ids: None,
                        custom_sid: None,
                        tv_archive: None,
                        direct_source: Some(line.to_string()),
                        tv_archive_duration: None,
                        is_adult: None,
                    });
                    channel_num += 1;
                    current_name.clear();
                    current_logo.clear();
                    current_tvg_id.clear();
                }
            }
        }
        
        Ok(channels)
    }
    
    fn extract_name(line: &str) -> String {
        // Name comes after the last comma in the EXTINF line
        if let Some(pos) = line.rfind(',') {
            line[pos + 1..].trim().to_string()
        } else {
            String::new()
        }
    }
    
    fn extract_attribute(line: &str, attr: &str) -> String {
        // Try with quotes first (tvg-logo="...")
        let pattern_with_quotes = format!("{}=\"", attr);
        if let Some(start) = line.find(&pattern_with_quotes) {
            let start = start + pattern_with_quotes.len();
            if let Some(end) = line[start..].find('"') {
                return line[start..start + end].to_string();
            }
        }
        
        // Try without quotes (group-title=value or tvg-id=value)
        let pattern_without_quotes = format!("{}=", attr);
        if let Some(start) = line.find(&pattern_without_quotes) {
            let start = start + pattern_without_quotes.len();
            // Find the end: either a space or a comma
            let remaining = &line[start..];
            if let Some(space_pos) = remaining.find(' ') {
                return remaining[..space_pos].trim().to_string();
            } else if let Some(comma_pos) = remaining.find(',') {
                return remaining[..comma_pos].trim().to_string();
            } else {
                return remaining.trim().to_string();
            }
        }
        
        String::new()
    }
    
    /// Parse M3U8 extended format with additional metadata
    pub fn parse_extended(content: &str) -> Result<Vec<Channel>, String> {
        if !content.starts_with("#EXTM3U") {
            return Err("Invalid M3U format: missing #EXTM3U header".to_string());
        }
        Self::parse_content(content)
    }
    
    /// Get all unique categories from parsed channels
    pub fn extract_categories(channels: &[Channel]) -> Vec<String> {
        let mut categories: Vec<String> = channels
            .iter()
            .map(|ch| ch.category_id.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_basic_extinf() {
        let content = r#"#EXTM3U
#EXTINF:-1,Test Channel
http://example.com/stream.m3u8
"#;
        let channels = M3UParser::parse_content(content).unwrap();
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].name, "Test Channel");
    }
    
    #[test]
    fn test_parse_with_attributes() {
        let content = r#"#EXTM3U
#EXTINF:-1 tvg-id="test.tv" tvg-logo="http://example.com/logo.png" group-title="News",Test Channel
http://example.com/stream.m3u8
"#;
        let channels = M3UParser::parse_content(content).unwrap();
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].name, "Test Channel");
        assert_eq!(channels[0].stream_icon, "http://example.com/logo.png");
        assert_eq!(channels[0].category_id, "News");
        assert_eq!(channels[0].epg_channel_id, Some("test.tv".to_string()));
    }
    
    #[test]
    fn test_extract_categories() {
        let content = r#"#EXTM3U
#EXTINF:-1 group-title="News",Channel 1
http://example.com/1.m3u8
#EXTINF:-1 group-title="Sports",Channel 2
http://example.com/2.m3u8
#EXTINF:-1 group-title="News",Channel 3
http://example.com/3.m3u8
"#;
        let channels = M3UParser::parse_content(content).unwrap();
        let categories = M3UParser::extract_categories(&channels);
        assert_eq!(categories.len(), 2);
        assert!(categories.contains(&"News".to_string()));
        assert!(categories.contains(&"Sports".to_string()));
    }
}
