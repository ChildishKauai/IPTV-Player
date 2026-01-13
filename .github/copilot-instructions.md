# IPTV Player - Copilot Instructions

## Project Overview
This is a modern IPTV Player built in Rust using the egui/eframe GUI framework.
It connects to IPTV services via the Xtream Codes API.

## Project Type
- **Language:** Rust
- **Framework:** egui/eframe for GUI
- **Build System:** Cargo

## Architecture
The project follows a modular, layered architecture:

```
src/
├── main.rs              # Entry point (minimal)
├── m3u.rs               # M3U playlist parser
├── api/                 # API client layer
├── models/              # Data models
└── ui/                  # UI layer
    ├── app.rs           # Main application state
    ├── theme.rs         # Theme configuration
    ├── messages.rs      # Message types
    ├── image_cache.rs   # Async image loading
    └── components/      # Reusable UI components
```

## Key Design Patterns
1. **Component-Based UI:** UI elements are stateless functions that return actions
2. **Message Passing:** Background threads communicate via mpsc channels
3. **Centralized State:** All app state in IPTVPlayerApp struct
4. **Theme System:** Centralized colors/dimensions for consistency

## Building
```bash
cargo build --release
```

## Running
```bash
cargo run --release
```

## Documentation
See ARCHITECTURE.txt for detailed design documentation.

- [ ] Ensure Documentation is Complete
	<!--
	Verify that all previous steps have been completed.
	Verify that README.md and the copilot-instructions.md file in the .github directory exists and contains current project information.
	Clean up the copilot-instructions.md file in the .github directory by removing all HTML comments.
	 -->
