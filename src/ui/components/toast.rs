//! Toast notifications.
//!
//! Short messages stacked at the bottom of the content area. Toasts can
//! carry one action button ("Try again", "Player settings"); the action
//! type is chosen by the caller and returned when the button is pressed.

use eframe::egui;
use std::time::{Duration, Instant};
use crate::ui::theme::{Theme, spacing, typography, radius};

/// Visual style of a toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

struct Toast<A> {
    id: u64,
    kind: ToastKind,
    message: String,
    action: Option<(String, A)>,
    shown_at: Instant,
    duration: Duration,
}

/// Most toasts visible at once; older ones are dropped.
const MAX_VISIBLE: usize = 3;

/// A stack of toasts with optional actions of type `A`.
pub struct Toasts<A> {
    toasts: Vec<Toast<A>>,
    next_id: u64,
}

impl<A: Clone> Default for Toasts<A> {
    fn default() -> Self {
        Self { toasts: Vec::new(), next_id: 0 }
    }
}

impl<A: Clone> Toasts<A> {
    /// Shows a toast. Errors and toasts with an action stay up longer.
    pub fn push(&mut self, kind: ToastKind, message: impl Into<String>, action: Option<(&str, A)>) {
        let duration = match (kind, action.is_some()) {
            (ToastKind::Error, _) => Duration::from_secs(10),
            (_, true) => Duration::from_secs(8),
            (ToastKind::Warning, false) => Duration::from_secs(6),
            _ => Duration::from_secs(4),
        };
        self.toasts.push(Toast {
            id: self.next_id,
            kind,
            message: message.into(),
            action: action.map(|(label, a)| (label.to_string(), a)),
            shown_at: Instant::now(),
            duration,
        });
        self.next_id += 1;
        if self.toasts.len() > MAX_VISIBLE {
            self.toasts.remove(0);
        }
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.push(ToastKind::Info, message, None);
    }

    pub fn error(&mut self, message: impl Into<String>, action: Option<(&str, A)>) {
        self.push(ToastKind::Error, message, action);
    }

    /// Draws the stack centred along the bottom of `area`.
    /// Returns the action of a pressed toast button.
    pub fn show(&mut self, ctx: &egui::Context, theme: &Theme, area: egui::Rect) -> Option<A> {
        // Hovered toasts don't expire, so they can be read
        let pointer = ctx.pointer_hover_pos();
        let now = Instant::now();
        self.toasts.retain(|t| now.duration_since(t.shown_at) < t.duration);
        if self.toasts.is_empty() {
            return None;
        }

        let width = (area.width() - spacing::LG * 2.0).min(480.0);
        let mut action = None;
        let mut dismissed = None;
        let mut hovered = None;

        egui::Area::new(egui::Id::new("toasts"))
            .order(egui::Order::Tooltip)
            .pivot(egui::Align2::CENTER_BOTTOM)
            .fixed_pos(egui::pos2(area.center().x, area.max.y - spacing::LG))
            .interactable(true)
            .show(ctx, |ui| {
                ui.set_width(width);
                ui.spacing_mut().item_spacing.y = spacing::SM;
                for toast in &self.toasts {
                    let (accent, icon) = match toast.kind {
                        ToastKind::Info => (theme.text_secondary, "ℹ"),
                        ToastKind::Success => (theme.success_color, "✔"),
                        ToastKind::Warning => (theme.warning_color, "⚠"),
                        ToastKind::Error => (theme.error_color, "⚠"),
                    };
                    let frame = egui::Frame::none()
                        .fill(theme.card_elevated)
                        .stroke(egui::Stroke::new(1.0, theme.border_strong))
                        .rounding(egui::Rounding::same(radius::LG))
                        .shadow(egui::epaint::Shadow {
                            offset: egui::vec2(0.0, 6.0),
                            blur: 18.0,
                            spread: 0.0,
                            color: theme.elevated_shadow(),
                        })
                        .inner_margin(egui::Margin::symmetric(spacing::LG, spacing::MD))
                        .show(ui, |ui| {
                            ui.set_width(width - spacing::LG * 2.0);
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = spacing::MD;
                                ui.label(egui::RichText::new(icon).size(typography::H3).color(accent));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let close = egui::Button::new(
                                        egui::RichText::new("🗙").size(typography::BODY_SM).color(theme.text_tertiary),
                                    )
                                    .frame(false)
                                    .min_size(egui::vec2(28.0, 28.0));
                                    if ui.add(close).on_hover_text("Dismiss").clicked() {
                                        dismissed = Some(toast.id);
                                    }
                                    if let Some((label, a)) = &toast.action {
                                        let button = egui::Button::new(
                                            egui::RichText::new(label).size(typography::BODY_SM).color(theme.accent_blue).strong(),
                                        )
                                        .fill(egui::Color32::TRANSPARENT)
                                        .stroke(egui::Stroke::new(1.0, theme.accent_blue))
                                        .rounding(egui::Rounding::same(radius::MD))
                                        .min_size(egui::vec2(0.0, 32.0));
                                        if ui.add(button).clicked() {
                                            action = Some(a.clone());
                                            dismissed = Some(toast.id);
                                        }
                                    }
                                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                        ui.add(egui::Label::new(
                                            egui::RichText::new(&toast.message)
                                                .size(typography::BODY_SM)
                                                .color(theme.text_primary),
                                        ).wrap());
                                    });
                                });
                            });
                        });
                    if pointer.is_some_and(|p| frame.response.rect.contains(p)) {
                        hovered = Some(toast.id);
                    }
                }
            });

        if let Some(id) = dismissed {
            self.toasts.retain(|t| t.id != id);
        }
        if let Some(id) = hovered {
            if let Some(t) = self.toasts.iter_mut().find(|t| t.id == id) {
                t.shown_at = Instant::now();
            }
        }

        // Wake up again when the next toast should disappear
        if let Some(next) = self.toasts.iter().map(|t| t.duration.saturating_sub(t.shown_at.elapsed())).min() {
            ctx.request_repaint_after(next);
        }

        action
    }
}
