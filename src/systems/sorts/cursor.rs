//! Text editor cursor management
//!
//! This module provides cursor management for the text editor, handling both
//! cursor position calculation and coordination with visual rendering.

#![allow(clippy::too_many_arguments)]

use crate::core::state::text_editor::{SortData, SortKind, SortLayoutMode, TextEditorState};
use crate::core::state::{AppState, TextEditorState as CoreTextEditorState};
use crate::rendering::entity_pools::EntityPools;
use crate::rendering::text_cursor::{self, CursorRenderingState};
use crate::ui::edit_mode_toolbar::text::TextPlacementMode;
use bevy::prelude::*;
use bevy::sprite::ColorMaterial;

// ============================================================================
// CURSOR POSITION CALCULATION
// ============================================================================

#[derive(Debug)]
struct BufferInfo {
    cursor_position: usize,
    root_position: Vec2,
    layout_mode: crate::core::state::text_editor::SortLayoutMode,
    buffer_id: crate::core::state::text_editor::buffer::BufferId,
}

/// Extract active buffer information from ECS queries
fn get_active_buffer_info(
    active_buffer: &Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
    buffer_query: &Query<(
        &crate::core::state::text_editor::text_buffer::TextBuffer,
        &crate::core::state::text_editor::text_buffer::BufferCursor,
    )>,
) -> Option<BufferInfo> {
    let active_buffer_res = active_buffer.as_ref()?;
    let buffer_entity = active_buffer_res.buffer_entity?;
    let (text_buffer, buffer_cursor) = buffer_query.get(buffer_entity).ok()?;

    warn!(
        "🎯 CURSOR: Using buffer entity {:?}, cursor: {}, layout: {:?}",
        buffer_entity, buffer_cursor.position, text_buffer.layout_mode
    );

    Some(BufferInfo {
        cursor_position: buffer_cursor.position,
        root_position: text_buffer.root_position,
        layout_mode: text_buffer.layout_mode.clone(),
        buffer_id: text_buffer.id,
    })
}

/// Get line height from font metrics
fn get_line_height(
    app_state: &Option<Res<AppState>>,
) -> f32 {
    if let Some(app_state) = app_state.as_ref() {
        let font_metrics = &app_state.workspace.info.metrics;
        let upm = font_metrics.units_per_em as f32;
        let descender = font_metrics.descender.unwrap_or(-256.0) as f32;
        upm - descender
    } else {
        1024.0 // Reasonable fallback
    }
}

/// Collect all sorts that belong to the specific buffer
fn collect_buffer_sorts(
    text_editor_state: &TextEditorState,
    buffer_id: crate::core::state::text_editor::buffer::BufferId,
) -> Vec<&SortData> {
    let sorts: Vec<&SortData> = text_editor_state
        .buffer
        .iter()
        .filter(|sort| sort.buffer_id == Some(buffer_id))
        .collect();

    warn!(
        "🔍 COLLECT BUFFER SORTS: Found {} sorts for buffer_id {:?}",
        sorts.len(),
        buffer_id.0
    );

    if sorts.is_empty() {
        warn!("  ⚠️ BUFFER IS EMPTY - no sorts to display");
    }

    for (i, sort) in sorts.iter().enumerate() {
        warn!("  🔄 LOOP ITERATION {}: Examining sort...", i);
        match &sort.kind {
            SortKind::Glyph { glyph_name, advance_width, .. } => {
                warn!(
                    "  [{}] GLYPH='{}', advance_width={:.1}, root_pos=({:.1},{:.1})",
                    i, glyph_name, advance_width, sort.root_position.x, sort.root_position.y
                );
            }
            SortKind::LineBreak => {
                warn!("  [{}] LINE BREAK, root_pos=({:.1},{:.1})", i, sort.root_position.x, sort.root_position.y);
            }
        }
    }

    sorts
}

