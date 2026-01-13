//! Category sidebar component (Netflix-style).

use eframe::egui;
use crate::models::Category;
use crate::ui::theme::{Theme, dimensions};
use crate::ui::messages::ContentType;

/// Category sidebar component for filtering content.
pub struct CategorySidebar;

impl CategorySidebar {
    /// Renders the Netflix-style category sidebar.
    /// Returns Some(category_id) if a category was selected, or Some("") for "All".
    pub fn show(
        ui: &mut egui::Ui,
        theme: &Theme,
        _content_type: ContentType,
        categories: &[Category],
        selected_category: &Option<String>,
        category_search: &mut String,
    ) -> Option<Option<String>> {
        let mut selection_changed: Option<Option<String>> = None;
        
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Genres")
            .size(16.0)
            .color(theme.text_primary)
            .strong());
        ui.add_space(12.0);
        
        // Search box with Netflix styling
        egui::Frame::none()
            .fill(theme.inactive_bg())
            .rounding(egui::Rounding::same(4.0))
            .inner_margin(egui::Margin::symmetric(8.0, 6.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔍").size(12.0).color(theme.text_secondary));
                    ui.add(egui::TextEdit::singleline(category_search)
                        .hint_text("Search...")
                        .desired_width(140.0)
                        .frame(false));
                });
            });
        
        ui.add_space(12.0);
        
        // Thin separator line
        let separator_rect = ui.available_rect_before_wrap();
        ui.painter().hline(
            separator_rect.min.x..=separator_rect.min.x + 180.0,
            separator_rect.min.y,
            egui::Stroke::new(1.0, theme.border_color),
        );
        ui.add_space(8.0);
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Only show "All" if search is empty
            if category_search.is_empty() {
                if Self::category_button(ui, theme, selected_category.is_none(), "All").clicked() {
                    selection_changed = Some(None);
                }
                ui.add_space(2.0);
            }
            
            // Filter categories by search
            let search_lower = category_search.to_lowercase();
            for category in categories {
                // Skip if doesn't match search
                if !category_search.is_empty() && 
                   !category.category_name.to_lowercase().contains(&search_lower) {
                    continue;
                }
                
                let is_selected = selected_category.as_ref() == Some(&category.category_id);
                if Self::category_button(ui, theme, is_selected, &category.category_name).clicked() {
                    selection_changed = Some(Some(category.category_id.clone()));
                }
                ui.add_space(1.0);
            }
        });
        
        selection_changed
    }
    
    /// Creates a Netflix-style category button.
    fn category_button(ui: &mut egui::Ui, theme: &Theme, is_selected: bool, text: &str) -> egui::Response {
        let (bg, fg) = if is_selected {
            (theme.accent_blue, egui::Color32::WHITE)
        } else {
            (egui::Color32::TRANSPARENT, theme.text_secondary)
        };
        
        // Truncate long category names
        let display_text = if text.chars().count() > 22 {
            format!("{}...", text.chars().take(19).collect::<String>())
        } else {
            text.to_string()
        };
        
        let btn = egui::Button::new(
            egui::RichText::new(&display_text)
                .size(13.0)
                .color(fg)
        )
        .fill(bg)
        .min_size(egui::vec2(dimensions::CATEGORY_BUTTON_WIDTH, dimensions::CATEGORY_BUTTON_HEIGHT))
        .rounding(egui::Rounding::same(4.0))
        .frame(false);
        
        let response = ui.add(btn);
        
        // Underline on hover if not selected
        if response.hovered() && !is_selected {
            let rect = response.rect;
            ui.painter().hline(
                rect.min.x + 8.0..=rect.max.x - 8.0,
                rect.max.y - 4.0,
                egui::Stroke::new(2.0, theme.accent_blue),
            );
        }
        
        response
    }
}
