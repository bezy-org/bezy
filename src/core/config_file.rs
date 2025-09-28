//! User configuration file handling
//!
//! Manages settings from ~/.config/bezy/settings.json

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

/// User configuration from ~/.config/bezy/settings.json
///
/// These settings override built-in defaults but are overridden by CLI arguments
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    /// Default theme to use (e.g., "dark", "light", "strawberry")
    pub default_theme: Option<String>,
    // Additional settings can be added here in the future
    // Examples could include:
    // - default_font_directory: Option<PathBuf>
    // - auto_save_interval: Option<u64>
    // - default_grid_settings: Option<GridSettings>
}

impl ConfigFile {
    /// Get the path to the user config file
    pub fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));
        config_dir.join("bezy").join("settings.json")
    }

    /// Get the path to the bezy config directory
    pub fn config_dir() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));
        config_dir.join("bezy")
    }

    /// Get the path to the logs directory
    pub fn logs_dir() -> PathBuf {
        Self::config_dir().join("logs")
    }

    /// Get the path to the current log file
    pub fn current_log_file() -> PathBuf {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d");
        Self::logs_dir().join(format!("bezy-{}.log", timestamp))
    }

    /// Initialize the logs directory
    pub fn initialize_logs_directory() -> anyhow::Result<()> {
        let logs_dir = Self::logs_dir();
        fs::create_dir_all(&logs_dir)?;
        debug!("Created logs directory: {:?}", logs_dir);
        Ok(())
    }

    /// Load configuration from the user config file
    pub fn load() -> Option<Self> {
        let path = Self::config_path();

        if !path.exists() {
            return None;
        }

        match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(config) => {
                    debug!("Loaded user settings from {:?}", path);
                    Some(config)
                }
                Err(e) => {
                    warn!("Failed to parse settings.json: {}", e);
                    None
                }
            },
            Err(e) => {
                warn!("Failed to read settings.json: {}", e);
                None
            }
        }
    }

    /// Save configuration to the user config file
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&path, contents)?;

        debug!("Saved settings to {:?}", path);
        Ok(())
    }

    /// Initialize the complete user configuration directory
    ///
    /// This creates:
    /// 1. The ~/.config/bezy directory structure
    /// 2. A settings.json file with default values
    /// 3. A themes/ directory with copies of all embedded themes
    pub fn initialize_config_directory() -> anyhow::Result<()> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("bezy");

        // Create the main config directory
        fs::create_dir_all(&config_dir)?;
        println!("Created config directory: {:?}", config_dir);

        // Create logs directory
        let logs_dir = config_dir.join("logs");
        fs::create_dir_all(&logs_dir)?;
        println!("Created logs directory: {:?}", logs_dir);

        // Create settings.json
        let settings_path = config_dir.join("settings.json");
        if !settings_path.exists() {
            let example = ConfigFile {
                default_theme: Some("dark".to_string()),
            };
            example.save()?;
            println!("Created settings file: {:?}", settings_path);
        } else {
            println!("Settings file already exists: {:?}", settings_path);
        }

        // Create themes directory and copy embedded themes
        let themes_dir = config_dir.join("themes");
        fs::create_dir_all(&themes_dir)?;
        println!("Created themes directory: {:?}", themes_dir);

        // Copy all embedded themes to the user directory
        use crate::ui::theme_system::embedded_themes;
        for (name, content) in embedded_themes::get_embedded_themes() {
            let theme_path = themes_dir.join(format!("{}.json", name));
            if !theme_path.exists() {
                fs::write(&theme_path, content)?;
                println!("  - Copied theme: {}.json", name);
            } else {
                println!("  - Theme already exists: {}.json", name);
            }
        }

        println!("\nConfiguration initialized successfully!");
        println!("You can now:");
        println!("  - Edit settings at: {:?}", settings_path);
        println!("  - Customize themes in: {:?}", themes_dir);
        println!("  - View application logs in: {:?}", logs_dir);

        Ok(())
    }

    /// Set up log redirection to ~/.config/bezy/logs/
    /// Used when running without TUI to capture logs to file
    pub fn setup_log_redirection() -> anyhow::Result<()> {
        // Check if config directory exists - if not, we'll fail gracefully
        let config_dir = Self::config_dir();
        if !config_dir.exists() {
            // Config directory doesn't exist, so don't try to create logs
            return Err(anyhow::anyhow!("Config directory doesn't exist"));
        }

        // Initialize logs directory
        Self::initialize_logs_directory()?;

        // Get the log file path
        let log_file_path = Self::current_log_file();

        // Create/open the log file - use truncate instead of append for single log file
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_file_path)?;

        // Redirect stdout and stderr to the log file
        unsafe {
            libc::dup2(log_file.as_raw_fd(), libc::STDOUT_FILENO);
            libc::dup2(log_file.as_raw_fd(), libc::STDERR_FILENO);
        }

        // Print initial log message to confirm redirection
        println!(
            "=== Bezy started at {} ===",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!("Logs redirected to: {:?}", log_file_path);

        Ok(())
    }
}
