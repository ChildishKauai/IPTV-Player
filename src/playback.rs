//! Playback sessions for external media players.
//!
//! Playback happens in an external player (mpv, VLC, ...). A session
//! launches the player, watches the process and reports back to the UI:
//!
//! - whether the player could be launched at all
//! - when playback has actually started
//! - the playback position, read from mpv's JSON IPC or VLC's remote
//!   control interface, so Continue Watching can resume where you left off
//! - how it ended, with a plain-language reason when the stream failed
//!
//! Events arrive on an `mpsc` channel that the UI drains every frame.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::models::{LaunchOptions, PlayerSettings, PlayerType};

/// Without position reports, a player that is still running after this long
/// is assumed to be playing.
const ASSUME_PLAYING_AFTER: Duration = Duration::from_secs(8);
/// Even with position tracking, stop showing "Opening…" after this long.
const GIVE_UP_WAITING_AFTER: Duration = Duration::from_secs(45);
/// How long to keep trying to reach the player's control interface.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
/// How often progress is reported to the UI.
const PROGRESS_INTERVAL: Duration = Duration::from_secs(1);
/// stderr lines kept for explaining failures.
const STDERR_LINES: usize = 60;

/// Something that happened to a playback session.
#[derive(Debug, Clone)]
pub enum PlaybackEvent {
    /// The player is up and the stream is playing.
    Playing { session: u64 },
    /// Latest position reported by the player (seconds).
    Progress { session: u64, position: f64, duration: f64 },
    /// The player exited.
    Ended {
        session: u64,
        outcome: PlaybackOutcome,
        position: f64,
        duration: f64,
    },
}

/// How a playback session ended.
#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackOutcome {
    /// The viewer closed the player or the video finished.
    Finished,
    /// Stopped from the app (Stop button or another stream was started).
    Stopped,
    /// The stream never started playing.
    Failed(String),
    /// Playback had started but the player exited with an error.
    Interrupted(String),
}

/// Why the player could not be launched.
#[derive(Debug, Clone)]
pub struct LaunchError {
    pub message: String,
    /// Whether changing the player settings is the likely fix.
    pub settings_fix: bool,
}

/// Handle to a running session, kept by the UI.
pub struct SessionHandle {
    stop: Arc<AtomicBool>,
}

