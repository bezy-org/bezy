//! UI Theme Interface
//!
//! This file serves as the main entry point for the theme system.
//! All theme-related constants, functions, and utilities are now organized
//! in the theme_system directory and re-exported here for compatibility.
//!
//! For creating custom themes, see the themes/ directory and docs/THEME_CREATION_GUIDE.md

// Re-export the theme system and all organized constants
pub use crate::ui::theme_system::*;
pub use crate::ui::themes::*;
