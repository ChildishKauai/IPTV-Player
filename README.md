# IPTV Player - Rust Edition

A high-performance IPTV player built with Rust and egui for fast, native UI experience.

## Features (Planned)

-  Modern, native UI with egui framework
-  Xtream Codes API support
-  M3U/M3U8 playlist parsing
-  Category browsing
-  Search functionality
-  Favorites system
-  Video playback with mpv
-  Playback controls (play, pause, skip)
-  Audio/subtitle track selection
-  Credential persistence
-  Resizable window

## Technology Stack

- **UI Framework**: egui 0.29 + eframe (immediate mode GUI)
- **HTTP Client**: reqwest 0.12 (async API calls)
- **Async Runtime**: tokio 1.x
- **Serialization**: serde + serde_json
- **Image Loading**: image 0.25
- **Config Storage**: dirs 5.0
- **Video Playback**: TBD (mpv or gstreamer bindings)

## Build Instructions

### Prerequisites
- Rust 1.70+ (install from https://rustup.rs/)
- C++ compiler (MSVC on Windows, GCC/Clang on Linux)

### Building
\\\ash
# Clone or navigate to project directory
cd iptv-player-rust

# Build in debug mode
cargo build

# Build in release mode (optimized)
cargo build --release

# Run the application
cargo run

# Run in release mode
cargo run --release
\\\

## Project Structure

\\\
iptv-player-rust/
 src/
    main.rs          # Entry point with basic egui UI
 Cargo.toml           # Dependencies and project metadata
 README.md            # This file
\\\

## Development Roadmap

1.  Basic project setup with egui
2.  Compile and test basic UI (in progress)
3.  Implement Xtream Codes API client
4.  Add M3U playlist parsing
5.  Design main UI layout (sidebar, content, search)
6.  Integrate video player (mpv)
7.  Add favorites and persistence
8.  Implement playback controls
9.  Add audio/subtitle selection
10.  Testing and optimization

## Comparison with Python Version

### Advantages
- **Performance**: Native compiled code, no GIL limitations
- **Memory**: Lower memory footprint
- **Startup**: Faster application startup
- **UI**: Hardware-accelerated rendering with egui
- **Dependencies**: Single binary output, no Python runtime needed

### Current Status
- Python version: Fully functional with CustomTkinter
- Rust version: In development, basic UI scaffold complete

## License

TBD
