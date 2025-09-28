//! Core application functionality
//!
//! This module contains the core application logic, including:
//! - Application initialization and configuration
//! - State management
//! - Settings and CLI handling
//! - Pointer and coordinate management
//! - Input system

pub mod app;
pub mod cli;
pub mod config_file;
pub mod errors;
pub mod io;
pub mod platform;
pub mod runner;
pub mod settings;
pub mod state;
pub mod tui_communication;

// Re-export commonly used items
pub use app::{create_app, create_app_with_tui};
pub use cli::CliArgs;
pub use io::input::{helpers, InputEvent, InputState};
pub use io::pointer::{PointerInfo, PointerPlugin};
pub use runner::run_app;
pub use settings::BezySettings;
pub use state::{AppState, GlyphNavigation};
