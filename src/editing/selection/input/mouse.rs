//! Mouse input handling for selection

use crate::core::io::input::{InputEvent, InputState, ModifierState};
use crate::core::state::TextEditorState;
use crate::editing::edit_type::EditType;
use crate::editing::selection::components::{
    GlyphPointReference, PointType, Selectable, Selected, SelectionRect, SelectionState,
};
use crate::editing::selection::coordinate_system::SelectionCoordinateSystem;
use crate::editing::selection::events::{ClickWorldPosition, SELECTION_MARGIN};
use crate::editing::selection::input::shortcuts::handle_selection_key_press;
use crate::editing::selection::nudge::EditEvent;
use crate::editing::selection::{DragPointState, DragSelectionState};
use crate::geometry::design_space::DPoint;
use bevy::input::mouse::MouseButton;
use bevy::prelude::*;

/// System to process selection input events from the new input system
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn process_selection_input_events(
    mut commands: Commands,
    mut input_events: EventReader<InputEvent>,
    input_state: Res<InputState>,
    mut drag_state: ResMut<DragSelectionState>,
    mut drag_point_state: ResMut<DragPointState>,
    mut event_writer: EventWriter<EditEvent>,
    #[allow(clippy::type_complexity)] selectable_query: Query<
        (
            Entity,
            &GlobalTransform,
            Option<&GlyphPointReference>,
            Option<&PointType>,
        ),
        With<Selectable>,
    >,
    selected_query: Query<(Entity, &Transform), With<Selected>>,
    selection_rect_query: Query<Entity, With<SelectionRect>>,
    mut selection_state: ResMut<SelectionState>,
    active_sort_state: Res<crate::editing::sort::ActiveSortState>,
    sort_point_entities: Query<&crate::systems::sort_manager::SortPointEntity>,
    select_mode: Option<Res<crate::ui::edit_mode_toolbar::select::SelectModeActive>>,
    text_editor_state: ResMut<TextEditorState>,
    app_state: Option<Res<crate::core::state::AppState>>,
    buffer_entities: Res<crate::systems::sorts::sort_entities::BufferSortEntities>,
) {
    // Early exit if no events to process - this prevents expensive logging every frame
    let event_count = input_events.len();
    if event_count == 0 {
        return;
    }

    debug!(
        "[process_selection_input_events] CALLED with {} events",
        event_count
    );

    // Check if select tool is active by checking InputMode
    if !crate::core::io::input::helpers::is_input_mode(
        &input_state,
        crate::core::io::input::InputMode::Select,
    ) {
        info!("[process_selection_input_events] Not in Select input mode (current: {:?}), skipping all events", input_state.mode);
        return;
    }
    debug!("[process_selection_input_events] In Select input mode - continuing");

    // Only log when we actually have events to process
    let mode_status = select_mode.as_ref().map(|s| s.0).unwrap_or(false);
    debug!(
        "[process_selection_input_events] SelectModeActive exists: {}, active: {}",
        select_mode.is_some(),
        mode_status
    );

    // Only process if in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            info!("[process_selection_input_events] SelectModeActive is false, exiting");
            return;
        }
    } else {
        info!("[process_selection_input_events] SelectModeActive resource not found, exiting");
        return;
    }
    debug!("[process_selection_input_events] All checks passed - processing events");

    for event in input_events.read() {
        debug!(
            "[process_selection_input_events] Processing event: {:?}",
            event
        );

        // Skip if UI is consuming input
        if crate::core::io::input::helpers::is_ui_consuming(&input_state) {
            debug!("Selection: Skipping event - UI is consuming input");
            continue;
        }

        // Only handle events that are relevant to selection
        match event {
            InputEvent::MouseMove { .. } => {
                // Only skip mouse move events when NOT dragging points
                if !drag_point_state.is_dragging {
                    continue;
                }
                // Mouse moves during point dragging are handled by handle_point_drag system
                continue;
            }
            InputEvent::MouseClick {
                button,
                position,
                modifiers,
            } => {
                debug!(
                    "[process_selection_input_events] GOT MOUSE CLICK: button={:?}, position={:?}",
                    button, position
                );
                if *button == MouseButton::Left {
                    let world_position = position.to_raw();
                    let handle_tolerance = 50.0;

                    // Try to detect sort handle clicks if AppState is available
                    let clicked_sort_handle = if let Some(state) = app_state.as_ref() {
                        let font_metrics = &state.workspace.info.metrics;
                        debug!(
                            "[sort-handle-hit] Mouse click at world position: ({:.1}, {:.1})",
                            world_position.x, world_position.y
                        );
                        debug!(
                            "[sort-handle-hit] Buffer has {} sorts",
                            text_editor_state.buffer.len()
                        );

                        // Only log detailed handle positions if we have a small number of sorts to avoid spam
                        if text_editor_state.buffer.len() <= 10 {
                            for i in 0..text_editor_state.buffer.len() {
                                if let Some(_sort) = text_editor_state.buffer.get(i) {
                                    if let Some(sort_pos) =
                                        text_editor_state.get_sort_visual_position(i)
                                    {
                                        let descender =
                                            font_metrics.descender.unwrap_or(-200.0) as f32;
                                        let handle_pos = sort_pos + Vec2::new(0.0, descender);
                                        let distance = world_position.distance(handle_pos);
                                        debug!("[sort-handle-hit] Sort {}: handle_pos=({:.1}, {:.1}), distance={:.1}, tolerance={:.1}",
                                            i, handle_pos.x, handle_pos.y, distance, handle_tolerance);
                                    }
                                }
                            }
                        }

                        if let Some(clicked_sort_index) = text_editor_state
                            .find_sort_handle_at_position(
                                world_position,
                                handle_tolerance,
                                Some(font_metrics),
                            )
                        {
                            info!("[process_selection_input_events] Clicked on sort handle at index {}", clicked_sort_index);

                            // Find the entity corresponding to this buffer index
                            if let Some(&sort_entity) =
                                buffer_entities.entities.get(&clicked_sort_index)
                            {
                                let is_ctrl_held = modifiers.ctrl;

                                if is_ctrl_held {
                                    // Multi-select: toggle selection
                                    if selection_state.selected.contains(&sort_entity) {
                                        // Remove from selection
                                        commands.entity(sort_entity).remove::<crate::editing::selection::components::Selected>();
                                        selection_state.selected.remove(&sort_entity);
                                        info!("[process_selection_input_events] Ctrl+click: removed sort {} from selection", clicked_sort_index);
                                    } else {
                                        // Add to selection
                                        commands.entity(sort_entity).insert(
                                            crate::editing::selection::components::Selected,
                                        );
                                        selection_state.selected.insert(sort_entity);
                                        info!("[process_selection_input_events] Ctrl+click: added sort {} to selection", clicked_sort_index);
                                    }
                                } else {
                                    // Single select: clear others and select this one
                                    // Clear all current selections
                                    for entity in selection_state.selected.clone() {
                                        commands.entity(entity).remove::<crate::editing::selection::components::Selected>();
                                    }
                                    selection_state.selected.clear();

                                    // Select this sort
                                    commands
                                        .entity(sort_entity)
                                        .insert(crate::editing::selection::components::Selected);
                                    selection_state.selected.insert(sort_entity);
                                    info!("[process_selection_input_events] Single click: selected sort {} exclusively", clicked_sort_index);
                                }
                            } else {
                                warn!("[process_selection_input_events] Could not find entity for sort index {}", clicked_sort_index);
                            }

                            true // Sort handle was clicked
                        } else {
                            false // No sort handle clicked
                        }
                    } else {
                        debug!("[sort-handle-hit] AppState not available (using FontIR) - skipping sort handle detection");
                        false // AppState not available, proceed to point selection
                    };

                    // If no sort handle was clicked, proceed with individual point selection
                    if !clicked_sort_handle {
                        debug!("[sort-handle-hit] No sort handle hit detected - calling handle_selection_click for individual points");

                        // Fallback to general selection click handling
                        // Use a dummy entity if no active sort exists
                        let active_sort_entity = active_sort_state
                            .active_sort_entity
                            .unwrap_or(Entity::PLACEHOLDER);
                        debug!("[sort-handle-hit] active_sort_state.active_sort_entity={:?}, using: {:?}", 
                              active_sort_state.active_sort_entity, active_sort_entity);
                        handle_selection_click(
                            &mut commands,
                            position,
                            modifiers,
                            &mut drag_state,
                            &mut drag_point_state,
                            &mut event_writer,
                            &selectable_query,
                            &selected_query,
                            &mut selection_state,
                            active_sort_entity,
                            &sort_point_entities,
                        );
                    }
                }
            }
            InputEvent::MouseDrag {
                button,
                start_position,
                current_position,
                delta,
                modifiers,
            } => {
                if *button == MouseButton::Left {
                    debug!(
                        "Selection: Processing mouse drag from {:?} to {:?} with modifiers {:?}",
                        start_position, current_position, modifiers
                    );
                    debug!(
                        "Selection: active_sort_entity={:?}",
                        active_sort_state.active_sort_entity
                    );

                    // Always allow drag selection, regardless of active sort state
                    // Use a dummy entity if no active sort exists
                    let active_sort_entity = active_sort_state
                        .active_sort_entity
                        .unwrap_or(Entity::PLACEHOLDER);
                    debug!("Selection: Calling handle_selection_drag...");
                    handle_selection_drag(
                        &mut commands,
                        start_position,
                        current_position,
                        delta,
                        modifiers,
                        &mut drag_state,
                        &mut drag_point_state,
                        &mut event_writer,
                        &selectable_query,
                        &mut selection_state,
                        active_sort_entity,
                        &sort_point_entities,
                        &selection_rect_query,
                    );
                    debug!("Selection: handle_selection_drag completed");
                }
            }
            InputEvent::MouseRelease {
                button,
                position,
                modifiers,
            } => {
                if *button == MouseButton::Left {
                    debug!(
                        "Selection: Processing mouse release at {:?} with modifiers {:?}",
                        position, modifiers
                    );
                    // Always handle mouse release for selection, regardless of active sort state
                    handle_selection_release(
                        &mut commands,
                        position,
                        modifiers,
                        &mut drag_state,
                        &mut drag_point_state,
                        &mut event_writer,
                        &mut selection_state,
                        &selection_rect_query,
                    );
                }
            }
            InputEvent::KeyPress { key, modifiers } => {
                if matches!(
                    key,
                    bevy::input::keyboard::KeyCode::KeyA | bevy::input::keyboard::KeyCode::Escape
                ) {
                    debug!(
                        "Selection: Processing key press {:?} with modifiers {:?}",
                        key, modifiers
                    );
                    // Always handle key presses for selection, regardless of active sort state
                    // Use a dummy entity if no active sort exists
                    let active_sort_entity = active_sort_state
                        .active_sort_entity
                        .unwrap_or(Entity::PLACEHOLDER);
                    handle_selection_key_press(
                        &mut commands,
                        key,
                        modifiers,
                        &selectable_query,
                        &selected_query,
                        &mut selection_state,
                        &mut event_writer,
                        active_sort_entity,
                        &sort_point_entities,
                    );
                }
            }
            _ => {}
        }
    }
}

