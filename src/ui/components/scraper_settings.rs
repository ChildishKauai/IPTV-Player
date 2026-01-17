//! Scraper settings dialog component for triggering fixture scraping.

use eframe::egui;
use crate::ui::theme::Theme;
use crate::api::ScrapingStatus;

/// Actions that can be triggered from the scraper settings dialog.
#[derive(Debug, Clone)]
pub enum ScraperAction {
    /// Trigger a scraping operation
    TriggerScrape,
    /// Close the dialog
    Close,
}

/// Scraper settings dialog component.
pub struct ScraperSettingsDialog;

impl ScraperSettingsDialog {
    /// Renders the scraper settings dialog.
    /// Returns any action that was triggered.
    pub fn show(
        ui: &mut egui::Ui,
        theme: &Theme,
        scraper_available: bool,
        scraping_status: &ScrapingStatus,
    ) -> Option<ScraperAction> {
        let mut action: Option<ScraperAction> = None;

        ui.heading("📊 Football Fixtures Scraper");
        ui.separator();

        if !scraper_available {
            ui.label(egui::RichText::new("⚠️ Soccer Scraper not found")
                .color(theme.warning_color));
            ui.label("Ensure Soccer-Scraper-main directory exists in the app folder");
        } else {
            match scraping_status {
                ScrapingStatus::Idle => {
                    ui.label("✅ Scraper ready");
                    ui.label("Click below to download today's fixtures from LiveSoccerTV");
                    
                    if ui.button("🔄 Scrape Fixtures Now").clicked() {
                        action = Some(ScraperAction::TriggerScrape);
                    }
                    ui.label("⏱️ Takes approximately 2 minutes");
                }
                ScrapingStatus::Scraping => {
                    ui.label(egui::RichText::new("⏳ Scraping in progress...")
                        .color(theme.accent_blue));
                    ui.label("Opening Chrome browser to bypass Cloudflare...");
                    ui.label("Please wait, this may take 2 minutes");
                }
                ScrapingStatus::Success(msg) => {
                    ui.label(egui::RichText::new("✅ Success!").color(theme.success_color));
                    ui.label(msg);
                    ui.label("Refresh the app to see updated fixtures");
                    
                    if ui.button("🔄 Scrape Again").clicked() {
                        action = Some(ScraperAction::TriggerScrape);
                    }
                }
                ScrapingStatus::Error(err) => {
                    ui.label(egui::RichText::new("❌ Error").color(theme.error_color));
                    ui.label(err);
                    ui.label("Ensure Python is installed and Soccer-Scraper-main has requirements installed");
                    
                    if ui.button("🔄 Try Again").clicked() {
                        action = Some(ScraperAction::TriggerScrape);
                    }
                }
            }
        }

        ui.separator();
        
        if ui.button("Close").clicked() {
            action = Some(ScraperAction::Close);
        }

        action
    }
}
