# M3U Parsing Module

This module provides comprehensive M3U/M3U8 playlist parsing functionality for the IPTV player.

## Features

✅ **Complete M3U/M3U8 Support**
- Parse M3U files from local filesystem
- Parse M3U content from URLs
- Parse M3U content from strings
- Support for extended M3U format (#EXTM3U)

✅ **Attribute Extraction**
- `tvg-id`: EPG channel identifier
- `tvg-logo`: Channel logo URL
- `group-title`: Category/group name
- Channel name extraction
- Stream URL extraction

✅ **Advanced Features**
- Category extraction and grouping
- Channel numbering
- Validation of M3U format
- Flexible attribute parsing (with/without quotes)
- Comprehensive error handling

## Usage

### Parse from File

```rust
use m3u::M3UParser;

let channels = M3UParser::parse_file("playlist.m3u")?;
println!("Parsed {} channels", channels.len());
```

### Parse from URL

```rust
let channels = M3UParser::parse_url("http://example.com/playlist.m3u")?;
```

### Parse from String Content

```rust
let m3u_content = r#"#EXTM3U
#EXTINF:-1 tvg-id="bbc1.uk" tvg-logo="logo.png" group-title="News",BBC One
http://example.com/stream.m3u8
"#;

let channels = M3UParser::parse_content(m3u_content)?;
```

### Extract Categories

```rust
let channels = M3UParser::parse_file("playlist.m3u")?;
let categories = M3UParser::extract_categories(&channels);

for category in categories {
    println!("Category: {}", category);
}
```

### Extended Format with Validation

```rust
// Validates that content starts with #EXTM3U
let channels = M3UParser::parse_extended(m3u_content)?;
```

## M3U Format Support

### Basic Format
```
#EXTM3U
#EXTINF:-1,Channel Name
http://example.com/stream.m3u8
```

### Extended Format with Attributes
```
#EXTM3U
#EXTINF:-1 tvg-id="channel.id" tvg-logo="http://example.com/logo.png" group-title="Category",Channel Name
http://example.com/stream.m3u8
```

### Supported Attributes

| Attribute | Description | Required |
|-----------|-------------|----------|
| `tvg-id` | EPG channel identifier | No |
| `tvg-logo` | URL to channel logo | No |
| `group-title` | Category/group name | No |
| Channel name | After the last comma | Yes |
| Stream URL | Line after EXTINF | Yes |

## Channel Structure

Parsed channels are converted to the `Channel` struct:

```rust
pub struct Channel {
    pub num: String,              // Sequential channel number
    pub name: String,             // Channel name
    pub stream_id: String,        // Unique identifier (m3u_X)
    pub stream_type: String,      // Always "live" for M3U
    pub stream_icon: String,      // Logo URL from tvg-logo
    pub epg_channel_id: Option<String>,  // From tvg-id
    pub category_id: String,      // From group-title or "Uncategorized"
    pub direct_source: Option<String>,   // The stream URL
    // ... other fields
}
```

## Examples

Run the example to see M3U parsing in action:

```bash
cargo run --example parse_m3u
```

This will parse a sample M3U playlist and display:
- All channels grouped by category
- Channel numbers and names
- EPG IDs and logos
- Stream URLs

## Error Handling

All parsing functions return `Result<Vec<Channel>, String>`:

```rust
match M3UParser::parse_file("playlist.m3u") {
    Ok(channels) => {
        println!("Successfully parsed {} channels", channels.len());
    }
    Err(e) => {
        eprintln!("Error parsing M3U: {}", e);
    }
}
```

## Implementation Details

### Attribute Parsing
The parser supports both quoted and unquoted attribute values:
- `tvg-logo="http://example.com/logo.png"` (with quotes)
- `group-title=News` (without quotes)

### Category Handling
- If `group-title` is not provided, channels are assigned to "Uncategorized"
- Categories are extracted and deduplicated for easy filtering

### Stream ID Generation
Each channel receives a unique ID in the format `m3u_0`, `m3u_1`, etc.

## Testing

The module includes comprehensive tests:

```rust
#[test]
fn test_parse_basic_extinf() { ... }

#[test]
fn test_parse_with_attributes() { ... }

#[test]
fn test_extract_categories() { ... }
```

## Integration

The M3U parser is integrated into the main IPTV player application and can be used alongside the Xtream Codes API for loading playlists from multiple sources.

## Future Enhancements

Potential improvements:
- [ ] Support for M3U8 variant playlists
- [ ] EPG integration using tvg-id
- [ ] Catchup/timeshift support
- [ ] Custom attribute parsing
- [ ] M3U playlist generation/export
