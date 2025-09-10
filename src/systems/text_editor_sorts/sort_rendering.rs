//! Text editor cursor management
//!
//! This module provides cursor management for the text editor, coordinating
//! between cursor position calculation and visual rendering.

mod cursor_calculation;
mod cursor_system;

/// Text editor sorts are rendered by the main mesh glyph outline system
/// This function exists for compatibility but the actual rendering happens
/// automatically through the ECS query in render_mesh_glyph_outline()
pub fn render_text_editor_sorts() {
    // Text editor sorts are rendered automatically by the mesh glyph outline system
    // since they are regular Sort entities with BufferSortIndex components.
    // No additional rendering logic needed here.
}

// Re-export the main cursor rendering system
pub use cursor_system::render_text_editor_cursor;