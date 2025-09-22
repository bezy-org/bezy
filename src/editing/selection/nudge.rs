use crate::core::settings::BezySettings;
use crate::editing::selection::components::{GlyphPointReference, PointType, Selected};
use crate::editing::selection::enhanced_point_component::EnhancedPointType;
use crate::editing::selection::point_movement::{
    find_connected_offcurve_points_nudge, sync_to_font_data, PointMovement,
};
use crate::editing::sort::manager::SortPointEntity;
use crate::editing::sort::ActiveSortState;
use bevy::log::{debug, info, warn};
use bevy::prelude::*;

/// Resource to track nudge state for preventing selection loss during nudging
#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct NudgeState {
    /// Whether we're currently nudging (to prevent selection loss)
    pub is_nudging: bool,
    /// Timestamp of the last nudge operation
    pub last_nudge_time: f32,
}

/// System to handle keyboard input for nudging selected points
/// This is the idiomatic Bevy ECS approach: direct system that queries and mutates
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn handle_nudge_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut queries: ParamSet<(
        Query<
            (
                Entity,
                &mut Transform,
                &GlyphPointReference,
                Option<&SortPointEntity>,
                Option<&PointType>,
            ),
            (With<Selected>, With<SortPointEntity>),
        >,
        Query<(&crate::editing::sort::Sort, &Transform)>,
        Query<
            (Entity, &mut Transform, &GlyphPointReference, &PointType),
            (With<SortPointEntity>, Without<Selected>),
        >,
    )>,
    _app_state: Option<ResMut<crate::core::state::AppState>>,
    _fontir_app_state: Option<ResMut<crate::core::state::FontIRAppState>>,
    mut event_writer: EventWriter<EditEvent>,
    mut nudge_state: ResMut<NudgeState>,
    time: Res<Time>,
    _active_sort_state: Res<ActiveSortState>, // Keep for potential future use
    settings: Res<BezySettings>,
) {
    // Debug: Log that the system is being called
    debug!(
        "[NUDGE] handle_nudge_input called - selected points: {}",
        queries.p0().iter().count()
    );

    // Debug: Check if any arrow keys are pressed
    let arrow_keys = [
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
    ];

    let pressed_arrows: Vec<KeyCode> = arrow_keys
        .iter()
        .filter(|&&key| keyboard_input.just_pressed(key))
        .copied()
        .collect();

    if !pressed_arrows.is_empty() {
        debug!("[NUDGE] Arrow keys pressed: {:?}", pressed_arrows);
        debug!(
            "[NUDGE] Selected points count: {}",
            queries.p0().iter().count()
        );
    }

    // Check for arrow key presses
    let nudge_amount = if keyboard_input.pressed(KeyCode::ShiftLeft)
        || keyboard_input.pressed(KeyCode::ShiftRight)
    {
        settings.nudge.shift
    } else if keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight)
        || keyboard_input.pressed(KeyCode::SuperLeft)
        || keyboard_input.pressed(KeyCode::SuperRight)
    {
        settings.nudge.cmd
    } else {
        settings.nudge.default
    };

    let mut nudge_direction = Vec2::ZERO;

    // Check each arrow key
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        nudge_direction.x = -nudge_amount;
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        nudge_direction.x = nudge_amount;
    } else if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        nudge_direction.y = nudge_amount;
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        nudge_direction.y = -nudge_amount;
    }

    // If we have a nudge direction, apply it to all selected points
    if nudge_direction != Vec2::ZERO {
        let selected_count = queries.p0().iter().count();
        if selected_count > 0 {
            debug!(
                "[NUDGE] Nudging {} selected points by {:?}",
                selected_count, nudge_direction
            );

            debug!("[NUDGE] Setting is_nudging = true");
            nudge_state.is_nudging = true;
            nudge_state.last_nudge_time = time.elapsed_secs();

            // ATOMIC UPDATE: Update FontIR working copies FIRST, then update Transforms
            // This ensures perfect sync between outline and points rendering

            // Collect all point movements using shared logic
            let mut all_movements = Vec::new();

            // Collect selected point data to avoid borrowing conflicts
            let selected_point_data: Vec<_> = queries
                .p0()
                .iter()
                .map(
                    |(entity, transform, point_ref, sort_point_entity_opt, point_type)| {
                        (
                            entity,
                            transform.translation.truncate(),
                            point_ref.clone(),
                            sort_point_entity_opt.is_some(),
                            point_type.copied(),
                        )
                    },
                )
                .collect();

            // Process selected points and collect their movements
            for (entity, old_pos, point_ref, has_sort_entity, point_type) in selected_point_data {
                let new_pos = old_pos + nudge_direction;

                debug!(
                    "[NUDGE] Preparing update for point {:?} from ({:.1}, {:.1}) to ({:.1}, {:.1})",
                    entity, old_pos.x, old_pos.y, new_pos.x, new_pos.y
                );

                // Add selected point movement
                if has_sort_entity {
                    all_movements.push(PointMovement {
                        entity,
                        point_ref: point_ref.clone(),
                        new_position: new_pos,
                        is_connected_offcurve: false,
                    });

                    // Find connected off-curve points if this is an on-curve point
                    if let Some(pt) = point_type {
                        if pt.is_on_curve {
                            let connected_movements = find_connected_offcurve_points_nudge(
                                &point_ref,
                                &pt,
                                nudge_direction,
                                &queries.p2(),
                            );
                            all_movements.extend(connected_movements);
                        }
                    }
                }
            }

            // STEP 1: Update Transform components FIRST (for immediate point rendering)
            for movement in &all_movements {
                if movement.is_connected_offcurve {
                    // Update connected off-curve points
                    if let Ok((_, mut transform, _, _)) = queries.p2().get_mut(movement.entity) {
                        transform.translation.x = movement.new_position.x;
                        transform.translation.y = movement.new_position.y;
                        debug!(
                            "[NUDGE] Transform: Updated connected off-curve position for {:?} to ({:.1}, {:.1})",
                            movement.entity, movement.new_position.x, movement.new_position.y
                        );
                    }
                } else {
                    // Update selected points
                    if let Ok((_, mut transform, _, _, _)) = queries.p0().get_mut(movement.entity) {
                        transform.translation.x = movement.new_position.x;
                        transform.translation.y = movement.new_position.y;
                        debug!(
                            "[NUDGE] Transform: Updated position for {:?} to ({:.1}, {:.1})",
                            movement.entity, movement.new_position.x, movement.new_position.y
                        );
                    }
                }
            }

            // Smooth constraints will be handled uniformly by update_glyph_data_from_selection
            // when it detects the Transform changes from nudging

            // STEP 2: Skip FontIR working copy updates during active nudging
            // Working copy will be updated when nudging completes to avoid timing issues
            debug!(
                "[NUDGE] Skipping FontIR updates during active nudging - will sync on completion"
            );

            // Create an edit event for undo/redo
            event_writer.write(EditEvent {});
        } else {
            debug!("[NUDGE] Arrow key pressed but no selected points found");
        }
    } else {
        // If nudging was active but no keys are pressed, sync immediately and reset state
        if nudge_state.is_nudging {
            debug!("[NUDGE] Keys released, syncing immediately and resetting nudge state");
            nudge_state.is_nudging = false;
        }
    }
}

