//! Theme and styling configuration for the IPTV Player UI.
//!
//! This module provides a centralized theme system with support for
//! dark and light modes, following Apple's Human Interface Guidelines.

use eframe::egui;

/// Theme colors and styling configuration.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Whether dark mode is enabled
    pub dark_mode: bool,
    
    /// Main background color
    pub bg_color: egui::Color32,
    
    /// Panel/sidebar background color
    pub panel_bg: egui::Color32,
    
    /// Card background color
    pub card_bg: egui::Color32,
    
    /// Primary text color
    pub text_primary: egui::Color32,
    
    /// Secondary/muted text color
    pub text_secondary: egui::Color32,
    
    /// Border color for cards and separators
    pub border_color: egui::Color32,
    
    /// Accent color (blue) for buttons and highlights
    pub accent_blue: egui::Color32,
    
    /// Error/danger color (red)
    #[allow(dead_code)]
    pub error_color: egui::Color32,
    
    /// Warning/favorite color (gold)
    pub warning_color: egui::Color32,
    
    /// Success color (green)
    #[allow(dead_code)]
    pub success_color: egui::Color32,
}

impl Theme {
    /// Creates a new theme with the specified mode
    pub fn new(dark_mode: bool) -> Self {
        if dark_mode {
            Self::dark()
        } else {
            Self::light()
        }
    }
    
    /// Creates a dark theme (Netflix-inspired)
    pub fn dark() -> Self {
        Self {
            dark_mode: true,
            bg_color: egui::Color32::from_rgb(20, 20, 20),           // Netflix dark bg
            panel_bg: egui::Color32::from_rgb(26, 26, 26),           // Slightly lighter panel
            card_bg: egui::Color32::from_rgb(35, 35, 35),            // Card background
            text_primary: egui::Color32::from_rgb(255, 255, 255),    // Pure white
            text_secondary: egui::Color32::from_rgb(180, 180, 180),  // Brighter secondary
            border_color: egui::Color32::from_rgb(50, 50, 50),       // Subtle borders
            accent_blue: egui::Color32::from_rgb(229, 9, 20),        // Netflix red (#E50914)
            error_color: egui::Color32::from_rgb(229, 9, 20),        // Also Netflix red
            warning_color: egui::Color32::from_rgb(255, 180, 0),     // Gold/yellow
            success_color: egui::Color32::from_rgb(70, 211, 105),    // Green
        }
    }
    
    /// Creates a light theme (still uses Netflix red accent)
    pub fn light() -> Self {
        Self {
            dark_mode: false,
            bg_color: egui::Color32::from_rgb(245, 245, 245),
            panel_bg: egui::Color32::from_rgb(255, 255, 255),
            card_bg: egui::Color32::from_rgb(255, 255, 255),
            text_primary: egui::Color32::from_rgb(20, 20, 20),
            text_secondary: egui::Color32::from_rgb(100, 100, 100),
            border_color: egui::Color32::from_rgb(230, 230, 230),
            accent_blue: egui::Color32::from_rgb(185, 9, 11),        // Darker Netflix red
            error_color: egui::Color32::from_rgb(185, 9, 11),
            warning_color: egui::Color32::from_rgb(255, 180, 0),
            success_color: egui::Color32::from_rgb(40, 167, 69),
        }
    }
    
    /// Returns the hover background color for interactive elements
    pub fn hover_bg(&self) -> egui::Color32 {
        if self.dark_mode {
            egui::Color32::from_rgb(55, 55, 55)
        } else {
            egui::Color32::from_rgb(235, 235, 235)
        }
    }
    
    /// Returns the inactive/button background color
    pub fn inactive_bg(&self) -> egui::Color32 {
        if self.dark_mode {
            egui::Color32::from_rgb(45, 45, 45)
        } else {
            egui::Color32::from_rgb(240, 240, 240)
        }
    }
    
    /// Returns the placeholder background color for loading states
    pub fn placeholder_bg(&self) -> egui::Color32 {
        if self.dark_mode {
            egui::Color32::from_rgb(45, 45, 45)
        } else {
            egui::Color32::from_rgb(240, 240, 245)
        }
    }
    
    /// Returns the placeholder icon color
    pub fn placeholder_icon(&self) -> egui::Color32 {
        if self.dark_mode {
            egui::Color32::from_rgb(100, 100, 100)
        } else {
            egui::Color32::from_rgb(180, 180, 190)
        }
    }
    
