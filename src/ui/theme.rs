//! Theme and styling configuration for the IPTV Player UI.
//!
//! A refined design system inspired by Apple's Human Interface Guidelines
//! and Airbnb's design language. Features subtle gradients, thoughtful
//! spacing, and a sophisticated color palette.

use eframe::egui;

/// Design tokens for consistent spacing throughout the app
pub mod spacing {
    /// Extra small spacing (4px)
    pub const XS: f32 = 4.0;
    /// Small spacing (8px)
    pub const SM: f32 = 8.0;
    /// Medium spacing (12px)
    pub const MD: f32 = 12.0;
    /// Large spacing (16px)
    pub const LG: f32 = 16.0;
    /// Extra large spacing (24px)
    pub const XL: f32 = 24.0;
    /// 2x Extra large spacing (32px)
    pub const XXL: f32 = 32.0;
    /// 3x Extra large spacing (48px)
    pub const XXXL: f32 = 48.0;
}

/// Typography scale
pub mod typography {
    /// Display heading (32px)
    pub const DISPLAY: f32 = 32.0;
    /// Large heading (24px)
    pub const H1: f32 = 24.0;
    /// Medium heading (20px)
    pub const H2: f32 = 20.0;
    /// Small heading (18px)
    pub const H3: f32 = 18.0;
    /// Body text (15px)
    pub const BODY: f32 = 15.0;
    /// Secondary text (14px)
    pub const BODY_SM: f32 = 14.0;
    /// Caption text (12px)
    pub const CAPTION: f32 = 12.0;
    /// Small label (11px)
    pub const LABEL: f32 = 11.0;
}

/// Border radius tokens
pub mod radius {
    /// Small radius for pills and chips (4px)
    pub const SM: f32 = 4.0;
    /// Medium radius for buttons and inputs (8px)
    pub const MD: f32 = 8.0;
    /// Large radius for cards (12px)
    pub const LG: f32 = 12.0;
    /// Extra large radius for panels (16px)
    pub const XL: f32 = 16.0;
    /// Full radius for circles/pills
    pub const FULL: f32 = 9999.0;
}

