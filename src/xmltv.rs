use crate::models::EpgProgram;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

/// XMLTV EPG parser for external EPG sources
pub struct XmltvParser;

impl XmltvParser {
    /// Parse XMLTV content from a string
    pub fn parse_content(content: &str) -> Result<HashMap<String, Vec<EpgProgram>>, String> {
        let mut reader = Reader::from_str(content);
        reader.trim_text(true);

        let mut programs_by_channel: HashMap<String, Vec<EpgProgram>> = HashMap::new();
        let mut buf = Vec::new();

        let mut current_channel_id = String::new();
        let mut current_title = String::new();
        let mut current_desc = String::new();
        let mut current_start = String::new();
        let mut current_stop = String::new();
        let mut current_lang = String::from("en");
        let mut in_title = false;
        let mut in_desc = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    match e.name().as_ref() {
                        b"programme" => {
                            // Reset current program data
                            current_channel_id.clear();
                            current_title.clear();
                            current_desc.clear();
                            current_start.clear();
                            current_stop.clear();
                            current_lang = String::from("en");

                            // Extract attributes
                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    match attr.key.as_ref() {
                                        b"channel" => {
                                            if let Ok(value) = std::str::from_utf8(&attr.value) {
                                                current_channel_id = value.to_string();
                                            }
                                        }
                                        b"start" => {
                                            if let Ok(value) = std::str::from_utf8(&attr.value) {
                                                current_start = value.to_string();
                                            }
                                        }
                                        b"stop" => {
                                            if let Ok(value) = std::str::from_utf8(&attr.value) {
                                                current_stop = value.to_string();
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        b"title" => {
                            in_title = true;
                            // Check for lang attribute
                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    if attr.key.as_ref() == b"lang" {
                                        if let Ok(value) = std::str::from_utf8(&attr.value) {
                                            current_lang = value.to_string();
                                        }
                                    }
                                }
                            }
                        }
                        b"desc" => {
                            in_desc = true;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(e)) => {
                    if in_title {
                        if let Ok(text) = e.unescape() {
                            current_title = text.to_string();
                        }
                    } else if in_desc {
                        if let Ok(text) = e.unescape() {
                            current_desc = text.to_string();
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    match e.name().as_ref() {
                        b"title" => {
                            in_title = false;
                        }
                        b"desc" => {
                            in_desc = false;
                        }
                        b"programme" => {
                            // Create EPG program if we have valid data
                            if !current_channel_id.is_empty() && !current_title.is_empty() {
                                let start_timestamp = Self::parse_xmltv_time(&current_start);
                                let stop_timestamp = Self::parse_xmltv_time(&current_stop);

                                // Convert to Unix timestamps (seconds)
                                let start_unix = Self::xmltv_to_unix(&current_start);
                                let stop_unix = Self::xmltv_to_unix(&current_stop);

                                let program = EpgProgram {
                                    id: format!("{}_{}", current_channel_id, start_unix),
                                    epg_id: current_channel_id.clone(),
                                    title: current_title.clone(),
                                    language: current_lang.clone(),
                                    start: start_unix,
                                    end: stop_unix,
                                    start_timestamp,
                                    stop_timestamp,
                                    description: current_desc.clone(),
                                    has_archive: 0,
                                    now_playing: 0,
                                };

                                programs_by_channel
                                    .entry(current_channel_id.clone())
                                    .or_insert_with(Vec::new)
                                    .push(program);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(format!("Error parsing XMLTV at position {}: {:?}", reader.buffer_position(), e));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(programs_by_channel)
    }

    /// Parse XMLTV from a URL with retry logic
    pub fn parse_url(url: &str) -> Result<HashMap<String, Vec<EpgProgram>>, String> {
        use std::time::Duration;

        // Create client with timeout
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .user_agent("IPTV-Player/1.0")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        // Retry up to 3 times with exponential backoff
        let mut last_error = String::new();
        for attempt in 0..3 {
            if attempt > 0 {
                let delay = Duration::from_secs(2u64.pow(attempt - 1)); // 0s, 2s, 4s
                eprintln!("[EPG] Retry attempt {} after {:?} delay", attempt + 1, delay);
                std::thread::sleep(delay);
            }

            match client.get(url).send() {
                Ok(response) => {
                    // Check status code
                    if !response.status().is_success() {
                        last_error = format!("HTTP {}", response.status());
                        eprintln!("[EPG] HTTP error: {}", last_error);
                        continue;
                    }

                    let content = if url.ends_with(".gz") {
                        // Handle gzipped content
                        use std::io::Read;
                        let mut decoder = flate2::read::GzDecoder::new(response);
                        let mut decompressed = String::new();
                        match decoder.read_to_string(&mut decompressed) {
                            Ok(_) => decompressed,
                            Err(e) => {
                                last_error = format!("Failed to decompress gzip: {}", e);
                                eprintln!("[EPG] {}", last_error);
                                continue;
                            }
                        }
                    } else {
                        match response.text() {
                            Ok(text) => text,
                            Err(e) => {
                                last_error = format!("Failed to read response: {}", e);
                                eprintln!("[EPG] {}", last_error);
                                continue;
                            }
                        }
                    };

                    // Success! Parse the content
                    return Self::parse_content(&content);
                }
                Err(e) => {
                    last_error = format!("Network error: {}", e);
                    eprintln!("[EPG] {}", last_error);
                }
            }
        }

        Err(format!("Failed after 3 attempts: {}", last_error))
    }

    /// Convert XMLTV time format (YYYYMMDDHHmmss +0000) to Unix timestamp
    fn xmltv_to_unix(xmltv_time: &str) -> String {
        use chrono::{DateTime, NaiveDateTime, Utc};

        // XMLTV format: 20240120120000 +0000
        let time_part = xmltv_time.split_whitespace().next().unwrap_or(xmltv_time);

        if time_part.len() < 14 {
            return "0".to_string();
        }

        // Parse: YYYYMMDDHHmmss
        let year = &time_part[0..4];
        let month = &time_part[4..6];
        let day = &time_part[6..8];
        let hour = &time_part[8..10];
        let minute = &time_part[10..12];
        let second = &time_part[12..14];

        let datetime_str = format!("{}-{}-{} {}:{}:{}", year, month, day, hour, minute, second);

        match NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S") {
            Ok(naive) => {
                let dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
                dt.timestamp().to_string()
            }
            Err(_) => "0".to_string()
        }
    }

    /// Convert XMLTV time to readable timestamp (YYYY-MM-DD HH:mm:ss)
    fn parse_xmltv_time(xmltv_time: &str) -> String {
        let time_part = xmltv_time.split_whitespace().next().unwrap_or(xmltv_time);

        if time_part.len() < 14 {
            return String::new();
        }

        let year = &time_part[0..4];
        let month = &time_part[4..6];
        let day = &time_part[6..8];
        let hour = &time_part[8..10];
        let minute = &time_part[10..12];
        let second = &time_part[12..14];

        format!("{}-{}-{} {}:{}:{}", year, month, day, hour, minute, second)
    }

    /// Get programs for a specific channel by its tvg-id
    pub fn get_programs_for_channel(
        programs_map: &HashMap<String, Vec<EpgProgram>>,
        tvg_id: &str
    ) -> Option<Vec<EpgProgram>> {
        programs_map.get(tvg_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xmltv_basic() {
        let xmltv = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE tv SYSTEM "xmltv.dtd">
<tv>
  <programme start="20240120120000 +0000" stop="20240120130000 +0000" channel="bbc1.uk">
    <title lang="en">Test Program</title>
    <desc lang="en">This is a test program description.</desc>
  </programme>
  <programme start="20240120130000 +0000" stop="20240120140000 +0000" channel="bbc1.uk">
    <title lang="en">Another Show</title>
    <desc lang="en">Another show description.</desc>
  </programme>
</tv>"#;

        let result = XmltvParser::parse_content(xmltv);
        assert!(result.is_ok());

        let programs = result.unwrap();
        assert_eq!(programs.len(), 1);
        assert!(programs.contains_key("bbc1.uk"));

        let bbc_programs = programs.get("bbc1.uk").unwrap();
        assert_eq!(bbc_programs.len(), 2);
        assert_eq!(bbc_programs[0].title, "Test Program");
        assert_eq!(bbc_programs[1].title, "Another Show");
    }

    #[test]
    fn test_xmltv_to_unix() {
        let time = "20240120120000 +0000";
        let unix = XmltvParser::xmltv_to_unix(time);
        assert!(!unix.is_empty());
        assert_ne!(unix, "0");
    }
}
