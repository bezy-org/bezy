//! Core application functionality
//!
//! This module contains the core application logic, including:
//! - Application initialization and configuration
//! - State management
//! - Settings and CLI handling
//! - Pointer and coordinate management
//! - Input system

pub mod app;
pub mod config;
pub mod errors;
pub mod platform;
pub mod runner;
pub mod state;
pub mod tui_communication;

// Re-export commonly used items
pub use app::{create_app, create_app_with_tui};
pub use config::{BezySettings, CliArgs, ConfigFile};
pub use runner::run_app;
pub use state::{AppState, GlyphNavigation};
