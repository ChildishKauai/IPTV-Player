//! Episode dialog component for viewing series episodes.

use eframe::egui;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use crate::api::XtreamClient;
use crate::models::PlayerSettings;

/// Actions that can be triggered from the episode dialog.
#[derive(Debug, Clone)]
pub enum EpisodeAction {
    /// Play an episode
    PlayEpisode {
        episode_id: String,
        series_name: String,
        season: i32,
        episode: i32,
        title: String,
        container: String,
    },
    /// Close the dialog
    Close,
}

/// Cached episode data for fast rendering.
#[derive(Clone)]
struct EpisodeData {
    id: String,
    num: i64,
    title: String,
    container: String,
    season: String,
}

/// Cached series data.
struct SeriesData {
    name: String,
    plot: String,
    seasons: Vec<(String, Vec<EpisodeData>)>,
}

/// Loading state for the dialog.
enum LoadingState {
    Loading,
    Loaded(SeriesData),
    Error(String),
}

/// Episode dialog state - stored in the app to persist between frames.
pub struct EpisodeDialogState {
    #[allow(dead_code)]
    series_id: i32,
    state: LoadingState,
    rx: Option<Receiver<Result<SeriesData, String>>>,
    selected_season: Option<String>,
    #[allow(dead_code)]
    server_url: String,
    #[allow(dead_code)]
    username: String,
    #[allow(dead_code)]
    password: String,
}

impl EpisodeDialogState {
    /// Create a new episode dialog state and start loading data.
    pub fn new(
        series_id: i32,
        server_url: String,
        username: String,
        password: String,
    ) -> Self {
        let (tx, rx) = channel();
        
        // Clone for background thread
        let srv = server_url.clone();
        let usr = username.clone();
        let pwd = password.clone();
        
        // Fetch data in background thread
        thread::spawn(move || {
            let client = XtreamClient::new(srv, usr, pwd);
            
            match client.get_series_info(series_id) {
                Ok(info) => {
                    // Extract series info
                    let mut name = String::new();
                    let mut plot = String::new();
                    
                    if let Some(info_obj) = info.get("info") {
                        name = info_obj.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown Series")
                            .to_string();
                        plot = info_obj.get("plot")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                    }
                    
                    // Extract episodes by season
                    let mut seasons: Vec<(String, Vec<EpisodeData>)> = Vec::new();
                    
                    if let Some(episodes) = info.get("episodes").and_then(|v| v.as_object()) {
                        let mut season_keys: Vec<_> = episodes.keys().collect();
                        season_keys.sort_by(|a, b| {
                            a.parse::<i32>().unwrap_or(0).cmp(&b.parse::<i32>().unwrap_or(0))
                        });
                        
                        for season_key in season_keys {
                            if let Some(season_episodes) = episodes.get(season_key).and_then(|v| v.as_array()) {
                                let eps: Vec<EpisodeData> = season_episodes.iter().map(|ep| {
                                    let id = if let Some(id_str) = ep.get("id").and_then(|v| v.as_str()) {
                                        id_str.to_string()
                                    } else if let Some(id_num) = ep.get("id").and_then(|v| v.as_i64()) {
                                        id_num.to_string()
                                    } else {
                                        String::new()
                                    };
                                    
                                    EpisodeData {
                                        id,
                                        num: ep.get("episode_num").and_then(|v| v.as_i64()).unwrap_or(0),
                                        title: ep.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                                        container: ep.get("container_extension").and_then(|v| v.as_str()).unwrap_or("mp4").to_string(),
                                        season: season_key.clone(),
                                    }
                                }).collect();
                                
                                seasons.push((season_key.clone(), eps));
                            }
                        }
                    }
                    
                    let _ = tx.send(Ok(SeriesData { name, plot, seasons }));
                }
                Err(e) => {
                    let _ = tx.send(Err(format!("Failed to load: {}", e)));
                }
            }
        });
        
        Self {
            series_id,
            state: LoadingState::Loading,
            rx: Some(rx),
            selected_season: None,
            server_url,
            username,
            password,
        }
    }
    
