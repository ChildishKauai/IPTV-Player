# EPG (Electronic Program Guide) Integration Guide

This IPTV player supports multiple EPG sources to provide TV program schedules for your channels.

## EPG Sources

### 1. Built-in Xtream API EPG (Default)
- Automatically loads EPG for Xtream Codes channels
- No configuration needed
- Shows current and next programs

### 2. External XMLTV EPG
- Support for external XMLTV format EPG sources
- Can supplement or override Xtream API EPG
- Works with both M3U and Xtream channels
- Supports iptv-org/epg and other XMLTV providers

## How to Configure External EPG

### Using Configuration File

Edit your configuration file at: `~/.iptv_player_config.json`

Add these fields:

```json
{
  "server_url": "http://your-server:port",
  "username": "your-username",
  "password": "your-password",
  "epg_url": "https://example.com/epg.xml",
  "epg_enabled": true,
  "favorites": [],
  "auto_login": false
}
```

### EPG URL Examples

#### iptv-org/epg (Free Community EPG)

Choose a guide from: https://github.com/iptv-org/epg

```json
"epg_url": "https://iptv-org.github.io/epg/guides/en/tvguide.com.epg.xml"
```

Available guides:
- UK: `https://iptv-org.github.io/epg/guides/gb/bbc.co.uk.epg.xml`
- US: `https://iptv-org.github.io/epg/guides/us/tvguide.com.epg.xml`
- France: `https://iptv-org.github.io/epg/guides/fr/programme-tv.net.epg.xml`
- Germany: `https://iptv-org.github.io/epg/guides/de/fernsehserien.de.epg.xml`

Browse all available guides at: https://github.com/iptv-org/epg

#### Self-Hosted EPG

If you host your own XMLTV file:

```json
"epg_url": "http://192.168.1.100:8080/epg.xml.gz"
```

Note: Both `.xml` and `.xml.gz` (gzipped) formats are supported.

### M3U Playlist EPG

If your M3U playlist includes an EPG URL in the header, it will be automatically detected:

```m3u
#EXTM3U x-tvg-url="http://example.com/epg.xml"
#EXTINF:-1 tvg-id="bbc1.uk" tvg-logo="logo.png",BBC One
http://stream-url
```

The player will automatically use the EPG URL from the playlist header.

## How EPG Matching Works

### Channel Matching

EPG programs are matched to channels using the `tvg-id` attribute:

```m3u
#EXTINF:-1 tvg-id="bbc1.uk" tvg-logo="logo.png" group-title="UK",BBC One
```

The `tvg-id` must match the channel ID in the XMLTV file:

```xml
<programme start="20260120120000 +0000" stop="20260120130000 +0000" channel="bbc1.uk">
  <title lang="en">BBC News</title>
  <desc lang="en">Latest news coverage</desc>
</programme>
```

### Priority Order

1. **External XMLTV EPG** (if enabled and channel has `tvg-id`)
2. **Xtream API EPG** (fallback for Xtream channels)
3. **No EPG** (shows "LIVE" badge)

## Supported XMLTV Features

- Program titles
- Program descriptions
- Start/end times with timezone support
- Multiple language support
- Gzip compression (`.xml.gz` files)

## EPG Display Features

- Current program with progress bar
- Next program time and title
- Color-coded progress indicator
- Automatic refresh every 5 minutes

## Troubleshooting

### EPG not showing

1. Check that `epg_enabled` is set to `true`
2. Verify the EPG URL is accessible
3. Ensure channels have correct `tvg-id` matching XMLTV channel IDs
4. Check console logs for EPG loading errors

### EPG loading slowly

- Large XMLTV files (>10MB) may take time to download and parse
- Use `.xml.gz` compressed format for faster loading
- EPG is loaded in background, wait ~10-30 seconds after connecting

### Mismatched programs

- Verify `tvg-id` in channel matches `channel` attribute in XMLTV
- Check timezone in XMLTV timestamps match your local timezone
- Ensure XMLTV uses correct date format: `YYYYMMDDHHmmss +0000`

## Example Full Configuration

```json
{
  "server_url": "http://iptv.example.com:8080",
  "username": "user123",
  "password": "pass456",
  "epg_url": "https://iptv-org.github.io/epg/guides/us/tvguide.com.epg.xml",
  "epg_enabled": true,
  "favorites": ["123", "456"],
  "auto_login": true,
  "player_settings": {
    "player_type": "MPV",
    "volume": 100,
    "hardware_acceleration": true,
    "subtitles_enabled": true
  }
}
```

## Resources

- **iptv-org/epg**: https://github.com/iptv-org/epg
- **XMLTV Format**: http://wiki.xmltv.org/index.php/XMLTVFormat
- **Free EPG Sources**: https://github.com/globetvapp/epg

## Future Enhancements

Planned features:
- UI settings panel for EPG configuration (no need to edit JSON)
- Multiple EPG source support with fallback
- EPG grid view showing schedule for multiple channels
- Program details modal with full description and cast
- Set reminders for upcoming programs
- EPG catchup/archive playback integration
