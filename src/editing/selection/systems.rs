#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unused_enumerate_index)]
#![allow(clippy::useless_vec)]

use super::components::*;
use super::coordinate_system::SelectionCoordinateSystem;
use super::events::ClickWorldPosition;
use super::DragPointState;
use super::DragSelectionState;
use crate::core::io::input::{helpers, InputEvent, InputState};
use crate::core::io::pointer::PointerInfo;
use crate::core::settings::BezySettings;
use crate::core::state::AppState;
use crate::core::state::FontMetrics;
use crate::core::state::TextEditorState;
use crate::editing::selection::nudge::{EditEvent, NudgeState};
#[allow(unused_imports)]
use crate::geometry::point::{EditPoint, EntityId, EntityKind};
#[allow(unused_imports)]
use crate::geometry::world_space::DPoint;
use bevy::ecs::system::ParamSet;
use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::log::info;
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy::window::PrimaryWindow;

/// Event to signal that app state has changed
#[derive(Event, Debug, Clone)]
pub struct AppStateChanged;

// Constants for selection
#[allow(dead_code)]
const SELECTION_MARGIN: f32 = 16.0; // Distance in pixels for selection hit testing

// Legacy handle_mouse_input system removed - replaced by handle_selection_input_events

/// System to handle selection shortcuts (Ctrl+A for select all, etc.)
#[allow(clippy::too_many_arguments)]
pub fn handle_selection_shortcuts(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    selected_query: Query<Entity, With<Selected>>,
    selectable_query: Query<Entity, With<Selectable>>,
    mut selection_state: ResMut<SelectionState>,
    mut event_writer: EventWriter<EditEvent>,
    select_mode: Option<Res<crate::ui::edit_mode_toolbar::select::SelectModeActive>>,
    knife_mode: Option<Res<crate::ui::edit_mode_toolbar::knife::KnifeModeActive>>,
    text_editor_state: Option<Res<crate::core::state::TextEditorState>>,
) {
    // Skip processing shortcuts if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            return;
        }
    }

    // Only process shortcuts when in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            return;
        }
    }

    // Only allow selection shortcuts when there's an active sort in text editor
    if let Some(text_editor_state) = text_editor_state.as_ref() {
        if text_editor_state.get_active_sort().is_none() {
            return;
        }
    } else {
        return;
    }

    // Handle Escape key to clear selection
    if keyboard_input.just_pressed(KeyCode::Escape) {
        for entity in &selected_query {
            commands.entity(entity).remove::<Selected>();
        }
        selection_state.selected.clear();
        debug!("Cleared selection");
    }

    // Handle Ctrl+A (select all)
    let ctrl_pressed = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight)
        || keyboard_input.pressed(KeyCode::SuperLeft)
        || keyboard_input.pressed(KeyCode::SuperRight);

    if ctrl_pressed && keyboard_input.just_pressed(KeyCode::KeyA) {
        debug!("Select all shortcut pressed");

        // Clear current selection
        for entity in &selected_query {
            commands.entity(entity).remove::<Selected>();
        }
        selection_state.selected.clear();

        // Select all selectable entities
        for entity in &selectable_query {
            selection_state.selected.insert(entity);
            commands.entity(entity).insert(Selected);
        }

        debug!("Selected all {} entities", selection_state.selected.len());

        // Send edit event
        event_writer.write(EditEvent {});
    }
}

