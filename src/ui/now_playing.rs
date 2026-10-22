//! What is playing right now.
//!
//! [`Media`] describes something the app can play and how it is recorded in
//! the watch history. [`PlaybackSession`] tracks a running external player,
//! and [`now_playing_bar`] shows its state at the bottom of the window.

use eframe::egui;
use crate::models::{NextEpisode, WatchProgress};
use crate::playback::SessionHandle;
use super::layout::Layout;
use super::theme::{Theme, spacing, typography, radius};

/// Something the app can play.
#[derive(Debug, Clone)]
pub enum Media {
    Channel {
        stream_id: String,
        name: String,
        icon: Option<String>,
    },
    Movie {
        stream_id: i64,
        name: String,
        container: String,
        thumbnail: Option<String>,
    },
    Episode {
        episode_id: String,
        series_id: Option<i32>,
        series_name: String,
        season: i32,
        episode: i32,
        title: String,
        container: String,
        thumbnail: Option<String>,
        next: Option<NextEpisode>,
    },
}

impl Media {
    pub fn is_live(&self) -> bool {
        matches!(self, Media::Channel { .. })
    }

    /// Key of this item in the watch history.
    pub fn content_id(&self) -> String {
        match self {
            Media::Channel { stream_id, .. } => format!("channel:{}", stream_id),
            // Same key as older versions, so their entries still resume
            Media::Movie { stream_id, .. } => stream_id.to_string(),
            Media::Episode { episode_id, .. } => format!("episode:{}", episode_id),
        }
    }

    /// Title for the player window and the now-playing bar.
    pub fn title(&self) -> String {
        match self {
            Media::Channel { name, .. } | Media::Movie { name, .. } => name.clone(),
            Media::Episode { series_name, season, episode, title, .. } => {
                format!("{} · S{}:E{} {}", series_name, season, episode, title).trim_end().to_string()
            }
        }
    }

    /// The episode after this one, if known.
    pub fn next_episode(&self) -> Option<Media> {
        match self {
            Media::Episode { series_id, series_name, thumbnail, next: Some(next), .. } => Some(Media::Episode {
                episode_id: next.episode_id.clone(),
                series_id: *series_id,
                series_name: series_name.clone(),
                season: next.season,
                episode: next.episode,
                title: next.title.clone(),
                container: next.container.clone(),
                thumbnail: thumbnail.clone(),
                next: None,
            }),
            _ => None,
        }
    }

    /// Rebuilds a playable item from a watch history entry. Entries written
    /// by older versions lack the stream details and return `None`.
    pub fn from_progress(p: &WatchProgress) -> Option<Media> {
        let stream_id = p.stream_id.clone()?;
        match p.content_type.as_str() {
            "channel" => Some(Media::Channel {
                stream_id,
                name: p.content_name.clone(),
                icon: p.thumbnail.clone(),
            }),
            "movie" => Some(Media::Movie {
                stream_id: stream_id.parse().ok()?,
                name: p.content_name.clone(),
                container: p.container.clone()?,
                thumbnail: p.thumbnail.clone(),
            }),
            "series" => Some(Media::Episode {
                episode_id: stream_id,
                series_id: p.series_id,
                series_name: p.content_name.clone(),
                season: p.season.unwrap_or(0),
                episode: p.episode.unwrap_or(0),
                title: p.episode_title.clone().unwrap_or_default(),
                container: p.container.clone()?,
                thumbnail: p.thumbnail.clone(),
                next: p.next_episode.clone(),
            }),
            _ => None,
        }
    }

    /// Watch history entry for this item at the given position.
    pub fn history_entry(&self, position: f64, duration: f64) -> WatchProgress {
        let mut entry = WatchProgress {
            content_id: self.content_id(),
            position_seconds: position,
            duration_seconds: duration,
            last_watched: chrono::Utc::now().timestamp(),
            ..Default::default()
        };
        match self {
            Media::Channel { stream_id, name, icon } => {
                entry.content_type = "channel".into();
                entry.content_name = name.clone();
                entry.thumbnail = icon.clone();
                entry.stream_id = Some(stream_id.clone());
                // Position is meaningless for live TV
                entry.position_seconds = 0.0;
                entry.duration_seconds = 0.0;
            }
            Media::Movie { stream_id, name, container, thumbnail } => {
                entry.content_type = "movie".into();
                entry.content_name = name.clone();
                entry.thumbnail = thumbnail.clone();
                entry.stream_id = Some(stream_id.to_string());
                entry.container = Some(container.clone());
            }
            Media::Episode { episode_id, series_id, series_name, season, episode, title, container, thumbnail, next } => {
                entry.content_type = "series".into();
                entry.content_name = series_name.clone();
                entry.thumbnail = thumbnail.clone();
                entry.stream_id = Some(episode_id.clone());
                entry.container = Some(container.clone());
                entry.series_id = *series_id;
                entry.season = Some(*season);
                entry.episode = Some(*episode);
                entry.episode_title = Some(title.clone()).filter(|t| !t.is_empty());
                entry.next_episode = next.clone();
            }
        }
        entry
    }
}

