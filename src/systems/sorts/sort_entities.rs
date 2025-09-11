//! Sort entity management for text editor sorts

use crate::core::state::text_editor::TextEditorState;
use crate::core::state::text_editor::{BufferMember, TextBuffer};
use crate::core::state::AppState;
use crate::core::state::FontMetrics;
use crate::core::state::SortLayoutMode;
use crate::editing::selection::components::Selected;
use crate::editing::sort::{ActiveSort, InactiveSort, Sort};
use bevy::prelude::*;
use std::collections::HashMap;

/// Component to track which buffer index this entity represents
#[derive(Component)]
pub struct BufferSortIndex(pub usize);

/// Resource to track which buffer sorts have entities
#[derive(Resource, Default)]
pub struct BufferSortEntities {
    pub entities: HashMap<usize, Entity>,
}

/// Resource to track buffer sorts that need entity respawning due to content changes
#[derive(Resource, Default)]
pub struct BufferSortRespawnQueue {
    pub indices: Vec<usize>,
}

/// Initialize text editor sorts
pub fn initialize_text_editor_sorts(mut commands: Commands) {
    commands.init_resource::<BufferSortEntities>();
    commands.init_resource::<BufferSortRespawnQueue>();
    info!("Initialized text editor sorts system");
}

/// Manage sort activation based on mouse clicks
#[allow(clippy::too_many_arguments)]
pub fn manage_sort_activation(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::rendering::cameras::DesignCamera>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    sort_query: Query<(Entity, &Transform, &Sort, Option<&BufferSortIndex>)>,
    _active_sort_query: Query<Entity, With<crate::editing::sort::ActiveSort>>,
    _text_editor_state: ResMut<TextEditorState>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
) {
    // System is working - debug confirmed
    // info!("[manage_sort_activation] System called, ui_hover_state.is_hovering_ui = {}", ui_hover_state.is_hovering_ui);

    // Don't process clicks when hovering over UI
    if ui_hover_state.is_hovering_ui {
        info!("[manage_sort_activation] Skipping - hovering over UI");
        return;
    }

    // Check for left mouse click
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        // info!("[manage_sort_activation] No left mouse click this frame");
        return;
    }

    // Get cursor position in world coordinates
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    info!(
        "[manage_sort_activation] Left mouse button pressed at screen pos: {:?}",
        window.cursor_position()
    );

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        debug!("[manage_sort_activation] Failed to convert cursor to world position");
        return;
    };

    // Check if click is on any sort
    let click_threshold = 200.0; // Increased click tolerance in design units

    debug!(
        "[manage_sort_activation] Mouse click at world position ({:.1}, {:.1})",
        world_position.x, world_position.y
    );
    debug!(
        "[manage_sort_activation] Checking {} sorts for handle clicks",
        sort_query.iter().count()
    );

    for (entity, transform, sort, _buffer_index) in sort_query.iter() {
        let sort_position = transform.translation.truncate();
        let distance = sort_position.distance(world_position);

        debug!(
            "Sort '{}' at ({:.1}, {:.1}), distance: {:.1}",
            sort.glyph_name, sort_position.x, sort_position.y, distance
        );

        if distance <= click_threshold {
            // TODO: Get modifier key state to support multi-select
            // For now, implement single-select behavior (clear others, select this one)

            // Clear selection from all other sorts
            let all_sorts = sort_query.iter().map(|(e, _, _, _)| e).collect::<Vec<_>>();
            for other_entity in all_sorts {
                if other_entity != entity {
                    commands
                        .entity(other_entity)
                        .remove::<crate::editing::selection::components::Selected>();
                }
            }

            // Add this sort to the selection
            commands
                .entity(entity)
                .insert(crate::editing::selection::components::Selected);
            info!(
                "Selected sort '{}' at position ({:.1}, {:.1})",
                sort.glyph_name, sort_position.x, sort_position.y
            );

            // Note: Activation will be handled separately based on selection state
            break;
        }
    }
}

