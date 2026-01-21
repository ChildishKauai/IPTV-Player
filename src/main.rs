//! IPTV Player - A modern Rust-based IPTV client.
//!
//! This application provides a sleek, Apple-inspired interface for viewing
//! IPTV content via the Xtream Codes API.
//!
//! # Architecture
//!
//! The application is organized into the following modules:
//!
//! - `api` - Xtream Codes API client
//! - `models` - Data models and configuration
//! - `m3u` - M3U playlist parsing
//! - `ui` - User interface components
//!
//! # Steam Deck Support
//!
//! The application is optimized for Steam Deck Game Mode with:
//! - Automatic fullscreen detection based on screen resolution
//! - Touch-friendly UI sizing (52px minimum touch targets)
//! - Gamepad navigation support via egui's built-in focus system
//!
//! # Usage
//!
//! ```bash
//! cargo run --release
//! ```

use eframe::egui;

mod api;
mod media_tools;
mod models;
mod m3u;
mod xmltv;
mod ui;

use ui::IPTVPlayerApp;

/// Detects if running on Steam Deck based on environment or display characteristics
fn is_steam_deck_environment() -> bool {
    // Check for Steam Deck specific environment variable
    if std::env::var("SteamDeck").is_ok() || std::env::var("STEAM_DECK").is_ok() {
        return true;
    }

    // Check for gamescope (Steam's compositor used in Game Mode)
    if std::env::var("GAMESCOPE_WAYLAND_DISPLAY").is_ok() {
        return true;
    }

    // Check for Steam runtime
    if let Ok(runtime) = std::env::var("SteamAppId") {
        if !runtime.is_empty() {
            return true;
        }
    }

    // Check /etc/os-release for SteamOS
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        if content.contains("SteamOS") || content.contains("steamos") {
            return true;
        }
    }

    false
}

/// Application entry point.
///
/// Initializes the eframe window and starts the IPTV Player application.
/// Automatically configures for Steam Deck Game Mode when detected.
fn main() -> Result<(), eframe::Error> {
    let is_steam_deck = is_steam_deck_environment();

    // Configure viewport based on environment
    let viewport = if is_steam_deck {
        // Steam Deck Game Mode: fullscreen at native resolution
        egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1280.0, 800.0])
            .with_fullscreen(true)
            .with_decorations(false)
            .with_resizable(false)
    } else {
        // Desktop mode: windowed with standard sizing
        egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 800.0])
            .with_min_inner_size([1000.0, 600.0])
            .with_resizable(true)
    };

    let options = eframe::NativeOptions {
        viewport,
        // Enable hardware acceleration for better performance on Steam Deck
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        // Vsync for smoother rendering on Steam Deck's 60Hz display
        vsync: true,
        // Multisampling for better rendering quality
        multisampling: 4,
        ..Default::default()
    };

    eframe::run_native(
        "IPTV Player",
        options,
        Box::new(|cc| Ok(Box::new(IPTVPlayerApp::new(cc)))),
    )
}
