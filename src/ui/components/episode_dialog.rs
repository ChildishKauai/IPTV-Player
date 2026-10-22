//! Episode dialog component - Modern, clean design
//!
//! Displays series episodes in a premium dialog with refined styling.
//! Features season tabs, episode list, and smooth interactions.

use eframe::egui;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use crate::api::XtreamClient;
use crate::models::{NextEpisode, WatchHistory};
use crate::ui::theme::{Theme, spacing, typography, radius};

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
        series_id: i32,
        /// Series poster, for Continue Watching
        cover: Option<String>,
        /// The following episode, offered as "Up next"
        next: Option<NextEpisode>,
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
    cover: Option<String>,
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

        let srv = server_url.clone();
        let usr = username.clone();
        let pwd = password.clone();

        thread::spawn(move || {
            let client = XtreamClient::new(srv, usr, pwd);

            match client.get_series_info(series_id) {
                Ok(info) => {
                    let mut name = String::new();
                    let mut plot = String::new();

                    if let Some(info_obj) = info.get("info") {
                        name = info_obj
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown Series")
                            .to_string();
                        plot = info_obj
                            .get("plot")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                    }

                    let seasons = parse_seasons(&info);
                    let cover = info
                        .get("info")
                        .and_then(|i| i.get("cover"))
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(String::from);

                    let _ = tx.send(Ok(SeriesData { name, plot, cover, seasons }));
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

    fn check_for_data(&mut self) {
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(data) => {
                        if self.selected_season.is_none() && !data.seasons.is_empty() {
                            self.selected_season = Some(data.seasons[0].0.clone());
                        }
                        self.state = LoadingState::Loaded(data);
                    }
                    Err(e) => {
                        self.state = LoadingState::Error(e);
                    }
                }
                self.rx = None;
            }
        }
    }
}

/// Episode dialog component - Modern design
pub struct EpisodeDialog;

