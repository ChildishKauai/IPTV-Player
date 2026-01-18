use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Available media player backends.
/// MPV is the default as it works more reliably across platforms including Steam Deck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlayerType {
    /// MPV media player (default - most reliable cross-platform)
    #[default]
    MPV,
    /// FFplay - part of FFmpeg suite (bundled fallback)
    FFplay,
    /// VLC media player
    VLC,
    /// MPC-HC (Media Player Classic - Home Cinema)
    MpcHc,
    /// PotPlayer
    PotPlayer,
    /// Custom player with user-defined path
    Custom,
}

impl PlayerType {
    /// Get display name for the player.
    pub fn display_name(&self) -> &'static str {
        match self {
            PlayerType::MPV => "MPV (Recommended)",
            PlayerType::FFplay => "FFplay (Bundled)",
            PlayerType::VLC => "VLC Media Player",
            PlayerType::MpcHc => "MPC-HC",
            PlayerType::PotPlayer => "PotPlayer",
            PlayerType::Custom => "Custom Player",
        }
    }
    
    /// Get the default executable name/command for the player.
    /// Returns a list of possible paths to try (first match wins).
    pub fn default_executables(&self) -> Vec<&'static str> {
        match self {
            PlayerType::FFplay => vec![
                "ffplay",
                "ffplay.exe",
                "bundled/ffplay.exe",
                "bundled\\ffplay.exe",
            ],
            PlayerType::VLC => vec![
                "vlc",
                "C:\\Program Files\\VideoLAN\\VLC\\vlc.exe",
                "C:\\Program Files (x86)\\VideoLAN\\VLC\\vlc.exe",
            ],
            PlayerType::MPV => vec![
                "mpv",
                "mpv.exe",
                "bundled/mpv.exe",
                "bundled\\mpv.exe",
                "C:\\Program Files\\mpv\\mpv.exe",
                "C:\\Program Files (x86)\\mpv\\mpv.exe",
                "C:\\Program Files\\Open TV\\deps\\mpv.exe",
                "C:\\Program Files\\mpv.net\\mpvnet.exe",
                "C:\\tools\\mpv\\mpv.exe",
                "C:\\mpv\\mpv.exe",
            ],
            PlayerType::MpcHc => vec![
                "mpc-hc64",
                "mpc-hc64.exe",
                "mpc-hc",
                "mpc-hc.exe",
                "C:\\Program Files\\MPC-HC\\mpc-hc64.exe",
                "C:\\Program Files (x86)\\MPC-HC\\mpc-hc.exe",
                "C:\\Program Files\\MPC-HC x64\\mpc-hc64.exe",
            ],
            PlayerType::PotPlayer => vec![
                "PotPlayerMini64",
                "PotPlayerMini64.exe",
                "PotPlayerMini",
                "PotPlayerMini.exe",
                "C:\\Program Files\\DAUM\\PotPlayer\\PotPlayerMini64.exe",
                "C:\\Program Files (x86)\\DAUM\\PotPlayer\\PotPlayerMini.exe",
            ],
            PlayerType::Custom => vec![],
        }
    }
    
    /// Get the default executable name/command for the player (first option).
    pub fn default_executable(&self) -> &'static str {
        self.default_executables().first().copied().unwrap_or("")
    }
    
    /// Get all available player types.
    /// Listed in recommended order: MPV (most reliable), FFplay (bundled), then others.
    pub fn all() -> &'static [PlayerType] {
        &[
            PlayerType::MPV,
            PlayerType::FFplay,
            PlayerType::VLC,
            PlayerType::MpcHc,
            PlayerType::PotPlayer,
            PlayerType::Custom,
        ]
    }
}

