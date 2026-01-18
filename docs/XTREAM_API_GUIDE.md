# Xtream Codes API Integration Guide

## Overview

The IPTV Player now includes improved Xtream Codes API support with better error handling, timeout management, and field validation.

## Recent Improvements

### ✅ Fixed Issues

1. **Better Error Messages**
   - Detailed error messages for authentication failures
   - Specific errors for categories and channels fetch failures
   - Network connection error reporting

2. **Robust Deserialization**
   - Added `#[serde(default)]` attributes to handle missing fields
   - Support for field aliases (e.g., `stream_id`)
   - Automatic `num` field population when missing

3. **Timeout Configuration**
   - Connection timeout: 10 seconds
   - Request timeout: 30 seconds
   - Redirect policy: Limited to 5 redirects

4. **URL Handling**
   - Automatically removes trailing slashes from server URLs
   - Proper URL encoding for parameters

5. **Authentication Validation**
   - Checks for valid JSON response
   - Verifies `user_info` field presence
   - Better error reporting for invalid credentials

## How to Use

### Server URL Format

The server URL should be in one of these formats:

```
http://example.com:8080
https://example.com
http://192.168.1.100:8000
```

**Do NOT include** `/player_api.php` - this is added automatically.

### Connection Process

1. Enter your server URL (without trailing slash)
2. Enter your username
3. Enter your password
4. Click "Connect"

The app will:
- Validate all fields are filled
- Remove any trailing slashes from URL
- Attempt authentication
- Fetch categories and channels
- Display detailed errors if anything fails

## API Endpoints Used

| Endpoint | Purpose | Response Type |
|----------|---------|---------------|
| `/player_api.php?username=X&password=Y` | Authentication | User info JSON |
| `action=get_live_categories` | Get channel categories | Array of categories |
| `action=get_live_streams` | Get all channels | Array of channels |
| `action=get_series_categories` | Get series categories | Array of categories |
| `action=get_series` | Get all series | Array of series |
| `action=get_series_info&series_id=X` | Get series details | Series info JSON |

## Data Structures

### Channel
```rust
{
    "num": "1",                    // Auto-generated if missing
    "name": "Channel Name",
    "stream_type": "live",
    "stream_id": "12345",
    "stream_icon": "logo.png",     // Optional, defaults to empty
    "epg_channel_id": "channel.id", // Optional
    "category_id": "1",            // Defaults to empty if missing
    "tv_archive": 1,               // Optional
    "direct_source": "http://...", // Optional
}
```

### Category
```rust
{
    "category_id": "1",
    "category_name": "News",
    "parent_id": 0                 // Defaults to 0 if missing
}
```

### Series
```rust
{
    "num": 1,                      // Auto-generated if missing
    "name": "Series Name",
    "series_id": 12345,
    "cover": "cover.jpg",          // Defaults to empty
    "genre": "Drama",              // Optional
    "category_id": "1",            // Defaults to empty
    // ... other optional fields
}
```

## Troubleshooting

### "Connection error" Messages

**Possible causes:**
- Incorrect server URL format
- Server is offline or unreachable
- Firewall blocking the connection
- SSL/TLS certificate issues (for HTTPS)

**Solutions:**
1. Verify the server URL is correct
2. Try accessing the URL in a web browser
3. Check your internet connection
4. Try HTTP instead of HTTPS if SSL issues occur

### "Authentication failed" Messages

**Possible causes:**
- Wrong username or password
- Account expired or suspended
- Server not returning valid user_info

**Solutions:**
1. Double-check your credentials
2. Verify your account is active
3. Contact your IPTV provider

### "Failed to fetch categories/channels" Messages

**Possible causes:**
- API endpoint not responding
- Malformed JSON response from server
- Server doesn't support the endpoint

**Solutions:**
1. Check if the server supports Xtream Codes API
2. Look at the detailed error message for parsing errors
3. Test the API directly with curl or a browser

### Empty Channel List

**Possible causes:**
- No channels assigned to your account
- Channels in a category you haven't selected
- Server returned empty array

**Solutions:**
1. Select "All" in the categories panel
2. Use the search function
3. Contact your provider to verify channel assignment

## Testing the API Manually

You can test your Xtream API with curl:

### Test Authentication
```bash
curl "http://your-server:port/player_api.php?username=YOUR_USER&password=YOUR_PASS"
```

Should return JSON with `user_info`, `server_info`, etc.

### Test Get Categories
```bash
curl "http://your-server:port/player_api.php?username=YOUR_USER&password=YOUR_PASS&action=get_live_categories"
```

Should return array of categories.

### Test Get Channels
```bash
curl "http://your-server:port/player_api.php?username=YOUR_USER&password=YOUR_PASS&action=get_live_streams"
```

Should return array of channels.

## Stream URL Format

Live streams use this format:
```
http://server:port/username/password/stream_id.m3u8
```

VOD/Series streams use:
```
http://server:port/series/username/password/episode_id.extension
```

## Common Server Response Issues

### Issue: Timeout Errors
- **Fix**: Server taking too long to respond (>30s)
- **Solution**: Check server load, contact provider

### Issue: Redirect Loop
- **Fix**: Server redirecting more than 5 times
- **Solution**: Check server configuration

### Issue: Invalid JSON
- **Fix**: Server returning HTML or malformed JSON
- **Solution**: Verify API endpoint URL, check server logs

## M3U Alternative

If Xtream API isn't working, you can use M3U playlists instead:

1. Get your M3U URL from your provider
2. Use the M3U parser: `M3UParser::parse_url("http://...")`
3. This bypasses the Xtream API entirely

## Support

For issues with:
- **This application**: Check error messages, review this guide
- **Your IPTV service**: Contact your provider
- **API specification**: Refer to Xtream Codes documentation

## Development Notes

### Adding New Endpoints

To add a new Xtream API endpoint:

1. Add method to `XtreamClient` in `src/api/xtream.rs`
2. Use the pattern:
```rust
pub fn get_something(&self) -> Result<Vec<Type>, Box<dyn std::error::Error>> {
    let url = self.api_url("action_name");
    let response = self.client.get(&url).send()?;
    
    if !response.status().is_success() {
        return Err(format!("API returned status: {}", response.status()).into());
    }
    
    let items: Vec<Type> = response.json()?;
    Ok(items)
}
```

### Field Mapping

If the API returns different field names than expected:
- Add `#[serde(alias = "api_field_name")]` to struct field
- Add `#[serde(default)]` for optional fields
- Use `Option<T>` for nullable fields

### Debugging Tips

1. Enable verbose logging by checking response text before parsing
2. Use `eprintln!` to log URLs and responses during development
3. Test with real server responses in unit tests
4. Check network traffic with tools like Wireshark or browser DevTools