/// System to update the actual glyph data when a point is moved
#[allow(clippy::type_complexity)]
pub fn update_glyph_data_from_selection(
    query: Query<
        (
            &Transform,
            &GlyphPointReference,
            Option<&crate::editing::sort::manager::SortPointEntity>,
        ),
        (With<Selected>, Changed<Transform>),
    >,
    sort_query: Query<(&crate::editing::sort::Sort, &Transform)>,
    mut app_state: ResMut<AppState>,
    // Track if we're in a nudging operation
    _nudge_state: Res<crate::editing::selection::nudge::NudgeState>,
    knife_mode: Option<Res<crate::ui::edit_mode_toolbar::knife::KnifeModeActive>>,
) {
    // Skip processing if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            return;
        }
    }

    // Early return if no points were moved
    if query.is_empty() {
        return;
    }

    debug!(
        "[update_glyph_data_from_selection] Processing {} moved points",
        query.iter().count()
    );

    let app_state = app_state.bypass_change_detection();
    let mut any_updates = false;

    for (transform, point_ref, sort_point_entity_opt) in query.iter() {
        // Default to world position if we can't get sort position
        let (relative_x, relative_y) = if let Some(sort_point_entity) = sort_point_entity_opt {
            if let Ok((_sort, sort_transform)) = sort_query.get(sort_point_entity.sort_entity) {
                let world_pos = transform.translation.truncate();
                let sort_pos = sort_transform.translation.truncate();
                let rel = world_pos - sort_pos;
                (rel.x as f64, rel.y as f64)
            } else {
                (
                    transform.translation.x as f64,
                    transform.translation.y as f64,
                )
            }
        } else {
            (
                transform.translation.x as f64,
                transform.translation.y as f64,
            )
        };

        let updated = app_state.set_point_position(
            &point_ref.glyph_name,
            point_ref.contour_index,
            point_ref.point_index,
            relative_x,
            relative_y,
        );

        debug!(
            "[update_glyph_data_from_selection] glyph='{}' contour={} point={} rel=({:.1}, {:.1}) updated={}",
            point_ref.glyph_name,
            point_ref.contour_index,
            point_ref.point_index,
            relative_x,
            relative_y,
            updated
        );

        if updated {
            any_updates = true;
            debug!(
                "Updated UFO glyph data for point {} in contour {} of glyph {}",
                point_ref.point_index, point_ref.contour_index, point_ref.glyph_name
            );
        } else {
            warn!(
                "Failed to update UFO glyph data for point {} in contour {} of glyph {} - invalid indices",
                point_ref.point_index, point_ref.contour_index, point_ref.glyph_name
            );
        }
    }

    // Log the results
    if any_updates {
        debug!(
            "[update_glyph_data_from_selection] Successfully updated {} outline points",
            query.iter().count()
        );
    } else {
        debug!("[update_glyph_data_from_selection] No outline updates needed");
    }
}

