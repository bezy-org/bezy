//! Text Editor Plugin
//!
//! This plugin replaces the old entity-based sort system with a text editor
//! approach that treats sorts as a linear buffer with cursor navigation.

use crate::systems::sorts::{
    despawn_inactive_sort_points_optimized, // NEW: Optimized instant point despawning
    despawn_missing_buffer_sort_entities,   // NEW: Despawn deleted buffer sorts
    detect_sort_glyph_changes, // NEW: Detect glyph changes and force point regeneration
    handle_arabic_text_input,  // NEW: Arabic and Unicode text input
    handle_sort_placement_input, // NEW: Uses centralized input system
    handle_text_editor_keyboard_input,
    handle_unicode_text_input, // NEW: Unicode character input using Bevy events
    initialize_rtl_shaping,    // NEW: Initialize RTL shaping resources
    initialize_text_editor_sorts,
    manage_sort_activation, // NEW: ECS-based sort activation management
    regenerate_points_on_fontir_change, // NEW: Regenerate points when FontIR data changes
    spawn_active_sort_points_optimized, // NEW: Optimized instant point spawning
    spawn_missing_sort_entities, // NEW: Spawn ECS entities for buffer sorts
    sync_buffer_sort_activation_state, // NEW: Sync activation state from buffer to entities
};

use bevy::prelude::*;

pub struct TextEditorPlugin;

impl Plugin for TextEditorPlugin {
    fn build(&self, app: &mut App) {
        debug!("[TextEditorPlugin] Building plugin...");
        app
            // Initialize resources
            .init_resource::<crate::core::state::text_editor::TextEditorState>()
            .init_resource::<crate::rendering::CursorRenderingState>()
            .init_resource::<crate::core::state::text_editor::ActiveSortEntity>()
            // Add buffer manager plugin
            .add_plugins(crate::systems::TextBufferManagerPlugin)
            // Initialize text editor state
            .add_systems(
                Startup,
                (
                    initialize_text_editor_sorts,
                    initialize_rtl_shaping, // Initialize RTL shaping resources
                ),
            )
            // Input handling
            .add_systems(
                Update,
                (
                    handle_unicode_text_input,
                    handle_arabic_text_input, // Handle Arabic text input with shaping
                    handle_sort_placement_input,
                )
                    .in_set(super::FontEditorSets::Input),
            )
            // Text buffer updates
            .add_systems(
                Update,
                (
                    spawn_missing_sort_entities,
                    sync_buffer_sort_activation_state, // NEW: Sync activation state after spawning
                    crate::systems::sorts::sort_entities::update_buffer_sort_positions,
                    crate::systems::sorts::sort_entities::auto_activate_selected_sorts,
                    manage_sort_activation,
                )
                    .chain()
                    .in_set(super::FontEditorSets::EntitySync),
            )
            // Entity spawning/despawning - must run AFTER sort entity management
            .add_systems(
                Update,
                (
                    detect_sort_glyph_changes, // Detect glyph changes and trigger point regeneration
                    spawn_active_sort_points_optimized,
                    despawn_inactive_sort_points_optimized,
                    regenerate_points_on_fontir_change, // Regenerate when FontIR data changes
                )
                    .chain()
                    .in_set(super::FontEditorSets::EntitySync)
                    .after(manage_sort_activation), // Ensure points spawn after sort activation
            )
            // Rendering systems
            .add_systems(
                Update,
                (crate::systems::sorts::cursor::render_text_editor_cursor,)
                    .in_set(super::FontEditorSets::Rendering),
            )
            // Cleanup systems (the old cleanup system is now replaced by component-relationship cleanup)
            .add_systems(
                Update,
                despawn_missing_buffer_sort_entities.in_set(super::FontEditorSets::Cleanup),
            );
    }
}