/// Modern ECS-based positioning for text sorts using buffer entities
/// This replaces the legacy get_text_sort_flow_position function to prevent buffer cross-contamination
fn calculate_buffer_local_position(
    global_buffer_index: usize,
    text_editor_state: &TextEditorState,
    font_metrics: &FontMetrics,
    buffer_id_to_entity: &std::collections::HashMap<
        crate::core::state::text_editor::buffer::BufferId,
        Entity,
    >,
    buffer_entity_query: &Query<(Entity, &TextBuffer)>,
) -> Option<Vec2> {
    // Get the sort entry we're positioning
    let sort_entry = text_editor_state.buffer.get(global_buffer_index)?;

    // Get the buffer ID for this sort
    let buffer_id = sort_entry.buffer_id?;

    // Find the buffer entity
    let buffer_entity = buffer_id_to_entity.get(&buffer_id)?;

    // Get buffer information
    let (_entity, text_buffer) = buffer_entity_query
        .iter()
        .find(|(entity, _)| entity == buffer_entity)?;

    let root_position = text_buffer.root_position;
    let _layout_mode = &text_buffer.layout_mode;

    // Find all sorts that belong to this specific buffer, in order
    let mut buffer_sorts = Vec::new();
    for (idx, sort) in text_editor_state.buffer.iter().enumerate() {
        if sort.buffer_id == Some(buffer_id) {
            buffer_sorts.push((idx, sort));
        }
    }

    // Find our position within this buffer
    let buffer_local_index = buffer_sorts
        .iter()
        .position(|(idx, _)| *idx == global_buffer_index)?;

    info!(
        "🔍 ECS POSITIONING: Sort at global[{}] is buffer-local[{}] in buffer {:?} with {} total sorts",
        global_buffer_index, buffer_local_index, buffer_id.0, buffer_sorts.len()
    );

    // Calculate position based on preceding sorts in THIS buffer only
    let mut x_offset = 0.0;
    let mut y_offset = 0.0;

    // Use font metrics to calculate line height (match legacy system)
    let upm = font_metrics.units_per_em as f32;
    let descender = font_metrics.descender.unwrap_or(-256.0) as f32;
    let line_height = upm - descender; // Legacy formula: upm - descender (descender is negative)

    // Calculate position by iterating through preceding sorts in this buffer
    // Direction depends on layout mode
    let layout_mode = &text_buffer.layout_mode;

    for i in 0..buffer_local_index {
        if let Some((_, sort)) = buffer_sorts.get(i) {
            match &sort.kind {
                crate::core::state::text_editor::SortKind::Glyph { advance_width, .. } => {
                    match layout_mode {
                        crate::core::state::SortLayoutMode::LTRText => {
                            // LTR: advance to the right
                            x_offset += advance_width;
                        }
                        crate::core::state::SortLayoutMode::RTLText => {
                            // RTL: advance to the left
                            x_offset -= advance_width;
                        }
                        crate::core::state::SortLayoutMode::Freeform => {
                            // Freeform: treat as LTR
                            x_offset += advance_width;
                        }
                    }
                }
                crate::core::state::text_editor::SortKind::LineBreak => {
                    x_offset = 0.0;
                    y_offset -= line_height;
                }
            }
        }
    }

    let final_position = Vec2::new(root_position.x + x_offset, root_position.y + y_offset);

    info!(
        "🎯 ECS POSITIONING: buffer-local[{}] '{}' at ({:.1}, {:.1}) (x_offset={:.1})",
        buffer_local_index,
        sort_entry.kind.glyph_name(),
        final_position.x,
        final_position.y,
        x_offset
    );

    Some(final_position)
}