    /// Returns a shadow color for cards
    pub fn card_shadow(&self) -> egui::Color32 {
        if self.dark_mode {
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120)
        } else {
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 30)
        }
    }
    
    /// Returns the hover overlay color for cards
    #[allow(dead_code)]
    pub fn card_hover_overlay(&self) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15)
    }
    
    /// Applies the theme to the egui context
    pub fn apply(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        // Netflix-style bold font sizing
        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::proportional(32.0)),
            (egui::TextStyle::Body, egui::FontId::proportional(14.0)),
            (egui::TextStyle::Button, egui::FontId::proportional(13.0)),
            (egui::TextStyle::Small, egui::FontId::proportional(11.0)),
            (egui::TextStyle::Monospace, egui::FontId::monospace(13.0)),
        ].into();
        
        // Tighter, more cinematic spacing
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(20.0, 10.0);
        style.spacing.indent = 16.0;
        style.spacing.window_margin = egui::Margin::same(12.0);
        
        // Slightly rounded corners (more subtle than before)
        style.visuals.window_rounding = egui::Rounding::same(8.0);
        style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(4.0);
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(4.0);
        style.visuals.widgets.active.rounding = egui::Rounding::same(4.0);
        
        // Widget colors
        style.visuals.widgets.noninteractive.bg_fill = self.panel_bg;
        style.visuals.widgets.inactive.bg_fill = self.inactive_bg();
        style.visuals.widgets.hovered.bg_fill = self.hover_bg();
        style.visuals.widgets.active.bg_fill = self.accent_blue;
        
        style.visuals.selection.bg_fill = self.accent_blue;
        style.visuals.selection.stroke = egui::Stroke::new(1.0, self.accent_blue);
        
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, self.text_primary);
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        
        ctx.set_style(style);
        
        // Set visuals based on theme
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals {
                dark_mode: true,
                panel_fill: self.bg_color,
                ..egui::Visuals::dark()
            });
        } else {
            ctx.set_visuals(egui::Visuals {
                dark_mode: false,
                panel_fill: self.bg_color,
                ..egui::Visuals::light()
            });
        }
    }
}

/// UI dimensions and sizing constants
#[allow(dead_code)]
pub mod dimensions {
    /// Breakpoints for responsive design
    pub const MOBILE_BREAKPOINT: f32 = 600.0;
    pub const TABLET_BREAKPOINT: f32 = 900.0;
    /// Steam Deck screen width (1280x800 in portable mode)
    pub const STEAM_DECK_WIDTH: f32 = 1280.0;
    pub const STEAM_DECK_HEIGHT: f32 = 800.0;
    
    /// Card dimensions (desktop) - Netflix poster style
    pub const CHANNEL_CARD_WIDTH: f32 = 200.0;
    pub const CHANNEL_CARD_HEIGHT: f32 = 130.0;
    pub const SERIES_CARD_WIDTH: f32 = 180.0;
    pub const SERIES_CARD_HEIGHT: f32 = 270.0;
    pub const MOVIE_CARD_WIDTH: f32 = 180.0;
    pub const MOVIE_CARD_HEIGHT: f32 = 270.0;
    
    /// Steam Deck optimized card dimensions (larger for touch)
    pub const STEAM_DECK_CARD_WIDTH: f32 = 220.0;
    pub const STEAM_DECK_CARD_HEIGHT: f32 = 150.0;
    pub const STEAM_DECK_SERIES_CARD_WIDTH: f32 = 200.0;
    pub const STEAM_DECK_SERIES_CARD_HEIGHT: f32 = 300.0;
    
    /// Image dimensions - taller poster ratio like Netflix
    pub const CHANNEL_ICON_SIZE: f32 = 80.0;
    pub const POSTER_WIDTH: f32 = 180.0;
    pub const POSTER_HEIGHT: f32 = 270.0;
    
    /// Sidebar dimensions
    pub const SIDEBAR_WIDTH: f32 = 200.0;
    pub const STEAM_DECK_SIDEBAR_WIDTH: f32 = 240.0;
    pub const CATEGORY_BUTTON_WIDTH: f32 = 175.0;
    pub const CATEGORY_BUTTON_HEIGHT: f32 = 40.0;
    pub const STEAM_DECK_CATEGORY_BUTTON_HEIGHT: f32 = 52.0;
    
    /// Button dimensions - Steam Deck friendly (larger touch targets)
    pub const BUTTON_HEIGHT: f32 = 44.0;
    pub const SMALL_BUTTON_SIZE: f32 = 44.0;
    /// Steam Deck minimum touch target (56px for comfortable touch/controller)
    pub const STEAM_DECK_BUTTON_HEIGHT: f32 = 56.0;
    pub const STEAM_DECK_TOUCH_TARGET: f32 = 56.0;
    
    /// Pagination
    pub const DEFAULT_PAGE_SIZE: usize = 30;
    pub const STEAM_DECK_PAGE_SIZE: usize = 18; // Fewer items for larger cards
    
    /// Check if likely running on Steam Deck (based on screen dimensions)
    pub fn is_steam_deck(screen_width: f32, screen_height: f32) -> bool {
        // Steam Deck has 1280x800 screen, or could be docked at various resolutions
        // Detect portable mode specifically
        (screen_width >= 1200.0 && screen_width <= 1300.0 && screen_height >= 750.0 && screen_height <= 850.0)
            || (screen_height >= 1200.0 && screen_height <= 1300.0 && screen_width >= 750.0 && screen_width <= 850.0) // Portrait
    }
    
    /// Check if in touch-friendly mode (Steam Deck or tablet)
    pub fn is_touch_mode(screen_width: f32, screen_height: f32) -> bool {
        is_steam_deck(screen_width, screen_height) || is_tablet(screen_width)
    }
    
