//! Application configuration management
//!
//! This module handles all configuration aspects:
//! - CLI arguments parsing
//! - User configuration files
//! - Application settings

pub mod cli;
pub mod settings;
pub mod user_config;

// Simple, clear re-exports
pub use cli::CliArgs;
pub use settings::{BezySettings, DEFAULT_WINDOW_SIZE, WINDOW_TITLE};
pub use user_config::ConfigFile;
