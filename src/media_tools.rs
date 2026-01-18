//! Media tools bundling support for Steam Deck
//! 
//! Handles detection and execution of bundled ffmpeg, vlc, and mpv
//! for Steam Deck deployments.

use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, Context};

/// Bundled media tools available in Steam Deck builds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaTool {
    FFmpeg,
    VLC,
    MPV,
}

impl MediaTool {
    pub fn name(&self) -> &'static str {
        match self {
            MediaTool::FFmpeg => "ffmpeg",
            MediaTool::VLC => "vlc",
            MediaTool::MPV => "mpv",
        }
    }
}

/// Manager for bundled media tools
pub struct MediaToolsManager {
    bundled_dir: PathBuf,
    is_bundled: bool,
}

impl MediaToolsManager {
    /// Create a new media tools manager
    pub fn new() -> Self {
        let is_bundled = cfg!(feature = "bundle-media-tools");
        let bundled_dir = Self::get_bundled_dir();
        
        MediaToolsManager {
            bundled_dir,
            is_bundled,
        }
    }
    
    /// Get the bundled media tools directory
    fn get_bundled_dir() -> PathBuf {
        // Try relative path from executable first (for bundled distributions)
        if let Ok(exe_path) = std::env::current_exe() {
            let bundle_path = exe_path.parent()
                .map(|p| p.join("media-tools"))
                .filter(|p| p.exists());
            
            if bundle_path.is_some() {
                return bundle_path.unwrap();
            }
        }
        
        // Fall back to standard system paths
        PathBuf::from("/usr/bin")
    }
    
    /// Check if a specific media tool is available
    pub fn is_available(&self, tool: MediaTool) -> bool {
        self.get_tool_path(tool).exists()
    }
    
    /// Get the full path to a media tool
    pub fn get_tool_path(&self, tool: MediaTool) -> PathBuf {
        if self.is_bundled {
            self.bundled_dir.join(tool.name())
        } else {
            // Fall back to system PATH
            PathBuf::from(tool.name())
        }
    }
    
    /// Execute a media tool with arguments
    pub fn execute(&self, tool: MediaTool, args: &[&str]) -> Result<std::process::Output> {
        let path = self.get_tool_path(tool);
        
        if !path.exists() && self.is_bundled {
            return Err(anyhow::anyhow!(
                "Bundled {} not found at {:?}. Bundled media tools may not be installed.",
                tool.name(),
                path
            ));
        }
        
        Command::new(&path)
            .args(args)
            .output()
            .context(format!("Failed to execute {}", tool.name()))
    }
    
    /// Get all available tools
    pub fn available_tools(&self) -> Vec<MediaTool> {
        vec![MediaTool::FFmpeg, MediaTool::VLC, MediaTool::MPV]
            .into_iter()
            .filter(|tool| self.is_available(*tool))
            .collect()
    }
    
    /// Check if this is a bundled build
    pub fn is_bundled_build(&self) -> bool {
        self.is_bundled
    }
}

impl Default for MediaToolsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_media_tool_names() {
        assert_eq!(MediaTool::FFmpeg.name(), "ffmpeg");
        assert_eq!(MediaTool::VLC.name(), "vlc");
        assert_eq!(MediaTool::MPV.name(), "mpv");
    }
    
    #[test]
    fn test_manager_creation() {
        let manager = MediaToolsManager::new();
        // Should not panic
        let _ = manager.available_tools();
    }
}
