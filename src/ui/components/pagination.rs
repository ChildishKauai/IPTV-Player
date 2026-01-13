//! Pagination component for navigating through content pages (Netflix-style).

use eframe::egui;
use crate::ui::theme::Theme;

/// Pagination component for navigating through pages of content.
pub struct Pagination;

impl Pagination {
    /// Renders Netflix-style pagination controls.
    /// Returns the new page number if changed.
    pub fn show(
        ui: &mut egui::Ui,
        theme: &Theme,
        current_page: usize,
        total_pages: usize,
    ) -> Option<usize> {
        if total_pages <= 1 {
            return None;
        }
        
        let mut new_page: Option<usize> = None;
        
        ui.add_space(24.0);
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            
            // Previous button - minimal Netflix style
            let prev_enabled = current_page > 0;
            let prev_color = if prev_enabled { theme.text_primary } else { theme.text_secondary };
            let prev_bg = if prev_enabled { theme.inactive_bg() } else { egui::Color32::TRANSPARENT };
            
            let prev_btn = egui::Button::new(
                egui::RichText::new("← Previous")
                    .size(13.0)
                    .color(prev_color)
            )
            .fill(prev_bg)
            .rounding(egui::Rounding::same(4.0))
            .min_size(egui::vec2(100.0, 36.0));
            
            if ui.add_enabled(prev_enabled, prev_btn).clicked() {
                new_page = Some(current_page.saturating_sub(1));
            }
            
            ui.add_space(16.0);
            
            // Page dots / numbers (show up to 5 pages)
            let start_page = if current_page < 2 { 0 } else { current_page.saturating_sub(2) };
            let end_page = (start_page + 5).min(total_pages);
            
            for page in start_page..end_page {
                let is_current = page == current_page;
                let page_btn = if is_current {
                    egui::Button::new(
                        egui::RichText::new(format!("{}", page + 1))
                            .size(13.0)
                            .color(egui::Color32::WHITE)
                    )
                    .fill(theme.accent_blue)
                    .min_size(egui::vec2(36.0, 36.0))
                    .rounding(egui::Rounding::same(4.0))
                } else {
                    egui::Button::new(
                        egui::RichText::new(format!("{}", page + 1))
                            .size(13.0)
                            .color(theme.text_secondary)
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .min_size(egui::vec2(36.0, 36.0))
                };
                
                if ui.add(page_btn).clicked() && !is_current {
                    new_page = Some(page);
                }
            }
            
            // Show ellipsis if there are more pages
            if end_page < total_pages {
                ui.label(egui::RichText::new("...").size(14.0).color(theme.text_secondary));
            }
            
            ui.add_space(16.0);
            
            // Next button - minimal Netflix style
            let next_enabled = current_page < total_pages - 1;
            let next_color = if next_enabled { theme.text_primary } else { theme.text_secondary };
            let next_bg = if next_enabled { theme.inactive_bg() } else { egui::Color32::TRANSPARENT };
            
            let next_btn = egui::Button::new(
                egui::RichText::new("Next →")
                    .size(13.0)
                    .color(next_color)
            )
            .fill(next_bg)
            .rounding(egui::Rounding::same(4.0))
            .min_size(egui::vec2(100.0, 36.0));
            
            if ui.add_enabled(next_enabled, next_btn).clicked() {
                new_page = Some(current_page + 1);
            }
        });
        
        new_page
    }
    
    /// Renders pagination info text showing the range of items.
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
            ui.label(egui::RichText::new(format!(
                "{} {}", 
                total_items,
                item_type
            ))
                .size(13.0)
                .color(theme.text_secondary));
            
            ui.label(egui::RichText::new("•").size(13.0).color(theme.text_secondary));
            
            ui.label(egui::RichText::new(format!(
                "Showing {}-{}", 
                if total_items > 0 { start_idx + 1 } else { 0 }, 
                end_idx
            ))
                .size(13.0)
                .color(theme.text_secondary));
        });
    }
}