/// Spawn missing sort entities for sorts in the text editor buffer
#[allow(clippy::too_many_arguments)]
pub fn spawn_missing_sort_entities(
    mut commands: Commands,
    text_editor_state: ResMut<TextEditorState>,
    mut buffer_entities: ResMut<BufferSortEntities>,
    mut respawn_queue: ResMut<BufferSortRespawnQueue>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    _existing_active_sorts: Query<Entity, With<crate::editing::sort::ActiveSort>>,
    buffer_entity_query: Query<(
        Entity,
        &crate::core::state::text_editor::text_buffer::TextBuffer,
    )>,
) {
    // Debug: Log buffer state (only when buffer has content to reduce spam)
    if !text_editor_state.buffer.is_empty() {
        info!(
            "📋 spawn_missing_sort_entities: Processing {} buffer entries",
            text_editor_state.buffer.len()
        );
        for i in 0..text_editor_state.buffer.len() {
            if let Some(sort) = text_editor_state.buffer.get(i) {
                info!(
                    "  📋 Buffer[{}]: glyph='{}', is_active={}, layout_mode={:?}, pos=({:.1},{:.1})",
                    i,
                    sort.kind.glyph_name(),
                    sort.is_active,
                    sort.layout_mode,
                    sort.root_position.x, sort.root_position.y
                );
            }
        }
    }
    // Silently skip logging when buffer is empty to reduce spam

    // Build a mapping from BufferId to buffer Entity
    let mut buffer_id_to_entity = std::collections::HashMap::new();
    for (buffer_entity, text_buffer) in buffer_entity_query.iter() {
        buffer_id_to_entity.insert(text_buffer.id, buffer_entity);
        info!(
            "🔗 BUFFER MAPPING: BufferId {:?} -> Entity {:?}",
            text_buffer.id.0, buffer_entity
        );
    }

    // Handle respawn requests first
    for &index in &respawn_queue.indices {
        if let Some(entity) = buffer_entities.entities.remove(&index) {
            info!(
                "🔄 Respawning entity for buffer index {} due to content change",
                index
            );
            commands.entity(entity).despawn();
        }
    }
    respawn_queue.indices.clear(); // Clear the queue after processing

    // Iterate through all sorts in the buffer
    for i in 0..text_editor_state.buffer.len() {
        // Skip if we already have an entity for this buffer index
        if buffer_entities.entities.contains_key(&i) {
            warn!("🔄 spawn_missing_sort_entities: Entity already exists for buffer index {} - SKIPPING", i);
            continue;
        }

        warn!("🆕 spawn_missing_sort_entities: No entity exists for buffer index {} - SPAWNING NEW ENTITY", i);

        if let Some(sort_entry) = text_editor_state.buffer.get(i) {
            // Skip spawning entities for line breaks - they are invisible
            if sort_entry.kind.is_line_break() {
                debug!("Skipping entity spawn for line break at buffer index {}", i);
                continue;
            }

            // Get font metrics from either FontIR or AppState
            let font_metrics = if let Some(fontir_state) = fontir_app_state.as_ref() {
                let fontir_metrics = fontir_state.get_font_metrics();
                crate::core::state::FontMetrics {
                    units_per_em: fontir_metrics.units_per_em as f64,
                    ascender: fontir_metrics.ascender.map(|a| a as f64),
                    descender: fontir_metrics.descender.map(|d| d as f64),
                    line_height: fontir_metrics.line_gap.unwrap_or(0.0) as f64,
                    x_height: None,
                    cap_height: None,
                    italic_angle: None,
                }
            } else if let Some(state) = app_state.as_ref() {
                state.workspace.info.metrics.clone()
            } else {
                // Fallback metrics
                crate::core::state::FontMetrics {
                    units_per_em: 1024.0,
                    ascender: Some(832.0),
                    descender: Some(-256.0),
                    line_height: 0.0,
                    x_height: None,
                    cap_height: None,
                    italic_angle: None,
                }
            };

            // Get the visual position for this sort using correct font metrics
            let position = match sort_entry.layout_mode {
                crate::core::state::SortLayoutMode::LTRText
                | crate::core::state::SortLayoutMode::RTLText => {
                    // Use modern ECS-based positioning to prevent buffer cross-contamination
                    let calculated_pos = calculate_buffer_local_position(
                        i,
                        &text_editor_state,
                        &font_metrics,
                        &buffer_id_to_entity,
                        &buffer_entity_query,
                    );
                    warn!(
                        "🔍 ECS ENTITY POSITIONING: buffer[{}] calculated at {:?}",
                        i, calculated_pos
                    );
                    calculated_pos
                }
                crate::core::state::SortLayoutMode::Freeform => Some(sort_entry.root_position),
            };

            if let Some(position) = position {
                warn!("🎯 spawn_missing_sort_entities: Got position ({:.1}, {:.1}) for buffer index {}", position.x, position.y, i);

                // Create Sort component
                let sort = Sort {
                    glyph_name: sort_entry.kind.glyph_name().to_string(),
                    layout_mode: sort_entry.layout_mode.clone(),
                };

                // Spawn entity with appropriate activation state
                let mut entity_commands = commands.spawn((
                    sort,
                    Transform::from_translation(position.extend(0.0)),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    BufferSortIndex(i),
                    crate::editing::selection::components::Selectable, // Make sorts selectable
                    Name::new(format!("BufferSort[{i}]")),
                ));

                // Check if this sort should be active based on the text editor state
                // Only make sorts active if they are explicitly marked as active
                // Only one sort should be active at a time - no special treatment for first characters
                let should_be_active = sort_entry.is_active;
                if should_be_active {
                    entity_commands.insert(ActiveSort);
                    warn!("🟢 Spawned ACTIVE sort entity for buffer index {} at position ({:.1}, {:.1}) - glyph '{}' (is_active: {})", 
                           i, position.x, position.y, sort_entry.kind.glyph_name(), sort_entry.is_active);
                } else {
                    entity_commands.insert(InactiveSort);
                    warn!("🔴 Spawned INACTIVE sort entity for buffer index {} at position ({:.1}, {:.1}) - glyph '{}' (is_active: {})", 
                           i, position.x, position.y, sort_entry.kind.glyph_name(), sort_entry.is_active);
                }

                let entity = entity_commands.id();

                // Add BufferMember component to link this sort to its buffer entity
                if let Some(buffer_id) = sort_entry.buffer_id {
                    if let Some(&buffer_entity) = buffer_id_to_entity.get(&buffer_id) {
                        // Calculate buffer-local index by counting sorts with the same buffer_id up to this point
                        let mut buffer_local_index = 0;
                        for idx in 0..i {
                            if let Some(sort) = text_editor_state.buffer.get(idx) {
                                if sort.buffer_id == Some(buffer_id) {
                                    buffer_local_index += 1;
                                }
                            }
                        }

                        entity_commands.insert(
                            crate::core::state::text_editor::text_buffer::BufferMember::new(
                                buffer_entity,
                                buffer_local_index,
                            ),
                        );

                        info!("🔗 BUFFER MEMBERSHIP: Linked sort entity {:?} to buffer entity {:?} at buffer-local index {}", 
                              entity, buffer_entity, buffer_local_index);
                    } else {
                        warn!("❌ No buffer entity found for buffer_id {:?} - sort will not have BufferMember component", buffer_id.0);
                    }
                } else {
                    info!("ℹ️ Sort at buffer[{}] has no buffer_id - adding as freeform sort (no BufferMember)", i);
                }

                // Track the entity
                buffer_entities.entities.insert(i, entity);

                // Debug logging
                info!(
                    "✅ Spawned buffer sort entity {} for glyph '{}' at position ({:.1}, {:.1})",
                    i,
                    sort_entry.kind.glyph_name(),
                    position.x,
                    position.y
                );
            } else {
                warn!("❌ spawn_missing_sort_entities: Failed to get position for buffer sort index {} - SKIPPING ENTITY SPAWN", i);
            }
        }
    }
}

