//! Pagination component - Modern pill-style design
//!
//! Clean, minimal pagination with subtle hover effects.
//! Inspired by modern web apps and Apple's design language.

use eframe::egui;
use crate::ui::theme::{Theme, spacing, typography, radius};

/// Pagination component - Modern design
pub struct Pagination;

impl Pagination {
    /// Renders modern pagination controls.
    /// Returns the new page number if changed.
    pub fn show(
        ui: &mut egui::Ui,
        theme: &Theme,
        current_page: usize,
        total_pages: usize,
        is_touch_mode: bool,
    ) -> Option<usize> {
        if total_pages <= 1 {
            return None;
        }

        let mut new_page: Option<usize> = None;

        // Touch-friendly sizing
        let btn_height = if is_touch_mode { 48.0 } else { 40.0 };
        let page_btn_size = if is_touch_mode { 44.0 } else { 36.0 };
        let font_size = if is_touch_mode { typography::BODY } else { typography::BODY_SM };

        ui.add_space(spacing::XL);

        // Center the pagination
        ui.horizontal(|ui| {
            // Calculate center offset
            let available = ui.available_width();
            let estimated_width = 400.0; // Approximate width of pagination
            let offset = ((available - estimated_width) / 2.0).max(0.0);
            ui.add_space(offset);

            // Previous button
            let prev_enabled = current_page > 0;
            if Self::nav_button(ui, theme, "Previous", prev_enabled, btn_height, font_size, false) {
                new_page = Some(current_page.saturating_sub(1));
            }

            ui.add_space(spacing::LG);

            // Page numbers (show up to 5 pages)
            let start_page = if current_page < 2 {
                0
            } else {
                current_page.saturating_sub(2)
            };
            let end_page = (start_page + 5).min(total_pages);

            // First page if not visible
            if start_page > 0 {
                if Self::page_button(ui, theme, 1, false, page_btn_size, font_size) {
                    new_page = Some(0);
                }
                if start_page > 1 {
                    ui.label(
                        egui::RichText::new("...")
                            .size(font_size)
                            .color(theme.text_muted),
                    );
                }
            }

            // Page buttons
            for page in start_page..end_page {
                let is_current = page == current_page;
                if Self::page_button(ui, theme, page + 1, is_current, page_btn_size, font_size) {
                    if !is_current {
                        new_page = Some(page);
                    }
                }
            }

            // Last page if not visible
            if end_page < total_pages {
                if end_page < total_pages - 1 {
                    ui.label(
                        egui::RichText::new("...")
                            .size(font_size)
                            .color(theme.text_muted),
                    );
                }
                if Self::page_button(ui, theme, total_pages, false, page_btn_size, font_size) {
                    new_page = Some(total_pages - 1);
                }
            }

            ui.add_space(spacing::LG);

            // Next button
            let next_enabled = current_page < total_pages - 1;
            if Self::nav_button(ui, theme, "Next", next_enabled, btn_height, font_size, true) {
                new_page = Some(current_page + 1);
            }
        });

        ui.add_space(spacing::MD);

        new_page
    }

    /// Creates a navigation button (Previous/Next)
    fn nav_button(
        ui: &mut egui::Ui,
        theme: &Theme,
        text: &str,
        enabled: bool,
        height: f32,
        font_size: f32,
        is_next: bool,
    ) -> bool {
        let text_color = if enabled {
            theme.text_primary
        } else {
            theme.text_muted
        };

        let label = if is_next {
            format!("{} →", text)
        } else {
            format!("← {}", text)
        };

        let btn = egui::Button::new(
            egui::RichText::new(label)
                .size(font_size)
                .color(text_color),
        )
        .fill(if enabled {
            theme.inactive_bg()
        } else {
            egui::Color32::TRANSPARENT
        })
        .rounding(egui::Rounding::same(radius::MD))
        .min_size(egui::vec2(100.0, height));

        ui.add_enabled(enabled, btn).clicked()
    }

    /// Creates a page number button
    fn page_button(
        ui: &mut egui::Ui,
        theme: &Theme,
        page_num: usize,
        is_current: bool,
        size: f32,
        font_size: f32,
    ) -> bool {
        let (bg, fg) = if is_current {
            (theme.accent_blue, egui::Color32::WHITE)
        } else {
            (egui::Color32::TRANSPARENT, theme.text_secondary)
        };

        let btn = egui::Button::new(
            egui::RichText::new(format!("{}", page_num))
                .size(font_size)
                .color(fg),
        )
        .fill(bg)
        .min_size(egui::vec2(size, size))
        .rounding(egui::Rounding::same(radius::MD));

        ui.add(btn).clicked()
    }

    /// Renders pagination info text
    pub fn show_info(
        ui: &mut egui::Ui,
        theme: &Theme,
        start_idx: usize,
        end_idx: usize,
        total_items: usize,
        item_type: &str,
        _current_page: usize,
        _total_pages: usize,
    ) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("{} {}", total_items, item_type))
                    .size(typography::CAPTION)
                    .color(theme.text_tertiary),
            );

            ui.label(
                egui::RichText::new("•")
                    .size(typography::CAPTION)
                    .color(theme.text_muted),
            );

            ui.label(
                egui::RichText::new(format!(
                    "Showing {}-{}",
                    if total_items > 0 { start_idx + 1 } else { 0 },
                    end_idx
                ))
                .size(typography::CAPTION)
                .color(theme.text_tertiary),
            );
        });
    }
}