/// Player settings for audio and subtitle track selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSettings {
    /// Selected media player
    #[serde(default)]
    pub player_type: PlayerType,
    /// Custom player executable path (used when player_type is Custom)
    #[serde(default)]
    pub custom_player_path: String,
    /// Custom player arguments template (use {url} for stream URL, {title} for window title)
    #[serde(default)]
    pub custom_player_args: String,
    /// Preferred audio track index (0 = first/default, -1 = disabled)
    #[serde(default)]
    pub audio_track: i32,
    /// Preferred subtitle track index (0 = first, -1 = disabled/none)
    #[serde(default = "default_subtitle_track")]
    pub subtitle_track: i32,
    /// Whether to show subtitles by default
    #[serde(default)]
    pub subtitles_enabled: bool,
    /// Preferred audio language code (e.g., "eng", "spa", "fra")
    #[serde(default)]
    pub preferred_audio_language: String,
    /// Preferred subtitle language code (e.g., "eng", "spa", "fra")
    #[serde(default)]
    pub preferred_subtitle_language: String,
    /// Audio sync offset in seconds (positive = delay audio, negative = advance)
    #[serde(default)]
    pub audio_sync_offset: f32,
    /// Subtitle sync offset in seconds
    #[serde(default)]
    pub subtitle_sync_offset: f32,
    /// Volume level (0-100)
    #[serde(default = "default_volume")]
    pub volume: i32,
    /// Enable hardware acceleration
    #[serde(default = "default_true")]
    pub hardware_acceleration: bool,
    /// Low latency mode for live streams
    #[serde(default)]
    pub low_latency_mode: bool,
    /// Buffer size in KB (0 = auto)
    #[serde(default)]
    pub buffer_size_kb: u32,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            player_type: PlayerType::default(),
            custom_player_path: String::new(),
            custom_player_args: String::new(),
            audio_track: 0,
            subtitle_track: default_subtitle_track(),
            subtitles_enabled: false,
            preferred_audio_language: String::new(),
            preferred_subtitle_language: String::new(),
            audio_sync_offset: 0.0,
            subtitle_sync_offset: 0.0,
            volume: default_volume(),
            hardware_acceleration: true,
            low_latency_mode: false,
            buffer_size_kb: 0,
        }
    }
}

fn default_subtitle_track() -> i32 { -1 }
fn default_volume() -> i32 { 100 }
fn default_true() -> bool { true }

impl PlayerSettings {
    /// Build ffplay arguments based on settings.
    pub fn build_ffplay_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        
        // Audio track selection
        if self.audio_track >= 0 {
            args.push("-ast".to_string());
            args.push(self.audio_track.to_string());
        }
        
        // Subtitle track selection
        if self.subtitles_enabled && self.subtitle_track >= 0 {
            args.push("-sst".to_string());
            args.push(self.subtitle_track.to_string());
        } else if !self.subtitles_enabled {
            // Disable subtitles
            args.push("-sn".to_string());
        }
        
        // Audio sync offset
        if self.audio_sync_offset.abs() > 0.001 {
            args.push("-sync".to_string());
            args.push("audio".to_string());
        }
        
        // Volume
        args.push("-volume".to_string());
        args.push(self.volume.to_string());
        
        // Hardware acceleration
        if self.hardware_acceleration {
            args.push("-hwaccel".to_string());
            args.push("auto".to_string());
        }
        
        // Low latency mode
        if self.low_latency_mode {
            args.push("-fflags".to_string());
            args.push("nobuffer".to_string());
            args.push("-flags".to_string());
            args.push("low_delay".to_string());
            args.push("-framedrop".to_string());
        }
        
        // Buffer size
        if self.buffer_size_kb > 0 {
            args.push("-bufsize".to_string());
            args.push(format!("{}k", self.buffer_size_kb));
        }
        
        // Netflix-style seamless fullscreen playback
        args.push("-fs".to_string());            // Start in fullscreen
        args.push("-autoexit".to_string());      // Exit when done
        args.push("-infbuf".to_string());        // Infinite buffer for network streams
        args.push("-hide_banner".to_string());   // No banner output
        