/// Theme colors and styling configuration.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Whether dark mode is enabled
    pub dark_mode: bool,

    // ─────────────────────────────────────────────────────────────────────
    // Background colors (layered system)
    // ─────────────────────────────────────────────────────────────────────

    /// Base background (lowest layer)
    pub bg_color: egui::Color32,

    /// Elevated surface (sidebar, panels)
    pub panel_bg: egui::Color32,

    /// Card/component surface
    pub card_bg: egui::Color32,

    /// Elevated card (hover states, dialogs)
    pub card_elevated: egui::Color32,

    // ─────────────────────────────────────────────────────────────────────
    // Text colors
    // ─────────────────────────────────────────────────────────────────────

    /// Primary text (headings, important content)
    pub text_primary: egui::Color32,

    /// Secondary text (body, descriptions)
    pub text_secondary: egui::Color32,

    /// Tertiary text (captions, metadata)
    pub text_tertiary: egui::Color32,

    /// Disabled/muted text
    pub text_muted: egui::Color32,

    // ─────────────────────────────────────────────────────────────────────
    // Border & Divider colors
    // ─────────────────────────────────────────────────────────────────────

    /// Subtle border for cards
    pub border_color: egui::Color32,

    /// Stronger border for focus states
    pub border_strong: egui::Color32,

    /// Divider lines
    pub divider: egui::Color32,

    // ─────────────────────────────────────────────────────────────────────
    // Accent colors
    // ─────────────────────────────────────────────────────────────────────

    /// Primary accent (brand color - refined coral/red)
    pub accent_blue: egui::Color32,

    /// Accent hover state
    pub accent_hover: egui::Color32,

    /// Accent pressed state
    pub accent_pressed: egui::Color32,

    /// Success color (green)
    pub success_color: egui::Color32,

    /// Warning color (amber)
    pub warning_color: egui::Color32,

    /// Error color (red)
    pub error_color: egui::Color32,

    // ─────────────────────────────────────────────────────────────────────
    // Interactive states
    // ─────────────────────────────────────────────────────────────────────

    /// Hover overlay
    pub hover_overlay: egui::Color32,

    /// Active/pressed overlay
    pub active_overlay: egui::Color32,
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

    /// Creates a refined dark theme
    /// Inspired by Apple TV+ and premium streaming apps
    pub fn dark() -> Self {
        Self {
            dark_mode: true,

            // Layered background system
            bg_color: egui::Color32::from_rgb(13, 13, 13),           // Near black
            panel_bg: egui::Color32::from_rgb(18, 18, 18),           // Elevated surface
            card_bg: egui::Color32::from_rgb(24, 24, 24),            // Card surface
            card_elevated: egui::Color32::from_rgb(32, 32, 32),      // Hover/dialog

            // Text hierarchy
            text_primary: egui::Color32::from_rgb(255, 255, 255),    // Pure white
            text_secondary: egui::Color32::from_rgb(170, 170, 170),  // 67% white
            text_tertiary: egui::Color32::from_rgb(128, 128, 128),   // 50% white
            text_muted: egui::Color32::from_rgb(85, 85, 85),         // 33% white

            // Borders
            border_color: egui::Color32::from_rgb(38, 38, 38),       // Subtle
            border_strong: egui::Color32::from_rgb(64, 64, 64),      // Prominent
            divider: egui::Color32::from_rgb(32, 32, 32),            // Very subtle

            // Accent - refined coral/red (Airbnb-inspired)
            accent_blue: egui::Color32::from_rgb(255, 90, 95),       // Coral red
            accent_hover: egui::Color32::from_rgb(255, 110, 115),    // Lighter on hover
            accent_pressed: egui::Color32::from_rgb(230, 70, 75),    // Darker on press

            // Semantic colors
            success_color: egui::Color32::from_rgb(52, 199, 89),     // Apple green
            warning_color: egui::Color32::from_rgb(255, 204, 0),     // Amber
            error_color: egui::Color32::from_rgb(255, 69, 58),       // Apple red

            // Overlays
            hover_overlay: egui::Color32::from_rgba_unmultiplied(255, 255, 255, 8),
            active_overlay: egui::Color32::from_rgba_unmultiplied(255, 255, 255, 12),
        }
    }

    /// Creates a refined light theme
    pub fn light() -> Self {
        Self {
            dark_mode: false,

            // Layered background system
            bg_color: egui::Color32::from_rgb(250, 250, 250),        // Off-white
            panel_bg: egui::Color32::from_rgb(255, 255, 255),        // Pure white
            card_bg: egui::Color32::from_rgb(255, 255, 255),         // White cards
            card_elevated: egui::Color32::from_rgb(255, 255, 255),   // White elevated

            // Text hierarchy
            text_primary: egui::Color32::from_rgb(17, 17, 17),       // Near black
            text_secondary: egui::Color32::from_rgb(72, 72, 72),     // Dark gray
            text_tertiary: egui::Color32::from_rgb(115, 115, 115),   // Medium gray
            text_muted: egui::Color32::from_rgb(170, 170, 170),      // Light gray

            // Borders
            border_color: egui::Color32::from_rgb(235, 235, 235),    // Light border
            border_strong: egui::Color32::from_rgb(200, 200, 200),   // Stronger border
            divider: egui::Color32::from_rgb(240, 240, 240),         // Very light

            // Accent
            accent_blue: egui::Color32::from_rgb(255, 56, 92),       // Airbnb pink/red
            accent_hover: egui::Color32::from_rgb(255, 80, 110),     // Lighter
            accent_pressed: egui::Color32::from_rgb(220, 40, 75),    // Darker

            // Semantic colors
            success_color: egui::Color32::from_rgb(40, 167, 69),     // Green
            warning_color: egui::Color32::from_rgb(255, 193, 7),     // Amber
            error_color: egui::Color32::from_rgb(220, 53, 69),       // Red

            // Overlays
            hover_overlay: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 5),
            active_overlay: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 8),
        }
    }

    /// Returns the hover background color for interactive elements
    pub fn hover_bg(&self) -> egui::Color32 {
        self.card_elevated
    }

    /// Returns the inactive/button background color
    pub fn inactive_bg(&self) -> egui::Color32 {
        if self.dark_mode {
            egui::Color32::from_rgb(38, 38, 38)
        } else {
            egui::Color32::from_rgb(245, 245, 245)
        }
    }

    /// Returns the placeholder background color for loading states
    pub fn placeholder_bg(&self) -> egui::Color32 {
        if self.dark_mode {
            egui::Color32::from_rgb(32, 32, 32)
        } else {
            egui::Color32::from_rgb(240, 240, 240)
        }
    }

    /// Returns the placeholder icon color
    pub fn placeholder_icon(&self) -> egui::Color32 {
        if self.dark_mode {
            egui::Color32::from_rgb(72, 72, 72)
        } else {
            egui::Color32::from_rgb(180, 180, 180)
        }
    }

    /// Returns a shadow color for cards
    pub fn card_shadow(&self) -> egui::Color32 {
        if self.dark_mode {
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 80)
        } else {
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 15)
        }
    }

    /// Returns a deeper shadow for elevated elements
    pub fn elevated_shadow(&self) -> egui::Color32 {
        if self.dark_mode {
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150)
        } else {
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 30)
        }
    }

    /// Returns the hover overlay color for cards
    #[allow(dead_code)]
    pub fn card_hover_overlay(&self) -> egui::Color32 {
        self.hover_overlay
    }

    /// Returns gradient overlay for cards (bottom darkening)
    pub fn gradient_overlay(&self) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180)
    }

    /// Returns a glass-like background for overlays
    pub fn glass_bg(&self) -> egui::Color32 {
        if self.dark_mode {
            egui::Color32::from_rgba_unmultiplied(13, 13, 13, 230)
        } else {
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 230)
        }
    }

    /// Returns badge/pill background
    pub fn badge_bg(&self) -> egui::Color32 {
        if self.dark_mode {
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15)
        } else {
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 8)
        }
    }

    /// Live badge color
    pub fn live_badge(&self) -> egui::Color32 {
        egui::Color32::from_rgb(255, 59, 48)  // Apple red
    }

    /// Applies the theme to the egui context
    pub fn apply(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // Typography scale - clean, modern sizing
        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::proportional(typography::H1)),
            (egui::TextStyle::Body, egui::FontId::proportional(typography::BODY)),
            (egui::TextStyle::Button, egui::FontId::proportional(typography::BODY_SM)),
            (egui::TextStyle::Small, egui::FontId::proportional(typography::CAPTION)),
            (egui::TextStyle::Monospace, egui::FontId::monospace(typography::BODY_SM)),
        ].into();

        // Refined spacing
        style.spacing.item_spacing = egui::vec2(spacing::SM, spacing::SM);
        style.spacing.button_padding = egui::vec2(spacing::LG, spacing::MD);
        style.spacing.indent = spacing::LG;
        style.spacing.window_margin = egui::Margin::same(spacing::LG);

        // Modern rounded corners
        style.visuals.window_rounding = egui::Rounding::same(radius::XL);
        style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(radius::MD);
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(radius::MD);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(radius::MD);
        style.visuals.widgets.active.rounding = egui::Rounding::same(radius::MD);

        // Widget colors - refined
        style.visuals.widgets.noninteractive.bg_fill = self.panel_bg;
        style.visuals.widgets.inactive.bg_fill = self.inactive_bg();
        style.visuals.widgets.hovered.bg_fill = self.hover_bg();
        style.visuals.widgets.active.bg_fill = self.accent_blue;

        // Subtle borders
        style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(0.0, self.border_color);
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, self.border_color);
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, self.border_strong);
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(0.0, self.accent_blue);

        // Selection styling
        style.visuals.selection.bg_fill = self.accent_blue.linear_multiply(0.3);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, self.accent_blue);

        // Text colors
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, self.text_primary);
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, self.text_primary);

        ctx.set_style(style);

        // Set visuals based on theme
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals {
                dark_mode: true,
                panel_fill: self.bg_color,
                window_fill: self.card_bg,
                window_stroke: egui::Stroke::new(1.0, self.border_color),
                ..egui::Visuals::dark()
            });
        } else {
            ctx.set_visuals(egui::Visuals {
                dark_mode: false,
                panel_fill: self.bg_color,
                window_fill: self.card_bg,
                window_stroke: egui::Stroke::new(1.0, self.border_color),
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

    /// Card dimensions (desktop) - refined poster style
    pub const CHANNEL_CARD_WIDTH: f32 = 340.0;
    pub const CHANNEL_CARD_HEIGHT: f32 = 100.0;
    pub const SERIES_CARD_WIDTH: f32 = 180.0;
    pub const SERIES_CARD_HEIGHT: f32 = 270.0;
    pub const MOVIE_CARD_WIDTH: f32 = 180.0;
    pub const MOVIE_CARD_HEIGHT: f32 = 270.0;

    /// Steam Deck optimized card dimensions
    pub const STEAM_DECK_CARD_WIDTH: f32 = 160.0;
    pub const STEAM_DECK_CARD_HEIGHT: f32 = 100.0;
    pub const STEAM_DECK_SERIES_CARD_WIDTH: f32 = 140.0;
    pub const STEAM_DECK_SERIES_CARD_HEIGHT: f32 = 210.0;

    /// Image dimensions
    pub const CHANNEL_ICON_SIZE: f32 = 64.0;
    pub const POSTER_WIDTH: f32 = 180.0;
    pub const POSTER_HEIGHT: f32 = 270.0;

    /// Sidebar dimensions
    pub const SIDEBAR_WIDTH: f32 = 240.0;
    pub const STEAM_DECK_SIDEBAR_WIDTH: f32 = 200.0;
    pub const CATEGORY_BUTTON_WIDTH: f32 = 210.0;
    pub const CATEGORY_BUTTON_HEIGHT: f32 = 44.0;
    pub const STEAM_DECK_CATEGORY_BUTTON_HEIGHT: f32 = 48.0;

    /// Button dimensions
    pub const BUTTON_HEIGHT: f32 = 44.0;
    pub const SMALL_BUTTON_SIZE: f32 = 40.0;
    pub const STEAM_DECK_BUTTON_HEIGHT: f32 = 52.0;
    pub const STEAM_DECK_TOUCH_TARGET: f32 = 52.0;

    /// Pagination
    pub const DEFAULT_PAGE_SIZE: usize = 30;
    pub const STEAM_DECK_PAGE_SIZE: usize = 24;

    /// Check if likely running on Steam Deck
    pub fn is_steam_deck(screen_width: f32, screen_height: f32) -> bool {
        (screen_width >= 1200.0 && screen_width <= 1300.0 && screen_height >= 750.0 && screen_height <= 850.0)
            || (screen_height >= 1200.0 && screen_height <= 1300.0 && screen_width >= 750.0 && screen_width <= 850.0)
    }

    /// Check if in touch-friendly mode
    pub fn is_touch_mode(screen_width: f32, screen_height: f32) -> bool {
        is_steam_deck(screen_width, screen_height) || is_tablet(screen_width)
    }

    /// Check if in mobile mode
    pub fn is_mobile(screen_width: f32) -> bool {
        screen_width < MOBILE_BREAKPOINT
    }

    /// Check if in tablet mode
    pub fn is_tablet(screen_width: f32) -> bool {
        screen_width >= MOBILE_BREAKPOINT && screen_width < TABLET_BREAKPOINT
    }

    /// Get responsive card width
    pub fn card_width(screen_width: f32) -> f32 {
        if is_mobile(screen_width) {
            ((screen_width - 48.0) / 2.0).max(140.0)
        } else if is_tablet(screen_width) {
            ((screen_width - 80.0) / 3.0).min(180.0)
        } else {
            MOVIE_CARD_WIDTH
        }
    }

    /// Get card width for touch mode
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

    /// Get poster height (3:2 aspect ratio)
    pub fn poster_height(width: f32) -> f32 {
        width * 1.5
    }

    /// Get responsive sidebar width
    pub fn sidebar_width(screen_width: f32) -> f32 {
        if is_mobile(screen_width) {
            screen_width * 0.85
        } else if is_tablet(screen_width) {
            200.0
        } else {
            SIDEBAR_WIDTH
        }
    }

    /// Get sidebar width for touch mode
    pub fn sidebar_width_touch(screen_width: f32, screen_height: f32) -> f32 {
        if is_steam_deck(screen_width, screen_height) {
            STEAM_DECK_SIDEBAR_WIDTH
        } else if is_mobile(screen_width) {
            screen_width * 0.85
        } else if is_tablet(screen_width) {
            220.0
        } else {
            SIDEBAR_WIDTH
        }
    }

    /// Get responsive channel card height
    pub fn channel_card_height(screen_width: f32) -> f32 {
        if is_mobile(screen_width) {
            88.0
        } else {
            CHANNEL_CARD_HEIGHT
        }
    }

    /// Get channel card height for touch
    pub fn channel_card_height_touch(screen_width: f32, screen_height: f32) -> f32 {
        if is_steam_deck(screen_width, screen_height) {
            STEAM_DECK_CARD_HEIGHT
        } else if is_mobile(screen_width) {
            96.0
        } else {
            CHANNEL_CARD_HEIGHT
        }
    }

    /// Get responsive icon size
    pub fn icon_size(screen_width: f32) -> f32 {
        if is_mobile(screen_width) {
            56.0
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

    /// Get category button height
    pub fn category_button_height(screen_width: f32, screen_height: f32) -> f32 {
        if is_steam_deck(screen_width, screen_height) || is_tablet(screen_width) {
            STEAM_DECK_CATEGORY_BUTTON_HEIGHT
        } else {
            CATEGORY_BUTTON_HEIGHT
        }
    }

    /// Get page size based on screen
    pub fn page_size(screen_width: f32) -> usize {
        if is_mobile(screen_width) {
            15
        } else if is_tablet(screen_width) {
            20
        } else {
            DEFAULT_PAGE_SIZE
        }
    }

    /// Get page size for touch mode
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
