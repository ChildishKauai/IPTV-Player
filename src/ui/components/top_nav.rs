//! Top navigation bar component (Netflix-style).

use eframe::egui;
use crate::ui::theme::Theme;
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
    /// Open scraper settings
    OpenScraperSettings,
    /// Toggle sidebar visibility (for mobile)
    ToggleSidebar,
}

/// Top navigation bar component (Netflix-style).
pub struct TopNavigation;

impl TopNavigation {
    /// Renders the Netflix-style top navigation bar.
    /// Returns any action that was triggered.
    pub fn show(
        ui: &mut egui::Ui,
        theme: &Theme,
        current_content: ContentType,
        search_query: &mut String,
        is_mobile: bool,
        is_touch_mode: bool, // Steam Deck or tablet
    ) -> Option<NavAction> {
        let mut action: Option<NavAction> = None;
        
        // Touch-friendly button size
        let touch_btn_size = if is_touch_mode { 56.0 } else { 44.0 };
        let icon_size = if is_touch_mode { 24.0 } else { 20.0 };
        
        ui.horizontal(|ui| {
            // Menu button for mobile/touch
            if is_mobile || is_touch_mode {
                let menu_btn = egui::Button::new(
                    egui::RichText::new("☰").size(icon_size).color(theme.text_primary)
                )
                .fill(egui::Color32::TRANSPARENT)
                .min_size(egui::vec2(touch_btn_size, touch_btn_size));
                
                if ui.add(menu_btn).clicked() {
                    action = Some(NavAction::ToggleSidebar);
                }
                
                ui.add_space(4.0);
            }
            
            // Netflix-style logo
            ui.label(egui::RichText::new("IPTV")
                .size(if is_mobile { 22.0 } else { 26.0 })
                .color(theme.accent_blue)
                .strong());
            
            if !is_mobile {
                ui.add_space(30.0);
                
                // Netflix-style horizontal tabs
                if Self::nav_link(ui, theme, matches!(current_content, ContentType::LiveTV), "Live TV").clicked() {
                    action = Some(NavAction::SwitchContent(ContentType::LiveTV));
                }
                
                if Self::nav_link(ui, theme, matches!(current_content, ContentType::ContinueWatching), "Continue").clicked() {
                    action = Some(NavAction::SwitchContent(ContentType::ContinueWatching));
                }
                
                if Self::nav_link(ui, theme, matches!(current_content, ContentType::Series), "Series").clicked() {
                    action = Some(NavAction::SwitchContent(ContentType::Series));
                }
                
                if Self::nav_link(ui, theme, matches!(current_content, ContentType::Movies), "Movies").clicked() {
                    action = Some(NavAction::SwitchContent(ContentType::Movies));
                }
                
                if Self::nav_link(ui, theme, matches!(current_content, ContentType::Favorites), "My List").clicked() {
                    action = Some(NavAction::SwitchContent(ContentType::Favorites));
                }
                
                if Self::nav_link(ui, theme, matches!(current_content, ContentType::Discover), "Discover").clicked() {
                    action = Some(NavAction::SwitchContent(ContentType::Discover));
                }
                
                if Self::nav_link(ui, theme, matches!(current_content, ContentType::FootballFixtures), "Sports").clicked() {
                    action = Some(NavAction::SwitchContent(ContentType::FootballFixtures));
                }
            } else {
                // Mobile/touch tabs - larger touch targets
                let tab_btn_size = if is_touch_mode { 52.0 } else { 44.0 };
                let tab_icon_size = if is_touch_mode { 20.0 } else { 16.0 };
                ui.add_space(8.0);
                if Self::mobile_tab_button(ui, theme, matches!(current_content, ContentType::LiveTV), "📡", tab_btn_size, tab_icon_size).clicked() {
                    action = Some(NavAction::SwitchContent(ContentType::LiveTV));
                }
                if Self::mobile_tab_button(ui, theme, matches!(current_content, ContentType::ContinueWatching), "▶️", tab_btn_size, tab_icon_size).clicked() {
                    action = Some(NavAction::SwitchContent(ContentType::ContinueWatching));
                }
                if Self::mobile_tab_button(ui, theme, matches!(current_content, ContentType::Series), "🎬", tab_btn_size, tab_icon_size).clicked() {
                    action = Some(NavAction::SwitchContent(ContentType::Series));
                }
                if Self::mobile_tab_button(ui, theme, matches!(current_content, ContentType::Movies), "🎥", tab_btn_size, tab_icon_size).clicked() {
                    action = Some(NavAction::SwitchContent(ContentType::Movies));
                }
                if Self::mobile_tab_button(ui, theme, matches!(current_content, ContentType::Favorites), "⭐", tab_btn_size, tab_icon_size).clicked() {
                    action = Some(NavAction::SwitchContent(ContentType::Favorites));
                }
                if Self::mobile_tab_button(ui, theme, matches!(current_content, ContentType::Discover), "🔥", tab_btn_size, tab_icon_size).clicked() {
                    action = Some(NavAction::SwitchContent(ContentType::Discover));
                }
                if Self::mobile_tab_button(ui, theme, matches!(current_content, ContentType::FootballFixtures), "⚽", tab_btn_size, tab_icon_size).clicked() {
                    action = Some(NavAction::SwitchContent(ContentType::FootballFixtures));
                }
            }
            
            // Right-aligned controls
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if is_mobile || is_touch_mode {
                    // Touch-friendly controls
                    let ctrl_icon_size = if is_touch_mode { 22.0 } else { 18.0 };
                    let settings_btn = egui::Button::new(egui::RichText::new("⚙").size(ctrl_icon_size).color(theme.text_primary))
                        .fill(egui::Color32::TRANSPARENT)
                        .min_size(egui::vec2(touch_btn_size, touch_btn_size));
                    if ui.add(settings_btn).clicked() {
                        action = Some(NavAction::OpenPlayerSettings);
                    }
                } else {
                    // Desktop controls
                    // Search bar with Netflix styling
                    egui::Frame::none()
                        .fill(theme.inactive_bg())
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🔍").size(14.0).color(theme.text_secondary));
                                let search_edit = egui::TextEdit::singleline(search_query)
                                    .hint_text(egui::RichText::new("Search...").color(theme.text_secondary))
                                    .desired_width(180.0)
                                    .frame(false);
                                if ui.add(search_edit).changed() {
                                    action = Some(NavAction::SearchChanged);
                                }
                            });
                        });
                    
