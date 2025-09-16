//! Point drag handling for selection

use crate::core::io::pointer::PointerInfo;
use crate::core::settings::BezySettings;
use crate::core::state::{AppState, FontIRAppState};
use crate::editing::selection::components::{GlyphPointReference, PointType, Selected};
use crate::editing::selection::nudge::{EditEvent, PointCoordinates};
use crate::editing::selection::point_movement::{find_connected_offcurve_points_drag, sync_to_font_data, PointMovement};
use crate::editing::selection::DragPointState;
use bevy::input::ButtonInput;
use bevy::log::{debug, warn};
use bevy::prelude::*;

/// System to handle advanced point dragging with constraints and snapping
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn handle_point_drag(
    pointer_info: Res<PointerInfo>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut drag_point_state: ResMut<DragPointState>,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &mut PointCoordinates,
            Option<&GlyphPointReference>,
            Option<&crate::editing::sort::manager::SortCrosshair>,
            Option<&PointType>,
        ),
        With<Selected>,
    >,
    mut all_points_query: Query<
        (
            Entity,
            &mut Transform,
            &mut PointCoordinates,
            &GlyphPointReference,
            &crate::editing::selection::components::PointType,
        ),
        Without<Selected>,
    >,
    mut app_state: Option<ResMut<AppState>>,
    mut fontir_app_state: Option<ResMut<FontIRAppState>>,
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
        let mut point_movements = Vec::new();

        // First, process selected points and collect movement data
        for (entity, mut transform, mut coordinates, point_ref, sort_crosshair, point_type) in &mut query {
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

                    // Add this point to movements list
                    point_movements.push(PointMovement {
                        entity,
                        point_ref: point_ref.clone(),
                        new_position: snapped_pos,
                        is_connected_offcurve: false,
                    });

                    // Find connected off-curve points if this is an on-curve point
                    if let Some(pt) = point_type {
                        if pt.is_on_curve {
                            let connected_movements = find_connected_offcurve_points_drag(
                                point_ref, pt, movement, &all_points_query
                            );
                            point_movements.extend(connected_movements);
                        }
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

        // Update connected off-curve points using shared utility
        for movement in &point_movements {
            if movement.is_connected_offcurve {
                if let Ok((_, mut transform, mut coordinates, _, _)) = all_points_query.get_mut(movement.entity) {
                    // Store original position if not already stored
                    if !drag_point_state.original_positions.contains_key(&movement.entity) {
                        drag_point_state.original_positions.insert(
                            movement.entity,
                            Vec2::new(transform.translation.x, transform.translation.y),
                        );
                    }

                    // Update to the new position (already calculated in shared utility)
                    transform.translation.x = movement.new_position.x;
                    transform.translation.y = movement.new_position.y;
                    transform.translation.z = 5.0;
                    coordinates.x = movement.new_position.x;
                    coordinates.y = movement.new_position.y;
                }
            }
        }

        // Sync all movements to font data using shared utility
        let result = sync_to_font_data(&point_movements, &mut fontir_app_state, &mut app_state);
        updated_count = result.points_moved + result.connected_offcurves_moved;

        if updated_count > 0 {
            debug!("Updated {} points ({} selected, {} connected off-curves) during drag",
                   updated_count, result.points_moved, result.connected_offcurves_moved);

            // Send edit event
            event_writer.write(EditEvent {
            });
        }
    }
}
