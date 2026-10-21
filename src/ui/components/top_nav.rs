//! Top navigation bar component - Modern, responsive design
//!
//! Adapts to the window size:
//! - Wide windows: logo, inline tabs and search on a single row
//! - Medium windows: search row plus horizontally scrollable tab chips
//! - Phones: compact header with search, tabs move to a bottom bar
//!
//! Secondary actions (settings, EPG, theme, sign out) live in one overflow
//! menu so they are reachable on every screen size.

use eframe::egui;
use crate::ui::theme::{Theme, spacing, typography, radius};
use crate::ui::layout::Layout;
use crate::ui::messages::ContentType;

/// Actions that can be triggered from the navigation bar.
#[derive(Debug, Clone, PartialEq)]
pub enum NavAction {
    /// Switch to a different content type
    SwitchContent(ContentType),
    /// Search query changed
    SearchChanged,
    /// Toggle dark/light mode
    ToggleTheme,
    /// Disconnect from server
    Disconnect,
    /// Open player settings
    OpenPlayerSettings,
    /// Open EPG settings
    OpenEpgSettings,
    /// Open scraper settings
    OpenScraperSettings,
    /// Toggle the category sidebar / drawer
    ToggleSidebar,
}

/// A navigation destination: icon, full label, short label, content type.
struct Tab(&'static str, &'static str, &'static str, ContentType);

const TABS: [Tab; 7] = [
    Tab("📺", "Live", "Live", ContentType::LiveTV),
    Tab("▶", "Continue", "Resume", ContentType::ContinueWatching),
    Tab("🎞", "Series", "Series", ContentType::Series),
    Tab("🎬", "Movies", "Movies", ContentType::Movies),
    Tab("★", "My List", "My List", ContentType::Favorites),
    Tab("🔥", "Discover", "Discover", ContentType::Discover),
    Tab("⚽", "Sports", "Sports", ContentType::FootballFixtures),
];

/// Minimum header width needed to fit the tabs inline next to the logo.
const INLINE_TABS_MIN_WIDTH: f32 = 960.0;

/// Top navigation bar component - Modern design
pub struct TopNavigation;

impl TopNavigation {
    /// Renders the navigation bar.
    /// Returns any action that was triggered.
    pub fn show(
        ui: &mut egui::Ui,
        theme: &Theme,
        current_content: ContentType,
        search_query: &mut String,
        layout: &Layout,
        show_menu_button: bool,
    ) -> Option<NavAction> {
        let mut action: Option<NavAction> = None;

        let btn_size = layout.touch_target();
        let icon_size = if layout.touch { 20.0 } else { 18.0 };
        let inline_tabs = ui.available_width() >= INLINE_TABS_MIN_WIDTH;

        ui.horizontal(|ui| {
            ui.set_min_height(btn_size);
            ui.spacing_mut().item_spacing = egui::vec2(spacing::XS, 0.0);

            if show_menu_button {
                if Self::icon_button(ui, "☰", icon_size, btn_size, theme.text_primary)
                    .on_hover_text("Categories")
                    .clicked()
                {
                    action = Some(NavAction::ToggleSidebar);
                }
                ui.add_space(spacing::XS);
            }

            Self::logo(ui, theme, layout);

            if inline_tabs {
                ui.add_space(spacing::XL);
                for tab in &TABS {
                    if Self::nav_tab(ui, theme, current_content == tab.3, tab.1).clicked() {
                        action = Some(NavAction::SwitchContent(tab.3));
                    }
                    ui.add_space(spacing::SM);
                }
            }

            // Right-aligned controls
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(spacing::XS, 0.0);

                if let Some(a) = Self::overflow_menu(ui, theme, icon_size, btn_size) {
                    action = Some(a);
                }

                if !layout.is_compact() {
                    let theme_icon = if theme.dark_mode { "☀" } else { "🌙" };
                    if Self::icon_button(ui, theme_icon, icon_size, btn_size, theme.text_secondary)
                        .on_hover_text("Toggle theme")
                        .clicked()
                    {
                        action = Some(NavAction::ToggleTheme);
                    }
                }

                if inline_tabs {
                    ui.add_space(spacing::SM);
                    let width = (ui.available_width() - spacing::LG).clamp(140.0, 280.0);
                    if Self::search_field(ui, theme, search_query, width, layout, "Search…") {
                        action = Some(NavAction::SearchChanged);
                    }
                }
            });
        });

        if !inline_tabs {
            ui.add_space(spacing::MD);
            let width = ui.available_width();
            if Self::search_field(ui, theme, search_query, width, layout, "Search channels, movies, series…") {
                action = Some(NavAction::SearchChanged);
            }

            // Phones get a bottom tab bar instead of chips
            if !layout.is_compact() {
                ui.add_space(spacing::MD);
                if let Some(a) = Self::tab_chips(ui, theme, current_content, layout) {
                    action = Some(a);
                }
            }
        }

        action
    }

    /// Renders the phone bottom tab bar. Returns the selected destination.
    pub fn bottom_bar(
        ui: &mut egui::Ui,
        theme: &Theme,
        current_content: ContentType,
    ) -> Option<NavAction> {
        let mut action = None;
        let height = 58.0;
        let item_width = ui.available_width() / TABS.len() as f32;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            for tab in &TABS {
                let is_selected = current_content == tab.3;
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(item_width, height), egui::Sense::click());

                let color = if is_selected {
                    theme.accent_blue
                } else if response.hovered() {
                    theme.text_primary
                } else {
                    theme.text_tertiary
                };

                // Selected indicator pill behind the icon
                if is_selected {
                    let pill = egui::Rect::from_center_size(
                        egui::pos2(rect.center().x, rect.min.y + 19.0),
                        egui::vec2((item_width - 12.0).min(52.0), 28.0),
                    );
                    ui.painter().rect_filled(pill, radius::FULL, theme.accent_blue.linear_multiply(0.15));
                }

                ui.painter().text(
                    egui::pos2(rect.center().x, rect.min.y + 19.0),
                    egui::Align2::CENTER_CENTER,
                    tab.0,
                    egui::FontId::proportional(18.0),
                    color,
                );
                let label = super::single_line(
                    ui,
                    tab.2,
                    egui::FontId::proportional(typography::LABEL - 1.0),
                    color,
                    item_width - 4.0,
                );
                ui.painter().galley(
                    egui::pos2(rect.center().x - label.size().x / 2.0, rect.min.y + 37.0),
                    label,
                    color,
                );

                if response.on_hover_text(tab.1).clicked() {
                    action = Some(NavAction::SwitchContent(tab.3));
                }
            }
        });

        action
    }

    /// Brand wordmark
    fn logo(ui: &mut egui::Ui, theme: &Theme, layout: &Layout) {
        let size = if layout.is_compact() { 20.0 } else { 24.0 };
        ui.label(
            egui::RichText::new("IPTV")
                .size(size)
                .color(theme.text_primary)
                .strong(),
        );
        // Accent dot
        let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, size), egui::Sense::hover());
        ui.painter().circle_filled(
            egui::pos2(rect.min.x + 3.0, rect.max.y - size * 0.3),
            3.0,
            theme.accent_blue,
        );
    }

    /// Settings / account overflow menu
    fn overflow_menu(
        ui: &mut egui::Ui,
        theme: &Theme,
        icon_size: f32,
        btn_size: f32,
    ) -> Option<NavAction> {
        let mut action = None;
        let button = egui::Button::new(
            egui::RichText::new("⚙").size(icon_size).color(theme.text_secondary),
        )
        .fill(egui::Color32::TRANSPARENT)
        .min_size(egui::vec2(btn_size, btn_size));

        let response = ui.add(button).on_hover_text("Settings");
        let popup_id = ui.make_persistent_id("nav_overflow_menu");
        if response.clicked() {
            ui.memory_mut(|m| m.toggle_popup(popup_id));
        }

        if !ui.memory(|m| m.is_popup_open(popup_id)) {
            return None;
        }

        // Right-aligned under the button so the menu never runs off-screen
        let area = egui::Area::new(popup_id)
            .order(egui::Order::Foreground)
            .pivot(egui::Align2::RIGHT_TOP)
            .fixed_pos(response.rect.right_bottom() + egui::vec2(0.0, spacing::XS))
            .constrain(true)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_width(220.0);
                    ui.spacing_mut().item_spacing.y = spacing::XS;
                    let theme_label = if theme.dark_mode { "☀  Light mode" } else { "🌙  Dark mode" };
                    let items = [
                        ("🎬  Player settings", NavAction::OpenPlayerSettings),
                        ("📺  TV guide (EPG)", NavAction::OpenEpgSettings),
                        ("⚽  Sports fixtures", NavAction::OpenScraperSettings),
                        (theme_label, NavAction::ToggleTheme),
                    ];
                    for (label, item_action) in items {
                        if Self::menu_item(ui, label, theme.text_primary).clicked() {
                            action = Some(item_action);
                        }
                    }
                    ui.separator();
                    if Self::menu_item(ui, "🚪  Sign out", theme.error_color).clicked() {
                        action = Some(NavAction::Disconnect);
                    }
                });
            });

        // Close after choosing an item or clicking anywhere else
        if action.is_some() || (area.response.clicked_elsewhere() && !response.clicked()) {
            ui.memory_mut(|m| m.close_popup());
        }

        action
    }

    fn menu_item(ui: &mut egui::Ui, label: &str, color: egui::Color32) -> egui::Response {
        ui.add(
            egui::Button::new(egui::RichText::new(label).size(typography::BODY_SM).color(color))
                .fill(egui::Color32::TRANSPARENT)
                .frame(false)
                .min_size(egui::vec2(ui.available_width(), 40.0)),
        )
    }

    /// Search box with a clear button. Returns true when the query changed.
    fn search_field(
        ui: &mut egui::Ui,
        theme: &Theme,
        search_query: &mut String,
        width: f32,
        layout: &Layout,
        hint: &str,
    ) -> bool {
        let mut changed = false;
        let font_size = if layout.touch { typography::BODY } else { typography::BODY_SM };
        let v_margin = if layout.touch { spacing::MD } else { spacing::SM };

        egui::Frame::none()
            .fill(theme.inactive_bg())
            .rounding(egui::Rounding::same(radius::FULL))
            .inner_margin(egui::Margin::symmetric(spacing::LG, v_margin))
            .show(ui, |ui| {
                ui.set_width(width - spacing::LG * 2.0);
                // Explicit direction (the parent may be right-to-left) in a
                // single bounded row
                let row = egui::vec2(ui.available_width(), font_size + 8.0);
                ui.allocate_ui_with_layout(row, egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = spacing::SM;
                    ui.label(egui::RichText::new("🔍").size(font_size).color(theme.text_muted));

                    let clear_width = if search_query.is_empty() { 0.0 } else { 28.0 };
                    let edit = egui::TextEdit::singleline(search_query)
                        .hint_text(egui::RichText::new(hint).size(font_size).color(theme.text_muted))
                        .desired_width((ui.available_width() - clear_width).max(40.0))
                        .font(egui::FontId::proportional(font_size))
                        .frame(false);
                    changed |= ui.add(edit).changed();

                    if !search_query.is_empty() {
                        let clear = egui::Button::new(
                            egui::RichText::new("🗙").size(font_size - 2.0).color(theme.text_tertiary),
                        )
                        .frame(false);
                        if ui.add(clear).on_hover_text("Clear search").clicked() {
                            search_query.clear();
                            changed = true;
                        }
                    }
                });
            });

        changed
    }

    /// Horizontally scrollable pill tabs for medium-width windows
    fn tab_chips(
        ui: &mut egui::Ui,
        theme: &Theme,
        current_content: ContentType,
        layout: &Layout,
    ) -> Option<NavAction> {
        let mut action = None;
        let height = if layout.touch { 40.0 } else { 34.0 };

        egui::ScrollArea::horizontal()
            .id_salt("nav_chips")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = spacing::SM;
                    for tab in &TABS {
                        let is_selected = current_content == tab.3;
                        let (fill, fg) = if is_selected {
                            (theme.accent_blue, egui::Color32::WHITE)
                        } else {
                            (theme.inactive_bg(), theme.text_secondary)
                        };
                        let chip = egui::Button::new(
                            egui::RichText::new(format!("{}  {}", tab.0, tab.1))
                                .size(typography::BODY_SM)
                                .color(fg),
                        )
                        .fill(fill)
                        .stroke(egui::Stroke::NONE)
                        .rounding(egui::Rounding::same(radius::FULL))
                        .min_size(egui::vec2(0.0, height));
                        if ui.add(chip).clicked() {
                            action = Some(NavAction::SwitchContent(tab.3));
                        }
                    }
                });
            });

        action
    }

    fn icon_button(
        ui: &mut egui::Ui,
        icon: &str,
        icon_size: f32,
        btn_size: f32,
        color: egui::Color32,
    ) -> egui::Response {
        ui.add(
            egui::Button::new(egui::RichText::new(icon).size(icon_size).color(color))
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::NONE)
                .rounding(egui::Rounding::same(radius::FULL))
                .min_size(egui::vec2(btn_size, btn_size)),
        )
    }

    /// Creates a navigation tab (wide layouts)
    fn nav_tab(
        ui: &mut egui::Ui,
        theme: &Theme,
        is_selected: bool,
        text: &str,
    ) -> egui::Response {
        let text_color = if is_selected {
            theme.text_primary
        } else {
            theme.text_secondary
        };

        let response = ui.add(
            egui::Button::new(
                egui::RichText::new(text)
                    .size(typography::BODY_SM)
                    .color(text_color),
            )
            .fill(egui::Color32::TRANSPARENT)
            .frame(false),
        );

        // Underline indicator
        if is_selected || response.hovered() {
            let rect = response.rect;
            let color = if is_selected {
                theme.accent_blue
            } else {
                theme.text_muted
            };
            ui.painter().hline(
                rect.min.x..=rect.max.x,
                rect.max.y + 4.0,
                egui::Stroke::new(2.0, color),
            );
        }

        response
    }
}
