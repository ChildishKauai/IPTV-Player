//! Shelf component - a titled, horizontally scrolling row of cards.
//!
//! Used by the Home screen. Rows scroll by dragging, with the mouse wheel
//! (Shift + wheel), with arrow buttons on hover, or by moving gamepad focus
//! along the row (focused cards scroll themselves into view).

use eframe::egui;
use crate::ui::layout::Layout;
use crate::ui::theme::{Theme, dimensions, spacing, typography, radius};

pub struct Shelf;

impl Shelf {
    /// Row heading with an optional "See all" link on the right.
    /// Returns true when "See all" was clicked.
    pub fn header(ui: &mut egui::Ui, theme: &Theme, layout: &Layout, title: &str, see_all: bool) -> bool {
        let mut clicked = false;
        let height = layout.touch_target();
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(
                    egui::RichText::new(title)
                        .size(if layout.is_compact() { typography::H3 } else { typography::H2 })
                        .color(theme.text_primary)
                        .strong(),
                );
                if see_all {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let button = egui::Button::new(
                            egui::RichText::new("See all  ›").size(typography::BODY_SM).color(theme.text_secondary),
                        )
                        .frame(false)
                        .min_size(egui::vec2(0.0, height));
                        clicked = ui.add(button).clicked();
                    });
                }
            },
        );
        ui.add_space(spacing::XS);
        clicked
    }

    /// Lays out `add_items` in a horizontally scrolling row.
    pub fn row(ui: &mut egui::Ui, theme: &Theme, layout: &Layout, id: &str, add_items: impl FnOnce(&mut egui::Ui)) {
        let gap = layout.grid_gap();
        let output = egui::ScrollArea::horizontal()
            .id_salt(("shelf", id))
            .auto_shrink([false, true])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    add_items(ui);
                });
            });

        // Arrow buttons for mouse users (touch and gamepad scroll directly).
        // `inner_rect` is clipped to the visible screen, so rebuild the row's
        // own rect from the content height.
        let view = egui::Rect::from_min_size(
            output.inner_rect.min,
            egui::vec2(output.inner_rect.width(), output.content_size.y),
        );
        let max_offset = (output.content_size.x - view.width()).max(0.0);
        let hovered = ui.rect_contains_pointer(view);
        if max_offset <= 1.0 || layout.touch || !hovered {
            return;
        }

        let mut state = output.state;
        let mut offset = state.offset.x;
        let step = view.width() * 0.8;
        let size = 40.0;
        // Centre on the artwork rather than the captions below it
        let y = view.min.y + (view.height() - 48.0).max(size) / 2.0;

        if offset > 1.0 {
            let rect = egui::Rect::from_center_size(egui::pos2(view.min.x + size / 2.0 + spacing::XS, y), egui::vec2(size, size));
            if Self::arrow(ui, theme, rect, "‹").on_hover_text("Scroll left").clicked() {
                offset = (offset - step).max(0.0);
            }
        }
        if offset < max_offset - 1.0 {
            let rect = egui::Rect::from_center_size(egui::pos2(view.max.x - size / 2.0 - spacing::XS, y), egui::vec2(size, size));
            if Self::arrow(ui, theme, rect, "›").on_hover_text("Scroll right").clicked() {
                offset = (offset + step).min(max_offset);
            }
        }

        if offset != state.offset.x {
            state.offset.x = offset;
            state.store(ui.ctx(), output.id);
            ui.ctx().request_repaint();
        }
    }

    fn arrow(ui: &mut egui::Ui, theme: &Theme, rect: egui::Rect, icon: &str) -> egui::Response {
        let response = ui.interact(rect, ui.id().with(("shelf_arrow", rect.min.x as i32, icon)), egui::Sense::click());
        let fill = if response.hovered() { theme.card_elevated } else { theme.glass_bg() };
        ui.painter().circle_filled(rect.center(), rect.width() / 2.0, fill);
        ui.painter().circle_stroke(rect.center(), rect.width() / 2.0, egui::Stroke::new(1.0, theme.border_strong));
        ui.painter().text(
            rect.center() - egui::vec2(0.0, 2.0),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(26.0),
            theme.text_primary,
        );
        response
    }

    /// Placeholder poster cards shown while a row is loading.
    pub fn skeleton_posters(ui: &mut egui::Ui, theme: &Theme, layout: &Layout, id: &str, card_width: f32) {
        let poster_height = dimensions::poster_height(card_width);
        // Gentle pulse so the row reads as "loading", not "empty"
        let t = ui.input(|i| i.time) as f32;
        let alpha = 0.55 + 0.25 * (t * 3.0).sin();
        ui.ctx().request_repaint();
        let count = ((ui.available_width() / card_width).ceil() as usize).clamp(2, 10);
        Self::row(ui, theme, layout, id, |ui| {
            for _ in 0..count {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(card_width, poster_height + 48.0), egui::Sense::hover());
                let color = theme.placeholder_bg().gamma_multiply(alpha);
                let poster = egui::Rect::from_min_size(rect.min, egui::vec2(card_width, poster_height));
                ui.painter().rect_filled(poster, radius::LG, color);
                let line = egui::Rect::from_min_size(
                    egui::pos2(rect.min.x, poster.max.y + spacing::SM + 2.0),
                    egui::vec2(card_width * 0.7, 12.0),
                );
                ui.painter().rect_filled(line, radius::SM, color);
            }
        });
    }

    /// A short hint in place of a row that has no items yet.
    pub fn hint(ui: &mut egui::Ui, theme: &Theme, text: &str) {
        egui::Frame::none()
            .fill(theme.card_bg)
            .stroke(egui::Stroke::new(1.0, theme.border_color))
            .rounding(egui::Rounding::same(radius::LG))
            .inner_margin(egui::Margin::symmetric(spacing::LG, spacing::LG))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new(text).size(typography::BODY_SM).color(theme.text_secondary));
            });
    }
}