/// Update positions of existing buffer sort entities to match text flow
pub fn update_buffer_sort_positions(
    text_editor_state: Res<TextEditorState>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    buffer_entities: Res<BufferSortEntities>,
    mut sort_query: Query<&mut Transform, With<BufferSortIndex>>,
    buffer_entity_query: Query<(
        Entity,
        &crate::core::state::text_editor::text_buffer::TextBuffer,
    )>,
) {
    // CRITICAL PERFORMANCE FIX: Early return if TextEditorState hasn't changed
    // Prevents O(N²) position calculations every frame
    if !text_editor_state.is_changed() {
        return;
    }

    debug!("Buffer position update triggered - TextEditorState changed");

    // Build a mapping from BufferId to buffer Entity for ECS positioning
    let mut buffer_id_to_entity = std::collections::HashMap::new();
    for (buffer_entity, text_buffer) in buffer_entity_query.iter() {
        buffer_id_to_entity.insert(text_buffer.id, buffer_entity);
    }

    // Get font metrics from either AppState or FontIR
    let font_metrics = if let Some(fontir_state) = fontir_app_state.as_ref() {
        let fontir_metrics = fontir_state.get_font_metrics();
        crate::core::state::FontMetrics {
            units_per_em: fontir_metrics.units_per_em as f64,
            ascender: fontir_metrics.ascender.map(|a| a as f64),
            descender: fontir_metrics.descender.map(|d| d as f64),
            line_height: fontir_metrics.line_gap.unwrap_or(0.0) as f64,
            x_height: None,
            cap_height: None,
            italic_angle: None,
        }
    } else if let Some(app_state) = app_state.as_ref() {
        app_state.workspace.info.metrics.clone()
    } else {
        warn!("Buffer sort position updates skipped - no font data available");
        return;
    };

    // Update Transform positions for all existing buffer sorts
    for (&buffer_index, &entity) in buffer_entities.entities.iter() {
        if let Ok(mut transform) = sort_query.get_mut(entity) {
            // Calculate correct position using the font metrics from app state
            if let Some(sort) = text_editor_state.buffer.get(buffer_index) {
                // Skip line breaks - they shouldn't have entities
                if sort.kind.is_line_break() {
                    warn!(
                        "Unexpected entity for line break at buffer index {}",
                        buffer_index
                    );
                    continue;
                }
                let new_position = match sort.layout_mode {
                    crate::core::state::SortLayoutMode::LTRText
                    | crate::core::state::SortLayoutMode::RTLText => {
                        // Use modern ECS-based positioning to prevent buffer cross-contamination
                        let calculated_pos = calculate_buffer_local_position(
                            buffer_index,
                            &text_editor_state,
                            &font_metrics,
                            &buffer_id_to_entity,
                            &buffer_entity_query,
                        );
                        warn!(
                            "🔍 ECS POSITION UPDATE: buffer[{}] calculated at {:?}",
                            buffer_index, calculated_pos
                        );
                        calculated_pos
                    }
                    crate::core::state::SortLayoutMode::Freeform => Some(sort.root_position),
                };

                if let Some(new_pos) = new_position {
                    let new_pos_3d = new_pos.extend(transform.translation.z);
                    transform.translation = new_pos_3d;
                    info!(
                        "Sort {} positioned at ({:.1}, {:.1})",
                        buffer_index, new_pos.x, new_pos.y
                    );
                }
            }
        }
    }
}

