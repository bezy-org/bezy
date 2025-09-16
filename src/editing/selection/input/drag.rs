//! Point drag handling for selection

use crate::core::io::pointer::PointerInfo;
use crate::core::settings::BezySettings;
use crate::core::state::{AppState, FontIRAppState};
use crate::editing::selection::components::{GlyphPointReference, PointType, Selected};
use crate::editing::selection::nudge::{EditEvent, PointCoordinates};
use crate::editing::selection::point_movement::{find_connected_offcurve_points_drag, sync_to_font_data, PointMovement};
use crate::editing::selection::smooth_curves::{find_all_smooth_constraints, apply_smooth_curve_constraints, update_smooth_constraint_transforms};
use crate::editing::selection::enhanced_point_component::EnhancedPointType;
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
    enhanced_points_query: Query<(Entity, &EnhancedPointType, &GlyphPointReference)>,
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

        // Handle smooth curve constraints for any off-curve points that were moved
        let mut smooth_adjustments = Vec::new();

        // Get the current glyph name from any point reference
        if let Some(first_movement) = point_movements.first() {
            let glyph_name = &first_movement.point_ref.glyph_name;

            // Create a simpler query interface by collecting data first
            let enhanced_point_data: Vec<_> = enhanced_points_query.iter().collect();
            let all_point_data: Vec<_> = all_points_query.iter()
                .map(|(entity, transform, _coords, point_ref, point_type)| {
                    (entity, transform.translation.truncate(), point_ref.clone(), *point_type)
                })
                .collect();

            // Find all smooth curve constraints manually
            for (_entity, enhanced, point_ref) in &enhanced_point_data {
                if point_ref.glyph_name == *glyph_name
                    && enhanced.is_on_curve
                    && enhanced.is_smooth()
                {
                    // Find handles for this smooth point
                    let mut left_handle = None;
                    let mut right_handle = None;

                    for (other_entity, _pos, other_ref, other_type) in &all_point_data {
                        if other_ref.glyph_name == point_ref.glyph_name
                            && other_ref.contour_index == point_ref.contour_index
                            && !other_type.is_on_curve
                        {
                            // Check if this is the previous off-curve (left handle)
                            if point_ref.point_index > 0
                                && other_ref.point_index == point_ref.point_index - 1
                            {
                                left_handle = Some(*other_entity);
                            }
                            // Check if this is the next off-curve (right handle)
                            else if other_ref.point_index == point_ref.point_index + 1 {
                                right_handle = Some(*other_entity);
                            }
                        }
                    }

                    // Check if any moved off-curve points are part of this constraint
                    for movement in &point_movements {
                        if movement.is_connected_offcurve {
                            if left_handle == Some(movement.entity) || right_handle == Some(movement.entity) {
                                // Calculate opposite handle position
                                let smooth_point_pos = enhanced.coords();
                                let smooth_point_vec2 = Vec2::new(smooth_point_pos.0 as f32, smooth_point_pos.1 as f32);
                                let handle_vector = movement.new_position - smooth_point_vec2;
                                let opposite_vector = -handle_vector;
                                let opposite_position = smooth_point_vec2 + opposite_vector;

                                // Apply the constraint to the opposite handle
                                if left_handle == Some(movement.entity) && right_handle.is_some() {
                                    // Left handle moved, adjust right handle
                                    smooth_adjustments.push((right_handle.unwrap(), opposite_position));
                                    debug!("Smooth constraint: left handle moved, adjusting right handle to ({:.1}, {:.1})", opposite_position.x, opposite_position.y);
                                } else if right_handle == Some(movement.entity) && left_handle.is_some() {
                                    // Right handle moved, adjust left handle
                                    smooth_adjustments.push((left_handle.unwrap(), opposite_position));
                                    debug!("Smooth constraint: right handle moved, adjusting left handle to ({:.1}, {:.1})", opposite_position.x, opposite_position.y);
                                }
                            }
                        }
                    }
                }
            }

            // Apply smooth constraint adjustments
            if !smooth_adjustments.is_empty() {
                for (entity, new_position) in &smooth_adjustments {
                    if let Ok((_, mut transform, mut coordinates, _, _)) = all_points_query.get_mut(*entity) {
                        transform.translation.x = new_position.x;
                        transform.translation.y = new_position.y;
                        transform.translation.z = 5.0;
                        coordinates.x = new_position.x;
                        coordinates.y = new_position.y;

                        // Store original position for newly adjusted points
                        if !drag_point_state.original_positions.contains_key(entity) {
                            drag_point_state.original_positions.insert(
                                *entity,
                                *new_position,
                            );
                        }
                    }
                }

                debug!("Applied {} smooth curve constraint adjustments", smooth_adjustments.len());
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
