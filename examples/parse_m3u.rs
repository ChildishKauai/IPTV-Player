// Example: Parse an M3U file or URL
// Run with: cargo run --example parse_m3u

use std::env;

// Since we can't import from src in examples, we'll duplicate the parser here
// In a real application, you'd create a library crate
mod m3u_parser {
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Channel {
        pub num: String,
        pub name: String,
        pub stream_type: String,
        pub stream_id: String,
        pub stream_icon: String,
        pub epg_channel_id: Option<String>,
        pub category_id: String,
        pub direct_source: Option<String>,
    }
    
    pub struct M3UParser;
    
    impl M3UParser {
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
                            category_id: if current_group.is_empty() { "Uncategorized".to_string() } else { current_group.clone() },
                            direct_source: Some(line.to_string()),
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
            if let Some(pos) = line.rfind(',') {
                line[pos + 1..].trim().to_string()
            } else {
                String::new()
            }
        }
        
        fn extract_attribute(line: &str, attr: &str) -> String {
            let pattern_with_quotes = format!("{}=\"", attr);
            if let Some(start) = line.find(&pattern_with_quotes) {
                let start = start + pattern_with_quotes.len();
                if let Some(end) = line[start..].find('"') {
                    return line[start..start + end].to_string();
                }
            }
            
            let pattern_without_quotes = format!("{}=", attr);
            if let Some(start) = line.find(&pattern_without_quotes) {
                let start = start + pattern_without_quotes.len();
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
    }
}

fn main() {
    // Example M3U content
    let sample_m3u = r#"#EXTM3U
#EXTINF:-1 tvg-id="bbc1.uk" tvg-logo="http://example.com/bbc1.png" group-title="UK Channels",BBC One
http://example.com/bbc1.m3u8
#EXTINF:-1 tvg-id="bbc2.uk" tvg-logo="http://example.com/bbc2.png" group-title="UK Channels",BBC Two
http://example.com/bbc2.m3u8
#EXTINF:-1 tvg-id="cnn.us" tvg-logo="http://example.com/cnn.png" group-title="News",CNN
http://example.com/cnn.m3u8
#EXTINF:-1 tvg-logo="http://example.com/sports.png" group-title="Sports",Sports Channel
http://example.com/sports.m3u8
"#;

    println!("=== M3U Parser Example ===\n");
    
    // Parse the content
    match m3u_parser::M3UParser::parse_content(sample_m3u) {
        Ok(channels) => {
            println!("Successfully parsed {} channels:\n", channels.len());
            
            // Group channels by category
            let mut categories: Vec<String> = channels.iter()
                .map(|ch| ch.category_id.clone())
                .collect();
            categories.sort();
            categories.dedup();
            
            for category in categories {
                println!("📁 Category: {}", category);
                for channel in channels.iter().filter(|ch| ch.category_id == category) {
                    println!("  {} - {} (ID: {})", 
                        channel.num,
                        channel.name,
                        channel.epg_channel_id.as_ref().unwrap_or(&"N/A".to_string())
                    );
                    if !channel.stream_icon.is_empty() {
                        println!("    Logo: {}", channel.stream_icon);
                    }
                    if let Some(url) = &channel.direct_source {
                        println!("    URL: {}", url);
                    }
                    println!();
                }
            }
            
            println!("\n✅ Total categories: {}", 
                channels.iter()
                    .map(|ch| ch.category_id.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .len()
            );
        }
        Err(e) => {
            eprintln!("❌ Error parsing M3U: {}", e);
        }
    }
    
    println!("\n=== Usage ===");
    println!("To parse your own M3U file:");
    println!("  cargo run --example parse_m3u <path_to_file.m3u>");
    println!("\nTo parse from URL (requires implementation):");
    println!("  cargo run --example parse_m3u http://example.com/playlist.m3u");
}
