//! Shared logic for point movement operations (dragging, nudging, etc.)
//!
//! This module provides common functionality for moving points and their
//! connected off-curve handles, ensuring consistent behavior across all
//! point movement operations.

use crate::core::state::{AppState, FontIRAppState};
use crate::editing::selection::components::{GlyphPointReference, PointType, Selected};
use crate::editing::sort::manager::SortPointEntity;
use bevy::log::{debug, warn};
use bevy::prelude::*;

// Type alias for mutable query (used in nudge.rs)
type UnselectedPointsQueryMut<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut Transform, &'static GlyphPointReference, &'static PointType),
    (With<SortPointEntity>, Without<Selected>),
>;

/// Information about a point that needs to be moved
#[derive(Debug, Clone)]
pub struct PointMovement {
    pub entity: Entity,
    pub point_ref: GlyphPointReference,
    pub new_position: Vec2,
    pub is_connected_offcurve: bool,
}

/// Result of a point movement operation
#[derive(Debug)]
pub struct MovementResult {
    pub points_moved: usize,
    pub connected_offcurves_moved: usize,
}

/// Find connected off-curve points for drag operations
pub fn find_connected_offcurve_points_drag(
    selected_point_ref: &GlyphPointReference,
    selected_point_type: &PointType,
    movement: Vec2,
    all_points_query: &Query<
        (
            Entity,
            &mut Transform,
            &mut crate::editing::selection::nudge::PointCoordinates,
            &GlyphPointReference,
            &PointType,
        ),
        Without<Selected>,
    >,
) -> Vec<PointMovement> {
    let mut connected_movements = Vec::new();

    // Only process on-curve points
    if !selected_point_type.is_on_curve {
        return connected_movements;
    }

    // Find adjacent off-curve points
    for (entity, transform, _, point_ref, point_type) in all_points_query.iter() {
        if point_ref.glyph_name == selected_point_ref.glyph_name
            && point_ref.contour_index == selected_point_ref.contour_index
            && !point_type.is_on_curve
        {
            // Check if this off-curve is adjacent (before or after)
            let is_next = point_ref.point_index == selected_point_ref.point_index + 1;
            let is_prev = selected_point_ref.point_index > 0
                && point_ref.point_index == selected_point_ref.point_index - 1;

            if is_next || is_prev {
                let current_pos = transform.translation.truncate();
                let new_pos = current_pos + movement;

                connected_movements.push(PointMovement {
                    entity,
                    point_ref: point_ref.clone(),
                    new_position: new_pos,
                    is_connected_offcurve: true,
                });

                debug!(
                    "[POINT_MOVEMENT] Found connected off-curve point {:?} to move from ({:.1}, {:.1}) to ({:.1}, {:.1})",
                    entity, current_pos.x, current_pos.y, new_pos.x, new_pos.y
                );
            }
        }
    }

    connected_movements
}

/// Find connected off-curve points for nudge operations
pub fn find_connected_offcurve_points_nudge(
    selected_point_ref: &GlyphPointReference,
    selected_point_type: &PointType,
    movement: Vec2,
    all_points_query: &UnselectedPointsQueryMut,
) -> Vec<PointMovement> {
    let mut connected_movements = Vec::new();

    // Only process on-curve points
    if !selected_point_type.is_on_curve {
        return connected_movements;
    }

    // Find adjacent off-curve points
    for (entity, transform, point_ref, point_type) in all_points_query.iter() {
        if point_ref.glyph_name == selected_point_ref.glyph_name
            && point_ref.contour_index == selected_point_ref.contour_index
            && !point_type.is_on_curve
        {
            // Check if this off-curve is adjacent (before or after)
            let is_next = point_ref.point_index == selected_point_ref.point_index + 1;
            let is_prev = selected_point_ref.point_index > 0
                && point_ref.point_index == selected_point_ref.point_index - 1;

            if is_next || is_prev {
                let current_pos = transform.translation.truncate();
                let new_pos = current_pos + movement;

                connected_movements.push(PointMovement {
                    entity,
                    point_ref: point_ref.clone(),
                    new_position: new_pos,
                    is_connected_offcurve: true,
                });

                debug!(
                    "[POINT_MOVEMENT] Found connected off-curve point {:?} to move from ({:.1}, {:.1}) to ({:.1}, {:.1})",
                    entity, current_pos.x, current_pos.y, new_pos.x, new_pos.y
                );
            }
        }
    }

    connected_movements
}

