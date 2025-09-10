//! Text editor sort rendering
//!
//! This module provides the rendering functions for text editor sorts.
//! Pure rendering logic only - no business logic or state management.

/// Text editor sorts are rendered by the main mesh glyph outline system
/// This function exists for compatibility but the actual rendering happens
/// automatically through the ECS query in render_mesh_glyph_outline()
pub fn render_text_editor_sorts() {
    // Text editor sorts are rendered automatically by the mesh glyph outline system
    // since they are regular Sort entities with BufferSortIndex components.
    // No additional rendering logic needed here.
}