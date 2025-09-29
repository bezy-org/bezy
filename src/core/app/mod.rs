//! Application initialization and management
//!
//! This module contains the core application setup, including:
//! - App builder functions
//! - Plugin organization
//! - Startup configuration

pub mod builder;
pub mod plugins;

// Re-export the main app creation functions for convenience
pub use builder::{create_app, create_app_with_tui};