/// Calculate cursor offset based on layout mode and cursor position
///
/// This is a thin wrapper around the unified text flow positioning system.
/// The cursor position is where new text will be inserted, so we calculate
/// the offset for that index in the buffer.
fn calculate_cursor_offset(
    buffer_sorts: &[&SortData],
    cursor_position: usize,
    layout_mode: &SortLayoutMode,
    line_height: f32,
) -> Vec2 {
    // Use the shared positioning function - single source of truth
    let offset = crate::systems::sorts::text_flow_positioning::calculate_text_flow_offset(
        buffer_sorts,
        cursor_position,
        line_height,
        layout_mode,
    );

    warn!(
        "🎯 CURSOR OFFSET: cursor_position={}, layout={:?}, offset=({:.1}, {:.1})",
        cursor_position, layout_mode, offset.x, offset.y
    );

    offset
}


/// Calculate cursor position using buffer entity system with full feature support
pub fn calculate_cursor_position(
    text_editor_state: &TextEditorState,
    app_state: &Option<Res<AppState>>,
    buffer_query: &Query<(
        &crate::core::state::text_editor::text_buffer::TextBuffer,
        &crate::core::state::text_editor::text_buffer::BufferCursor,
    )>,
    active_buffer: &Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
) -> Option<Vec2> {
    let buffer_info = get_active_buffer_info(active_buffer, buffer_query)?;
    let line_height = get_line_height(app_state);
    let buffer_sorts = collect_buffer_sorts(text_editor_state, buffer_info.buffer_id);

    let offset = calculate_cursor_offset(
        &buffer_sorts,
        buffer_info.cursor_position,
        &buffer_info.layout_mode,
        line_height,
    );

    let final_position = buffer_info.root_position + offset;

    warn!(
        "🎯 FINAL CURSOR POSITION: root=({:.1}, {:.1}) + offset=({:.1}, {:.1}) = ({:.1}, {:.1})",
        buffer_info.root_position.x, buffer_info.root_position.y,
        offset.x, offset.y,
        final_position.x, final_position.y
    );

    Some(final_position)
}

// ============================================================================
// CURSOR RENDERING SYSTEM
// ============================================================================

/// System to manage text editor cursor rendering (internal)
pub(crate) fn render_text_editor_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    text_editor_state: Option<Res<CoreTextEditorState>>,
    current_placement_mode: Res<TextPlacementMode>,
    app_state: Option<Res<AppState>>,
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
    let current_camera_scale = camera_scale.scale_factor();

    // Get cursor position from active buffer entity
    let current_buffer_cursor_position = active_buffer
        .as_ref()
        .and_then(|active| active.buffer_entity)
        .and_then(|buffer_entity| {
            buffer_query
                .get(buffer_entity)
                .ok()
                .map(|(_, cursor)| cursor.position)
        });

    // Calculate current cursor position using business logic
    let current_cursor_position = text_editor_state.as_ref().and_then(|state| {
        calculate_cursor_position(
            state,
            &app_state,
            &buffer_query,
            &active_buffer,
        )
    });

    // Check if anything changed (change detection optimization)
    let tool_changed = cursor_state.last_tool.as_deref() != current_tool_name;
    let placement_mode_changed =
        cursor_state.last_placement_mode != Some(current_placement_mode_value);
    let buffer_cursor_changed =
        cursor_state.last_buffer_cursor_position != current_buffer_cursor_position;
    let cursor_position_changed = cursor_state.last_cursor_position != current_cursor_position;
    let camera_scale_changed = cursor_state.last_camera_scale != Some(current_camera_scale);

    if !tool_changed
        && !placement_mode_changed
        && !buffer_cursor_changed
        && !cursor_position_changed
        && !camera_scale_changed
    {
        warn!("🔒 CURSOR RENDERING: No changes detected, skipping render");
        return; // No changes, skip rendering
    }

    warn!(
        "🔄 CURSOR RENDERING: Changes detected - tool:{} placement:{} buffer_cursor:{} cursor_pos:{} camera:{}",
        tool_changed, placement_mode_changed, buffer_cursor_changed, cursor_position_changed, camera_scale_changed
    );
    warn!(
        "   Last cursor pos: {:?}, Current cursor pos: {:?}",
        cursor_state.last_cursor_position, current_cursor_position
    );
    warn!(
        "   Last buffer cursor: {:?}, Current buffer cursor: {:?}",
        cursor_state.last_buffer_cursor_position, current_buffer_cursor_position
    );

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
            let (upm, descender) = if let Some(app_state) = app_state.as_ref() {
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