/// A running external player.
pub struct PlaybackSession {
    pub handle: SessionHandle,
    pub media: Media,
    /// Player name for status text ("mpv", "VLC")
    pub player: &'static str,
    /// The stream has started playing
    pub playing: bool,
    pub position: f64,
    pub duration: f64,
}

/// Formats seconds as `m:ss` or `h:mm:ss`.
pub fn format_time(seconds: f64) -> String {
    let total = seconds.max(0.0) as u64;
    let (h, m, s) = (total / 3600, total / 60 % 60, total % 60);
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m, s)
    } else {
        format!("{}:{:02}", m, s)
    }
}

/// Renders the now-playing bar. Returns true when Stop is pressed.
pub fn now_playing_bar(ui: &mut egui::Ui, theme: &Theme, layout: &Layout, session: &PlaybackSession) -> bool {
    let mut stop = false;
    let height = layout.touch_target();
    let show_progress = !session.media.is_live() && session.duration > 0.0;

    ui.horizontal(|ui| {
        ui.set_min_height(height);
        ui.spacing_mut().item_spacing.x = spacing::MD;

        // Status indicator: spinner while opening, live dot or play icon after
        if session.playing {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
            let color = if session.media.is_live() { theme.live_badge() } else { theme.accent_blue };
            ui.painter().circle_filled(rect.center(), 5.0, color);
        } else {
            ui.add(egui::Spinner::new().size(16.0).color(theme.accent_blue));
        }

        let status = match (session.playing, session.media.is_live()) {
            (false, _) => format!("Opening in {}…", session.player),
            (true, true) => format!("Live in {}", session.player),
            (true, false) if show_progress => format!(
                "{} / {}",
                format_time(session.position),
                format_time(session.duration)
            ),
            (true, false) => format!("Playing in {}", session.player),
        };

        // Stop button on the right; the text takes the remaining width
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let label = if layout.is_compact() { "■" } else { "■  Stop" };
            let button = egui::Button::new(
                egui::RichText::new(label).size(typography::BODY_SM).color(theme.text_primary),
            )
            .fill(theme.inactive_bg())
            .stroke(egui::Stroke::NONE)
            .rounding(egui::Rounding::same(radius::FULL))
            .min_size(egui::vec2(height, height - 8.0));
            if ui.add(button).on_hover_text("Close the player").clicked() {
                stop = true;
            }

            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.add(egui::Label::new(
                        egui::RichText::new(session.media.title())
                            .size(typography::BODY_SM)
                            .color(theme.text_primary)
                            .strong(),
                    ).truncate());
                    ui.label(egui::RichText::new(status).size(typography::CAPTION).color(theme.text_secondary));
                });
            });
        });
    });

    // Thin progress line along the top edge of the bar
    if show_progress {
        let rect = ui.max_rect();
        let pct = (session.position / session.duration).clamp(0.0, 1.0) as f32;
        let top = rect.min.y - spacing::SM;
        let full = egui::Rect::from_min_max(
            egui::pos2(rect.min.x - spacing::LG, top),
            egui::pos2(rect.max.x + spacing::LG, top + 2.0),
        );
        ui.painter().rect_filled(
            egui::Rect::from_min_size(full.min, egui::vec2(full.width() * pct, full.height())),
            0.0,
            theme.accent_blue,
        );
    }

    stop
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_times() {
        assert_eq!(format_time(0.0), "0:00");
        assert_eq!(format_time(75.4), "1:15");
        assert_eq!(format_time(3725.0), "1:02:05");
    }

    #[test]
    fn episode_round_trips_through_history() {
        let media = Media::Episode {
            episode_id: "55".into(),
            series_id: Some(3),
            series_name: "Show".into(),
            season: 1,
            episode: 2,
            title: "Pilot".into(),
            container: "mkv".into(),
            thumbnail: None,
            next: Some(NextEpisode { episode_id: "56".into(), season: 1, episode: 3, title: "Next".into(), container: "mkv".into() }),
        };
        let entry = media.history_entry(120.0, 1500.0);
        assert_eq!(entry.content_id, "episode:55");
        let back = Media::from_progress(&entry).expect("playable");
        assert_eq!(back.content_id(), "episode:55");
        let next = back.next_episode().expect("next episode");
        assert_eq!(next.content_id(), "episode:56");
    }
}
