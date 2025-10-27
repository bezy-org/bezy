//! Theme definitions for the Bezy font editor
//!
//! This module contains all theme implementations as simple Rust structs.
//! To create a new theme, copy an existing theme file and modify the colors.

pub mod dark;
pub mod light;
pub mod strawberry;
pub mod campfire;

pub use dark::DarkTheme;
pub use light::LightTheme;
pub use strawberry::StrawberryTheme;
pub use campfire::CampfireTheme;

pub use crate::ui::theme_system::{
    get_theme_registry, BezyTheme, CurrentTheme, ThemeRegistry, ThemeVariant,
};
