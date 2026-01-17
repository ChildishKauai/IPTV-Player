# IPTV Player - Copilot Instructions

## Project Overview
Rust IPTV player using **egui/eframe** (immediate mode GUI) connecting to IPTV services via **Xtream Codes API**. Netflix-inspired dark theme UI. Integrated with **Soccer-Scraper** for football fixture scraping.

### Target Platform
- **Cross-platform**: Linux (Wayland/X11) and Windows—avoid OS-specific code, use `std::path` and `dirs` crate for paths
- **Input**: Touch screen devices—ensure all UI elements have adequate touch targets (min 44px)
- **Design**: Mobile-first responsive layout using `dimensions::is_mobile(screen_width)`

## Quick Commands
```bash
cargo run --release     # Build and run optimized
cargo check             # Fast error checking without build
cargo fmt               # Format code
cargo test              # Run tests
```

## Architecture

### Layer Structure
```
src/
├── main.rs           # Minimal entry point
├── api/              # Xtream API client + Football Scraper integration
├── models/           # Data types with custom serde deserializers
└── ui/
    ├── app.rs        # IPTVPlayerApp - ALL state lives here (1700+ lines)
    ├── theme.rs      # Theme + dimensions:: module for responsive sizing
    ├── messages.rs   # AppMessage enum for thread communication
    ├── image_cache.rs # Thread-safe async image loading
    └── components/   # Stateless UI components
```

### Component Pattern (Critical)
Components are **stateless structs with static `show()` methods** that return `Option<Action>`:
```rust
// Pattern: Props down, actions up
impl ChannelCard {
    pub fn show(ui, ctx, theme, channel, is_favorite, image_cache, screen_width, epg_info)
        -> Option<ChannelAction>
}

pub enum ChannelAction {
    Play(Channel),
    ToggleFavorite(String),
}
```
Always define an `Action` enum when a component can trigger user interactions.

### Threading Model
- **Main thread**: egui event loop, UI rendering, state updates
- **Background threads**: API calls, image loading, Python scraper via `std::thread::spawn`
- **Communication**: `mpsc::channel` with `AppMessage` enum
```rust
// Spawn work
thread::spawn(move || {
    let result = api_call();
    tx.send(AppMessage::Connected(data)).unwrap();
});
// Poll in update()
if let Ok(msg) = self.rx.try_recv() { /* handle */ }
```

## Key Conventions

### Theme Usage
Always use `Theme` and `dimensions::` for styling—never hardcode colors/sizes:
```rust
let card_width = dimensions::card_width(screen_width);
let is_mobile = dimensions::is_mobile(screen_width);
ui.painter().rect_filled(rect, 8.0, theme.card_bg);
```

### Model Deserializers
Xtream API returns inconsistent JSON (strings vs numbers). Use custom deserializers in [src/models/types.rs](src/models/types.rs):
```rust
#[serde(deserialize_with = "deserialize_string_or_int")]
pub category_id: String,
```

### Image Loading
Use `ImageCache` for all remote images—it handles background loading:
```rust
image_cache.load(ctx, url.clone());  // Non-blocking
if let Some(texture) = image_cache.get(&url) { /* render */ }
```

### Config Persistence
User config saves to `~/.iptv_player_config.json`. Managed in [src/models/config.rs](src/models/config.rs).

### Football Fixtures (Soccer Scraper Integration)
The **ScraperManager** in [src/api/scraper_integration.rs](src/api/scraper_integration.rs) invokes the Python scraper:
```rust
let mut scraper = ScraperManager::new();
// Spawn background thread to scrape
thread::spawn(move || {
    let status = scraper.scrape_blocking(); // Returns Success/Error/Scraping
});
```
UI trigger: Settings button (📊) opens **ScraperSettingsDialog** in [src/ui/components/scraper_settings.rs](src/ui/components/scraper_settings.rs).
Database: `Soccer-Scraper-main/output/fixtures.db`

## Adding New Features

### New Component Checklist
1. Create `src/ui/components/your_component.rs`
2. Define `YourAction` enum with all possible user actions
3. Implement `YourComponent::show(...) -> Option<YourAction>`
4. Export in [src/ui/components/mod.rs](src/ui/components/mod.rs)
5. Handle actions in `app.rs` render methods

### New API Endpoint
1. Add method to `XtreamClient` in [src/api/xtream.rs](src/api/xtream.rs)
2. Add `AppMessage` variant in [src/ui/messages.rs](src/ui/messages.rs)
3. Spawn background thread in `app.rs`, send result via channel
4. Handle message in `process_messages()` method

## Scraper Integration Workflow

### First-time Setup
1. Install Python dependencies in Soccer-Scraper-main:
   ```bash
   python -m venv .venv
   source .venv/bin/activate  # or .venv\Scripts\activate on Windows
   pip install -r requirements.txt
   ```

2. Click the **📊 button** in the app's top navigation (desktop only) to open Football Fixtures Scraper

3. Click **"Scrape Fixtures Now"** to download today's matches from LiveSoccerTV (~2 minutes)

### Database Schema
The scraper populates `fixtures.db` with:
- **fixtures** table: home_team, away_team, competition, fixture_date, fixture_time, venue
- **broadcasters** table: country, channel (broadcasting which match)
- Data persists; re-scrape anytime to update

### Custom Scraper Queries (Future)
The `ScraperManager` supports:
- `query_today()` - today's fixtures
- `query_country(country)` - fixtures by country broadcast
- `query_competition(name)` - fixtures by league

## Documentation
See [ARCHITECTURE.txt](ARCHITECTURE.txt) for detailed design rationale.