/// System to reset nudge state after a short delay
pub fn reset_nudge_state(mut nudge_state: ResMut<NudgeState>, time: Res<Time>) {
    if nudge_state.is_nudging && time.elapsed_secs() - nudge_state.last_nudge_time > 0.5 {
        debug!("[NUDGE] Resetting nudge state after timeout");
        nudge_state.is_nudging = false;
    }
}

/// System to sync nudged points back to font data when nudging completes
#[allow(clippy::type_complexity)]
pub fn sync_nudged_points_on_completion(
    nudge_state: Res<NudgeState>,
    selected_query: Query<
        (
            &Transform,
            &GlyphPointReference,
            Option<&SortPointEntity>,
            Option<&PointType>,
        ),
        With<Selected>,
    >,
    all_points_query: Query<
        (&Transform, &GlyphPointReference, &PointType),
        (With<SortPointEntity>, Without<Selected>),
    >,
    _sort_query: Query<(&crate::editing::sort::Sort, &Transform)>,
    mut app_state: Option<ResMut<crate::core::state::AppState>>,
    mut fontir_app_state: Option<ResMut<crate::core::state::FontIRAppState>>,
    mut last_nudge_state: Local<bool>,
) {
    // Only sync when transitioning from nudging to not nudging
    if *last_nudge_state && !nudge_state.is_nudging {
        debug!("[NUDGE] Nudging completed, syncing points to font data");

        let mut all_sync_movements = Vec::new();

        // Collect selected points and their connected off-curves for syncing
        for (transform, point_ref, _sort_point_entity_opt, point_type) in selected_query.iter() {
            let current_pos = transform.translation.truncate();

            // Add selected point
            all_sync_movements.push(PointMovement {
                entity: Entity::PLACEHOLDER, // We don't need entity for syncing
                point_ref: point_ref.clone(),
                new_position: current_pos,
                is_connected_offcurve: false,
            });

            // Find connected off-curve points if this is an on-curve point
            if let Some(pt) = point_type {
                if pt.is_on_curve {
                    for (other_transform, other_ref, other_type) in all_points_query.iter() {
                        if other_ref.glyph_name == point_ref.glyph_name
                            && other_ref.contour_index == point_ref.contour_index
                            && !other_type.is_on_curve
                        {
                            // Check if this off-curve is adjacent (before or after)
                            let is_next = other_ref.point_index == point_ref.point_index + 1;
                            let is_prev = point_ref.point_index > 0
                                && other_ref.point_index == point_ref.point_index - 1;

                            if is_next || is_prev {
                                let other_pos = other_transform.translation.truncate();
                                all_sync_movements.push(PointMovement {
                                    entity: Entity::PLACEHOLDER, // We don't need entity for syncing
                                    point_ref: other_ref.clone(),
                                    new_position: other_pos,
                                    is_connected_offcurve: true,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Use shared utility to sync all movements
        let result = sync_to_font_data(&all_sync_movements, &mut fontir_app_state, &mut app_state);
        let total_synced = result.points_moved + result.connected_offcurves_moved;

        if total_synced > 0 {
            debug!(
                "[NUDGE] Successfully synced {} points ({} selected, {} connected off-curves) to font data",
                total_synced, result.points_moved, result.connected_offcurves_moved
            );
        }
    }

    *last_nudge_state = nudge_state.is_nudging;
}

/// Plugin for the nudge system
pub struct NudgePlugin;

impl Plugin for NudgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NudgeState>().add_systems(
            Update,
            (
                handle_nudge_input,
                reset_nudge_state,
                sync_nudged_points_on_completion,
            )
                .chain()
                .before(super::systems::update_glyph_data_from_selection),
        );
    }
}

/// Event for nudge operations
#[derive(Event)]
pub struct EditEvent {}

/// Point coordinates component
#[derive(Component, Debug, Clone, Copy)]
pub struct PointCoordinates {
    pub x: f32,
    pub y: f32,
}