/// Update Transform components for a list of point movements
pub fn update_transforms<T>(
    point_movements: &[PointMovement],
    query: &mut Query<&mut Transform, T>,
) -> usize
where
    T: bevy::ecs::query::QueryFilter,
{
    let mut updated_count = 0;

    for movement in point_movements {
        if let Ok(mut transform) = query.get_mut(movement.entity) {
            transform.translation.x = movement.new_position.x;
            transform.translation.y = movement.new_position.y;
            // Keep Z as is for different layers (glyph points vs crosshairs)
            updated_count += 1;

            debug!(
                "[POINT_MOVEMENT] Transform: Updated {} point {:?} to ({:.1}, {:.1})",
                if movement.is_connected_offcurve {
                    "connected off-curve"
                } else {
                    "selected"
                },
                movement.entity,
                movement.new_position.x,
                movement.new_position.y
            );
        }
    }

    updated_count
}

/// Sync point movements to FontIR and AppState data
pub fn sync_to_font_data(
    point_movements: &[PointMovement],
    fontir_app_state: &mut Option<ResMut<FontIRAppState>>,
    app_state: &mut Option<ResMut<AppState>>,
) -> MovementResult {
    let mut points_moved = 0;
    let mut connected_offcurves_moved = 0;

    for movement in point_movements {
        let mut handled = false;

        // Try FontIR first, then fallback to UFO AppState
        if let Some(ref mut fontir_state) = fontir_app_state {
            match fontir_state.update_point_position(
                &movement.point_ref.glyph_name,
                movement.point_ref.contour_index,
                movement.point_ref.point_index,
                movement.new_position.x as f64,
                movement.new_position.y as f64,
            ) {
                Ok(was_updated) => {
                    if was_updated {
                        if movement.is_connected_offcurve {
                            connected_offcurves_moved += 1;
                        } else {
                            points_moved += 1;
                        }
                        handled = true;
                        debug!(
                            "[POINT_MOVEMENT] FontIR: Updated {} point {} in glyph '{}'",
                            if movement.is_connected_offcurve {
                                "connected off-curve"
                            } else {
                                "selected"
                            },
                            movement.point_ref.point_index,
                            movement.point_ref.glyph_name
                        );
                    }
                }
                Err(e) => {
                    warn!("[POINT_MOVEMENT] Failed to update FontIR point: {}", e);
                }
            }
        }

        // Fallback to UFO AppState if FontIR didn't handle it
        if !handled && app_state.is_some() {
            if let Some(ref mut state) = app_state {
                let updated = state.set_point_position(
                    &movement.point_ref.glyph_name,
                    movement.point_ref.contour_index,
                    movement.point_ref.point_index,
                    movement.new_position.x as f64,
                    movement.new_position.y as f64,
                );
                if updated {
                    if movement.is_connected_offcurve {
                        connected_offcurves_moved += 1;
                    } else {
                        points_moved += 1;
                    }
                    handled = true;
                    debug!(
                        "[POINT_MOVEMENT] UFO: Updated {} point: glyph='{}' contour={} point={} pos=({:.1}, {:.1})",
                        if movement.is_connected_offcurve { "connected off-curve" } else { "selected" },
                        movement.point_ref.glyph_name,
                        movement.point_ref.contour_index,
                        movement.point_ref.point_index,
                        movement.new_position.x,
                        movement.new_position.y
                    );
                }
            }
        }

        // If neither FontIR nor UFO handled it, just track the Transform update
        if !handled {
            if movement.is_connected_offcurve {
                connected_offcurves_moved += 1;
            } else {
                points_moved += 1;
            }
            debug!(
                "[POINT_MOVEMENT] Point update handled via Transform only (no source data update)"
            );
        }
    }

    MovementResult {
        points_moved,
        connected_offcurves_moved,
    }
}