/// Handle mouse click for selection
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn handle_selection_click(
    commands: &mut Commands,
    position: &DPoint,
    modifiers: &ModifierState,
    _drag_state: &mut ResMut<DragSelectionState>,
    drag_point_state: &mut ResMut<DragPointState>,
    event_writer: &mut EventWriter<EditEvent>,
    selectable_query: &Query<
        (
            Entity,
            &GlobalTransform,
            Option<&GlyphPointReference>,
            Option<&PointType>,
        ),
        With<Selectable>,
    >,
    selected_query: &Query<(Entity, &Transform), With<Selected>>,
    selection_state: &mut ResMut<SelectionState>,
    active_sort_entity: Entity,
    sort_point_entities: &Query<&crate::systems::sort_manager::SortPointEntity>,
) {
    debug!(
        "=== HANDLE SELECTION CLICK === position={:?}, active_sort={:?}",
        position, active_sort_entity
    );
    let cursor_pos = position.to_raw();
    debug!("Click position: {:?} (raw: {:?})", position, cursor_pos);
    debug!("Modifiers: {:?}", modifiers);
    debug!("Active sort entity: {:?}", active_sort_entity);
    debug!(
        "Current selection count: {}",
        selection_state.selected.len()
    );

    // Count selectable points in active sort
    let mut total_selectable = 0;
    let mut active_sort_selectable = 0;
    for (entity, _, _, _) in selectable_query.iter() {
        total_selectable += 1;
        if let Ok(sort_point_entity) = sort_point_entities.get(entity) {
            if sort_point_entity.sort_entity == active_sort_entity {
                active_sort_selectable += 1;
            }
        }
    }
    debug!(
        "Total selectable entities: {}, in active sort: {}",
        total_selectable, active_sort_selectable
    );

    debug!(
        "Selection click at ({:.1}, {:.1})",
        cursor_pos.x, cursor_pos.y
    );

    let mut best_hit = None;
    let mut min_dist_sq = SELECTION_MARGIN * SELECTION_MARGIN;
    debug!("Using selection margin: {}", SELECTION_MARGIN);

    // Find the closest selectable entity that belongs to the active sort
    let mut checked_points = 0;
    let mut points_in_active_sort = 0;

    for (entity, transform, glyph_ref, point_type) in selectable_query.iter() {
        checked_points += 1;

        // Check if this entity belongs to the active sort
        if let Ok(sort_point_entity) = sort_point_entities.get(entity) {
            // If we have a valid active sort, filter by it
            if active_sort_entity != Entity::PLACEHOLDER
                && sort_point_entity.sort_entity != active_sort_entity
            {
                continue; // Skip points that don't belong to the active sort
            }
            points_in_active_sort += 1;
        } else {
            debug!("Entity {:?} has no SortPointEntity component", entity);
            continue; // Skip entities that aren't sort points
        }

        let pos = transform.translation().truncate();
        let dist_sq = cursor_pos.distance_squared(pos);
        let distance = dist_sq.sqrt();

        debug!(
            "Point {:?} at {:?}, distance: {:.1} (squared: {:.1})",
            entity, pos, distance, dist_sq
        );

        if dist_sq < min_dist_sq {
            min_dist_sq = dist_sq;
            best_hit = Some((entity, pos, glyph_ref, point_type));
            debug!("New best hit: {:?} at distance {:.1}", entity, distance);
        }
    }

    debug!(
        "Checked {} total points, {} in active sort",
        checked_points, points_in_active_sort
    );
    debug!("Best hit found: {:?}", best_hit.map(|(e, p, _, _)| (e, p)));

    if let Some((entity, pos, glyph_ref, point_type)) = best_hit {
        // Clicked on a selectable entity in the active sort
        commands.insert_resource(ClickWorldPosition);

        let shift_held = modifiers.shift;

        if !shift_held && selection_state.selected.contains(&entity) {
            // Clicked on already selected entity - start drag
            debug!(
                "Clicked on already-selected entity {:?} - starting drag",
                entity
            );
        } else {
            // Handle selection
            if !shift_held {
                // Clear previous selection
                debug!(
                    "Clearing previous selection (count: {})",
                    selection_state.selected.len()
                );
                for (e, _) in selected_query.iter() {
                    commands.entity(e).remove::<Selected>();
                }
                selection_state.selected.clear();
            }

            // Add to selection
            selection_state.selected.insert(entity);
            commands.entity(entity).insert(Selected);

            // Log point type for debugging
            if let Some(pt) = point_type {
                let point_type_str = if pt.is_on_curve {
                    "on-curve"
                } else {
                    "off-curve"
                };
                debug!(
                    "Selected {} point at ({:.1}, {:.1})",
                    point_type_str, pos.x, pos.y
                );
            }

            if let Some(glyph_ref) = glyph_ref {
                debug!(
                    "Selected point in glyph '{}', contour {}, point {}",
                    glyph_ref.glyph_name, glyph_ref.contour_index, glyph_ref.point_index
                );
            }
        }

        // Start drag operation
        if drag_point_state.is_dragging {
            debug!("WARNING: Starting drag while already dragging - resetting drag state");
            drag_point_state.is_dragging = false;
            drag_point_state.original_positions.clear();
            drag_point_state.dragged_entities.clear();
        }

        drag_point_state.is_dragging = true;
        drag_point_state.start_position = Some(cursor_pos);
        drag_point_state.current_position = Some(cursor_pos);

        // Include all currently selected entities in the drag operation
        drag_point_state.dragged_entities = selection_state.selected.iter().cloned().collect();
        debug!(
            "Starting drag with {} entities",
            drag_point_state.dragged_entities.len()
        );

        // Save original positions
        drag_point_state.original_positions.clear();
        for (entity, transform) in selected_query.iter() {
            if selection_state.selected.contains(&entity) {
                let pos = Vec2::new(transform.translation.x, transform.translation.y);
                drag_point_state.original_positions.insert(entity, pos);
            }
        }

        // Also store position of newly clicked entity
        drag_point_state
            .original_positions
            .entry(entity)
            .or_insert(pos);

        event_writer.write(EditEvent {
            edit_type: EditType::Normal,
        });

        debug!(
            "Selection updated and drag started. Current selection count: {}",
            selection_state.selected.len()
        );
    } else {
        // Clicked on empty space
        debug!("Clicked on empty space - clearing selection");
        commands.insert_resource(ClickWorldPosition);

        // Clear selection if not multi-selecting
        if !modifiers.shift {
            for (entity, _) in selected_query.iter() {
                commands.entity(entity).remove::<Selected>();
            }
            selection_state.selected.clear();
            debug!("Cleared all selections");
        }
    }
}

