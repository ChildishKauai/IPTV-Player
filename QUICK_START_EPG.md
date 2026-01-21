# Quick Start: EPG Setup

This guide will help you set up Electronic Program Guide (EPG) for your IPTV channels in under 5 minutes.

## What is EPG?

EPG shows you what's currently playing and what's coming next on your TV channels, just like a TV guide.

## Option 1: Auto-Detect from M3U Playlist (Easiest)

If your M3U playlist has an EPG URL in the header, it will be automatically detected. No setup needed!

Example M3U header:
```m3u
#EXTM3U x-tvg-url="http://example.com/epg.xml"
```

## Option 2: Manual Configuration

### Step 1: Find Your Config File

The config file is located at:
- **Windows**: `C:\Users\YourName\.iptv_player_config.json`
- **Linux**: `~/.iptv_player_config.json`
- **macOS**: `~/.iptv_player_config.json`

### Step 2: Add EPG Settings

Open the config file and add these two lines:

```json
{
  "server_url": "...",
  "username": "...",
  "password": "...",
  "epg_url": "https://epgshare01.online/epgshare01/epg_ripper_US2.xml.gz",
  "epg_enabled": true
}
```

### Step 3: Choose Your EPG Source

Replace the `epg_url` with one of these free sources from EPGShare01:

#### United States
```json
"epg_url": "https://epgshare01.online/epgshare01/epg_ripper_US2.xml.gz"
```

#### United Kingdom
```json
"epg_url": "https://epgshare01.online/epgshare01/epg_ripper_UK1.xml.gz"
```

#### France
```json
"epg_url": "https://epgshare01.online/epgshare01/epg_ripper_FR1.xml.gz"
```

#### Germany
```json
"epg_url": "https://epgshare01.online/epgshare01/epg_ripper_DE1.xml.gz"
```

#### Spain
```json
"epg_url": "https://epgshare01.online/epgshare01/epg_ripper_ES1.xml.gz"
```

#### Italy
```json
"epg_url": "https://epgshare01.online/epgshare01/epg_ripper_IT1.xml.gz"
```

#### Canada
```json
"epg_url": "https://epgshare01.online/epgshare01/epg_ripper_CA2.xml.gz"
```

#### Australia
```json
"epg_url": "https://epgshare01.online/epgshare01/epg_ripper_AU1.xml.gz"
```

**Browse 60+ countries:** https://epgshare01.online/epgshare01/

### Step 4: Restart the App

Close and reopen the IPTV player. EPG data will load automatically in the background.

## How to Tell if EPG is Working

1. Connect to your IPTV service
2. Browse to Live TV channels
3. Look for:
   - Current program name under channel logo
   - Progress bar showing how far into the program
   - Next program time and title

If you see "LIVE" badge instead, the EPG may not have data for that channel.

## Troubleshooting

### EPG not showing

**Check 1:** Verify `epg_enabled` is set to `true`

**Check 2:** Make sure channels have `tvg-id` matching your EPG source

Example M3U channel:
```m3u
#EXTINF:-1 tvg-id="BBCOne.uk" tvg-logo="logo.png",BBC One
```

The `tvg-id` must match what's in the XMLTV file.

**Check 3:** Wait 30 seconds after connecting for EPG to load

### Wrong programs showing

Your channel `tvg-id` doesn't match the EPG. Check the XMLTV file to find the correct channel ID.

### EPG loading slowly

Large EPG files take time to download. Use `.xml.gz` compressed format when available:
```json
"epg_url": "https://example.com/epg.xml.gz"
```

## Complete Example Config

```json
{
  "server_url": "http://my-iptv-server.com:8080",
  "username": "myuser",
  "password": "mypass",
  "epg_url": "https://epgshare01.online/epgshare01/epg_ripper_US2.xml.gz",
  "epg_enabled": true,
  "favorites": [],
  "auto_login": true,
  "player_settings": {
    "player_type": "MPV",
    "volume": 100,
    "hardware_acceleration": true
  }
}
```

## Where to Find EPG URLs

1. **EPGShare01** (Free, Regularly Updated)
   - Website: https://epgshare01.online/
   - Coverage: 60+ countries, thousands of channels
   - Format: XMLTV (`.xml.gz` compressed)
   - Updated: Daily

2. **iptv-org/epg** (Free, Community-maintained)
   - Website: https://github.com/iptv-org/epg
   - Coverage: 100+ countries, 1000+ channels
   - Format: XMLTV (`.xml` and `.xml.gz`)

3. **Your IPTV Provider**
   - Many providers include EPG URLs in their service
   - Check your provider's documentation

4. **Self-Hosted**
   - Generate your own using tools like [WebGrab+Plus](http://www.webgrabplus.com/)
   - Host on your local network

## Advanced: Multiple EPG Sources

The app currently supports one EPG source at a time. To use multiple sources:

1. Merge XMLTV files manually
2. Use your primary source and supplement missing channels through M3U `tvg-id` attributes

## Need More Help?

See the full guide: [docs/EPG_GUIDE.md](docs/EPG_GUIDE.md)

---

**Happy viewing!** 📺
