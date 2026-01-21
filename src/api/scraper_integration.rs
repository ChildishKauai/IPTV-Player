//! Integration with the Soccer Scraper Python utility.
//!
//! This module manages scraping of football fixtures from LiveSoccerTV
//! by invoking the Soccer-Scraper-main Python scripts.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Status of a scraping operation
#[derive(Debug, Clone)]
pub enum ScrapingStatus {
    /// Not currently scraping
    Idle,
    /// Currently scraping fixtures
    Scraping,
    /// Scraping completed successfully
    Success(String),
    /// Scraping failed with error message
    Error(String),
}

/// Manager for Python scraper integration
pub struct ScraperManager {
    scraper_dir: PathBuf,
    status: ScrapingStatus,
}

impl ScraperManager {
    /// Create a new scraper manager
    pub fn new() -> Self {
        let scraper_dir = PathBuf::from("Soccer-Scraper-main");
        Self {
            scraper_dir,
            status: ScrapingStatus::Idle,
        }
    }

    /// Get the current scraping status
    pub fn status(&self) -> ScrapingStatus {
        self.status.clone()
    }

    /// Check if scraper is available (directory exists and has required files)
    pub fn is_available(&self) -> bool {
        self.scraper_dir.exists()
            && self.scraper_dir.join("fixtures.py").exists()
            && self.scraper_dir.join("requirements.txt").exists()
    }

    /// Get database path
    pub fn get_database_path(&self) -> PathBuf {
        self.scraper_dir.join("output").join("fixtures.db")
    }

    /// Trigger a scraping operation (non-blocking)
    ///
    /// Returns immediately with Scraping status.
    /// The actual scraping happens in the background.
    pub fn trigger_scrape(&mut self) -> ScrapingStatus {
        if !self.is_available() {
            let err = "Soccer Scraper not found. Ensure Soccer-Scraper-main directory exists.".to_string();
            self.status = ScrapingStatus::Error(err.clone());
            return ScrapingStatus::Error(err);
        }

        self.status = ScrapingStatus::Scraping;
        ScrapingStatus::Scraping
    }

    /// Execute scraping in background and return immediately
    ///
    /// Call this from a background thread to actually run the scraper.
    pub fn scrape_blocking(&mut self) -> ScrapingStatus {
        if !self.is_available() {
            let err = "Soccer Scraper not found. Ensure Soccer-Scraper-main directory exists.".to_string();
            self.status = ScrapingStatus::Error(err.clone());
            return self.status.clone();
        }

        // Detect Python executable
        let python_cmd = if cfg!(windows) { "python" } else { "python3" };

        // Try to run: python fixtures.py scrape
        let result = Command::new(python_cmd)
            .current_dir(&self.scraper_dir)
            .arg("fixtures.py")
            .arg("scrape")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    let message = format!(
                        "Scraping completed successfully. Database: {:?}",
                        self.get_database_path()
                    );
                    self.status = ScrapingStatus::Success(message.clone());
                    ScrapingStatus::Success(message)
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let error_msg = format!("Scraping failed: {}", stderr);
                    self.status = ScrapingStatus::Error(error_msg.clone());
                    ScrapingStatus::Error(error_msg)
                }
            }
            Err(e) => {
                let error_msg = format!(
                    "Failed to run scraper. Ensure Python is installed and in PATH. Error: {}",
                    e
                );
                self.status = ScrapingStatus::Error(error_msg.clone());
                ScrapingStatus::Error(error_msg)
            }
        }
    }

    /// Query today's fixtures
    pub fn query_today(&self) -> Result<String, String> {
        self.query_command(&["today"])
    }

    /// Query tomorrow's fixtures
    pub fn query_tomorrow(&self) -> Result<String, String> {
        self.query_command(&["tomorrow"])
    }

    /// Query fixtures by country
    pub fn query_country(&self, country: &str) -> Result<String, String> {
        self.query_command(&["country", country])
    }

    /// Generic command query
    fn query_command(&self, args: &[&str]) -> Result<String, String> {
        if !self.is_available() {
            return Err("Soccer Scraper not found".to_string());
        }

        let python_cmd = if cfg!(windows) { "python" } else { "py" };

        let mut cmd = Command::new(python_cmd);
        cmd.current_dir(&self.scraper_dir)
            .arg("fixtures.py")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Err(String::from_utf8_lossy(&output.stderr).to_string())
                }
            }
            Err(e) => Err(format!("Failed to run query: {}", e)),
        }
    }
}

impl Default for ScraperManager {
    fn default() -> Self {
        Self::new()
    }
}
