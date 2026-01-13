//! UI Module - User interface components and application state.
//!
//! This module contains all UI-related code for the IPTV Player:
//!
//! - `app` - Main application state and logic
//! - `theme` - Theme and styling configuration  
//! - `messages` - Application message types
//! - `image_cache` - Async image loading and caching
//! - `epg_cache` - EPG data caching and background loading
//! - `components` - Reusable UI components

pub mod app;
pub mod theme;
pub mod messages;
pub mod image_cache;
pub mod epg_cache;
pub mod components;

pub use app::IPTVPlayerApp;
