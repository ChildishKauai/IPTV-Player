//! Category sidebar component - Modern, clean design
//!
//! A refined sidebar with clean category buttons and subtle interactions.
//! Features smooth scrolling and clear visual hierarchy.

use eframe::egui;
use crate::models::Category;
use crate::ui::theme::{Theme, spacing, typography, radius};
use crate::ui::messages::ContentType;

/// Category sidebar component - Modern design
pub struct CategorySidebar;

impl CategorySidebar {
    /// Renders the sidebar with category list.
    /// Returns Some(category_id) if a category was selected, or Some(None) for "All".
    pub fn show(
        ui: &mut egui::Ui,
        theme: &Theme,
        _content_type: ContentType,
        categories: &[Category],
        selected_category: &Option<String>,
        category_search: &mut String,
    ) -> Option<Option<String>> {
        let mut selection_changed: Option<Option<String>> = None;

        // Sizing
        let item_height = 44.0;
        let font_size = typography::BODY_SM;

        ui.add_space(spacing::SM);

        // Header
        ui.label(
            egui::RichText::new("Categories")
                .size(typography::CAPTION)
                .color(theme.text_tertiary),
        );

        ui.add_space(spacing::MD);

        // Search input
        egui::Frame::none()
            .fill(theme.inactive_bg())
            .rounding(egui::Rounding::same(radius::MD))
            .inner_margin(egui::Margin::symmetric(spacing::MD, spacing::SM))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("🔍")
                            .size(14.0)
                            .color(theme.text_muted),
                    );
                    ui.add_space(spacing::XS);

                    let search_edit = egui::TextEdit::singleline(category_search)
                        .hint_text(
                            egui::RichText::new("Filter...")
                                .size(font_size)
                                .color(theme.text_muted),
                        )
                        .desired_width(ui.available_width() - 30.0)
                        .font(egui::FontId::proportional(font_size))
                        .frame(false);

                    ui.add(search_edit);
                });
            });

        ui.add_space(spacing::MD);

        // Category list with scroll
        let scroll_height = ui.available_height() - spacing::LG;
        egui::ScrollArea::vertical()
            .max_height(scroll_height)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, spacing::XS);

                    // "All" option (only if no search)
                    if category_search.is_empty() {
                        let is_all_selected = selected_category.is_none();
                        let all_response = Self::category_button(
                            ui,
                            theme,
                            "All Categories",
                            is_all_selected,
                            item_height,
                            font_size,
                        );
                        if all_response.clicked() {
                            selection_changed = Some(None);
                        }

                        // Divider
                        ui.add_space(spacing::SM);
                        ui.add(egui::Separator::default().spacing(0.0));
                        ui.add_space(spacing::SM);
                    }

                    // Filter categories
                    let search_lower = category_search.to_lowercase();
                    let filtered_categories: Vec<_> = categories
                        .iter()
                        .filter(|c| {
                            search_lower.is_empty()
                                || c.category_name.to_lowercase().contains(&search_lower)
                        })
                        .collect();

                    // Category count
                    ui.label(
                        egui::RichText::new(format!("{} categories", filtered_categories.len()))
                            .size(typography::LABEL)
                            .color(theme.text_muted),
                    );

                    ui.add_space(spacing::SM);

                    // Category buttons
                    for category in filtered_categories {
                        let is_selected = selected_category
                            .as_ref()
                            .map(|id| id == &category.category_id)
                            .unwrap_or(false);

                        let response = Self::category_button(
                            ui,
                            theme,
                            &category.category_name,
                            is_selected,
                            item_height,
                            font_size,
                        );

                        if response.clicked() {
                            selection_changed = Some(Some(category.category_id.clone()));
                        }
                    }

                    // Bottom padding
                    ui.add_space(spacing::XL);
                });
            });

        selection_changed
    }

    /// Creates a category button with modern styling
    fn category_button(
        ui: &mut egui::Ui,
        theme: &Theme,
        text: &str,
        is_selected: bool,
        height: f32,
        font_size: f32,
    ) -> egui::Response {
        let available_width = ui.available_width();
        let display_text = truncate_text(text, 26);

        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(available_width, height), egui::Sense::click());

        let is_hovered = response.hovered();

        // Background
        let bg_color = if is_selected {
            theme.accent_blue.linear_multiply(0.15)
        } else if is_hovered {
            theme.hover_overlay
        } else {
            egui::Color32::TRANSPARENT
        };

        ui.painter().rect_filled(rect, radius::MD, bg_color);

        // Left accent bar for selected
        if is_selected {
            let accent_rect = egui::Rect::from_min_size(rect.min, egui::vec2(3.0, height));
            ui.painter()
                .rect_filled(accent_rect, radius::SM, theme.accent_blue);
        }

        // Text
        let text_color = if is_selected {
            theme.text_primary
        } else if is_hovered {
            theme.text_primary
        } else {
            theme.text_secondary
        };

        let galley = ui.painter().layout_no_wrap(
            display_text,
            egui::FontId::proportional(font_size),
            text_color,
        );

        let text_pos = egui::pos2(
            rect.min.x + spacing::LG,
            rect.center().y - galley.size().y / 2.0,
        );
        ui.painter().galley(text_pos, galley, text_color);

        response
    }
}

/// Truncates text to a maximum length with ellipsis.
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.chars().count() > max_len {
        format!(
            "{}...",
            text.chars().take(max_len.saturating_sub(3)).collect::<String>()
        )
    } else {
        text.to_string()
    }
}
