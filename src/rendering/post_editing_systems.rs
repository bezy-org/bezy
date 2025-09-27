//! Post-Editing Rendering SystemSet
//!
//! This module consolidates the system ordering for rendering systems that need
//! to run after editing operations are complete. It eliminates duplicate
//! .after() dependencies and provides a unified scheduling approach.

use bevy::prelude::*;

/// SystemSet for rendering operations that run after editing is complete
///
/// This set ensures all post-editing rendering happens in the correct order:
/// 1. After point spawning/updates (`spawn_active_sort_points_optimized`)
/// 2. After nudge input handling (`handle_nudge_input`)
/// 3. Before final frame presentation
///
/// Systems in this set:
/// - `detect_sort_changes` (glyph rendering)
/// - `render_points_with_meshes` (point rendering)
/// - `update_handle_lines` (outline rendering)
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PostEditingRenderingSet;

/// Plugin that consolidates post-editing rendering system scheduling
///
/// This replaces the individual .after() dependencies in each rendering plugin
/// with a unified SystemSet approach, reducing scheduling complexity.
pub struct PostEditingRenderingPlugin;

impl Plugin for PostEditingRenderingPlugin {
    fn build(&self, app: &mut App) {
        // Configure the system set to run after editing operations
        app.configure_sets(
            Update,
            PostEditingRenderingSet
                .after(crate::systems::sorts::spawn_active_sort_points_optimized)
                .after(crate::editing::selection::nudge::handle_nudge_input),
        );
    }
}