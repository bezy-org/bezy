//! Sort placement handling for text editor
//!
//! This module handles mouse-based sort placement when using text tools.

#![allow(clippy::too_many_arguments)]

use crate::rendering::checkerboard::calculate_dynamic_grid_size;
use crate::ui::themes::CurrentTheme;
use bevy::prelude::*;

/// Handle sort placement input (mouse clicks in text modes)
pub fn handle_sort_placement_input(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<
        (&Camera, &GlobalTransform, &Projection),
        With<crate::rendering::cameras::DesignCamera>,
    >,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    mut current_placement_mode: ResMut<crate::ui::edit_mode_toolbar::text::TextPlacementMode>,
    mut text_editor_state: ResMut<crate::core::state::TextEditorState>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    theme: Res<CurrentTheme>,
) {
    use crate::ui::edit_mode_toolbar::text::TextPlacementMode;

    // Only handle input when text tool is active
    let current_tool_name = current_tool.get_current();
    if current_tool_name != Some("text") {
        return;
    }

    debug!("🖱️ SORT PLACEMENT: ✅ Text tool is active, checking other conditions...");

    // Only handle text placement modes, not insert mode
    match *current_placement_mode {
        TextPlacementMode::LTRText | TextPlacementMode::RTLText => {
            // Continue with placement
            debug!("🖱️ SORT PLACEMENT: ✅ Text tool active with placement mode {:?} - READY TO PLACE SORTS!", *current_placement_mode);
        }
        TextPlacementMode::Insert | TextPlacementMode::Freeform => {
            // These modes don't place sorts on click
            if mouse_button_input.just_pressed(MouseButton::Left) && !ui_hover_state.is_hovering_ui
            {
                debug!(
                    "🖱️ SORT PLACEMENT: Click ignored - in {:?} mode (not placement mode)",
                    *current_placement_mode
                );
            }
            return;
        }
    }

    // Don't process clicks when hovering over UI
    if ui_hover_state.is_hovering_ui {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            debug!("🖱️ SORT PLACEMENT: ⚠️  Click ignored - hovering over UI");
        }
        return;
    }

    // Check for left mouse click
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    debug!("🖱️ SORT PLACEMENT: ✅ Left mouse click detected, UI not hovering");

    debug!("🖱️ SORT PLACEMENT: Left mouse click detected - processing placement");

    // Get camera, transform, and projection
    let Ok((camera, camera_transform, projection)) = camera_query.single() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(raw_world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
    else {
        return;
    };

    // Get zoom scale from camera projection for grid snapping
    let zoom_scale = if let Projection::Orthographic(ortho) = projection {
        ortho.scale
    } else {
        1.0
    };

    // Apply grid snapping to match the preview
    let grid_size = calculate_dynamic_grid_size(zoom_scale, &theme);
    let snapped_position = (raw_world_position / grid_size).round() * grid_size;

    // Always create a new sort when clicking in placement mode
    // This allows placing multiple sorts of the same glyph or different glyphs
    let existing_sorts_count = text_editor_state.get_text_sorts().len();
    debug!(
        "🖱️ SORT PLACEMENT: Creating new sort at position ({:.1}, {:.1}), existing sorts: {}",
        snapped_position.x, snapped_position.y, existing_sorts_count
    );

    // CRITICAL FIX: Deactivate all existing sorts before creating new active sort
    // This prevents multiple active sorts from existing simultaneously
    // NOTE: Each text flow (LTR/RTL) maintains its own buffer and text flow chain
    // Buffer[0] might be LTR, Buffer[1] might be RTL - they are independent text flows
    for i in 0..text_editor_state.buffer.len() {
        if let Some(sort) = text_editor_state.buffer.get_mut(i) {
            if sort.is_active {
                debug!(
                    "🔻 SORT PLACEMENT: Deactivating existing sort - glyph '{}'",
                    sort.kind.glyph_name()
                );
                sort.is_active = false;
            }
        }
    }

    // Create a new independent sort with buffer entity
    debug!("🖱️ SORT PLACEMENT: About to call create_independent_sort_with_fontir");
    let new_buffer_entity = create_independent_sort_with_fontir(
        &mut commands,
        &mut text_editor_state,
        snapped_position,
        current_placement_mode.to_sort_layout_mode(),
        fontir_app_state.as_deref(),
    );

    // CRITICAL: Update the ActiveTextBuffer resource to point to the new buffer entity
    commands.insert_resource(
        crate::core::state::text_editor::text_buffer::ActiveTextBuffer {
            buffer_entity: Some(new_buffer_entity),
        },
    );

    debug!(
        "🖱️ SORT PLACEMENT: Set active buffer entity to {:?}",
        new_buffer_entity
    );

    // CRITICAL: Mark the text editor state as changed to trigger entity spawning
    text_editor_state.set_changed();

    // AUTO-SWITCH TO INSERT MODE: After placing LTR/RTL text buffer sorts, switch to Insert mode
    // for natural text editing UX. Freeform sorts stay in placement mode for multi-placement.
    let previous_mode = *current_placement_mode;
    match previous_mode {
        TextPlacementMode::LTRText | TextPlacementMode::RTLText => {
            *current_placement_mode = TextPlacementMode::Insert;
            debug!(
                "🖱️ SORT PLACEMENT: Auto-switched to Insert mode after placing {:?} text buffer sort",
                previous_mode
            );
        }
        TextPlacementMode::Freeform => {
            // Stay in Freeform mode for multi-placement workflow
            debug!("🖱️ SORT PLACEMENT: Staying in Freeform mode for multi-placement");
        }
        TextPlacementMode::Insert => {
            // Already in Insert mode (shouldn't happen in placement, but handle gracefully)
            debug!("🖱️ SORT PLACEMENT: Already in Insert mode");
        }
    }

    debug!("🖱️ SORT PLACEMENT: create_independent_sort_with_fontir completed");

    debug!(
        "🖱️ SORT PLACEMENT: Created new sort, total sorts now: {}",
        text_editor_state.get_text_sorts().len()
    );
}