impl SessionHandle {
    /// Asks the player to close. An `Ended` event follows.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// Position shared between the tracker thread and the session thread.
#[derive(Default)]
struct Position {
    position: f64,
    duration: f64,
    /// The control interface connected
    connected: bool,
}

/// Launches the configured player and starts watching it.
///
/// `notify` is called whenever an event is sent, so the UI can repaint.
pub fn start(
    settings: &PlayerSettings,
    session: u64,
    url: &str,
    title: &str,
    is_live: bool,
    start_at: Option<f64>,
    events: Sender<PlaybackEvent>,
    notify: impl Fn() + Send + 'static,
) -> Result<SessionHandle, LaunchError> {
    let player = settings.player_type;
    let mut options = LaunchOptions { start_at, ..Default::default() };
    let tracker = match player {
        PlayerType::MPV => {
            let endpoint = mpv_ipc_endpoint(session);
            options.mpv_ipc = Some(endpoint.clone());
            Some(Tracker::Mpv(endpoint))
        }
        PlayerType::VLC => free_local_port().map(|port| {
            options.vlc_rc_port = Some(port);
            Tracker::Vlc(port)
        }),
        _ => None,
    };

    let mut child = settings
        .launch_player(url, title, is_live, &options)
        .map_err(|e| describe_launch_error(&e, player, settings))?;

    let stop = Arc::new(AtomicBool::new(false));
    let handle = SessionHandle { stop: stop.clone() };

    // Keep the tail of stderr to explain failures, and still log it
    let stderr_tail = Arc::new(Mutex::new(VecDeque::new()));
    if let Some(stderr) = child.stderr.take() {
        let tail = stderr_tail.clone();
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if is_status_line(&line) {
                    continue;
                }
                eprintln!("[Player] {}", line);
                let mut tail = tail.lock().unwrap();
                if tail.len() == STDERR_LINES {
                    tail.pop_front();
                }
                tail.push_back(line);
            }
        });
    }

    let position = Arc::new(Mutex::new(Position::default()));
    let tracked = tracker.is_some();
    if let Some(tracker) = tracker {
        let position = position.clone();
        let stop = stop.clone();
        thread::spawn(move || tracker.run(&position, &stop));
    }

    thread::spawn(move || {
        let started = Instant::now();
        let mut playing = false;
        let mut stopped_by_app = false;
        let mut last_report = Instant::now();
        let mut reported = (0.0, 0.0);

        let status = loop {
            if stop.load(Ordering::Relaxed) && !stopped_by_app {
                stopped_by_app = true;
                let _ = child.kill();
            }
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) => {}
                Err(_) => break None,
            }

            let (pos, dur, connected) = {
                let p = position.lock().unwrap();
                (p.position, p.duration, p.connected)
            };

            if !playing {
                let elapsed = started.elapsed();
                // With tracking, wait for the position to move; otherwise
                // (or if the control interface never came up) assume the
                // stream is playing once the player has stayed open a while.
                playing = pos > 0.0
                    || (elapsed >= ASSUME_PLAYING_AFTER && !(tracked && connected))
                    || elapsed >= GIVE_UP_WAITING_AFTER;
                if playing {
                    let _ = events.send(PlaybackEvent::Playing { session });
                    notify();
                }
            }

            if last_report.elapsed() >= PROGRESS_INTERVAL && (pos, dur) != reported {
                reported = (pos, dur);
                last_report = Instant::now();
                let _ = events.send(PlaybackEvent::Progress { session, position: pos, duration: dur });
                notify();
            }

            thread::sleep(Duration::from_millis(250));
        };

        // Let the stderr reader catch the last lines
        thread::sleep(Duration::from_millis(150));
        stop.store(true, Ordering::Relaxed);

        let (pos, dur) = {
            let p = position.lock().unwrap();
            (p.position, p.duration)
        };
        let lines: Vec<String> = stderr_tail.lock().unwrap().iter().cloned().collect();
        // Closing the window, or the system asking the player to quit,
        // is a normal end - not an error worth reporting
        let success = status.map(|s| s.success() || ended_by_request(&s)).unwrap_or(false);

        let outcome = if stopped_by_app {
            PlaybackOutcome::Stopped
        } else if success {
            PlaybackOutcome::Finished
        } else if !playing {
            PlaybackOutcome::Failed(describe_failure(&lines, player))
        } else {
            PlaybackOutcome::Interrupted(describe_failure(&lines, player))
        };

        let _ = events.send(PlaybackEvent::Ended { session, outcome, position: pos, duration: dur });
        notify();
    });

    Ok(handle)
}

// ─────────────────────────────────────────────────────────────────────────
// Error messages
// ─────────────────────────────────────────────────────────────────────────

/// Warnings players print during normal playback (audio output probing,
/// hardware decoder fallback) that say nothing about why a stream failed.
fn is_noise(line: &str) -> bool {
    let line = line.to_lowercase();
    [
        "[ao", "alsa", "jack", "pulse", "pipewire", "pw.", "audio device", "audio driver",
        "v4l2", "vaapi", "vdpau", "hwdec", "cuda", "nvdec", "vulkan", "could not open codec",
        "client-rt.conf", "server socket", "server request channel",
    ]
    .iter()
    .any(|n| line.contains(n))
}

/// Whether the player exited because it was asked to quit (SIGTERM,
/// SIGINT, SIGHUP, or mpv's matching exit code) rather than by crashing.
fn ended_by_request(status: &std::process::ExitStatus) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return matches!(signal, 1 | 2 | 15);
        }
    }
    // mpv exits with 4 when it quits because of a signal
    status.code() == Some(4)
}

/// mpv's once-per-frame status line ("AV: 00:01:02 / 00:45:00 (2%)") would
/// flood the log and push real errors out of the kept tail.
fn is_status_line(line: &str) -> bool {
    let line = line.trim_start_matches(['\r', ' ']);
    ["AV:", "A:", "V:", "(Paused)", "(Buffering)"].iter().any(|p| line.starts_with(p))
}

