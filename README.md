# IPTV Player - Rust Edition

A high-performance IPTV player built with Rust and egui for fast, native UI experience. Netflix-inspired dark theme with touch-friendly controls for Steam Deck and desktop.

## Features

- ✅ Modern, native UI with egui framework
- ✅ Xtream Codes API support
- ✅ M3U/M3U8 playlist parsing
- ✅ Category browsing (Live TV, Movies, Series)
- ✅ Search functionality
- ✅ Favorites system
- ✅ Video playback with mpv/vlc/ffmpeg
- ✅ **EPG (Electronic Program Guide) support**
  - Built-in Xtream API EPG
  - External XMLTV EPG (iptv-org/epg compatible)
  - Auto-detect EPG from M3U playlists
  - Current/next program display with progress bars
- ✅ Image caching
- ✅ Credential persistence
- ✅ Responsive design (mobile/desktop)
- ✅ Steam Deck support with bundled media tools
- ✅ Football fixtures scraper integration

## Technology Stack

- **UI Framework**: egui 0.29 + eframe (immediate mode GUI)
- **HTTP Client**: reqwest 0.12 (async API calls)
- **Async Runtime**: tokio 1.x
- **Serialization**: serde + serde_json
- **Image Loading**: image 0.25
- **Database**: rusqlite (bundled SQLite)
- **Config Storage**: dirs 5.0
- **Video Playback**: mpv, vlc, ffmpeg (bundled for Steam Deck)

## Quick Start

```bash
# Clone the repository
git clone https://github.com/ChildishKauai/IPTV-Player.git
cd IPTV-Player

# Build and run
cargo run --release
```

### Steam Deck Build

```bash
# On Steam Deck, bundle media tools first
./scripts/bundle-media-tools.sh
cargo build --release --features bundle-media-tools
```

See [docs/STEAM_DECK_BUILD.md](docs/STEAM_DECK_BUILD.md) for detailed instructions.

## Project Structure

```
iptv-player-rust/
├── src/
│   ├── main.rs              # Entry point
│   ├── m3u.rs               # M3U playlist parser
│   ├── xmltv.rs             # XMLTV EPG parser
│   ├── media_tools.rs       # FFmpeg/VLC/MPV manager
│   ├── api/                 # API clients
│   │   ├── xtream.rs        # Xtream Codes API
│   │   ├── football.rs      # Football API
│   │   ├── tmdb.rs          # TMDB metadata
│   │   └── scraper_integration.rs
│   ├── models/              # Data types
│   │   ├── types.rs         # Channel, Movie, Series
│   │   ├── config.rs        # User configuration
│   │   └── watch_history.rs
│   └── ui/                  # UI components
│       ├── app.rs           # Main app state
│       ├── theme.rs         # Netflix-inspired theme
│       ├── image_cache.rs   # Async image loading
│       └── components/      # Reusable UI components
├── docs/                    # Documentation
│   ├── ARCHITECTURE.txt     # Design rationale
│   ├── STEAM_DECK_BUILD.md  # Steam Deck setup
│   ├── XTREAM_API_GUIDE.md  # API reference
│   └── ...
├── scripts/                 # Build scripts
│   ├── bundle-media-tools.sh   # Steam Deck bundler
│   └── bundle-media-tools.bat  # Windows helper
├── tests/                   # Tests and fixtures
│   ├── fixtures/            # Sample API responses
│   └── xtream_api_test.rs
├── Soccer-Scraper-main/     # Football fixture scraper
├── examples/                # Example code
├── Cargo.toml               # Dependencies
└── README.md
```

## Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.txt](docs/ARCHITECTURE.txt) | Design rationale and architecture |
| [EPG_GUIDE.md](docs/EPG_GUIDE.md) | EPG configuration and setup guide |
| [STEAM_DECK_BUILD.md](docs/STEAM_DECK_BUILD.md) | Steam Deck build instructions |
| [XTREAM_API_GUIDE.md](docs/XTREAM_API_GUIDE.md) | Xtream Codes API reference |
| [DEVELOPMENT_GUIDE.md](docs/DEVELOPMENT_GUIDE.md) | Development guidelines |
| [M3U_PARSING.md](docs/M3U_PARSING.md) | M3U parsing documentation |

## Development

```bash
# Fast error checking
cargo check

# Run tests
cargo test

# Format code
cargo fmt

# Build optimized release
cargo build --release
```

## License

MIT License - see LICENSE file for details.