/// System to auto-activate sorts when exactly one is selected
pub fn auto_activate_selected_sorts(
    mut commands: Commands,
    mut text_editor_state: ResMut<TextEditorState>,
    _buffer_entities: Res<BufferSortEntities>,
    selected_sorts: Query<
        Entity,
        (
            With<Sort>,
            With<crate::editing::selection::components::Selected>,
        ),
    >,
    active_sorts: Query<Entity, With<ActiveSort>>,
    buffer_index_query: Query<&BufferSortIndex>,
) {
    let selected_count = selected_sorts.iter().count();
    let active_count = active_sorts.iter().count();

    // Debug: Log when this system runs and what it finds
    if selected_count > 0 || active_count > 0 {
        info!(
            "auto_activate_selected_sorts: selected={}, active={}",
            selected_count, active_count
        );
    }

    if selected_count == 1 {
        // Exactly one sort is selected - activate it
        if let Ok(selected_sort) = selected_sorts.single() {
            // Find the buffer index of the selected sort
            let selected_buffer_index =
                if let Ok(buffer_index) = buffer_index_query.get(selected_sort) {
                    Some(buffer_index.0)
                } else {
                    None
                };

            // Deactivate all currently active sorts when another sort is selected
            for active_entity in active_sorts.iter() {
                // Always deactivate active sorts when a different sort is explicitly selected
                // This allows any active sort to become inactive when user clicks other handles
                if active_entity != selected_sort {
                    commands.entity(active_entity).remove::<ActiveSort>();
                    commands.entity(active_entity).insert(InactiveSort);

                    // Update text editor state to deactivate the sort
                    if let Ok(buffer_index) = buffer_index_query.get(active_entity) {
                        if let Some(sort) = text_editor_state.buffer.get_mut(buffer_index.0) {
                            info!(
                                "🔻 Deactivating buffer sort {} - glyph '{}'",
                                buffer_index.0,
                                sort.kind.glyph_name()
                            );
                            sort.is_active = false;
                        }
                    }
                }
            }

            // Activate the selected sort
            commands.entity(selected_sort).remove::<InactiveSort>();
            commands.entity(selected_sort).insert(ActiveSort);

            // Update text editor state to activate the sort
            if let Some(buffer_index) = selected_buffer_index {
                info!(
                    "🔼 Activating buffer sort {} via auto_activate_selected_sorts",
                    buffer_index
                );
                text_editor_state.activate_sort(buffer_index);
            }

            info!("Auto-activated selected sort {:?}", selected_sort);
        }
    } else if selected_count == 0 {
        // No sorts selected - keep current state for now
        // TODO: Could deactivate all if we want that behavior
        debug!("No sorts selected - keeping current activation state");
    } else {
        // Multiple sorts selected - keep current activation state for now
        // TODO: Could implement "first selected becomes active" logic here
        debug!(
            "Multiple sorts selected ({}) - keeping current activation state",
            selected_count
        );
    }
}

