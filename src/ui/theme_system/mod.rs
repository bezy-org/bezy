//! Theme system infrastructure
//!
//! This module contains the infrastructure code that powers the theming system,
//! including the core trait definitions, theme registry, JSON loading, hot reloading,
//! and runtime theme switching.
//!
//! Actual theme definitions live in ../themes/

pub mod core;
pub mod embedded_themes;
pub mod hot_reload;
pub mod json_theme;
pub mod runtime_reload;
// Temporarily disabled - requires toml crate
// pub mod hot_reload_toml;

// Theme constants and utilities
pub mod layout_constants;
pub mod theme_interface;

// Re-export commonly used items
pub use core::{get_theme_registry, BezyTheme, CurrentTheme, ThemeRegistry, ThemeVariant};
pub use json_theme::{JsonThemeManager, ToolbarBorderRadius, UiBorderRadius, WidgetBorderRadius};
pub use runtime_reload::RuntimeThemePlugin;

// Re-export constants and utilities for easy access
pub use layout_constants::*;
pub use theme_interface::*;