/// System to spawn point entities for the active sort using ECS as source of truth
pub fn spawn_active_sort_points(
    mut commands: Commands,
    active_sort_state: Res<crate::editing::sort::ActiveSortState>,
    sort_query: Query<(Entity, &crate::editing::sort::Sort, &Transform)>,
    point_entities: Query<Entity, With<crate::editing::sort::manager::SortPointEntity>>,
    app_state: Res<AppState>,
    _selection_state: ResMut<crate::editing::selection::SelectionState>,
) {
    // Only spawn points if there's an active sort
    if let Some(active_sort_entity) = active_sort_state.active_sort_entity {
        if let Ok((sort_entity, sort, transform)) = sort_query.get(active_sort_entity) {
            // Check if points already exist for this sort
            let existing_points = point_entities.iter().any(|_entity| {
                // Check if points already exist for this sort
                true // Simplified check
            });

            if !existing_points {
                let position = transform.translation.truncate();
                debug!("[spawn_active_sort_points] Spawning points for active sort: '{}' at position {:?}", 
                      sort.glyph_name, position);

                // Get glyph data for the active sort
                if let Some(glyph_data) = app_state.workspace.font.get_glyph(&sort.glyph_name) {
                    if let Some(outline) = &glyph_data.outline {
                        let mut point_count = 0;

                        for (contour_index, contour) in outline.contours.iter().enumerate() {
                            for (point_index, point) in contour.points.iter().enumerate() {
                                // Calculate world position: sort position + point offset
                                let point_world_pos =
                                    position + Vec2::new(point.x as f32, point.y as f32);
                                point_count += 1;

                                // Debug: Print first few point positions
                                if point_count <= 5 {
                                    debug!("[spawn_active_sort_points] Point {}: local=({:.1}, {:.1}), world=({:.1}, {:.1})", 
                                          point_count, point.x, point.y, point_world_pos.x, point_world_pos.y);
                                }

                                let glyph_point_ref =
                                    crate::editing::selection::components::GlyphPointReference {
                                        glyph_name: sort.glyph_name.clone(),
                                        contour_index,
                                        point_index,
                                    };

                                let _entity = commands
                                    .spawn((
                                        crate::geometry::point::EditPoint {
                                            position: kurbo::Point::new(point.x, point.y),
                                            point_type: point.point_type,
                                        },
                                        glyph_point_ref,
                                        crate::editing::selection::components::PointType {
                                            is_on_curve: matches!(point.point_type,
                                            crate::core::state::font_data::PointTypeData::Move |
                                            crate::core::state::font_data::PointTypeData::Line |
                                            crate::core::state::font_data::PointTypeData::Curve),
                                        },
                                        Transform::from_translation(point_world_pos.extend(0.0)),
                                        Visibility::Visible,
                                        InheritedVisibility::default(),
                                        ViewVisibility::default(),
                                        crate::editing::selection::components::Selectable,
                                        crate::editing::sort::manager::SortPointEntity {
                                            sort_entity,
                                        },
                                    ))
                                    .id();
                            }
                        }
                        debug!(
                            "[spawn_active_sort_points] Successfully spawned {} point entities",
                            point_count
                        );
                    } else {
                        warn!(
                            "[spawn_active_sort_points] No outline found for glyph '{}'",
                            sort.glyph_name
                        );
                    }
                } else {
                    warn!(
                        "[spawn_active_sort_points] No glyph data found for '{}'",
                        sort.glyph_name
                    );
                }
            } else {
                debug!("[spawn_active_sort_points] Points already exist for active sort, skipping spawn");
            }
        } else {
            warn!("[spawn_active_sort_points] Active sort entity not found in sort query");
        }
    } else {
        debug!("[spawn_active_sort_points] No active sort, skipping point spawn");
    }
}

/// System to despawn point entities when active sort changes
pub fn despawn_inactive_sort_points(
    mut commands: Commands,
    active_sort_state: Res<crate::editing::sort::ActiveSortState>,
    point_entities: Query<(Entity, &crate::editing::sort::manager::SortPointEntity)>,
    mut selection_state: ResMut<crate::editing::selection::SelectionState>,
) {
    // Despawn points for sorts that are no longer active
    for (entity, sort_point) in point_entities.iter() {
        let is_active = active_sort_state.active_sort_entity == Some(sort_point.sort_entity);

        if !is_active {
            // Remove from selection state if selected
            if selection_state.selected.contains(&entity) {
                selection_state.selected.remove(&entity);
                debug!(
                    "[despawn_inactive_sort_points] Removed despawned entity {:?} from selection",
                    entity
                );
            }

            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.despawn();
            }
            debug!(
                "[despawn_inactive_sort_points] Despawned point entity {:?} for inactive sort {:?}",
                entity, sort_point.sort_entity
            );
        }
    }
}