fn describe_launch_error(e: &std::io::Error, player: PlayerType, settings: &PlayerSettings) -> LaunchError {
    let name = player.short_name();
    let message = match e.kind() {
        std::io::ErrorKind::NotFound if player == PlayerType::Custom && settings.custom_player_path.is_empty() => {
            "No custom player is set. Choose one in Player settings.".to_string()
        }
        std::io::ErrorKind::NotFound => {
            format!("{} isn't installed or couldn't be found. Install it or pick another player.", name)
        }
        std::io::ErrorKind::PermissionDenied => {
            format!("{} couldn't be started (permission denied).", name)
        }
        _ => format!("{} couldn't be started: {}", name, e),
    };
    LaunchError { message, settings_fix: true }
}

/// Turns the player's error output into a short explanation.
pub fn describe_failure(stderr: &[String], player: PlayerType) -> String {
    let lines: Vec<&String> = stderr.iter().filter(|l| !is_noise(l)).collect();
    let text = lines.iter().map(|l| l.as_str()).collect::<Vec<_>>().join("\n").to_lowercase();
    let has = |needles: &[&str]| needles.iter().any(|n| text.contains(n));

    if has(&["403", "forbidden"]) {
        "The server refused this stream (403). Your subscription may have expired or reached its connection limit.".into()
    } else if has(&["401", "unauthorized"]) {
        "The server rejected your login for this stream (401).".into()
    } else if has(&["404", "not found"]) && !has(&["command not found"]) {
        "This stream isn't available on the server (404).".into()
    } else if has(&["429", "too many"]) {
        "Too many connections. Close other players or devices using this account and try again.".into()
    } else if has(&["500 ", "502", "503", "504", "server error"]) {
        "The stream server had a problem. Try again in a moment.".into()
    } else if has(&[
        "connection refused",
        "timed out",
        "timeout",
        "failed to resolve",
        "name or service not known",
        "no route to host",
        "network is unreachable",
    ]) {
        "Couldn't reach the stream server. Check your internet connection.".into()
    } else if has(&["no video or audio streams", "unrecognized file format", "invalid data found"]) {
        "The stream sent no playable video. It may be offline right now.".into()
    } else if has(&["unsupported codec", "no decoder", "decoder not found", "codec not supported", "unknown codec"]) {
        format!("{} can't decode this stream's format.", player.short_name())
    } else if let Some(line) = lines.iter().rev().find(|l| {
        let l = l.to_lowercase();
        l.contains("error") || l.contains("failed")
    }) {
        let line = line.trim();
        let short: String = line.chars().take(140).collect();
        format!("{} stopped: {}", player.short_name(), short)
    } else {
        format!("{} closed before the stream started.", player.short_name())
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Position tracking
// ─────────────────────────────────────────────────────────────────────────

enum Tracker {
    /// mpv JSON IPC endpoint
    Mpv(String),
    /// VLC remote control port
    Vlc(u16),
}

impl Tracker {
    fn run(self, position: &Mutex<Position>, stop: &AtomicBool) {
        match self {
            Tracker::Mpv(endpoint) => {
                if let Some(conn) = connect_with_retry(stop, || mpv_connect(&endpoint)) {
                    position.lock().unwrap().connected = true;
                    let _ = track_mpv(conn, position);
                }
                #[cfg(unix)]
                let _ = std::fs::remove_file(&endpoint);
            }
            Tracker::Vlc(port) => {
                let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
                if let Some(conn) = connect_with_retry(stop, || std::net::TcpStream::connect(addr)) {
                    position.lock().unwrap().connected = true;
                    track_vlc(conn, position, stop);
                }
            }
        }
    }
}

fn connect_with_retry<T>(stop: &AtomicBool, mut connect: impl FnMut() -> std::io::Result<T>) -> Option<T> {
    let started = Instant::now();
    while started.elapsed() < CONNECT_TIMEOUT && !stop.load(Ordering::Relaxed) {
        if let Ok(conn) = connect() {
            return Some(conn);
        }
        thread::sleep(Duration::from_millis(250));
    }
    None
}

fn mpv_ipc_endpoint(session: u64) -> String {
    let name = format!("iptv-player-mpv-{}-{}", std::process::id(), session);
    if cfg!(windows) {
        format!(r"\\.\pipe\{}", name)
    } else {
        std::env::temp_dir().join(format!("{}.sock", name)).to_string_lossy().into_owned()
    }
}

#[cfg(unix)]
fn mpv_connect(endpoint: &str) -> std::io::Result<std::os::unix::net::UnixStream> {
    std::os::unix::net::UnixStream::connect(endpoint)
}

#[cfg(windows)]
fn mpv_connect(endpoint: &str) -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new().read(true).write(true).open(endpoint)
}

/// Subscribes to mpv's position and duration and follows them until mpv
/// closes the connection.
fn track_mpv<S: Read + Write>(mut conn: S, position: &Mutex<Position>) -> std::io::Result<()> {
    conn.write_all(b"{\"command\":[\"observe_property\",1,\"time-pos\"]}\n")?;
    conn.write_all(b"{\"command\":[\"observe_property\",2,\"duration\"]}\n")?;
    conn.flush()?;

    for line in BufReader::new(conn).lines() {
        let line = line?;
        let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
        if msg.get("event").and_then(|e| e.as_str()) != Some("property-change") {
            continue;
        }
        let Some(value) = msg.get("data").and_then(|d| d.as_f64()) else { continue };
        let mut p = position.lock().unwrap();
        match msg.get("name").and_then(|n| n.as_str()) {
            Some("time-pos") => p.position = value.max(0.0),
            Some("duration") => p.duration = value.max(0.0),
            _ => {}
        }
    }
    Ok(())
}

/// Polls VLC's remote control interface for position and length.
fn track_vlc(mut conn: std::net::TcpStream, position: &Mutex<Position>, stop: &AtomicBool) {
    let _ = conn.set_read_timeout(Some(Duration::from_millis(300)));
    let mut pending = String::new();
    while !stop.load(Ordering::Relaxed) {
        let time = match vlc_query(&mut conn, &mut pending, "get_time") {
            Ok(v) => v,
            Err(()) => return,
        };
        let length = match vlc_query(&mut conn, &mut pending, "get_length") {
            Ok(v) => v,
            Err(()) => return,
        };
        {
            let mut p = position.lock().unwrap();
            if let Some(t) = time {
                p.position = t;
            }
            if let Some(l) = length {
                p.duration = l;
            }
        }
        thread::sleep(Duration::from_secs(1));
    }
}

/// Sends one command and waits for a numeric reply. `Err` means VLC closed
/// the connection.
fn vlc_query(conn: &mut std::net::TcpStream, pending: &mut String, command: &str) -> Result<Option<f64>, ()> {
    // Drop anything left over (banner, prompts, late replies)
    pending.clear();
    conn.write_all(format!("{}\n", command).as_bytes()).map_err(|_| ())?;

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut buf = [0u8; 512];
    loop {
        while let Some(nl) = pending.find('\n') {
            let line: String = pending.drain(..=nl).collect();
            if let Some(value) = parse_vlc_number(&line) {
                return Ok(Some(value));
            }
        }
        if Instant::now() > deadline {
            return Ok(None);
        }
        match conn.read(&mut buf) {
            Ok(0) => return Err(()),
            Ok(n) => pending.push_str(&String::from_utf8_lossy(&buf[..n])),
            Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {}
            Err(_) => return Err(()),
        }
    }
}

/// VLC answers with a bare number, sometimes after a "> " prompt.
fn parse_vlc_number(line: &str) -> Option<f64> {
    let value = line.trim().trim_start_matches('>').trim();
    if !value.is_empty() && value.chars().all(|c| c.is_ascii_digit()) {
        value.parse().ok()
    } else {
        None
    }
}

fn free_local_port() -> Option<u16> {
    std::net::TcpListener::bind(("127.0.0.1", 0))
        .and_then(|l| l.local_addr())
        .map(|a| a.port())
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<String> {
        text.lines().map(String::from).collect()
    }

    #[test]
    fn explains_common_stream_failures() {
        let msg = describe_failure(&lines("[ffmpeg] http: HTTP error 403 Forbidden\nFailed to open http://x"), PlayerType::MPV);
        assert!(msg.contains("403"), "{msg}");
        let msg = describe_failure(&lines("HTTP error 404 Not Found"), PlayerType::MPV);
        assert!(msg.contains("404"), "{msg}");
        let msg = describe_failure(&lines("tcp: Connection refused"), PlayerType::FFplay);
        assert!(msg.contains("internet connection"), "{msg}");
        // Hardware-decoder and audio probing noise isn't a reason
        let noise = lines("[ffmpeg/video] h264_v4l2m2m: can't configure decoder\nCould not open codec.\n[ao] Failed to initialize audio driver 'sdl'");
        assert_eq!(describe_failure(&noise, PlayerType::MPV), "mpv closed before the stream started.");
        let msg = describe_failure(&[], PlayerType::VLC);
        assert_eq!(msg, "VLC closed before the stream started.");
    }

    #[test]
    fn parses_vlc_replies() {
        assert_eq!(parse_vlc_number("> 125\r\n"), Some(125.0));
        assert_eq!(parse_vlc_number("42"), Some(42.0));
        assert_eq!(parse_vlc_number("> \r\n"), None);
        assert_eq!(parse_vlc_number("Command Line Interface initialized."), None);
    }

    #[test]
    fn skips_mpv_status_lines() {
        assert!(is_status_line("AV: 00:00:02 / 00:00:20 (10%) A-V:  0.000"));
        assert!(is_status_line("\rV: 00:00:02 / 00:00:20 (10%)"));
        assert!(!is_status_line("[ffmpeg] http: HTTP error 403 Forbidden"));
    }

    /// A server answering 403 should produce the subscription explanation.
    /// Needs mpv on PATH: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn explains_mpv_http_403() {
        let server = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/live/user/pass/1.ts", server.local_addr().unwrap());
        thread::spawn(move || {
            for stream in server.incoming().flatten() {
                let mut stream = stream;
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                let _ = stream.write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            }
        });

        let (tx, rx) = std::sync::mpsc::channel();
        start(&PlayerSettings::default(), 2, &url, "test", true, None, tx, || {}).expect("mpv launches");
        loop {
            if let PlaybackEvent::Ended { outcome, .. } = rx.recv_timeout(Duration::from_secs(30)).expect("event") {
                match outcome {
                    PlaybackOutcome::Failed(reason) => assert!(reason.contains("403"), "{reason}"),
                    other => panic!("expected a failure, got {other:?}"),
                }
                break;
            }
        }
    }

    /// Plays a generated clip in a real mpv and checks that the position
    /// comes back. Needs mpv on PATH: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn tracks_mpv_position() {
        let (tx, rx) = std::sync::mpsc::channel();
        let settings = PlayerSettings::default();
        let url = std::env::var("IPTV_TEST_MEDIA").expect("set IPTV_TEST_MEDIA to a short video file");
        let handle = start(&settings, 1, &url, "test", false, Some(2.0), tx, || {}).expect("mpv launches");

        let mut saw_playing = false;
        let mut last = (0.0, 0.0);
        loop {
            match rx.recv_timeout(Duration::from_secs(30)).expect("event") {
                PlaybackEvent::Playing { .. } => saw_playing = true,
                PlaybackEvent::Progress { position, duration, .. } => {
                    last = (position, duration);
                    if position > 3.0 {
                        handle.stop();
                    }
                }
                PlaybackEvent::Ended { outcome, position, duration, .. } => {
                    assert_eq!(outcome, PlaybackOutcome::Stopped);
                    assert!(position >= 2.0, "resumed from --start: {position}");
                    assert!(duration > 0.0);
                    break;
                }
            }
        }
        assert!(saw_playing);
        assert!(last.1 > 0.0);
    }
}
