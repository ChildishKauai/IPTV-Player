//! Top navigation bar component - Modern, clean design
//!
//! A refined navigation experience with subtle hover effects
//! and clean typography. Inspired by Apple TV+ and Spotify.

use eframe::egui;
use crate::ui::theme::{Theme, spacing, typography, radius};
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
    /// Toggle sidebar visibility (for mobile)
    ToggleSidebar,
}

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
        is_mobile: bool,
        is_touch_mode: bool,
    ) -> Option<NavAction> {
        let mut action: Option<NavAction> = None;

        // Touch-friendly sizing
        let btn_size = if is_touch_mode { 48.0 } else { 40.0 };
        let icon_size = if is_touch_mode { 22.0 } else { 18.0 };

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(spacing::XS, 0.0);

            // Menu button for mobile/touch
            if is_mobile || is_touch_mode {
                let menu_btn = egui::Button::new(
                    egui::RichText::new("☰")
                        .size(icon_size)
                        .color(theme.text_primary),
                )
                .fill(egui::Color32::TRANSPARENT)
                .min_size(egui::vec2(btn_size, btn_size));

                if ui.add(menu_btn).clicked() {
                    action = Some(NavAction::ToggleSidebar);
                }

                ui.add_space(spacing::SM);
            }

            // Logo - clean wordmark
            ui.label(
                egui::RichText::new("IPTV")
                    .size(if is_mobile { 20.0 } else { 24.0 })
                    .color(theme.text_primary)
                    .strong(),
            );

            if !is_mobile {
                ui.add_space(spacing::XXL);

                // Navigation tabs - clean, minimal
                let tabs = [
                    ("Live", ContentType::LiveTV),
                    ("Continue", ContentType::ContinueWatching),
                    ("Series", ContentType::Series),
                    ("Movies", ContentType::Movies),
                    ("My List", ContentType::Favorites),
                    ("Discover", ContentType::Discover),
                    ("Sports", ContentType::FootballFixtures),
                ];

                for (label, content_type) in tabs {
                    let is_selected = current_content == content_type;
                    if Self::nav_tab(ui, theme, is_selected, label).clicked() {
                        action = Some(NavAction::SwitchContent(content_type));
                    }
                }
            } else {
                // Mobile tabs - icon only
                ui.add_space(spacing::MD);

                let mobile_tabs = [
                    ("Live", ContentType::LiveTV),
                    ("Continue", ContentType::ContinueWatching),
                    ("Series", ContentType::Series),
                    ("Movies", ContentType::Movies),
                    ("List", ContentType::Favorites),
                    ("Hot", ContentType::Discover),
                    ("Sports", ContentType::FootballFixtures),
                ];

                for (label, content_type) in mobile_tabs {
                    let is_selected = current_content == content_type;
                    if Self::mobile_tab(ui, theme, is_selected, label, btn_size).clicked() {
                        action = Some(NavAction::SwitchContent(content_type));
                    }
                }
            }

            // Right-aligned controls
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(spacing::XS, 0.0);

                if is_mobile || is_touch_mode {
                    // Mobile: Settings only
                    let settings_btn = egui::Button::new(
                        egui::RichText::new("⚙")
                            .size(icon_size)
                            .color(theme.text_secondary),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .min_size(egui::vec2(btn_size, btn_size));

                    if ui.add(settings_btn).clicked() {
                        action = Some(NavAction::OpenPlayerSettings);
                    }
                } else {
                    // Desktop controls

                    // Sign out
                    let signout_btn = egui::Button::new(
                        egui::RichText::new("Sign out")
                            .size(typography::CAPTION)
                            .color(theme.text_tertiary),
                    )
                    .fill(egui::Color32::TRANSPARENT);

                    if ui.add(signout_btn).clicked() {
                        action = Some(NavAction::Disconnect);
                    }

                    ui.add_space(spacing::MD);

                    // Theme toggle
                    let theme_icon = if theme.dark_mode { "☀" } else { "🌙" };
                    let theme_btn = egui::Button::new(
                        egui::RichText::new(theme_icon)
                            .size(icon_size)
                            .color(theme.text_secondary),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .min_size(egui::vec2(36.0, 36.0));

                    if ui
                        .add(theme_btn)
                        .on_hover_text("Toggle theme")
                        .clicked()
                    {
                        action = Some(NavAction::ToggleTheme);
                    }

                    // Settings button
                    let settings_btn = egui::Button::new(
                        egui::RichText::new("⚙")
                            .size(icon_size)
                            .color(theme.text_secondary),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .min_size(egui::vec2(36.0, 36.0));

                    if ui
                        .add(settings_btn)
                        .on_hover_text("Player settings")
                        .clicked()
                    {
                        action = Some(NavAction::OpenPlayerSettings);
                    }

                    // EPG settings
                    let epg_btn = egui::Button::new(
                        egui::RichText::new("📺")
                            .size(icon_size)
                            .color(theme.text_secondary),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .min_size(egui::vec2(36.0, 36.0));

                    if ui
                        .add(epg_btn)
                        .on_hover_text("EPG settings")
                        .clicked()
                    {
                        action = Some(NavAction::OpenEpgSettings);
                    }

                    // Scraper settings
                    let scraper_btn = egui::Button::new(
                        egui::RichText::new("📊")
                            .size(icon_size)
                            .color(theme.text_secondary),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .min_size(egui::vec2(36.0, 36.0));

                    if ui
                        .add(scraper_btn)
                        .on_hover_text("Sports scraper")
                        .clicked()
                    {
                        action = Some(NavAction::OpenScraperSettings);
                    }

                    ui.add_space(spacing::MD);

                    // Search bar
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

                                let search_edit = egui::TextEdit::singleline(search_query)
                                    .hint_text(
                                        egui::RichText::new("Search...")
                                            .color(theme.text_muted)
                                            .size(typography::BODY_SM),
                                    )
                                    .desired_width(200.0)
                                    .font(egui::FontId::proportional(typography::BODY_SM))
                                    .frame(false);

                                if ui.add(search_edit).changed() {
                                    action = Some(NavAction::SearchChanged);
                                }
                            });
                        });
                }
            });
        });

        // Mobile search bar (below nav)
        if is_mobile || is_touch_mode {
            ui.add_space(spacing::MD);

            egui::Frame::none()
                .fill(theme.inactive_bg())
                .rounding(egui::Rounding::same(radius::MD))
                .inner_margin(egui::Margin::symmetric(
                    spacing::LG,
                    if is_touch_mode { spacing::MD } else { spacing::SM },
                ))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("🔍")
                                .size(if is_touch_mode { 18.0 } else { 16.0 })
                                .color(theme.text_muted),
                        );
                        ui.add_space(spacing::SM);

                        let search_edit = egui::TextEdit::singleline(search_query)
                            .hint_text(
                                egui::RichText::new("Search channels, movies, series...")
                                    .size(if is_touch_mode {
                                        typography::BODY
                                    } else {
                                        typography::BODY_SM
                                    })
                                    .color(theme.text_muted),
                            )
                            .desired_width(ui.available_width() - 40.0)
                            .font(egui::FontId::proportional(if is_touch_mode {
                                typography::BODY
                            } else {
                                typography::BODY_SM
                            }))
                            .frame(false);

                        if ui.add(search_edit).changed() {
                            action = Some(NavAction::SearchChanged);
                        }
                    });
                });
        }

        action
    }

    /// Creates a navigation tab (desktop)
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
                rect.max.y + 2.0,
                egui::Stroke::new(2.0, color),
            );
        }

        response
    }

    /// Creates a mobile tab button
    fn mobile_tab(
        ui: &mut egui::Ui,
        theme: &Theme,
        is_selected: bool,
        label: &str,
        btn_size: f32,
    ) -> egui::Response {
        let text_color = if is_selected {
            theme.text_primary
        } else {
            theme.text_tertiary
        };

        // Use first 2 characters as abbreviation
        let abbrev: String = label.chars().take(2).collect();

        let response = ui.add(
            egui::Button::new(
                egui::RichText::new(abbrev)
                    .size(typography::CAPTION)
                    .color(text_color),
            )
            .fill(egui::Color32::TRANSPARENT)
            .min_size(egui::vec2(btn_size, btn_size)),
        );

        // Underline for selected
        if is_selected {
            let rect = response.rect;
            ui.painter().hline(
                rect.min.x + 8.0..=rect.max.x - 8.0,
                rect.max.y + 1.0,
                egui::Stroke::new(2.0, theme.accent_blue),
            );
        }

        response
    }
}
