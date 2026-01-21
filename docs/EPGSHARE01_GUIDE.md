# EPGShare01 Integration Guide

EPGShare01 (https://epgshare01.online/) is now the default EPG provider for the IPTV Player. This guide explains the integration and how to use it.

## What is EPGShare01?

EPGShare01 is a free, daily-updated XMLTV EPG service that provides program guide data for TV channels worldwide. It offers:

- **60+ countries** supported
- **Daily updates** with fresh program data
- **Compressed format** (.xml.gz) for faster downloads
- **WebGrab+Plus** powered scraping

## Quick Setup

### Option 1: Using the UI (Recommended)

1. Click the **📺 button** in the top navigation bar
2. Enable **"Enable External EPG"** checkbox
3. Click one of the quick-select country buttons:
   - 🇺🇸 US
   - 🇬🇧 UK
   - 🇫🇷 France
   - 🇩🇪 Germany
   - 🇪🇸 Spain
   - 🇮🇹 Italy
   - 🇨🇦 Canada
   - 🇦🇺 Australia
4. Click **Save**
5. Wait 10-30 seconds for EPG to load

### Option 2: Manual Configuration

Edit your config file (`~/.iptv_player_config.json`):

```json
{
  "epg_url": "https://epgshare01.online/epgshare01/epg_ripper_FR1.xml.gz",
  "epg_enabled": true
}
```

## Available Countries

EPGShare01 provides EPG for the following countries:

| Country | URL |
|---------|-----|
| United States | `https://epgshare01.online/epgshare01/epg_ripper_US2.xml.gz` |
| UK | `https://epgshare01.online/epgshare01/epg_ripper_UK1.xml.gz` |
| France | `https://epgshare01.online/epgshare01/epg_ripper_FR1.xml.gz` |
| Germany | `https://epgshare01.online/epgshare01/epg_ripper_DE1.xml.gz` |
| Spain | `https://epgshare01.online/epgshare01/epg_ripper_ES1.xml.gz` |
| Italy | `https://epgshare01.online/epgshare01/epg_ripper_IT1.xml.gz` |
| Canada | `https://epgshare01.online/epgshare01/epg_ripper_CA2.xml.gz` |
| Australia | `https://epgshare01.online/epgshare01/epg_ripper_AU1.xml.gz` |
| Poland | `https://epgshare01.online/epgshare01/epg_ripper_PL1.xml.gz` |
| Romania | `https://epgshare01.online/epgshare01/epg_ripper_RO1.xml.gz` |
| Netherlands | `https://epgshare01.online/epgshare01/epg_ripper_NL1.xml.gz` |
| Belgium | `https://epgshare01.online/epgshare01/epg_ripper_BE2.xml.gz` |
| Switzerland | `https://epgshare01.online/epgshare01/epg_ripper_CH1.xml.gz` |
| Austria | `https://epgshare01.online/epgshare01/epg_ripper_AT1.xml.gz` |
| Greece | `https://epgshare01.online/epgshare01/epg_ripper_GR1.xml.gz` |
| Turkey | `https://epgshare01.online/epgshare01/epg_ripper_TR1.xml.gz` |
| Portugal | `https://epgshare01.online/epgshare01/epg_ripper_PT1.xml.gz` |
| Sweden | `https://epgshare01.online/epgshare01/epg_ripper_SE1.xml.gz` |
| Norway | `https://epgshare01.online/epgshare01/epg_ripper_NO1.xml.gz` |
| Denmark | `https://epgshare01.online/epgshare01/epg_ripper_DK1.xml.gz` |
| Finland | `https://epgshare01.online/epgshare01/epg_ripper_FI1.xml.gz` |
| Czech Republic | `https://epgshare01.online/epgshare01/epg_ripper_CZ1.xml.gz` |
| Slovakia | `https://epgshare01.online/epgshare01/epg_ripper_SK1.xml.gz` |
| Hungary | `https://epgshare01.online/epgshare01/epg_ripper_HU1.xml.gz` |
| Croatia | `https://epgshare01.online/epgshare01/epg_ripper_HR1.xml.gz` |
| Serbia | `https://epgshare01.online/epgshare01/epg_ripper_RS1.xml.gz` |
| Bulgaria | `https://epgshare01.online/epgshare01/epg_ripper_BG1.xml.gz` |
| Ireland | `https://epgshare01.online/epgshare01/epg_ripper_IE1.xml.gz` |
| India | `https://epgshare01.online/epgshare01/epg_ripper_IN1.xml.gz` |
| Japan | `https://epgshare01.online/epgshare01/epg_ripper_JP1.xml.gz` |
| South Korea | `https://epgshare01.online/epgshare01/epg_ripper_KR1.xml.gz` |
| Hong Kong | `https://epgshare01.online/epgshare01/epg_ripper_HK1.xml.gz` |
| Singapore | `https://epgshare01.online/epgshare01/epg_ripper_SG1.xml.gz` |
| Malaysia | `https://epgshare01.online/epgshare01/epg_ripper_MY1.xml.gz` |
| Thailand | `https://epgshare01.online/epgshare01/epg_ripper_TH1.xml.gz` |
| Indonesia | `https://epgshare01.online/epgshare01/epg_ripper_ID1.xml.gz` |
| Philippines | `https://epgshare01.online/epgshare01/epg_ripper_PH2.xml.gz` |
| Vietnam | `https://epgshare01.online/epgshare01/epg_ripper_VN1.xml.gz` |
| Brazil | `https://epgshare01.online/epgshare01/epg_ripper_BR1.xml.gz` |
| Mexico | `https://epgshare01.online/epgshare01/epg_ripper_MX1.xml.gz` |
| Argentina | `https://epgshare01.online/epgshare01/epg_ripper_AR1.xml.gz` (if available) |
| UAE | `https://epgshare01.online/epgshare01/epg_ripper_AE1.xml.gz` |
| South Africa | `https://epgshare01.online/epgshare01/epg_ripper_ZA1.xml.gz` |
| Israel | `https://epgshare01.online/epgshare01/epg_ripper_IL1.xml.gz` |
| New Zealand | `https://epgshare01.online/epgshare01/epg_ripper_NZ1.xml.gz` |

**Full list:** https://epgshare01.online/epgshare01/

## Specialized EPG Files

EPGShare01 also provides specialized EPG files:

- **All Sources Combined**: `epg_ripper_ALL_SOURCES1.xml.gz` (182 MB - includes all channels)
- **US Local Channels**: `epg_ripper_US_LOCALS1.xml.gz` (56 MB - local US stations)
- **US Sports**: `epg_ripper_US_SPORTS1.xml.gz` (519 KB - sports channels)
- **Plex**: `epg_ripper_PLEX1.xml.gz` (4.7 MB)
- **Rakuten**: `epg_ripper_RAKUTEN1.xml.gz` (7.2 MB)
- **BeIN Sports**: `epg_ripper_BEIN1.xml.gz` (36 KB)
- **Al Jazeera**: `epg_ripper_ALJAZEERA1.xml.gz` (5 KB)

## How It Works

1. **Channel Matching**: EPG data is matched to channels using the `tvg-id` attribute in your M3U playlist or Xtream API channel metadata
2. **Priority System**: XMLTV (EPGShare01) → Xtream API → No EPG
3. **Background Loading**: EPG data loads in a background thread to avoid blocking the UI
4. **Caching**: Loaded EPG is cached in memory for fast access

## Troubleshooting

### EPG Not Showing

**Check 1:** Verify EPG is enabled
- Open EPG settings (📺 button)
- Ensure "Enable External EPG" is checked

**Check 2:** Wait for loading
- EPG loads in the background after connecting
- Wait 10-30 seconds for first load
- Large files (like US_LOCALS) may take longer

**Check 3:** Check channel tvg-id
- Your channels must have `tvg-id` attributes that match EPGShare01's channel IDs
- Check the XMLTV file to see available channel IDs

**Check 4:** Check console output
- The app logs EPG loading progress to the console:
  ```
  [EPG] Loading XMLTV from: https://epgshare01.online/...
  [EPG] Loaded XMLTV data for X channels
  ```

### Wrong Programs Showing

Your channel's `tvg-id` doesn't match the EPGShare01 channel ID. You'll need to:
1. Download the EPG file manually
2. Find the correct channel ID in the XMLTV
3. Update your M3U playlist with the correct `tvg-id`

### Slow Loading

EPG files are compressed (.xml.gz) for faster downloads. If loading is still slow:
- Check your internet connection
- Use a smaller country-specific file instead of ALL_SOURCES
- Consider caching the EPG file locally

## M3U Auto-Detection

If your M3U playlist includes an EPG URL in the header, it will be automatically detected:

```m3u
#EXTM3U x-tvg-url="https://epgshare01.online/epgshare01/epg_ripper_FR1.xml.gz"
#EXTINF:-1 tvg-id="TF1.fr" tvg-logo="...",TF1
http://example.com/tf1
```

The app will:
1. Detect the `x-tvg-url` attribute
2. Automatically load EPG from that URL
3. Override any manually configured EPG URL

## Technical Details

### File Format
- **Format**: XMLTV (XML TV listings format)
- **Compression**: Gzip (.xml.gz)
- **Encoding**: UTF-8
- **Update Frequency**: Daily

### Implementation
- **Parser**: Custom XMLTV parser in [src/xmltv.rs](../src/xmltv.rs)
- **Cache**: Thread-safe Arc<Mutex<HashMap>> in [src/ui/epg_cache.rs](../src/ui/epg_cache.rs)
- **UI**: EPG settings dialog in [src/ui/components/epg_settings.rs](../src/ui/components/epg_settings.rs)

### Dependencies
```toml
quick-xml = "0.31"  # XML parsing
flate2 = "1.0"      # Gzip decompression
```

## Advantages of EPGShare01

1. **Daily Updates**: Fresh program data every day
2. **Compressed Format**: Faster downloads with .xml.gz
3. **Wide Coverage**: 60+ countries supported
4. **Free**: No registration or API keys required
5. **Reliable**: Maintained infrastructure
6. **WebGrab+Plus**: Professional scraping tool

## Alternative EPG Sources

If EPGShare01 doesn't work for you, alternatives include:

1. **iptv-org/epg** (GitHub community project)
2. **Your IPTV provider's EPG** (check provider docs)
3. **Self-hosted WebGrab+Plus** (generate your own)

## Need Help?

See also:
- [Quick Start EPG Guide](../QUICK_START_EPG.md)
- [Full EPG Guide](EPG_GUIDE.md)
- [EPGShare01 Website](https://epgshare01.online/)

---

**EPGShare01 Last Updated**: 2026-01-20
**Guide Version**: 1.0