                    ui.add_space(16.0);
                    
                    // Scraper settings button
                    let scraper_btn = egui::Button::new(egui::RichText::new("📊").size(18.0).color(theme.text_primary))
                        .fill(egui::Color32::TRANSPARENT);
                    
                    if ui.add(scraper_btn).on_hover_text("Football Fixtures Scraper").clicked() {
                        action = Some(NavAction::OpenScraperSettings);
                    }
                    
                    ui.add_space(8.0);
                    
                    let settings_btn = egui::Button::new(egui::RichText::new("⚙").size(18.0).color(theme.text_primary))
                        .fill(egui::Color32::TRANSPARENT);
                    
                    if ui.add(settings_btn).on_hover_text("Player Settings").clicked() {
                        action = Some(NavAction::OpenPlayerSettings);
                    }
                    
                    ui.add_space(8.0);
                    
                    let theme_icon = if theme.dark_mode { "☀" } else { "🌙" };
                    let theme_btn = egui::Button::new(egui::RichText::new(theme_icon).size(18.0).color(theme.text_primary))
                        .fill(egui::Color32::TRANSPARENT);
                    
                    if ui.add(theme_btn).on_hover_text("Toggle Theme").clicked() {
                        action = Some(NavAction::ToggleTheme);
                    }
                    
                    ui.add_space(8.0);
                    
                    let disconnect_btn = egui::Button::new(
                        egui::RichText::new("Sign Out").size(12.0).color(theme.text_secondary)
                    )
                    .fill(egui::Color32::TRANSPARENT);
                    
                    if ui.add(disconnect_btn).clicked() {
                        action = Some(NavAction::Disconnect);
                    }
                }
            });
        });
        
        // Mobile/Touch search bar (below nav) - shown for mobile OR touch mode (Steam Deck)
        if is_mobile || is_touch_mode {
            ui.add_space(if is_touch_mode { 12.0 } else { 8.0 });
            let inner_margin = if is_touch_mode { 
                egui::Margin::symmetric(14.0, 12.0) 
            } else { 
                egui::Margin::symmetric(10.0, 8.0) 
            };
            let icon_size = if is_touch_mode { 20.0 } else { 16.0 };
            let font_size = if is_touch_mode { 18.0 } else { 14.0 };
            
            egui::Frame::none()
                .fill(theme.inactive_bg())
                .rounding(egui::Rounding::same(6.0))
                .inner_margin(inner_margin)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🔍").size(icon_size).color(theme.text_secondary));
                        ui.add_space(8.0);
                        let search_edit = egui::TextEdit::singleline(search_query)
                            .hint_text(egui::RichText::new("Search channels, movies, series...").size(font_size))
                            .desired_width(ui.available_width() - 40.0)
                            .font(egui::TextStyle::Body)
                            .frame(false);
                        if ui.add(search_edit).changed() {
                            action = Some(NavAction::SearchChanged);
                        }
                    });
                });
        }
        
        action
    }
    
    /// Creates a Netflix-style navigation link (text that lights up on hover/selected).
    fn nav_link(ui: &mut egui::Ui, theme: &Theme, is_selected: bool, text: &str) -> egui::Response {
        let color = if is_selected {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        
        let response = ui.add(
            egui::Button::new(
                egui::RichText::new(text)
                    .size(14.0)
                    .color(color)
            )
            .fill(egui::Color32::TRANSPARENT)
            .frame(false)
        );
        
        // Underline on hover
        if response.hovered() && !is_selected {
            let rect = response.rect;
            ui.painter().hline(
                rect.min.x..=rect.max.x,
                rect.max.y + 2.0,
                egui::Stroke::new(2.0, theme.accent_blue),
            );
        }
        
        // Underline for selected
        if is_selected {
            let rect = response.rect;
            ui.painter().hline(
                rect.min.x..=rect.max.x,
                rect.max.y + 2.0,
                egui::Stroke::new(2.0, theme.accent_blue),
            );
        }
        
        response
    }
    
    /// Creates a touch-friendly icon tab button for mobile/Steam Deck.
    fn mobile_tab_button(ui: &mut egui::Ui, theme: &Theme, is_selected: bool, icon: &str, btn_size: f32, icon_size: f32) -> egui::Response {
        let fg = if is_selected {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        
        let response = ui.add(
            egui::Button::new(egui::RichText::new(icon).size(icon_size).color(fg))
                .fill(egui::Color32::TRANSPARENT)
                .min_size(egui::vec2(btn_size, btn_size))
        );
        
        // Underline for selected
        if is_selected {
            let rect = response.rect;
            ui.painter().hline(
                rect.min.x + 4.0..=rect.max.x - 4.0,
                rect.max.y + 1.0,
                egui::Stroke::new(3.0, theme.accent_blue),
            );
        }
        
        response
    }
}
