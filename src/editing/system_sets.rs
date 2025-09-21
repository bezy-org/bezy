//! System Sets for Font Editor
//!
//! This module defines the execution order for font editor systems using Bevy's SystemSet pattern.
//! This prevents race conditions and ensures predictable system execution order.

use bevy::prelude::*;

/// System sets that define the execution order for font editor operations
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum FontEditorSets {
    /// Handle keyboard and mouse input
    Input,

    /// Update text buffer state (add/remove characters)
    TextBuffer,

    /// Synchronize ECS entities with buffer state (spawn/despawn sorts)
    EntitySync,

    /// Create visual elements (metrics, outlines, points)
    Rendering,

    /// Clean up orphaned entities and resources
    Cleanup,
}

/// Plugin to configure system set ordering
pub struct FontEditorSystemSetsPlugin;

impl Plugin for FontEditorSystemSetsPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                FontEditorSets::Input,
                FontEditorSets::TextBuffer,
                FontEditorSets::EntitySync,
                FontEditorSets::Rendering,
                FontEditorSets::Cleanup,
            )
                .chain(),
        );

        debug!("FontEditor system sets configured with guaranteed execution order");
    }
}