/// System to update point positions when sort position changes
#[allow(clippy::type_complexity)]
pub fn sync_point_positions_to_sort(
    mut param_set: ParamSet<(
        Query<
            (Entity, &crate::editing::sort::Sort, &Transform),
            Changed<crate::editing::sort::Sort>,
        >,
        Query<(
            &mut Transform,
            &crate::editing::sort::manager::SortPointEntity,
            &crate::editing::selection::components::GlyphPointReference,
        )>,
    )>,
    app_state: Res<AppState>,
) {
    // First, collect all the sort positions that have changed
    let mut sort_positions = std::collections::HashMap::new();

    for (sort_entity, sort, sort_transform) in param_set.p0().iter() {
        let position = sort_transform.translation.truncate();
        sort_positions.insert(sort_entity, (sort.glyph_name.clone(), position));
    }

    // Then update all point transforms based on the collected positions
    for (mut point_transform, sort_point, glyph_ref) in param_set.p1().iter_mut() {
        if let Some((glyph_name, position)) = sort_positions.get(&sort_point.sort_entity) {
            // Get the original point data from the glyph
            if let Some(glyph_data) = app_state.workspace.font.get_glyph(glyph_name) {
                if let Some(outline) = &glyph_data.outline {
                    if let Some(contour) = outline.contours.get(glyph_ref.contour_index) {
                        if let Some(point) = contour.points.get(glyph_ref.point_index) {
                            // Calculate new world position: sort position + original point offset
                            let point_world_pos =
                                *position + Vec2::new(point.x as f32, point.y as f32);
                            point_transform.translation = point_world_pos.extend(0.0);

                            debug!("[sync_point_positions_to_sort] Updated point {} in contour {} to position {:?}", 
                                   glyph_ref.point_index, glyph_ref.contour_index, point_world_pos);
                        }
                    }
                }
            }
        }
    }
}

/// System to handle key releases for nudging
pub fn handle_key_releases(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut nudge_state: ResMut<NudgeState>,
) {
    // Reset nudging state if no arrow keys are pressed
    let arrow_keys = [
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
    ];

    let any_arrow_pressed = arrow_keys.iter().any(|key| keyboard_input.pressed(*key));

    if !any_arrow_pressed {
        nudge_state.is_nudging = false;
    }
}

/// System to clear selection when app state changes (e.g., when codepoint changes)
pub fn clear_selection_on_app_change(
    mut commands: Commands,
    query: Query<Entity, With<Selected>>,
    mut selection_state: ResMut<SelectionState>,
    mut events: EventReader<AppStateChanged>,
) {
    for _ in events.read() {
        // Clear the selection when app state changes (e.g., when codepoint changes)
        selection_state.selected.clear();

        // Also remove the Selected component from all entities
        for entity in &query {
            commands.entity(entity).remove::<Selected>();
        }

        debug!("Selection cleared due to app state change");
    }
}

/// System to handle advanced point dragging with constraints and snapping
#[allow(clippy::type_complexity)]
pub fn handle_point_drag(
    pointer_info: Res<PointerInfo>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut drag_point_state: ResMut<DragPointState>,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &mut crate::editing::selection::nudge::PointCoordinates,
            Option<&GlyphPointReference>,
            Option<&crate::editing::sort::manager::SortCrosshair>,
        ),
        With<Selected>,
    >,
    mut app_state: ResMut<AppState>,
    mut event_writer: EventWriter<EditEvent>,
    settings: Res<BezySettings>,
) {
    // Only drag if the resource says we are
    if !drag_point_state.is_dragging {
        return;
    }

    let cursor_pos = pointer_info.design.to_raw();
    drag_point_state.current_position = Some(cursor_pos);

    if let Some(start_pos) = drag_point_state.start_position {
        let total_movement = cursor_pos - start_pos;
        let mut movement = total_movement;

        // Handle constrained movement with Shift key
        if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight)
        {
            if total_movement.x.abs() > total_movement.y.abs() {
                movement.y = 0.0; // Constrain to horizontal
            } else {
                movement.x = 0.0; // Constrain to vertical
            }
        }

        let mut updated_count = 0;

        for (entity, mut transform, mut coordinates, point_ref, sort_crosshair) in &mut query {
            if let Some(original_pos) = drag_point_state.original_positions.get(&entity) {
                let new_pos = *original_pos + movement;

                // Handle sort crosshair drag (no snapping, keep on top)
                if sort_crosshair.is_some() {
                    transform.translation.x = new_pos.x;
                    transform.translation.y = new_pos.y;
                    transform.translation.z = 25.0; // Keep crosshairs on top
                    coordinates.x = new_pos.x;
                    coordinates.y = new_pos.y;
                }
                // Handle glyph point drag (with snapping)
                else if let Some(point_ref) = point_ref {
                    // Apply grid snapping if enabled
                    let snapped_pos = settings.apply_grid_snap(new_pos);

                    transform.translation.x = snapped_pos.x;
                    transform.translation.y = snapped_pos.y;
                    transform.translation.z = 5.0; // Keep glyph points above background
                    coordinates.x = snapped_pos.x;
                    coordinates.y = snapped_pos.y;

                    // Update UFO data for glyph points
                    let updated = app_state.set_point_position(
                        &point_ref.glyph_name,
                        point_ref.contour_index,
                        point_ref.point_index,
                        transform.translation.x as f64, // Convert f32 to f64
                        transform.translation.y as f64, // Convert f32 to f64
                    );
                    if updated {
                        updated_count += 1;
                    }
                }
                // Handle other draggable entities (no snapping, normal Z layer)
                else {
                    transform.translation.x = new_pos.x;
                    transform.translation.y = new_pos.y;
                    transform.translation.z = 10.0; // Middle layer
                    coordinates.x = new_pos.x;
                    coordinates.y = new_pos.y;
                }
            }
        }

        if updated_count > 0 {
            debug!("Updated {} UFO points during drag", updated_count);

            // Send edit event
            event_writer.write(EditEvent {});
        }
    }
}