/// Create an independent sort that can coexist with other sorts
/// This now uses the new buffer entity system for proper buffer management
fn create_independent_sort_with_fontir(
    commands: &mut Commands,
    text_editor_state: &mut crate::core::state::TextEditorState,
    world_position: bevy::math::Vec2,
    layout_mode: crate::core::state::text_editor::SortLayoutMode,
    fontir_app_state: Option<&crate::core::state::FontIRAppState>,
) -> bevy::prelude::Entity {
    use crate::core::state::text_editor::buffer::BufferId;
    use crate::core::state::text_editor::{SortData, SortKind, SortLayoutMode};
    use crate::systems::text_buffer_manager::create_text_buffer;

    debug!("🖱️ INSIDE create_independent_sort_with_fontir: Starting function");

    // Choose appropriate default glyph based on layout mode
    let (placeholder_glyph, placeholder_codepoint) =
        crate::core::state::text_editor::editor::get_default_glyph_for_direction(&layout_mode);

    let advance_width = if let Some(fontir_state) = fontir_app_state {
        fontir_state.get_glyph_advance_width(&placeholder_glyph)
    } else {
        // Fallback to reasonable default if FontIR not available
        500.0
    };

    // BUFFER SEPARATION POLICY:
    // Each click with the text tool creates a NEW independent text flow
    // This ensures clean separation between different text placement operations
    // Even if the same layout mode (RTL/LTR) exists, we create a new buffer for independence

    // NEW BUFFER ENTITY SYSTEM: Create a buffer entity first, then add sort to it

    // Create a new unique buffer ID for complete isolation
    let buffer_id = BufferId::new();

    // CURSOR POSITIONING: Start cursor after initial character for natural typing flow
    let initial_cursor_position = 1;

    // Create the buffer entity with cursor storage
    let buffer_entity = create_text_buffer(
        commands,
        buffer_id,
        layout_mode.clone(),
        world_position,
        initial_cursor_position,
    );

    debug!(
        "🖱️ Creating new {} buffer (Entity: {:?}, ID: {:?}) at position ({:.1}, {:.1})",
        match layout_mode {
            SortLayoutMode::RTLText => "RTL",
            SortLayoutMode::LTRText => "LTR",
            SortLayoutMode::Freeform => "Freeform",
        },
        buffer_entity,
        buffer_id,
        world_position.x,
        world_position.y
    );

    // CREATE INITIAL CHARACTER SORT: This provides a clear visual starting point
    // This is NOT a "root sort" - just the first character in the buffer like any other
    let initial_sort = SortData {
        kind: SortKind::Glyph {
            glyph_name: placeholder_glyph.clone(),
            codepoint: Some(placeholder_codepoint),
            advance_width,
        },
        layout_mode: layout_mode.clone(),
        is_active: true, // Make this sort active for immediate editing
        root_position: world_position,
        buffer_cursor_position: None, // Deprecated field - cursor stored in buffer entity now
        buffer_id: Some(buffer_id),   // For compatibility, though deprecated
    };

    // Insert the initial sort into the text editor buffer at index 0
    text_editor_state.buffer.insert(0, initial_sort);

    debug!(
        "📍 SORT PLACEMENT: Created buffer entity {:?} with layout_mode: {:?}, added initial '{}' character at index 0", 
        buffer_entity, layout_mode, placeholder_glyph
    );

    debug!(
        "🖱️ Created new buffer entity {:?} with cursor at position {} and initial character '{}' at world position ({:.1}, {:.1})",
        buffer_entity, initial_cursor_position, placeholder_glyph, world_position.x, world_position.y
    );

    // Return the buffer entity for the caller to use
    buffer_entity
}
