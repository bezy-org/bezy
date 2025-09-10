//! Text editor cursor system
//!
//! This module provides the main system for managing text cursor rendering,
//! coordinating between cursor position calculation and visual rendering.

#![allow(clippy::too_many_arguments)]

use crate::core::state::{AppState, TextEditorState};
use crate::rendering::text_cursor::{self, CursorRenderingState};
use crate::rendering::entity_pools::EntityPools;
use crate::ui::edit_mode_toolbar::text::TextPlacementMode;
use bevy::prelude::*;
use bevy::sprite::ColorMaterial;

use super::cursor_calculation;

/// System to manage text editor cursor rendering
pub fn render_text_editor_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    text_editor_state: Option<Res<TextEditorState>>,
    current_placement_mode: Res<TextPlacementMode>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    camera_scale: Res<crate::rendering::zoom_aware_scaling::CameraResponsiveScale>,
    mut cursor_state: ResMut<CursorRenderingState>,
    mut entity_pools: ResMut<EntityPools>,
    buffer_query: Query<(
        &crate::core::state::text_editor::TextBuffer,
        &crate::core::state::text_editor::BufferCursor,
    )>,
    active_buffer: Option<Res<crate::core::state::text_editor::ActiveTextBuffer>>,
) {
    // Only render cursor when Text tool is active AND in Insert mode
    let should_show_cursor = current_tool.get_current() == Some("text")
        && matches!(*current_placement_mode, TextPlacementMode::Insert);
        
    if !should_show_cursor {
        text_cursor::clear_cursor_entities(&mut commands, &mut entity_pools);
        return;
    }

    // Get current state for change detection
    let current_tool_name = current_tool.get_current();
    let current_placement_mode_value = *current_placement_mode;
    let current_camera_scale = camera_scale.scale_factor;
    
    // Get cursor position from active buffer entity
    let current_buffer_cursor_position = active_buffer
        .as_ref()
        .and_then(|active| active.buffer_entity)
        .and_then(|buffer_entity| {
            buffer_query.get(buffer_entity).ok().map(|(_, cursor)| cursor.position)
        });

    // Calculate current cursor position using business logic
    let current_cursor_position = text_editor_state.as_ref().and_then(|state| {
        cursor_calculation::calculate_cursor_position(
            state,
            &app_state,
            &fontir_app_state,
            &buffer_query,
            &active_buffer,
        )
    });

    // Check if anything changed (change detection optimization)
    let tool_changed = cursor_state.last_tool.as_deref() != current_tool_name;
    let placement_mode_changed = cursor_state.last_placement_mode != Some(current_placement_mode_value);
    let buffer_cursor_changed = cursor_state.last_buffer_cursor_position != current_buffer_cursor_position;
    let cursor_position_changed = cursor_state.last_cursor_position != current_cursor_position;
    let camera_scale_changed = cursor_state.last_camera_scale != Some(current_camera_scale);

    if !tool_changed && !placement_mode_changed && !buffer_cursor_changed 
        && !cursor_position_changed && !camera_scale_changed {
        return; // No changes, skip rendering
    }

    // Clear existing cursor entities before re-rendering
    text_cursor::clear_cursor_entities(&mut commands, &mut entity_pools);

    // Update state tracking
    cursor_state.last_tool = current_tool_name.map(|s| s.to_string());
    cursor_state.last_placement_mode = Some(current_placement_mode_value);
    cursor_state.last_buffer_cursor_position = current_buffer_cursor_position;
    cursor_state.last_cursor_position = current_cursor_position;
    cursor_state.last_camera_scale = Some(current_camera_scale);

    // Render cursor if we have a valid position
    if let Some(cursor_world_pos) = current_cursor_position {
        if text_editor_state.is_some() {
            // Get font metrics for proper cursor height
            let (upm, descender) = if let Some(fontir_state) = fontir_app_state.as_ref() {
                let metrics = fontir_state.get_font_metrics();
                (metrics.units_per_em, metrics.descender.unwrap_or(-256.0))
            } else if let Some(app_state) = app_state.as_ref() {
                let font_metrics = &app_state.workspace.info.metrics;
                (
                    font_metrics.units_per_em as f32,
                    font_metrics.descender.unwrap_or(-256.0) as f32,
                )
            } else {
                return; // No font metrics available
            };

            // Render the cursor using the rendering module
            text_cursor::render_cursor_at_position(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut entity_pools,
                cursor_world_pos,
                upm,
                descender,
                &camera_scale,
            );
        }
    }
}