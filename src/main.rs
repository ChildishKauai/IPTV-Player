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
//! # Usage
//!
//! ```bash
//! cargo run --release
//! ```

use eframe::egui;

mod api;
mod models;
mod m3u;
mod ui;

use ui::IPTVPlayerApp;

/// Application entry point.
///
/// Initializes the eframe window and starts the IPTV Player application.
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 800.0])
            .with_min_inner_size([1000.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "IPTV Player - Rust Edition",
        options,
        Box::new(|cc| Ok(Box::new(IPTVPlayerApp::new(cc)))),
    )
}