/// Handle mouse drag for selection
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn handle_selection_drag(
    commands: &mut Commands,
    start_position: &DPoint,
    current_position: &DPoint,
    _delta: &Vec2,
    modifiers: &ModifierState,
    drag_state: &mut ResMut<DragSelectionState>,
    _drag_point_state: &mut ResMut<DragPointState>,
    _event_writer: &mut EventWriter<EditEvent>,
    selectable_query: &Query<
        (
            Entity,
            &GlobalTransform,
            Option<&GlyphPointReference>,
            Option<&PointType>,
        ),
        With<Selectable>,
    >,
    selection_state: &mut ResMut<SelectionState>,
    _active_sort_entity: Entity,
    _sort_point_entities: &Query<&crate::systems::sort_manager::SortPointEntity>,
    _selection_rect_query: &Query<Entity, With<SelectionRect>>,
) {
    info!(
        "[handle_selection_drag] Called: start={:?}, current={:?}, is_dragging={}",
        start_position, current_position, drag_state.is_dragging
    );
    if !drag_state.is_dragging {
        info!(
            "[handle_selection_drag] Starting new drag selection at start={:?}, end={:?}",
            start_position.to_raw(),
            current_position.to_raw()
        );

        // Initialize drag state
        drag_state.is_dragging = true;
        drag_state.start_position = Some(*start_position);
        drag_state.current_position = Some(*current_position);
        drag_state.is_multi_select = modifiers.shift;

        // Store previous selection for multi-select
        if modifiers.shift {
            drag_state.previous_selection = selection_state.selected.iter().cloned().collect();
        }

        // Spawn selection rectangle entity
        let rect_entity = commands
            .spawn(SelectionRect {
                start: start_position.to_raw(),
                end: current_position.to_raw(),
            })
            .id();
        drag_state.selection_rect_entity = Some(rect_entity);
        info!(
            "[handle_selection_drag] SelectionRect entity created with ID {:?}",
            rect_entity
        );
    } else {
        // Only update current_position during drag
        info!("Continuing existing drag selection...");
        drag_state.current_position = Some(*current_position);
        // Only update the entity if it exists
        if let Some(rect_entity) = drag_state.selection_rect_entity {
            info!("Updating SelectionRect entity {:?}", rect_entity);
            if let Ok(mut entity_commands) = commands.get_entity(rect_entity) {
                entity_commands.insert(SelectionRect {
                    start: drag_state
                        .start_position
                        .unwrap_or(*start_position)
                        .to_raw(),
                    end: current_position.to_raw(),
                });
                info!(
                    "SelectionRect entity updated: start={:?}, end={:?}",
                    drag_state
                        .start_position
                        .unwrap_or(*start_position)
                        .to_raw(),
                    current_position.to_raw()
                );
            } else {
                info!(
                    "ERROR: Could not get entity commands for SelectionRect entity {:?}",
                    rect_entity
                );
            }
        } else {
            info!("ERROR: No SelectionRect entity found in drag_state!");
        }
    }

    // Update selection based on what's inside the rectangle
    if let (Some(start_pos), Some(current_pos)) =
        (drag_state.start_position, drag_state.current_position)
    {
        info!(
            "Selection: Marquee rect coordinates - start: {:?}, current: {:?}",
            start_pos, current_pos
        );

        // In multi-select mode, start with previous selection
        if drag_state.is_multi_select {
            // Reset to previous selection
            for (entity, _, _, _) in selectable_query.iter() {
                if !drag_state.previous_selection.contains(&entity) {
                    commands.entity(entity).remove::<Selected>();
                    selection_state.selected.remove(&entity);
                }
            }

            for &entity in &drag_state.previous_selection {
                if !selection_state.selected.contains(&entity) {
                    commands.entity(entity).insert(Selected);
                    selection_state.selected.insert(entity);
                }
            }
        } else {
            // Clear selection for non-multi-select
            for (entity, _, _, _) in selectable_query.iter() {
                commands.entity(entity).remove::<Selected>();
            }
            selection_state.selected.clear();
        }

        // Add entities in the rectangle to selection
        let mut points_in_rect = 0;
        let mut points_selected = 0;

        info!(
            "Selection: Checking {} selectable entities for marquee selection",
            selectable_query.iter().count()
        );

        // Collect entity positions for debugging and coordinate system analysis
        let entity_positions: Vec<(Entity, Vec2)> = selectable_query
            .iter()
            .map(|(entity, transform, _, _)| (entity, transform.translation().truncate()))
            .collect();

        // Use centralized coordinate system for debugging
        let debug_info = SelectionCoordinateSystem::debug_coordinate_ranges(
            &entity_positions,
            &start_pos,
            &current_pos,
        );
        info!("Selection: {}", debug_info);

        for (entity, entity_pos) in &entity_positions {
            // Use centralized coordinate system to check if entity is inside the marquee rectangle
            if SelectionCoordinateSystem::is_point_in_rectangle(
                entity_pos,
                &start_pos,
                &current_pos,
            ) {
                points_in_rect += 1;
                info!(
                    "Selection: Entity {:?} is inside marquee rect! Position: {:?}",
                    entity, entity_pos
                );
                if drag_state.is_multi_select && drag_state.previous_selection.contains(entity) {
                    // Toggle off if previously selected
                    selection_state.selected.remove(entity);
                    commands.entity(*entity).remove::<Selected>();
                    info!("Selection: Toggled off entity {:?}", entity);
                } else {
                    // Add to selection
                    selection_state.selected.insert(*entity);
                    commands.entity(*entity).insert(Selected);
                    points_selected += 1;
                    info!("Selection: Added entity {:?} to selection", entity);
                }
            } else {
                // Debug: Show why entity is not selected using centralized coordinate system
                let rect_entity_start =
                    SelectionCoordinateSystem::design_to_entity_coordinates(&start_pos);
                let rect_entity_end =
                    SelectionCoordinateSystem::design_to_entity_coordinates(&current_pos);

                let distance_x = if entity_pos.x < rect_entity_start.x.min(rect_entity_end.x) {
                    rect_entity_start.x.min(rect_entity_end.x) - entity_pos.x
                } else if entity_pos.x > rect_entity_start.x.max(rect_entity_end.x) {
                    entity_pos.x - rect_entity_start.x.max(rect_entity_end.x)
                } else {
                    0.0
                };

                let distance_y = if entity_pos.y < rect_entity_start.y.min(rect_entity_end.y) {
                    rect_entity_start.y.min(rect_entity_end.y) - entity_pos.y
                } else if entity_pos.y > rect_entity_start.y.max(rect_entity_end.y) {
                    entity_pos.y - rect_entity_start.y.max(rect_entity_end.y)
                } else {
                    0.0
                };

                if distance_x > 0.0 || distance_y > 0.0 {
                    info!(
                        "Selection: Entity {:?} outside rect - X: {:.1} units, Y: {:.1} units",
                        entity, distance_x, distance_y
                    );
                }
            }
        }

        info!(
            "Marquee selection: {} points in rect, {} points selected",
            points_in_rect, points_selected
        );

        info!(
            "Marquee selection updated: {} points selected",
            selection_state.selected.len()
        );
    }
}

