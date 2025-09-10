//! Text editor cursor management
//!
//! This module provides cursor management for the text editor, handling both
//! cursor position calculation and coordination with visual rendering.

#![allow(clippy::too_many_arguments)]

use crate::core::state::text_editor::{SortData, SortKind, SortLayoutMode, TextEditorState};
use crate::core::state::{AppState, TextEditorState as CoreTextEditorState};
use crate::rendering::text_cursor::{self, CursorRenderingState};
use crate::rendering::entity_pools::EntityPools;
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
    
    info!(
        "🎯 CURSOR: Using buffer entity {:?}, cursor: {}, layout: {:?}",
        buffer_entity,
        buffer_cursor.position,
        text_buffer.layout_mode
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
    fontir_app_state: &Option<Res<crate::core::state::FontIRAppState>>,
    app_state: &Option<Res<AppState>>,
) -> f32 {
    if let Some(fontir_state) = fontir_app_state.as_ref() {
        let metrics = fontir_state.get_font_metrics();
        let upm = metrics.units_per_em;
        let descender = metrics.descender.unwrap_or(-256.0);
        upm - descender
    } else if let Some(app_state) = app_state.as_ref() {
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
    text_editor_state
        .buffer
        .iter()
        .filter(|sort| sort.buffer_id == Some(buffer_id))
        .collect()
}

/// Calculate cursor offset based on layout mode and cursor position
fn calculate_cursor_offset(
    buffer_sorts: &[&SortData],
    cursor_position: usize,
    layout_mode: &SortLayoutMode,
    line_height: f32,
) -> Vec2 {
    match layout_mode {
        SortLayoutMode::RTLText => {
            calculate_rtl_cursor_offset(buffer_sorts, cursor_position, line_height)
        }
        _ => calculate_ltr_cursor_offset(buffer_sorts, cursor_position, line_height),
    }
}

/// Calculate RTL cursor offset (right-to-left text positioning)
/// 
/// RTL POSITIONING LOGIC:
/// ===================
/// In RTL text, cursor positioning works BACKWARDS from LTR:
/// 1. START: Root position (rightmost edge) 
/// 2. DIRECTION: Move LEFT by subtracting widths
/// 3. RULE: Accumulate widths of text AT OR AFTER cursor position
/// 
/// This positions cursor at LEFT EDGE of existing text (insertion point)
fn calculate_rtl_cursor_offset(
    buffer_sorts: &[&SortData],
    cursor_position: usize,
    line_height: f32,
) -> Vec2 {
    info!(
        "🎯 RTL CURSOR: Found {} sorts in buffer, cursor at position {}",
        buffer_sorts.len(),
        cursor_position
    );
    
    // RTL starts at RIGHT EDGE (x=0) and moves LEFT (negative x)
    let mut horizontal_offset = 0.0;
    let mut vertical_offset = 0.0;
    
    // TODO(human): Debug this RTL cursor positioning logic
    
    // CRITICAL RTL RULE: Process characters AT OR AFTER cursor position
    // This moves cursor leftward to the insertion point
    for (sort_index, sort_entry) in buffer_sorts.iter().enumerate() {
        if sort_index < cursor_position {
            // SKIP: Characters BEFORE cursor don't affect RTL cursor position
            continue;
        }
        
        // Process characters AT OR AFTER cursor position
        match &sort_entry.kind {
            SortKind::LineBreak => {
                if sort_index == cursor_position {
                    // Cursor exactly at line break - move to next line
                    vertical_offset -= line_height;
                    info!("🎯 RTL CURSOR: Cursor at line break {}", sort_index);
                    break;
                }
                // Line breaks AFTER cursor don't affect position
            }
            
            SortKind::Glyph { advance_width, .. } => {
                // RTL KEY OPERATION: Move LEFT by subtracting width
                horizontal_offset -= advance_width;
                
                info!(
                    "🎯 RTL: Sort[{}] '{}' at/after cursor → moved LEFT by {:.1} \
                     → offset now ({:.1}, {:.1})",
                    sort_index,
                    sort_entry.kind.glyph_name(),
                    advance_width,
                    horizontal_offset,
                    vertical_offset
                );
            }
        }
    }
    
    info!(
        "🎯 RTL RESULT: Cursor at LEFT EDGE for insertion → ({:.1}, {:.1})",
        horizontal_offset,
        vertical_offset
    );
    
    Vec2::new(horizontal_offset, vertical_offset)
}

/// Calculate LTR cursor offset (left-to-right text positioning)
/// 
/// LTR POSITIONING LOGIC:
/// ====================
/// In LTR text, cursor positioning is intuitive:
/// 1. START: Root position (leftmost edge)
/// 2. DIRECTION: Move RIGHT by adding widths
/// 3. RULE: Accumulate widths of text BEFORE cursor position
/// 
/// This positions cursor AFTER existing text (insertion point)
fn calculate_ltr_cursor_offset(
    buffer_sorts: &[&SortData],
    cursor_position: usize,
    line_height: f32,
) -> Vec2 {
    info!("🎯 LTR CURSOR: Using standard LTR cursor positioning logic");
    
    // LTR starts at LEFT EDGE (x=0) and moves RIGHT (positive x)
    let mut horizontal_offset = 0.0;
    let mut vertical_offset = 0.0;
    
    // Process each sort in the buffer
    for (sort_index, sort_entry) in buffer_sorts.iter().enumerate() {
        
        if sort_index < cursor_position {
            // BEFORE CURSOR: These characters affect cursor position
            match &sort_entry.kind {
                SortKind::LineBreak => {
                    // Line break: Reset to start of next line
                    horizontal_offset = 0.0;
                    vertical_offset -= line_height;
                    
                    info!(
                        "🎯 LTR: Line break[{}] → moved to next line (y: {:.1})",
                        sort_index,
                        vertical_offset
                    );
                }
                
                SortKind::Glyph { advance_width, .. } => {
                    // LTR KEY OPERATION: Move RIGHT by adding width
                    horizontal_offset += advance_width;
                    
                    info!(
                        "🎯 LTR: Sort[{}] '{}' before cursor → moved RIGHT by {:.1} \
                         → offset now ({:.1}, {:.1})",
                        sort_index,
                        sort_entry.kind.glyph_name(),
                        advance_width,
                        horizontal_offset,
                        vertical_offset
                    );
                }
            }
        } 
        
        else if sort_index == cursor_position {
            // AT CURSOR: Special case for line breaks
            if let SortKind::LineBreak = &sort_entry.kind {
                // Cursor exactly at line break - show at start of new line
                horizontal_offset = 0.0;
                vertical_offset -= line_height;
                
                info!(
                    "🎯 LTR: Cursor AT line break[{}] → show at new line start",
                    sort_index
                );
                break;
            }
            // For glyphs: cursor positioned BEFORE the glyph (no offset change)
        }
        
        // AFTER CURSOR: These characters don't affect cursor position (skip)
    }
    
    info!(
        "🎯 LTR RESULT: Cursor AFTER existing text → ({:.1}, {:.1})",
        horizontal_offset,
        vertical_offset
    );
    
    Vec2::new(horizontal_offset, vertical_offset)
}

/// Calculate cursor position using buffer entity system with full feature support
pub fn calculate_cursor_position(
    text_editor_state: &TextEditorState,
    app_state: &Option<Res<AppState>>,
    fontir_app_state: &Option<Res<crate::core::state::FontIRAppState>>,
    buffer_query: &Query<(
        &crate::core::state::text_editor::text_buffer::TextBuffer,
        &crate::core::state::text_editor::text_buffer::BufferCursor,
    )>,
    active_buffer: &Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
) -> Option<Vec2> {
    let buffer_info = get_active_buffer_info(active_buffer, buffer_query)?;
    let line_height = get_line_height(fontir_app_state, app_state);
    let buffer_sorts = collect_buffer_sorts(text_editor_state, buffer_info.buffer_id);
    
    let offset = calculate_cursor_offset(
        &buffer_sorts,
        buffer_info.cursor_position,
        &buffer_info.layout_mode,
        line_height,
    );
    
    Some(buffer_info.root_position + offset)
}

// ============================================================================
// CURSOR RENDERING SYSTEM
// ============================================================================

/// System to manage text editor cursor rendering
pub fn render_text_editor_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    text_editor_state: Option<Res<CoreTextEditorState>>,
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
        calculate_cursor_position(
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