    /// Check if data has been received from background thread.
    fn check_for_data(&mut self) {
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(data) => {
                        // Auto-select first season
                        if self.selected_season.is_none() && !data.seasons.is_empty() {
                            self.selected_season = Some(data.seasons[0].0.clone());
                        }
                        self.state = LoadingState::Loaded(data);
                    }
                    Err(e) => {
                        self.state = LoadingState::Error(e);
                    }
                }
                self.rx = None; // Done receiving
            }
        }
    }
}

/// Episode dialog component (Netflix-style).
pub struct EpisodeDialog;

impl EpisodeDialog {
    /// Renders the Netflix-style episode dialog using cached state.
    /// Returns true if the dialog should be closed.
    pub fn show(
        ctx: &egui::Context,
        state: &mut EpisodeDialogState,
        _player_settings: &PlayerSettings,
    ) -> Option<EpisodeAction> {
        // Check for background data
        state.check_for_data();
        
        let mut action: Option<EpisodeAction> = None;
        
        egui::Window::new("")
            .resizable(true)
            .collapsible(false)
            .title_bar(false)
            .default_width(800.0)
            .default_height(550.0)
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(24, 24, 24))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(0.0)))
            .show(ctx, |ui| {
                // Netflix-style dark dialog
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(24, 24, 24))
                    .inner_margin(egui::Margin::same(24.0))
                    .show(ui, |ui| {
                        match &state.state {
                            LoadingState::Loading => {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label(egui::RichText::new("Loading episodes...")
                                        .color(egui::Color32::from_rgb(180, 180, 180)));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add(egui::Button::new(
                                            egui::RichText::new("✕").size(18.0).color(egui::Color32::WHITE)
                                        ).fill(egui::Color32::TRANSPARENT)).clicked() {
                                            action = Some(EpisodeAction::Close);
                                        }
                                    });
                                });
                            }
                            LoadingState::Error(e) => {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("❌ {}", e))
                                        .color(egui::Color32::from_rgb(229, 9, 20)));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add(egui::Button::new(
                                            egui::RichText::new("✕").size(18.0).color(egui::Color32::WHITE)
                                        ).fill(egui::Color32::TRANSPARENT)).clicked() {
                                            action = Some(EpisodeAction::Close);
                                        }
                                    });
                                });
                            }
                            LoadingState::Loaded(data) => {
                                // Header with close button
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&data.name)
                                        .size(24.0)
                                        .color(egui::Color32::WHITE)
                                        .strong());
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add(egui::Button::new(
                                            egui::RichText::new("✕").size(18.0).color(egui::Color32::WHITE)
                                        ).fill(egui::Color32::TRANSPARENT)).clicked() {
                                            action = Some(EpisodeAction::Close);
                                        }
                                    });
                                });
                                
                                // Plot (truncated)
                                if !data.plot.is_empty() {
                                    ui.add_space(8.0);
                                    let plot_display = if data.plot.chars().count() > 250 {
                                        let truncated: String = data.plot.chars().take(250).collect();
                                        format!("{}...", truncated)
                                    } else {
                                        data.plot.clone()
                                    };
                                    ui.label(egui::RichText::new(plot_display)
                                        .size(13.0)
                                        .color(egui::Color32::from_rgb(160, 160, 160)));
                                }
                                
                                ui.add_space(20.0);
                                
                                // Season tabs (Netflix-style pills)
                                ui.horizontal_wrapped(|ui| {
                                    for (season_key, _) in &data.seasons {
                                        let is_selected = state.selected_season.as_ref() == Some(season_key);
                                        let btn_text = format!("Season {}", season_key);
                                        
                                        let (bg, fg) = if is_selected {
                                            (egui::Color32::WHITE, egui::Color32::BLACK)
                                        } else {
                                            (egui::Color32::from_rgb(50, 50, 50), egui::Color32::WHITE)
                                        };
                                        
                                        let btn = egui::Button::new(
                                            egui::RichText::new(&btn_text)
                                                .size(13.0)
                                                .color(fg)
                                        )
                                        .fill(bg)
                                        .rounding(egui::Rounding::same(16.0))
                                        .min_size(egui::vec2(90.0, 32.0));
                                        
                                        if ui.add(btn).clicked() {
                                            state.selected_season = Some(season_key.clone());
                                        }
                                    }
                                });
                                
                                ui.add_space(16.0);
                                
                                // Episodes list
                                if let Some(selected) = &state.selected_season {
                                    if let Some((_, episodes)) = data.seasons.iter().find(|(s, _)| s == selected) {
                                        let total_episodes = episodes.len();
                                        ui.label(egui::RichText::new(format!("{} Episodes", total_episodes))
                                            .size(12.0)
                                            .color(egui::Color32::from_rgb(140, 140, 140)));
                                        ui.add_space(12.0);
                                        
                                        // Netflix-style episode list
                                        let row_height = 56.0;
                                        
                                        egui::ScrollArea::vertical()
                                            .auto_shrink([false, false])
                                            .show_rows(ui, row_height, total_episodes, |ui, row_range| {
                                                for idx in row_range {
                                                    let ep = &episodes[idx];
                                                    
                                                    // Episode row with hover effect
                                                    egui::Frame::none()
                                                        .fill(egui::Color32::from_rgb(35, 35, 35))
                                                        .rounding(egui::Rounding::same(4.0))
                                                        .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                                                        .show(ui, |ui| {
                                                            ui.horizontal(|ui| {
                                                                // Episode number
                                                                ui.label(egui::RichText::new(format!("{}", ep.num))
                                                                    .size(22.0)
                                                                    .color(egui::Color32::from_rgb(120, 120, 120)));
                                                                
                                                                ui.add_space(16.0);
                                                                
                                                                // Episode title
                                                                let title_display = if ep.title.chars().count() > 50 {
                                                                    let truncated: String = ep.title.chars().take(50).collect();
                                                                    format!("{}...", truncated)
                                                                } else {
                                                                    ep.title.clone()
                                                                };
                                                                
                                                                ui.vertical(|ui| {
                                                                    ui.label(egui::RichText::new(&title_display)
                                                                        .size(14.0)
                                                                        .color(egui::Color32::WHITE));
                                                                    ui.label(egui::RichText::new(format!("S{}:E{}", ep.season, ep.num))
                                                                        .size(11.0)
                                                                        .color(egui::Color32::from_rgb(120, 120, 120)));
                                                                });
                                                                
                                                                // Play button on the right
                                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                    let play_btn = egui::Button::new(
                                                                        egui::RichText::new("▶")
                                                                            .size(16.0)
                                                                            .color(egui::Color32::WHITE)
                                                                    )
                                                                    .fill(egui::Color32::from_rgb(229, 9, 20))
                                                                    .min_size(egui::vec2(40.0, 32.0))
                                                                    .rounding(egui::Rounding::same(4.0));
                                                                    
                                                                    if ui.add(play_btn).clicked() {
                                                                        action = Some(EpisodeAction::PlayEpisode {
                                                                            episode_id: ep.id.clone(),
                                                                            series_name: data.name.clone(),
                                                                            season: ep.season.parse().unwrap_or(0),
                                                                            episode: ep.num as i32,
                                                                            title: ep.title.clone(),
                                                                            container: ep.container.clone(),
                                                                        });
                                                                    }
                                                                });
                                                            });
                                                        });
                                                    ui.add_space(4.0);
                                                }
                                            });
                                    }
                                }
                            }
                        }
                    });
            });
        
        action
    }
}