/// System to clean up the click resource
pub fn cleanup_click_resource(mut commands: Commands) {
    commands.remove_resource::<ClickWorldPosition>();
}

/// System to process selection input events from the new input system
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn process_selection_input_events(
    mut commands: Commands,
    mut input_events: EventReader<crate::core::io::input::InputEvent>,
    input_state: Res<crate::core::io::input::InputState>,
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
    sort_point_entities: Query<&crate::editing::sort::manager::SortPointEntity>,
    select_mode: Option<Res<crate::ui::edit_mode_toolbar::select::SelectModeActive>>,
    text_editor_state: ResMut<TextEditorState>,
    app_state: Res<crate::core::state::AppState>,
) {
    debug!("[process_selection_input_events] Called");

    // Check if select tool is active by checking InputMode
    if !crate::core::io::input::helpers::is_input_mode(
        &input_state,
        crate::core::io::input::InputMode::Select,
    ) {
        debug!("[process_selection_input_events] Not in Select input mode, returning early");
        return;
    }

    // Only process if in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            debug!("[process_selection_input_events] Not in select mode, returning early");
            return;
        }
    }
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
            crate::core::io::input::InputEvent::MouseClick {
                button,
                position,
                modifiers,
            } => {
                if *button == MouseButton::Left {
                    let world_position = position.to_raw();
                    let handle_tolerance = 50.0;
                    let font_metrics = &app_state.workspace.info.metrics;
                    debug!(
                        "[sort-handle-hit] Click at world position: ({:.1}, {:.1})",
                        world_position.x, world_position.y
                    );
                    // Print all handle positions and distances
                    for i in 0..text_editor_state.buffer.len() {
                        if let Some(_sort) = text_editor_state.buffer.get(i) {
                            if let Some(sort_pos) = text_editor_state.get_sort_visual_position(i) {
                                let descender = font_metrics.descender.unwrap_or(-200.0) as f32;
                                let handle_pos = sort_pos + Vec2::new(0.0, descender);
                                let distance = world_position.distance(handle_pos);
                                debug!("[sort-handle-hit] Sort {}: handle_pos=({:.1}, {:.1}), distance={:.1}, tolerance={:.1}",
                                    i, handle_pos.x, handle_pos.y, distance, handle_tolerance);
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
                        debug!(
                            "[process_selection_input_events] Clicked near sort handle at index {}",
                            clicked_sort_index
                        );
                        let is_ctrl_held = modifiers.ctrl;
                        if is_ctrl_held {
                            // OLD: ECS-based selection: activate the clicked sort directly
                            // text_editor_state.activate_sort(clicked_sort_index);
                            debug!("[process_selection_input_events] Ctrl: skipping activation (handled by selection system)");
                        } else {
                            // OLD: ECS-based selection: activate the clicked sort directly
                            // text_editor_state.activate_sort(clicked_sort_index);
                            debug!("[process_selection_input_events] Regular click: skipping activation (handled by selection system)");
                        }
                        // Early return: don't run the rest of the selection logic for this click
                        return;
                    } else {
                        debug!("[sort-handle-hit] No sort handle hit detected");

                        // Fallback to general selection click handling
                        // Use a dummy entity if no active sort exists
                        let active_sort_entity = active_sort_state
                            .active_sort_entity
                            .unwrap_or(Entity::PLACEHOLDER);
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
            crate::core::io::input::InputEvent::MouseDrag {
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
            crate::core::io::input::InputEvent::MouseRelease {
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
            crate::core::io::input::InputEvent::KeyPress { key, modifiers } => {
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
    modifiers: &crate::core::io::input::ModifierState,
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
    sort_point_entities: &Query<&crate::editing::sort::manager::SortPointEntity>,
) {
    debug!("=== HANDLE SELECTION CLICK ===");
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
            if sort_point_entity.sort_entity != active_sort_entity {
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

        event_writer.write(EditEvent {});

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
    delta: &Vec2,
    modifiers: &crate::core::io::input::ModifierState,
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
    _sort_point_entities: &Query<&crate::editing::sort::manager::SortPointEntity>,
    _selection_rect_query: &Query<Entity, With<SelectionRect>>,
) {
    debug!(
        "[handle_selection_drag] Called: start={:?}, current={:?}, delta={:?}, is_dragging={}",
        start_position, current_position, delta, drag_state.is_dragging
    );
    if !drag_state.is_dragging {
        debug!(
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
        debug!(
            "[handle_selection_drag] SelectionRect entity created with ID {:?}",
            rect_entity
        );
    } else {
        // Only update current_position during drag
        debug!("Continuing existing drag selection...");
        drag_state.current_position = Some(*current_position);
        // Only update the entity if it exists
        if let Some(rect_entity) = drag_state.selection_rect_entity {
            debug!("Updating SelectionRect entity {:?}", rect_entity);
            if let Ok(mut entity_commands) = commands.get_entity(rect_entity) {
                entity_commands.insert(SelectionRect {
                    start: drag_state
                        .start_position
                        .unwrap_or(*start_position)
                        .to_raw(),
                    end: current_position.to_raw(),
                });
                debug!(
                    "SelectionRect entity updated: start={:?}, end={:?}",
                    drag_state
                        .start_position
                        .unwrap_or(*start_position)
                        .to_raw(),
                    current_position.to_raw()
                );
            } else {
                debug!(
                    "ERROR: Could not get entity commands for SelectionRect entity {:?}",
                    rect_entity
                );
            }
        } else {
            debug!("ERROR: No SelectionRect entity found in drag_state!");
        }
    }

    // Update selection based on what's inside the rectangle
    if let (Some(start_pos), Some(current_pos)) =
        (drag_state.start_position, drag_state.current_position)
    {
        debug!(
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

        debug!(
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
        debug!("Selection: {}", debug_info);

        for (entity, entity_pos) in &entity_positions {
            // Use centralized coordinate system to check if entity is inside the marquee rectangle
            if SelectionCoordinateSystem::is_point_in_rectangle(
                entity_pos,
                &start_pos,
                &current_pos,
            ) {
                points_in_rect += 1;
                debug!(
                    "Selection: Entity {:?} is inside marquee rect! Position: {:?}",
                    entity, entity_pos
                );
                if drag_state.is_multi_select && drag_state.previous_selection.contains(entity) {
                    // Toggle off if previously selected
                    selection_state.selected.remove(entity);
                    commands.entity(*entity).remove::<Selected>();
                    debug!("Selection: Toggled off entity {:?}", entity);
                } else {
                    // Add to selection
                    selection_state.selected.insert(*entity);
                    commands.entity(*entity).insert(Selected);
                    points_selected += 1;
                    debug!("Selection: Added entity {:?} to selection", entity);
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
                    debug!(
                        "Selection: Entity {:?} outside rect - X: {:.1} units, Y: {:.1} units",
                        entity, distance_x, distance_y
                    );
                }
            }
        }

        debug!(
            "Marquee selection: {} points in rect, {} points selected",
            points_in_rect, points_selected
        );

        debug!(
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
    _modifiers: &crate::core::io::input::ModifierState,
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

        debug!(
            "Marquee selection complete. {} points selected.",
            selection_state.selected.len()
        );
        // Clean up the selection rectangle entity
        if let Some(rect_entity) = rect_entity {
            if let Ok(mut entity_commands) = commands.get_entity(rect_entity) {
                entity_commands.despawn();
            }
            debug!("SelectionRect entity despawned on release");
        }
    }
}

/// Handle key press for selection shortcuts
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn handle_selection_key_press(
    commands: &mut Commands,
    key: &KeyCode,
    modifiers: &crate::core::io::input::ModifierState,
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
    event_writer: &mut EventWriter<EditEvent>,
    active_sort_entity: Entity,
    sort_point_entities: &Query<&crate::editing::sort::manager::SortPointEntity>,
) {
    match key {
        KeyCode::KeyA => {
            if modifiers.ctrl {
                // Ctrl+A: Select all points in the active sort
                debug!("Select all shortcut triggered for active sort");
                let mut selected_count = 0;

                for (entity, _, _, _) in selectable_query.iter() {
                    // Check if this entity belongs to the active sort
                    if let Ok(sort_point_entity) = sort_point_entities.get(entity) {
                        if sort_point_entity.sort_entity != active_sort_entity {
                            continue; // Skip points that don't belong to the active sort
                        }
                    } else {
                        continue; // Skip entities that aren't sort points
                    }

                    if !selection_state.selected.contains(&entity) {
                        selection_state.selected.insert(entity);
                        commands.entity(entity).insert(Selected);
                        selected_count += 1;
                    }
                }

                event_writer.write(EditEvent {});
                debug!("Selected all {} points in active sort", selected_count);
            }
        }
        KeyCode::Escape => {
            // Escape: Clear selection
            debug!("Escape key pressed - clearing selection");
            for (entity, _) in selected_query.iter() {
                commands.entity(entity).remove::<Selected>();
            }
            selection_state.selected.clear();
            event_writer.write(EditEvent {});
        }
        _ => {}
    }
}

/// TEMP: Debug system to print all SelectionRect entities every frame
pub fn debug_print_selection_rects(selection_rects: Query<(Entity, &SelectionRect)>) {
    let count = selection_rects.iter().count();
    if count > 0 {
        debug!(
            "[debug_print_selection_rects] Found {} SelectionRect entities:",
            count
        );
        for (entity, rect) in selection_rects.iter() {
            debug!(
                "  Entity {:?}: start={:?}, end={:?}",
                entity, rect.start, rect.end
            );
        }
    }
}

#[cfg(debug_assertions)]
pub fn debug_validate_point_entity_uniqueness(
    point_entities: Query<
        (
            &crate::editing::selection::components::GlyphPointReference,
            Entity,
        ),
        With<crate::editing::sort::manager::SortPointEntity>,
    >,
) {
    use std::collections::HashMap;
    let mut seen: HashMap<(String, usize, usize), Entity> = HashMap::new();
    for (point_ref, entity) in point_entities.iter() {
        let key = (
            point_ref.glyph_name.clone(),
            point_ref.contour_index,
            point_ref.point_index,
        );
        if let Some(existing) = seen.insert(key.clone(), entity) {
            warn!(
                "[debug_validate_point_entity_uniqueness] Duplicate ECS entities for glyph='{}' contour={} point={}: {:?} and {:?}",
                point_ref.glyph_name, point_ref.contour_index, point_ref.point_index, existing, entity
            );
        }
    }
}