/// Handle mouse release for selection
#[allow(clippy::too_many_arguments)]
pub fn handle_selection_release(
    commands: &mut Commands,
    position: &DPoint,
    _modifiers: &ModifierState,
    drag_state: &mut ResMut<DragSelectionState>,
    drag_point_state: &mut ResMut<DragPointState>,
    _event_writer: &mut EventWriter<EditEvent>,
    selection_state: &mut ResMut<SelectionState>,
    _selection_rect_query: &Query<Entity, With<SelectionRect>>,
) {
    let release_pos = position.to_raw();
    debug!(
        "Selection release at ({:.1}, {:.1})",
        release_pos.x, release_pos.y
    );

    if drag_point_state.is_dragging {
        drag_point_state.is_dragging = false;
        drag_point_state.start_position = None;
        drag_point_state.current_position = None;
        drag_point_state.dragged_entities.clear();
        drag_point_state.original_positions.clear();
    } else if drag_state.is_dragging {
        // End drag selection
        drag_state.is_dragging = false;
        let rect_entity = drag_state.selection_rect_entity;
        drag_state.selection_rect_entity = None;
        drag_state.start_position = None;
        drag_state.current_position = None;
        drag_state.is_multi_select = false;

        debug!(
            "Ended drag selection. Final selection count: {}",
            selection_state.selected.len()
        );

        info!(
            "Marquee selection complete. {} points selected.",
            selection_state.selected.len()
        );
        // Clean up the selection rectangle entity
        if let Some(rect_entity) = rect_entity {
            commands.entity(rect_entity).despawn();
            info!("SelectionRect entity despawned on release");
        }
    }
}
