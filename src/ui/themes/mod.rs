//! Theme definitions for the Bezy font editor
//!
//! This module contains all the actual theme implementations. The theming system
//! infrastructure lives in `../theme_system/`.

#![allow(clippy::uninlined_format_args)]
//!
//! ## Creating a Custom Theme - Super Easy! 🎨
//!
//! To create a custom theme:
//! 1. Create a new `.json` file in this directory (e.g., `ocean.json`)
//! 2. Copy an existing theme JSON and customize the colors
//! 3. Your theme is automatically available!
//! 4. Enjoy hot reloading - changes appear instantly!
//!
//! Example JSON structure:
//! ```json
//! {
//!   "name": "Ocean",
//!   "background": [0.05, 0.15, 0.25],
//!   "normal_text": [0.8, 0.9, 1.0],
//!   // ... other color values
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

// All themes are now JSON-based for easy editing and hot reloading
// Theme files: dark.json, light.json, campfire.json, strawberry.json

// Re-export core theming infrastructure from theme_system
pub use crate::ui::theme_system::{
    get_theme_registry, BezyTheme, CurrentTheme, ThemeRegistry, ThemeVariant, ToolbarBorderRadius,
    UiBorderRadius, WidgetBorderRadius,
};