/// Sync activation state from text editor buffer to entities
pub fn sync_buffer_sort_activation_state(
    mut commands: Commands,
    text_editor_state: Res<TextEditorState>,
    buffer_entities: Res<BufferSortEntities>,
    active_sort_query: Query<Entity, With<ActiveSort>>,
    inactive_sort_query: Query<Entity, With<InactiveSort>>,
    _buffer_index_query: Query<&BufferSortIndex>,
) {
    // Only run when text editor state has changed to avoid unnecessary work
    if !text_editor_state.is_changed() {
        return;
    }

    debug!(
        "🔄 sync_buffer_sort_activation_state: Syncing activation state from buffer to entities"
    );

    // Iterate through all tracked entities and sync their activation state
    for (&buffer_index, &entity) in buffer_entities.entities.iter() {
        if let Some(sort_entry) = text_editor_state.buffer.get(buffer_index) {
            let should_be_active = sort_entry.is_active;

            // Check current entity state
            let is_currently_active = active_sort_query.get(entity).is_ok();
            let is_currently_inactive = inactive_sort_query.get(entity).is_ok();

            if should_be_active && !is_currently_active {
                // Should be active but isn't - activate it
                if is_currently_inactive {
                    commands.entity(entity).remove::<InactiveSort>();
                }
                commands.entity(entity).insert(ActiveSort);
                info!(
                    "🟢 ACTIVATION SYNC: Activated entity for buffer[{}] - glyph '{}'",
                    buffer_index,
                    sort_entry.kind.glyph_name()
                );
            } else if !should_be_active && is_currently_active {
                // Should be inactive but is active - deactivate it
                commands.entity(entity).remove::<ActiveSort>();
                commands.entity(entity).insert(InactiveSort);
                info!(
                    "🔴 ACTIVATION SYNC: Deactivated entity for buffer[{}] - glyph '{}'",
                    buffer_index,
                    sort_entry.kind.glyph_name()
                );
            }
        }
    }
}

