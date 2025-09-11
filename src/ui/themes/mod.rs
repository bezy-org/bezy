//! Theme definitions for the Bezy font editor
//!
//! This module contains all the actual theme implementations. The theming system
//! infrastructure lives in `../theme_system/`.

#![allow(clippy::uninlined_format_args)]
//!
//! ## Creating a Custom Theme - Super Easy! 🎨
//!
//! To create a custom theme:
//! 1. Create a new `.rs` file in `src/ui/themes/` (e.g., `ocean.rs`)
//! 2. Copy the template below and customize colors
//! 3. Add `pub mod ocean;` to this file
//! 4. Your theme is automatically available!
//!
//! ```rust
//! use bevy::prelude::*;
//! use crate::ui::theme_system::BezyTheme;
//!
//! pub struct OceanTheme;
//! impl BezyTheme for OceanTheme {
//!     fn name(&self) -> &'static str { "Ocean" }
//!     fn background_color(&self) -> Color { Color::srgb(0.05, 0.15, 0.25) }
//!     // ... customize other colors
//! }
//! ```
//!
//! ## Theme Structure
//!
//! Themes are organized into logical groups:
//! - **Typography**: Font paths, sizes, and text colors
//! - **Layout**: Spacing, margins, padding, and dimensions
//! - **Colors**: All color constants used throughout the app
//! - **Rendering**: Glyph points, paths, selections, and tools
//! - **UI Components**: Buttons, toolbars, panels, and widgets
//! - **Interaction**: Hover states, selection feedback, and tool previews

// Import all theme implementations
pub mod campfire;
pub mod darkmode;
pub mod lightmode;
pub mod strawberry;

// Re-export core theming infrastructure from theme_system
pub use crate::ui::theme_system::{
    BezyTheme, ThemeRegistry, ThemeVariant, CurrentTheme, get_theme_registry,
    WidgetBorderRadius, ToolbarBorderRadius, UiBorderRadius
};
