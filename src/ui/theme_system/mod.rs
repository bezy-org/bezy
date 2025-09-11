//! Theme system infrastructure
//!
//! This module contains the infrastructure code that powers the theming system,
//! including the core trait definitions, theme registry, JSON loading, hot reloading, 
//! and runtime theme switching.
//! 
//! Actual theme definitions live in ../themes/

pub mod core;
pub mod json_theme;
pub mod runtime_reload;
pub mod hot_reload;
// Temporarily disabled - requires toml crate
// pub mod hot_reload_toml;

// Theme constants and utilities
pub mod legacy_constants;
pub mod theme_interface;
pub mod layout_constants;

// Re-export commonly used items
pub use core::{BezyTheme, ThemeRegistry, ThemeVariant, CurrentTheme, get_theme_registry};
pub use json_theme::{JsonThemeManager, WidgetBorderRadius, ToolbarBorderRadius, UiBorderRadius};
pub use runtime_reload::RuntimeThemePlugin;

// Re-export constants and utilities for easy access
pub use legacy_constants::*;
pub use theme_interface::*;
pub use layout_constants::*;