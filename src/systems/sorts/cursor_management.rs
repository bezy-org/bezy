//! Text editor cursor management
//!
//! This module provides cursor management for the text editor, coordinating
//! between cursor position calculation and visual rendering.

mod cursor_calculation;
mod cursor_system;

// Re-export the main cursor rendering system
pub use cursor_system::render_text_editor_cursor;