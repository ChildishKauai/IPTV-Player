//! Responsive layout system.
//!
//! Every frame the app computes a [`Layout`] from the current window size.
//! Components ask it how to size themselves instead of hard-coding
//! breakpoints, so the same UI adapts from a phone-sized window up to a
//! TV, including Steam Deck Game Mode.

use eframe::egui;

/// Width-based size classes (similar to Material's window size classes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeClass {
    /// Phones and very narrow windows (< 600px)
    Compact,
    /// Tablets, split-screen and small laptop windows (600-1023px)
    Medium,
    /// Desktops, Steam Deck, TVs (>= 1024px)
    Expanded,
}

/// Breakpoint where the layout switches from Compact to Medium.
pub const MEDIUM_BREAKPOINT: f32 = 600.0;
/// Breakpoint where the layout switches from Medium to Expanded.
pub const EXPANDED_BREAKPOINT: f32 = 1024.0;

/// Responsive layout metrics for the current frame.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub width: f32,
    pub size_class: SizeClass,
    /// Use larger hit targets (gamepad, touch screens, narrow windows)
    pub touch: bool,
}

impl Layout {
    /// Builds the layout for a window of the given size.
    /// `force_touch` is set when running in Steam Deck Game Mode.
    pub fn new(width: f32, force_touch: bool) -> Self {
        let size_class = if width < MEDIUM_BREAKPOINT {
            SizeClass::Compact
        } else if width < EXPANDED_BREAKPOINT {
            SizeClass::Medium
        } else {
            SizeClass::Expanded
        };

        Self {
            width,
            size_class,
            touch: force_touch || size_class != SizeClass::Expanded,
        }
    }

    pub fn is_compact(&self) -> bool {
        self.size_class == SizeClass::Compact
    }

    pub fn is_expanded(&self) -> bool {
        self.size_class == SizeClass::Expanded
    }

    /// Whether the category sidebar is docked beside the content.
    /// Smaller layouts show it as a slide-over drawer instead.
    pub fn docked_sidebar(&self) -> bool {
        self.is_expanded()
    }

    /// Width of the docked sidebar or the drawer.
    pub fn sidebar_width(&self) -> f32 {
        match self.size_class {
            SizeClass::Expanded if self.touch => 200.0,
            SizeClass::Expanded => 248.0,
            _ => (self.width * 0.85).min(320.0),
        }
    }

    /// Horizontal padding around the main content.
    pub fn content_padding(&self) -> f32 {
        match self.size_class {
            SizeClass::Compact => 12.0,
            SizeClass::Medium => 20.0,
            SizeClass::Expanded if self.width >= 1920.0 => 40.0,
            SizeClass::Expanded => 28.0,
        }
    }

    /// Gap between cards in a grid.
    pub fn grid_gap(&self) -> f32 {
        match self.size_class {
            SizeClass::Compact => 10.0,
            SizeClass::Medium => 14.0,
            SizeClass::Expanded => 18.0,
        }
    }

    /// Minimum size for an interactive element.
    pub fn touch_target(&self) -> f32 {
        if self.touch { 48.0 } else { 36.0 }
    }

    /// Page heading size.
    pub fn heading_size(&self) -> f32 {
        if self.is_compact() { 22.0 } else { 28.0 }
    }

    /// Grid for poster cards (series, movies, discover).
    pub fn poster_grid(&self, available_width: f32) -> Grid {
        let (min, max) = match self.size_class {
            // Two posters side by side on a 360px phone
            SizeClass::Compact => (130.0, 200.0),
            SizeClass::Medium => (150.0, 210.0),
            SizeClass::Expanded => (160.0, 230.0),
        };
        Grid::fit(available_width, min, max, self.grid_gap())
    }

    /// Grid for wide cards (channels, continue watching).
    pub fn wide_grid(&self, available_width: f32) -> Grid {
        Grid::fit(available_width, 290.0, 440.0, self.grid_gap())
    }

    /// Items per page, rounded to fill complete rows.
    pub fn page_size(&self, columns: usize) -> usize {
        let target = match self.size_class {
            SizeClass::Compact => 16,
            SizeClass::Medium => 24,
            SizeClass::Expanded if self.touch => 20,
            SizeClass::Expanded => 30,
        };
        let columns = columns.max(1);
        let rows = (target + columns - 1) / columns;
        (rows * columns).max(columns)
    }
}

/// A computed grid: how many columns fit and how wide each item is.
#[derive(Debug, Clone, Copy)]
pub struct Grid {
    pub columns: usize,
    pub item_width: f32,
    pub gap: f32,
}

impl Grid {
    /// Fits as many columns as possible so that each item is between
    /// `min_width` and `max_width`, and the row fills the available width.
    pub fn fit(available_width: f32, min_width: f32, max_width: f32, gap: f32) -> Self {
        let available = available_width.max(1.0);

        // Fewest columns that keep items under max_width...
        let mut columns = ((available + gap) / (max_width + gap)).ceil().max(1.0) as usize;
        // ...unless that makes them narrower than min_width.
        if Self::width_for(available, columns, gap) < min_width {
            columns = ((available + gap) / (min_width + gap)).floor().max(1.0) as usize;
        }

        Self {
            columns,
            item_width: Self::width_for(available, columns, gap).floor(),
            gap,
        }
    }

    fn width_for(available: f32, columns: usize, gap: f32) -> f32 {
        (available - gap * (columns as f32 - 1.0)) / columns as f32
    }

    /// Lays `items` out row by row, calling `add_item` for each one.
    pub fn show<T>(&self, ui: &mut egui::Ui, items: &[T], mut add_item: impl FnMut(&mut egui::Ui, &T)) {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = self.gap;
            for row in items.chunks(self.columns) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = self.gap;
                    for item in row {
                        add_item(ui, item);
                    }
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_classes() {
        assert_eq!(Layout::new(360.0, false).size_class, SizeClass::Compact);
        assert_eq!(Layout::new(800.0, false).size_class, SizeClass::Medium);
        assert_eq!(Layout::new(1280.0, false).size_class, SizeClass::Expanded);
        assert!(Layout::new(1280.0, true).touch);
        assert!(!Layout::new(1920.0, false).touch);
    }

    #[test]
    fn grid_fills_row_within_bounds() {
        for width in [300.0, 360.0, 412.0, 768.0, 1024.0, 1440.0, 1920.0, 3840.0] {
            let grid = Grid::fit(width, 130.0, 200.0, 10.0);
            let used = grid.item_width * grid.columns as f32 + grid.gap * (grid.columns as f32 - 1.0);
            assert!(used <= width, "row overflows at {width}");
            assert!(grid.item_width <= 200.0, "item too wide at {width}");
            if grid.columns > 1 {
                assert!(grid.item_width >= 130.0, "item too narrow at {width}");
            }
        }
        // Phones get two posters per row
        assert_eq!(Grid::fit(336.0, 130.0, 200.0, 10.0).columns, 2);
    }

    #[test]
    fn page_size_fills_rows() {
        let layout = Layout::new(1440.0, false);
        for columns in 1..=8 {
            assert_eq!(layout.page_size(columns) % columns, 0);
        }
    }
}