impl EpisodeDialog {
    /// Renders the episode dialog. `history` marks episodes that were
    /// started and offers to resume them.
    pub fn show(
        ctx: &egui::Context,
        theme: &Theme,
        state: &mut EpisodeDialogState,
        history: &WatchHistory,
    ) -> Option<EpisodeAction> {
        state.check_for_data();

        let mut action: Option<EpisodeAction> = None;

        // Colors
        let bg = theme.card_bg;
        let card_bg = theme.inactive_bg();
        let text_primary = theme.text_primary;
        let text_secondary = theme.text_secondary;
        let text_tertiary = theme.text_tertiary;
        let accent = theme.accent_blue;

        egui::Window::new("")
            .resizable(true)
            .collapsible(false)
            .title_bar(false)
            // Fit inside small windows (phones, split screen)
            .default_width((850.0f32).min(ctx.screen_rect().width() - 72.0))
            .max_width(ctx.screen_rect().width() - 72.0)
            .max_height(ctx.screen_rect().height() - 72.0)
            .default_height((600.0f32).min(ctx.screen_rect().height() - 72.0))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(
                egui::Frame::none()
                    .fill(bg)
                    .rounding(egui::Rounding::same(radius::XL))
                    .inner_margin(egui::Margin::same(0.0))
                    .stroke(egui::Stroke::new(1.0, theme.border_color)),
            )
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(bg)
                    .inner_margin(egui::Margin::same(spacing::XL))
                    .show(ui, |ui| {
                        match &state.state {
                            LoadingState::Loading => {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.add_space(spacing::SM);
                                    ui.label(
                                        egui::RichText::new("Loading episodes...")
                                            .size(typography::BODY)
                                            .color(text_secondary),
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        egui::RichText::new("🗙")
                                                            .size(20.0)
                                                            .color(text_secondary),
                                                    )
                                                    .fill(egui::Color32::TRANSPARENT)
                                                    .min_size(egui::vec2(40.0, 40.0)),
                                                )
                                                .clicked()
                                            {
                                                action = Some(EpisodeAction::Close);
                                            }
                                        },
                                    );
                                });
                            }
                            LoadingState::Error(e) => {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("Error: {}", e))
                                            .size(typography::BODY)
                                            .color(theme.error_color),
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        egui::RichText::new("🗙")
                                                            .size(20.0)
                                                            .color(text_secondary),
                                                    )
                                                    .fill(egui::Color32::TRANSPARENT)
                                                    .min_size(egui::vec2(40.0, 40.0)),
                                                )
                                                .clicked()
                                            {
                                                action = Some(EpisodeAction::Close);
                                            }
                                        },
                                    );
                                });
                            }
                            LoadingState::Loaded(data) => {
                                // Header
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(&data.name)
                                            .size(typography::H1)
                                            .color(text_primary)
                                            .strong(),
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        egui::RichText::new("🗙")
                                                            .size(20.0)
                                                            .color(text_secondary),
                                                    )
                                                    .fill(egui::Color32::TRANSPARENT)
                                                    .min_size(egui::vec2(40.0, 40.0)),
                                                )
                                                .clicked()
                                            {
                                                action = Some(EpisodeAction::Close);
                                            }
                                        },
                                    );
                                });

                                // Plot
                                if !data.plot.is_empty() {
                                    ui.add_space(spacing::SM);
                                    let plot_display = truncate_text(&data.plot, 200);
                                    ui.label(
                                        egui::RichText::new(plot_display)
                                            .size(typography::BODY_SM)
                                            .color(text_tertiary),
                                    );
                                }

                                ui.add_space(spacing::XL);

                                // Season tabs
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing = egui::vec2(spacing::SM, spacing::SM);

                                    for (season_key, _) in &data.seasons {
                                        let is_selected =
                                            state.selected_season.as_ref() == Some(season_key);
                                        let btn_text = format!("Season {}", season_key);

                                        let (bg_color, fg_color) = if is_selected {
                                            (text_primary, bg)
                                        } else {
                                            (card_bg, text_secondary)
                                        };

                                        let btn = egui::Button::new(
                                            egui::RichText::new(&btn_text)
                                                .size(typography::BODY_SM)
                                                .color(fg_color),
                                        )
                                        .fill(bg_color)
                                        .rounding(egui::Rounding::same(radius::FULL))
                                        .min_size(egui::vec2(100.0, 36.0));

                                        if ui.add(btn).clicked() {
                                            state.selected_season = Some(season_key.clone());
                                        }
                                    }
                                });

                                ui.add_space(spacing::LG);

                                // Episodes list
                                if let Some(selected) = &state.selected_season {
                                    if let Some((_, episodes)) =
                                        data.seasons.iter().find(|(s, _)| s == selected)
                                    {
                                        let total_episodes = episodes.len();
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{} Episodes",
                                                total_episodes
                                            ))
                                            .size(typography::CAPTION)
                                            .color(text_tertiary),
                                        );

                                        ui.add_space(spacing::MD);

                                        let row_height = 72.0;

                                        egui::ScrollArea::vertical()
                                            .auto_shrink([false, false])
                                            .show_rows(
                                                ui,
                                                row_height,
                                                total_episodes,
                                                |ui, row_range| {
                                                    for idx in row_range {
                                                        let ep = &episodes[idx];
                                                        let watched = history.get_progress(&format!("episode:{}", ep.id));
                                                        let resume = watched.and_then(|p| p.resume_position());
                                                        let finished = watched.is_some_and(|p| p.is_nearly_finished());

                                                        egui::Frame::none()
                                                            .fill(card_bg)
                                                            .rounding(egui::Rounding::same(
                                                                radius::MD,
                                                            ))
                                                            .inner_margin(egui::Margin::symmetric(
                                                                spacing::LG,
                                                                spacing::MD,
                                                            ))
                                                            .show(ui, |ui| {
                                                                ui.horizontal(|ui| {
                                                                    // Episode number
                                                                    ui.label(
                                                                        egui::RichText::new(
                                                                            format!("{}", ep.num),
                                                                        )
                                                                        .size(typography::H2)
                                                                        .color(text_tertiary),
                                                                    );

                                                                    ui.add_space(spacing::LG);

                                                                    // Episode info
                                                                    ui.vertical(|ui| {

                                                                        // Leave room for the Play button; elide long titles

                                                                        ui.set_max_width((ui.available_width() - 112.0).max(80.0));

                                                                        ui.add(

                                                                            egui::Label::new(

                                                                                egui::RichText::new(&ep.title)

                                                                                    .size(typography::BODY)

                                                                                    .color(text_primary),

                                                                            )

                                                                            .truncate(),

                                                                        );
                                                                        let status = match (watched, resume) {
                                                                            (Some(p), Some(_)) => format!(
                                                                                " · {} min left",
                                                                                ((p.duration_seconds - p.position_seconds) / 60.0).ceil().max(1.0) as i64
                                                                            ),
                                                                            _ if finished => " · Watched".to_string(),
                                                                            _ => String::new(),
                                                                        };
                                                                        ui.label(
                                                                            egui::RichText::new(
                                                                                format!(
                                                                                    "S{}:E{}{}",
                                                                                    ep.season,
                                                                                    ep.num,
                                                                                    status
                                                                                ),
                                                                            )
                                                                            .size(
                                                                                typography::CAPTION,
                                                                            )
                                                                            .color(text_tertiary),
                                                                        );
                                                                    });

                                                                    // Play button
                                                                    ui.with_layout(
                                                                        egui::Layout::right_to_left(
                                                                            egui::Align::Center,
                                                                        ),
                                                                        |ui| {
                                                                            let play_btn =
                                                                                egui::Button::new(
                                                                                    egui::RichText::new(
                                                                                        if resume.is_some() { "▶ Resume" } else { "▶ Play" },
                                                                                    )
                                                                                    .size(
                                                                                        typography::BODY_SM,
                                                                                    )
                                                                                    .color(
                                                                                        egui::Color32::WHITE,
                                                                                    ),
                                                                                )
                                                                                .fill(accent)
                                                                                .min_size(egui::vec2(
                                                                                    96.0, 36.0,
                                                                                ))
                                                                                .rounding(
                                                                                    egui::Rounding::same(
                                                                                        radius::MD,
                                                                                    ),
                                                                                );

                                                                            if ui
                                                                                .add(play_btn)
                                                                                .clicked()
                                                                            {
                                                                                action = Some(
                                                                                    EpisodeAction::PlayEpisode {
                                                                                        episode_id: ep.id.clone(),
                                                                                        series_name: data.name.clone(),
                                                                                        season: ep.season.parse().unwrap_or(0),
                                                                                        episode: ep.num as i32,
                                                                                        title: ep.title.clone(),
                                                                                        container: ep.container.clone(),
                                                                                        series_id: state.series_id,
                                                                                        cover: data.cover.clone(),
                                                                                        next: next_after(&data.seasons, &ep.id),
                                                                                    },
                                                                                );
                                                                            }
                                                                        },
                                                                    );
                                                                });
                                                            });

                                                        ui.add_space(spacing::SM);
                                                    }
                                                },
                                            );
                                    }
                                }
                            }
                        }
                    });
            });

        action
    }
}