        args
    }
    
    /// Build VLC arguments based on settings.
    pub fn build_vlc_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        
        // Audio track selection
        if self.audio_track >= 0 {
            args.push(format!("--audio-track={}", self.audio_track));
        }
        
        // Subtitle track selection
        if self.subtitles_enabled && self.subtitle_track >= 0 {
            args.push(format!("--sub-track={}", self.subtitle_track));
        } else if !self.subtitles_enabled {
            args.push("--no-sub-autodetect-file".to_string());
            args.push("--sub-track=-1".to_string());
        }
        
        // Audio language preference
        if !self.preferred_audio_language.is_empty() {
            args.push(format!("--audio-language={}", self.preferred_audio_language));
        }
        
        // Subtitle language preference
        if self.subtitles_enabled && !self.preferred_subtitle_language.is_empty() {
            args.push(format!("--sub-language={}", self.preferred_subtitle_language));
        }
        
        // Audio sync offset (VLC uses milliseconds)
        if self.audio_sync_offset.abs() > 0.001 {
            args.push(format!("--audio-desync={}", (self.audio_sync_offset * 1000.0) as i32));
        }
        
        // Subtitle sync offset
        if self.subtitle_sync_offset.abs() > 0.001 {
            args.push(format!("--sub-delay={}", (self.subtitle_sync_offset * 10.0) as i32));
        }
        
        // Volume (VLC uses 0-512, with 256 being 100%)
        let vlc_volume = (self.volume as f32 * 2.56) as i32;
        args.push(format!("--volume={}", vlc_volume));
        
        // Hardware acceleration
        if self.hardware_acceleration {
            args.push("--avcodec-hw=any".to_string());
        }
        
        // Low latency mode
        if self.low_latency_mode {
            args.push("--network-caching=300".to_string());
            args.push("--live-caching=300".to_string());
        } else if self.buffer_size_kb > 0 {
            args.push(format!("--network-caching={}", self.buffer_size_kb));
        }
        
        // Additional VLC options for Netflix-style seamless playback
        args.push("--fullscreen".to_string());           // Start in fullscreen
        args.push("--play-and-exit".to_string());        // Exit when done
        args.push("--no-video-title-show".to_string());  // No title overlay
        args.push("--no-playlist-enqueue".to_string());  // Don't enqueue, just play
        args.push("--one-instance".to_string());         // Use single VLC instance
        args.push("--no-qt-video-autoresize".to_string()); // Don't resize window
        args.push("--mouse-hide-timeout=1500".to_string()); // Quick mouse hide
        
        args
    }
    
    /// Build MPV arguments based on settings.
    pub fn build_mpv_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        
        // Audio track selection
        if self.audio_track >= 0 {
            args.push(format!("--aid={}", self.audio_track + 1)); // MPV uses 1-based indexing
        }
        
        // Subtitle track selection
        if self.subtitles_enabled && self.subtitle_track >= 0 {
            args.push(format!("--sid={}", self.subtitle_track + 1));
        } else if !self.subtitles_enabled {
            args.push("--sid=no".to_string());
        }
        
        // Audio language preference
        if !self.preferred_audio_language.is_empty() {
            args.push(format!("--alang={}", self.preferred_audio_language));
        }
        
        // Subtitle language preference
        if self.subtitles_enabled && !self.preferred_subtitle_language.is_empty() {
            args.push(format!("--slang={}", self.preferred_subtitle_language));
        }
        
        // Audio sync offset
        if self.audio_sync_offset.abs() > 0.001 {
            args.push(format!("--audio-delay={:.3}", self.audio_sync_offset));
        }
        
        // Subtitle sync offset
        if self.subtitle_sync_offset.abs() > 0.001 {
            args.push(format!("--sub-delay={:.3}", self.subtitle_sync_offset));
        }
        
        // Volume
        args.push(format!("--volume={}", self.volume));
        
        // Hardware acceleration
        if self.hardware_acceleration {
            args.push("--hwdec=auto".to_string());
        }
        
        // Low latency mode
        if self.low_latency_mode {
            args.push("--profile=low-latency".to_string());
            args.push("--untimed".to_string());
        }
        
        // Buffer size
        if self.buffer_size_kb > 0 {
            args.push(format!("--cache-secs={}", self.buffer_size_kb / 1024));
        }
        
        // Netflix-style seamless playback
        args.push("--fullscreen".to_string());           // Start in fullscreen
        args.push("--no-border".to_string());            // Borderless window
        args.push("--osd-level=1".to_string());          // Minimal OSD
        args.push("--osd-duration=1000".to_string());    // Quick OSD fade
        args.push("--cursor-autohide=1000".to_string()); // Quick cursor hide
        args.push("--keep-open=no".to_string());         // Close when done
        
        args
    }
    
    /// Build MPC-HC arguments based on settings.
    pub fn build_mpchc_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        
        // Audio track selection
        if self.audio_track >= 0 {
            args.push("/audio".to_string());
            args.push(self.audio_track.to_string());
        }
        
        // Subtitle track selection
        if self.subtitles_enabled && self.subtitle_track >= 0 {
            args.push("/sub".to_string());
            args.push(self.subtitle_track.to_string());
        } else if !self.subtitles_enabled {
            args.push("/nosub".to_string());
        }
        
        // Volume
        args.push("/volume".to_string());
        args.push(self.volume.to_string());
        
        // Netflix-style fullscreen playback
        args.push("/fullscreen".to_string());
        args.push("/close".to_string());  // Close when done
        
        args
    }
    
    /// Build PotPlayer arguments based on settings.
    pub fn build_potplayer_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        
        // Audio track selection
        if self.audio_track >= 0 {
            args.push(format!("/audio={}", self.audio_track));
        }
        
        // Subtitle track selection
        if self.subtitles_enabled && self.subtitle_track >= 0 {
            args.push(format!("/sub={}", self.subtitle_track));
        } else if !self.subtitles_enabled {
            args.push("/nosub".to_string());
        }
        
        // Volume
        args.push(format!("/volume={}", self.volume));
        
        // Netflix-style fullscreen playback
        args.push("/fullscreen".to_string());
        
        args
    }
    
    /// Build arguments for custom player using template.
    pub fn build_custom_args(&self, url: &str, title: &str) -> Vec<String> {
        if self.custom_player_args.is_empty() {
            return vec![url.to_string()];
        }
        
        self.custom_player_args
            .replace("{url}", url)
            .replace("{title}", title)
            .replace("{volume}", &self.volume.to_string())
            .replace("{audio_track}", &self.audio_track.to_string())
            .replace("{subtitle_track}", &self.subtitle_track.to_string())
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }
    
    /// Get the executable path for the current player.
    /// Tries multiple common paths and returns the first one that exists.
    pub fn get_player_executable(&self) -> String {
        match self.player_type {
            PlayerType::Custom => self.custom_player_path.clone(),
            _ => {
                // Try each possible executable path from defaults
                for path in self.player_type.default_executables() {
                    // Check if it's a full path and exists
                    if path.contains('\\') || path.contains('/') {
                        if std::path::Path::new(path).exists() {
                            return path.to_string();
                        }
                    } else {
                        // It's just a command name, try to find it using `where` on Windows
                        #[cfg(windows)]
                        {
                            use std::os::windows::process::CommandExt;
                            if let Ok(output) = std::process::Command::new("where")
                                .arg(path)
                                .creation_flags(0x08000000) // CREATE_NO_WINDOW for this check
                                .output()
                            {
                                if output.status.success() {
                                    // Return the first found path from where output
                                    if let Ok(stdout) = String::from_utf8(output.stdout) {
                                        if let Some(first_path) = stdout.lines().next() {
                                            let trimmed = first_path.trim();
                                            if !trimmed.is_empty() {
                                                return trimmed.to_string();
                                            }
                                        }
                                    }
                                    return path.to_string();
                                }
                            }
                        }
                        
                        #[cfg(not(windows))]
                        if let Ok(output) = std::process::Command::new("which")
                            .arg(path)
                            .output()
                        {
                            if output.status.success() {
                                return path.to_string();
                            }
                        }
                    }
                }
                
                // Try additional dynamic paths based on player type
                let additional_paths = self.get_additional_player_paths();
                for path in additional_paths {
                    if std::path::Path::new(&path).exists() {
                        return path;
                    }
                }
                
                // Fallback to first option
                self.player_type.default_executable().to_string()
            }
        }
    }
    
    /// Get additional dynamic paths to search for the player executable.
    fn get_additional_player_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        
        #[cfg(windows)]
        {
            // Get user profile directory
            if let Ok(userprofile) = std::env::var("USERPROFILE") {
                let exe_name = match self.player_type {
                    PlayerType::MPV => "mpv.exe",
                    PlayerType::VLC => "vlc.exe",
                    PlayerType::MpcHc => "mpc-hc64.exe",
                    PlayerType::PotPlayer => "PotPlayerMini64.exe",
                    PlayerType::FFplay => "ffplay.exe",
                    PlayerType::Custom => return paths,
                };
                
                // Scoop installation
                paths.push(format!("{}\\scoop\\apps\\{}\\current\\{}", userprofile, 
                    match self.player_type {
                        PlayerType::MPV => "mpv",
                        PlayerType::VLC => "vlc",
                        PlayerType::MpcHc => "mpc-hc",
                        PlayerType::PotPlayer => "potplayer",
                        _ => "",
                    }, exe_name));
                
                // Chocolatey installation
                paths.push(format!("C:\\ProgramData\\chocolatey\\bin\\{}", exe_name));
            }
            
            // Local app data
            if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
                match self.player_type {
                    PlayerType::MPV => {
                        paths.push(format!("{}\\Programs\\mpv\\mpv.exe", localappdata));
                        paths.push(format!("{}\\mpv\\mpv.exe", localappdata));
                    }
                    PlayerType::VLC => {
                        paths.push(format!("{}\\Programs\\VLC\\vlc.exe", localappdata));
                    }
                    _ => {}
                }
            }
        }
        
        paths
    }
    
    /// Launch the configured player with the given URL and title.
    pub fn launch_player(&self, url: &str, title: &str, is_live: bool) -> std::io::Result<std::process::Child> {
        let executable = self.get_player_executable();
        
        // Create command - GUI players need to show their window, don't use CREATE_NO_WINDOW
        // Only use DETACHED_PROCESS to separate from our console
        #[cfg(windows)]
        let mut cmd = {
            use std::os::windows::process::CommandExt;
            let mut c = std::process::Command::new(&executable);
            // DETACHED_PROCESS = 0x00000008 - Runs independently without console
            // Don't use CREATE_NO_WINDOW (0x08000000) as it prevents GUI from showing
            c.creation_flags(0x00000008);
            c
        };
        
        #[cfg(not(windows))]
        let mut cmd = std::process::Command::new(&executable);
        
        match self.player_type {
            PlayerType::FFplay => {
                cmd.arg("-window_title").arg(title);
                for arg in self.build_ffplay_args() {
                    cmd.arg(arg);
                }
                if is_live && self.low_latency_mode {
                    cmd.arg("-probesize").arg("32");
                    cmd.arg("-analyzeduration").arg("0");
                }
                cmd.arg(url);
            }
            PlayerType::VLC => {
                cmd.arg(format!("--meta-title={}", title));
                for arg in self.build_vlc_args() {
                    cmd.arg(arg);
                }
                cmd.arg(url);
            }
            PlayerType::MPV => {
                cmd.arg(format!("--title={}", title));
                for arg in self.build_mpv_args() {
                    cmd.arg(arg);
                }
                cmd.arg(url);
            }
            PlayerType::MpcHc => {
                for arg in self.build_mpchc_args() {
                    cmd.arg(arg);
                }
                cmd.arg(url);
            }
            PlayerType::PotPlayer => {
                for arg in self.build_potplayer_args() {
                    cmd.arg(arg);
                }
                cmd.arg(url);
            }
            PlayerType::Custom => {
                for arg in self.build_custom_args(url, title) {
                    cmd.arg(arg);
                }
            }
        }
        
        cmd.spawn()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub username: String,
    pub password: String,
    pub favorites: HashSet<String>,
    pub auto_login: bool,
    /// Player settings for audio/subtitle configuration
    #[serde(default)]
    pub player_settings: PlayerSettings,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;
        Ok(home.join(".iptv_player_config.json"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            username: String::new(),
            password: String::new(),
            favorites: HashSet::new(),
            auto_login: false,
            player_settings: PlayerSettings::default(),
        }
    }
}