/// Despawn missing buffer sort entities
#[allow(clippy::too_many_arguments)]
pub fn despawn_missing_buffer_sort_entities(
    mut commands: Commands,
    text_editor_state: Res<TextEditorState>,
    mut buffer_entities: ResMut<BufferSortEntities>,
    mut glyph_entities: ResMut<crate::rendering::glyph_renderer::GlyphRenderEntities>,
    mut metrics_entities: ResMut<crate::rendering::metrics::MetricsLineEntities>,
    mut handle_entities: ResMut<crate::rendering::sort_visuals::SortHandleEntities>,
    sort_query: Query<Entity, With<BufferSortIndex>>,
    glyph_element_query: Query<(
        Entity,
        &crate::rendering::glyph_renderer::GlyphRenderElement,
    )>,
    point_query: Query<Entity, With<crate::editing::sort::manager::SortPointEntity>>,
    sort_point_query: Query<&crate::editing::sort::manager::SortPointEntity>,
    sort_name_text_query: Query<(Entity, &crate::rendering::sort_renderer::SortGlyphNameText)>,
    sort_unicode_text_query: Query<(Entity, &crate::rendering::sort_renderer::SortUnicodeText)>,
) {
    // Always log to see if system is running
    info!("🔍 despawn_missing_buffer_sort_entities: SYSTEM CALLED - Buffer length: {}, Tracked entities: {}", 
          text_editor_state.buffer.len(), buffer_entities.entities.len());

    // Debug: Log current state
    if !buffer_entities.entities.is_empty() {
        info!("despawn_missing_buffer_sort_entities: Checking {} tracked entities against buffer length {}", 
              buffer_entities.entities.len(), text_editor_state.buffer.len());

        // Log all tracked entities
        for (&idx, &entity) in buffer_entities.entities.iter() {
            info!("  Tracked entity: buffer[{}] -> entity {:?}", idx, entity);
        }
    }

    // Remove entities for buffer indices that no longer exist
    let mut to_remove = Vec::new();
    let mut despawn_operations = 0;
    const MAX_DESPAWN_PER_FRAME: usize = 50; // Limit to prevent command queue overflow

    for (&buffer_index, &entity) in buffer_entities.entities.iter() {
        // Break early if we've hit our despawn limit for this frame
        if despawn_operations >= MAX_DESPAWN_PER_FRAME {
            info!(
                "⚠️ Reached despawn limit ({}) for this frame - will continue next frame",
                MAX_DESPAWN_PER_FRAME
            );
            break;
        }
        // Check if this buffer index still exists in the actual buffer
        if buffer_index >= text_editor_state.buffer.len() {
            // Buffer index no longer exists, despawn entity
            if sort_query.get(entity).is_ok() {
                info!(
                    "🗑️ Despawning sort entity for deleted buffer index {} (buffer len: {})",
                    buffer_index,
                    text_editor_state.buffer.len()
                );

                // First, despawn all glyph render elements associated with this sort
                let mut glyph_count = 0;

                // Despawn all glyph elements (points, outlines, handles)
                if let Some(element_entities) = glyph_entities.elements.get(&entity) {
                    for &element_entity in element_entities.iter() {
                        commands.entity(element_entity).despawn();
                        glyph_count += 1;
                        despawn_operations += 1;
                    }
                }

                // Also despawn any loose glyph elements that might not be tracked
                for (glyph_entity, glyph_element) in glyph_element_query.iter() {
                    if glyph_element.sort_entity == entity {
                        commands.entity(glyph_entity).despawn();
                        glyph_count += 1;
                        despawn_operations += 1;
                    }
                }

                // Despawn all point entities associated with this sort
                let mut point_count = 0;
                for point_entity in point_query.iter() {
                    if let Ok(sort_point) = sort_point_query.get(point_entity) {
                        if sort_point.sort_entity == entity {
                            commands.entity(point_entity).despawn();
                            point_count += 1;
                            despawn_operations += 1;
                        }
                    }
                }

                // Initialize metrics count for later cleanup
                let mut metrics_count = 0;

                // Despawn all sort label text entities associated with this sort
                let mut label_count = 0;

                // Despawn glyph name text entities
                for (text_entity, name_text) in sort_name_text_query.iter() {
                    if name_text.sort_entity == entity {
                        commands.entity(text_entity).despawn();
                        label_count += 1;
                        despawn_operations += 1;
                    }
                }

                // Despawn unicode text entities
                for (text_entity, unicode_text) in sort_unicode_text_query.iter() {
                    if unicode_text.sort_entity == entity {
                        commands.entity(text_entity).despawn();
                        label_count += 1;
                        despawn_operations += 1;
                    }
                }

                // Despawn all metrics line entities associated with this sort
                if let Some(metrics_entity_list) = metrics_entities.lines.get(&entity) {
                    info!(
                        "🗑️ CLEANING UP METRICS: Found {} metrics entities for sort {:?}",
                        metrics_entity_list.len(),
                        entity
                    );
                    for &metrics_entity in metrics_entity_list.iter() {
                        info!("🗑️ Despawning metrics entity {:?}", metrics_entity);
                        commands.entity(metrics_entity).despawn();
                        metrics_count += 1;
                        despawn_operations += 1;
                    }
                } else {
                    info!(
                        "⚠️ NO METRICS FOUND: Sort {:?} has no metrics entities to clean up",
                        entity
                    );
                }

                // Despawn all sort handle entities associated with this sort
                let mut handle_count = 0;
                if let Some(handle_entity_list) = handle_entities.handles.get(&entity) {
                    for &handle_entity in handle_entity_list.iter() {
                        commands.entity(handle_entity).despawn();
                        handle_count += 1;
                        despawn_operations += 1;
                    }
                }

                info!("🗑️ Despawned {} glyph elements, {} point entities, {} metrics line entities, {} label entities, and {} handle entities for sort {:?} (total operations: {})", 
                      glyph_count, point_count, metrics_count, label_count, handle_count, entity, despawn_operations);

                // Remove from glyph render tracking
                glyph_entities.elements.remove(&entity);

                // Remove from metrics tracking
                metrics_entities.lines.remove(&entity);

                // Remove from handle tracking
                handle_entities.handles.remove(&entity);

                // Finally, despawn the sort entity itself
                commands.entity(entity).despawn();
                despawn_operations += 1;
                to_remove.push(buffer_index);
            } else {
                warn!(
                    "Entity {:?} for buffer index {} already despawned or invalid",
                    entity, buffer_index
                );
                to_remove.push(buffer_index);
            }
        }
    }

    // Store count before consuming the vector
    let despawn_count = to_remove.len();

    // Remove from tracking
    for index in to_remove {
        buffer_entities.entities.remove(&index);
        info!("🗑️ Removed buffer index {} from entity tracking", index);
    }

    // Log final summary
    if despawn_operations > 0 {
        info!("📊 despawn_missing_buffer_sort_entities: Completed {} total despawn operations this frame (despawned {} sorts)", despawn_operations, despawn_count);
    }

    if despawn_count > 0 {
        info!("🗑️ Despawned {} entities total", despawn_count);
    }
}