/// Reads the seasons and episodes from a `get_series_info` response,
/// sorted by season number.
fn parse_seasons(info: &serde_json::Value) -> Vec<(String, Vec<EpisodeData>)> {
    let mut seasons: Vec<(String, Vec<EpisodeData>)> = Vec::new();

    if let Some(episodes) = info.get("episodes").and_then(|v| v.as_object()) {
        let mut season_keys: Vec<_> = episodes.keys().collect();
        season_keys.sort_by(|a, b| {
            a.parse::<i32>()
                .unwrap_or(0)
                .cmp(&b.parse::<i32>().unwrap_or(0))
        });

        for season_key in season_keys {
            if let Some(season_episodes) =
                episodes.get(season_key).and_then(|v| v.as_array())
            {
                let eps: Vec<EpisodeData> = season_episodes
                    .iter()
                    .map(|ep| {
                        let id = if let Some(id_str) =
                            ep.get("id").and_then(|v| v.as_str())
                        {
                            id_str.to_string()
                        } else if let Some(id_num) =
                            ep.get("id").and_then(|v| v.as_i64())
                        {
                            id_num.to_string()
                        } else {
                            String::new()
                        };

                        EpisodeData {
                            id,
                            num: ep
                                .get("episode_num")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0),
                            title: ep
                                .get("title")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown")
                                .to_string(),
                            container: ep
                                .get("container_extension")
                                .and_then(|v| v.as_str())
                                .unwrap_or("mp4")
                                .to_string(),
                            season: season_key.clone(),
                        }
                    })
                    .collect();

                seasons.push((season_key.clone(), eps));
            }
        }
    }

    seasons
}

/// The episode that follows `episode_id` in a `get_series_info` response,
/// continuing into the next season.
pub fn find_next_episode(info: &serde_json::Value, episode_id: &str) -> Option<NextEpisode> {
    next_after(&parse_seasons(info), episode_id)
}

fn next_after(seasons: &[(String, Vec<EpisodeData>)], episode_id: &str) -> Option<NextEpisode> {
    let mut episodes = seasons.iter().flat_map(|(_, eps)| eps.iter());
    episodes.find(|ep| ep.id == episode_id)?;
    episodes.next().map(|ep| NextEpisode {
        episode_id: ep.id.clone(),
        season: ep.season.parse().unwrap_or(0),
        episode: ep.num as i32,
        title: ep.title.clone(),
        container: ep.container.clone(),
    })
}

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