    /// Check if in mobile mode based on screen width
    pub fn is_mobile(screen_width: f32) -> bool {
        screen_width < MOBILE_BREAKPOINT
    }
    
    /// Check if in tablet mode based on screen width
    pub fn is_tablet(screen_width: f32) -> bool {
        screen_width >= MOBILE_BREAKPOINT && screen_width < TABLET_BREAKPOINT
    }
    
    /// Get responsive card width based on screen size (poster style)
    pub fn card_width(screen_width: f32) -> f32 {
        if is_mobile(screen_width) {
            // Two columns on mobile
            ((screen_width - 48.0) / 2.0).max(140.0)
        } else if is_tablet(screen_width) {
            // Three columns on tablet
            ((screen_width - 80.0) / 3.0).min(180.0)
        } else {
            MOVIE_CARD_WIDTH
        }
    }
    
    /// Get card width optimized for Steam Deck/touch
    pub fn card_width_touch(screen_width: f32, screen_height: f32) -> f32 {
        if is_steam_deck(screen_width, screen_height) {
            STEAM_DECK_SERIES_CARD_WIDTH
        } else if is_mobile(screen_width) {
            ((screen_width - 48.0) / 2.0).max(160.0)
        } else if is_tablet(screen_width) {
            ((screen_width - 80.0) / 3.0).min(200.0)
        } else {
            MOVIE_CARD_WIDTH
        }
    }
    
    /// Get poster height based on width (3:2 aspect ratio for Netflix-style)
    pub fn poster_height(width: f32) -> f32 {
        width * 1.5
    }
    
    /// Get responsive sidebar width
    pub fn sidebar_width(screen_width: f32) -> f32 {
        if is_mobile(screen_width) {
            screen_width * 0.85 // 85% of screen on mobile
        } else if is_tablet(screen_width) {
            180.0
        } else {
            SIDEBAR_WIDTH
        }
    }
    
    /// Get sidebar width for Steam Deck/touch mode
    pub fn sidebar_width_touch(screen_width: f32, screen_height: f32) -> f32 {
        if is_steam_deck(screen_width, screen_height) {
            STEAM_DECK_SIDEBAR_WIDTH
        } else if is_mobile(screen_width) {
            screen_width * 0.85
        } else if is_tablet(screen_width) {
            200.0
        } else {
            SIDEBAR_WIDTH
        }
    }
    
    /// Get responsive channel card height
    pub fn channel_card_height(screen_width: f32) -> f32 {
        if is_mobile(screen_width) {
            90.0
        } else {
            CHANNEL_CARD_HEIGHT
        }
    }
    
    /// Get channel card height for Steam Deck/touch
    pub fn channel_card_height_touch(screen_width: f32, screen_height: f32) -> f32 {
        if is_steam_deck(screen_width, screen_height) {
            STEAM_DECK_CARD_HEIGHT
        } else if is_mobile(screen_width) {
            100.0
        } else {
            CHANNEL_CARD_HEIGHT
        }
    }
    
    /// Get responsive icon size
    pub fn icon_size(screen_width: f32) -> f32 {
        if is_mobile(screen_width) {
            60.0
        } else {
            CHANNEL_ICON_SIZE
        }
    }
    
    /// Get button height for current mode
    pub fn button_height(screen_width: f32, screen_height: f32) -> f32 {
        if is_steam_deck(screen_width, screen_height) || is_tablet(screen_width) {
            STEAM_DECK_BUTTON_HEIGHT
        } else if is_mobile(screen_width) {
            48.0
        } else {
            BUTTON_HEIGHT
        }
    }
    
    /// Get minimum touch target size
    pub fn touch_target(screen_width: f32, screen_height: f32) -> f32 {
        if is_steam_deck(screen_width, screen_height) || is_tablet(screen_width) {
            STEAM_DECK_TOUCH_TARGET
        } else if is_mobile(screen_width) {
            48.0
        } else {
            SMALL_BUTTON_SIZE
        }
    }
    
    /// Get category button height for current mode
    pub fn category_button_height(screen_width: f32, screen_height: f32) -> f32 {
        if is_steam_deck(screen_width, screen_height) || is_tablet(screen_width) {
            STEAM_DECK_CATEGORY_BUTTON_HEIGHT
        } else {
            CATEGORY_BUTTON_HEIGHT
        }
    }
    
    /// Get page size based on screen size
    pub fn page_size(screen_width: f32) -> usize {
        if is_mobile(screen_width) {
            15
        } else if is_tablet(screen_width) {
            20
        } else {
            DEFAULT_PAGE_SIZE
        }
    }
    
    /// Get page size for Steam Deck/touch mode
    pub fn page_size_touch(screen_width: f32, screen_height: f32) -> usize {
        if is_steam_deck(screen_width, screen_height) {
            STEAM_DECK_PAGE_SIZE
        } else if is_mobile(screen_width) {
            15
        } else if is_tablet(screen_width) {
            20
        } else {
            DEFAULT_PAGE_SIZE
        }
    }
}
