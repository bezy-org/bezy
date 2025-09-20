//! Smooth curve handling for UFO-compliant point editing
//!
//! This module implements the logic for maintaining smooth curve tangents
//! when editing points with the smooth=true flag.

use crate::core::state::ufo_point::{UfoPoint, UfoPointType};
use crate::editing::selection::components::{GlyphPointReference, PointType};
use crate::editing::selection::enhanced_point_component::EnhancedPointType;
use bevy::ecs::system::ParamSet;
use bevy::log::{debug, info};
use bevy::prelude::*;

/// Simple function to find direct neighbor handles in a contour
/// Returns (left_handle, right_handle) where either or both can be None
pub fn find_direct_neighbor_handles(
    smooth_point_ref: &GlyphPointReference,
    point_data: &[(Entity, Vec2, GlyphPointReference, PointType)],
) -> (
    Option<(Entity, Vec2, GlyphPointReference)>,
    Option<(Entity, Vec2, GlyphPointReference)>,
) {
    // Get all points in the same contour
    let mut contour_points: Vec<_> = point_data
        .iter()
        .filter(|(_, _, p_ref, _)| {
            p_ref.glyph_name == smooth_point_ref.glyph_name
                && p_ref.contour_index == smooth_point_ref.contour_index
        })
        .collect();

    // Sort by point index
    contour_points.sort_by_key(|(_, _, p_ref, _)| p_ref.point_index);

    let num_points = contour_points.len();
    if num_points == 0 {
        return (None, None);
    }

    // Find our smooth point position
    let smooth_pos = contour_points
        .iter()
        .position(|(_, _, p_ref, _)| p_ref.point_index == smooth_point_ref.point_index);

    if let Some(pos) = smooth_pos {
        // Check previous point (wrapping around for closed contours)
        let prev_idx = if pos > 0 { pos - 1 } else { num_points - 1 };
        let left_handle =
            if let Some((entity, pos, p_ref, point_type)) = contour_points.get(prev_idx) {
                if !point_type.is_on_curve {
                    Some((*entity, *pos, (*p_ref).clone()))
                } else {
                    None
                }
            } else {
                None
            };

        // Check next point (wrapping around for closed contours)
        let next_idx = (pos + 1) % num_points;
        let right_handle =
            if let Some((entity, pos, p_ref, point_type)) = contour_points.get(next_idx) {
                if !point_type.is_on_curve {
                    Some((*entity, *pos, (*p_ref).clone()))
                } else {
                    None
                }
            } else {
                None
            };

        (left_handle, right_handle)
    } else {
        (None, None)
    }
}
use kurbo::Point;

/// Information about a smooth curve constraint
#[derive(Debug, Clone)]
pub struct SmoothCurveConstraint {
    /// The on-curve point that is smooth
    pub smooth_point: Entity,
    /// The point reference for the smooth point
    pub smooth_point_ref: GlyphPointReference,
    /// Left handle (previous off-curve point)
    pub left_handle: Option<Entity>,
    /// Right handle (next off-curve point)
    pub right_handle: Option<Entity>,
}

/// Result of applying smooth curve constraints
#[derive(Debug)]
pub struct SmoothCurveResult {
    /// Points that were automatically adjusted to maintain smoothness
    pub adjusted_points: Vec<(Entity, Vec2)>,
}

/// Find the actual control handles that connect to a smooth point through curve segments
///
/// ## How Smooth Point Collinear System Works:
///
/// When an on-curve point is marked as "smooth", any off-curve control points that are
/// directly adjacent to it in the contour path should be positioned collinearly with
/// the smooth on-curve point. This ensures smooth curve transitions without kinks.
///
/// **Example contour sequence:**
/// ```
/// [on-curve] → [off-curve] → [SMOOTH on-curve] → [off-curve] → [on-curve]
///                    ↑              ↑                ↑
///                left handle    smooth point    right handle
/// ```
///
/// The left and right off-curve handles must be collinear through the smooth point:
/// - If you move one handle, the opposite handle automatically adjusts to maintain collinearity
/// - The handles don't need to be the same distance from the smooth point
/// - They just need to lie on the same straight line passing through the smooth point
///
/// This function analyzes the curve structure rather than just looking at consecutive point indices,
/// because point storage order may not match the actual curve flow.
pub fn find_curve_handles_for_smooth_point(
    smooth_entity: Entity,
    smooth_point_ref: &GlyphPointReference,
    point_data: &[(Entity, Vec2, GlyphPointReference, PointType)],
) -> Option<SmoothCurveConstraint> {
    // Get all points in the same contour, sorted by point index
    let mut contour_points: Vec<_> = point_data
        .iter()
        .filter(|(_, _, p_ref, _)| {
            p_ref.glyph_name == smooth_point_ref.glyph_name
                && p_ref.contour_index == smooth_point_ref.contour_index
        })
        .collect();

    // Sort by point index to understand the contour order
    contour_points.sort_by_key(|(_, _, p_ref, _)| p_ref.point_index);

    // Find the position of our smooth point in the sorted list
    let smooth_position = contour_points
        .iter()
        .position(|(entity, _, _, _)| *entity == smooth_entity);

    let smooth_idx = if let Some(pos) = smooth_position {
        pos
    } else {
        debug!("Could not find smooth point in contour");
        return None;
    };

    let mut left_handle = None;
    let mut right_handle = None;

    // Look backwards from smooth point to find the previous on-curve point and any handles
    let mut idx = if smooth_idx == 0 {
        contour_points.len() - 1
    } else {
        smooth_idx - 1
    };
    while idx != smooth_idx {
        let (entity, _, _, point_type) = contour_points[idx];

        if point_type.is_on_curve {
            // Found the previous on-curve point, stop looking
            break;
        } else {
            // This is an off-curve point (handle) between previous on-curve and our smooth point
            left_handle = Some(*entity);
            debug!(
                "Found left handle {:?} for smooth point {:?} (moving backwards)",
                entity, smooth_entity
            );
            break; // We want the handle closest to our smooth point
        }
    }

    // Look forwards from smooth point to find the next on-curve point and any handles
    idx = (smooth_idx + 1) % contour_points.len();
    while idx != smooth_idx {
        let (entity, _, _, point_type) = contour_points[idx];

        if point_type.is_on_curve {
            // Found the next on-curve point, stop looking
            break;
        } else {
            // This is an off-curve point (handle) between our smooth point and next on-curve
            right_handle = Some(*entity);
            debug!(
                "Found right handle {:?} for smooth point {:?} (moving forwards)",
                entity, smooth_entity
            );
            break; // We want the handle closest to our smooth point
        }
    }

    // Only create constraint if we have at least one handle
    if left_handle.is_some() || right_handle.is_some() {
        debug!(
            "Created smooth constraint for point {:?} with left_handle: {:?}, right_handle: {:?}",
            smooth_entity, left_handle, right_handle
        );
        Some(SmoothCurveConstraint {
            smooth_point: smooth_entity,
            smooth_point_ref: smooth_point_ref.clone(),
            left_handle,
            right_handle,
        })
    } else {
        debug!("No handles found for smooth point {:?}", smooth_entity);
        None
    }
}

/// Find smooth curve constraints for a given smooth on-curve point
pub fn find_smooth_curve_constraints(
    smooth_point_entity: Entity,
    smooth_point_ref: &GlyphPointReference,
    all_points_query: &Query<(Entity, &Transform, &GlyphPointReference, &PointType)>,
) -> Option<SmoothCurveConstraint> {
    let mut left_handle = None;
    let mut right_handle = None;

    // Find adjacent off-curve points
    for (entity, _transform, point_ref, point_type) in all_points_query.iter() {
        if point_ref.glyph_name == smooth_point_ref.glyph_name
            && point_ref.contour_index == smooth_point_ref.contour_index
            && !point_type.is_on_curve
        {
            // Check if this is the previous off-curve (left handle)
            if smooth_point_ref.point_index > 0
                && point_ref.point_index == smooth_point_ref.point_index - 1
            {
                left_handle = Some(entity);
                debug!(
                    "Found left handle {:?} for smooth point {:?}",
                    entity, smooth_point_entity
                );
            }
            // Check if this is the next off-curve (right handle)
            else if point_ref.point_index == smooth_point_ref.point_index + 1 {
                right_handle = Some(entity);
                debug!(
                    "Found right handle {:?} for smooth point {:?}",
                    entity, smooth_point_entity
                );
            }
        }
    }

    // Only create constraint if we have at least one handle
    if left_handle.is_some() || right_handle.is_some() {
        Some(SmoothCurveConstraint {
            smooth_point: smooth_point_entity,
            smooth_point_ref: smooth_point_ref.clone(),
            left_handle,
            right_handle,
        })
    } else {
        None
    }
}

/// Apply smooth curve constraints when a handle is moved
/// This ensures collinearity between handles across the smooth point
pub fn apply_smooth_curve_constraints(
    constraint: &SmoothCurveConstraint,
    moved_handle: Entity,
    new_handle_position: Vec2,
    all_points_query: &Query<(Entity, &Transform, &GlyphPointReference, &PointType)>,
) -> SmoothCurveResult {
    let mut adjusted_points = Vec::new();

    // Get the smooth point position
    let smooth_point_pos =
        if let Ok((_, transform, _, _)) = all_points_query.get(constraint.smooth_point) {
            transform.translation.truncate()
        } else {
            debug!("Could not find smooth point position");
            return SmoothCurveResult { adjusted_points };
        };

    // Determine which handle was moved and calculate the opposite handle position
    let moved_is_left = constraint.left_handle == Some(moved_handle);
    let moved_is_right = constraint.right_handle == Some(moved_handle);

    if !moved_is_left && !moved_is_right {
        debug!("Moved handle is not part of this constraint");
        return SmoothCurveResult { adjusted_points };
    }

    // Calculate the vector from smooth point to moved handle
    let handle_vector = new_handle_position - smooth_point_pos;

    // The opposite handle should be on the opposite side of the smooth point
    // maintaining the same distance and opposite direction for perfect smoothness
    let opposite_vector = -handle_vector;
    let opposite_position = smooth_point_pos + opposite_vector;

    // Apply the constraint to the opposite handle
    if moved_is_left {
        // Left handle moved, adjust right handle
        if let Some(right_entity) = constraint.right_handle {
            adjusted_points.push((right_entity, opposite_position));
            debug!(
                "Smooth constraint: left handle moved to ({:.1}, {:.1}), adjusting right handle to ({:.1}, {:.1})",
                new_handle_position.x, new_handle_position.y,
                opposite_position.x, opposite_position.y
            );
        }
    } else if moved_is_right {
        // Right handle moved, adjust left handle
        if let Some(left_entity) = constraint.left_handle {
            adjusted_points.push((left_entity, opposite_position));
            debug!(
                "Smooth constraint: right handle moved to ({:.1}, {:.1}), adjusting left handle to ({:.1}, {:.1})",
                new_handle_position.x, new_handle_position.y,
                opposite_position.x, opposite_position.y
            );
        }
    }

    SmoothCurveResult { adjusted_points }
}

/// Check if a point is marked as smooth using enhanced point data
pub fn is_point_smooth(entity: Entity, enhanced_points: &Query<&EnhancedPointType>) -> bool {
    if let Ok(enhanced) = enhanced_points.get(entity) {
        enhanced.is_smooth()
    } else {
        false
    }
}

/// Find all smooth curve constraints in the current selection/glyph
pub fn find_all_smooth_constraints(
    enhanced_points: &Query<(Entity, &EnhancedPointType, &GlyphPointReference)>,
    all_points_query: &Query<(Entity, &Transform, &GlyphPointReference, &PointType)>,
    glyph_name: &str,
) -> Vec<SmoothCurveConstraint> {
    let mut constraints = Vec::new();

    // Find all smooth on-curve points
    for (entity, enhanced, point_ref) in enhanced_points.iter() {
        if point_ref.glyph_name == glyph_name && enhanced.is_on_curve && enhanced.is_smooth() {
            if let Some(constraint) =
                find_smooth_curve_constraints(entity, point_ref, all_points_query)
            {
                debug!(
                    "Found smooth constraint for point {:?} with {} handles",
                    entity,
                    constraint.left_handle.is_some() as usize
                        + constraint.right_handle.is_some() as usize
                );
                constraints.push(constraint);
            }
        }
    }

    debug!("Found {} smooth curve constraints", constraints.len());
    constraints
}

/// Update transforms for points affected by smooth curve constraints
pub fn update_smooth_constraint_transforms<T>(
    adjusted_points: &[(Entity, Vec2)],
    transform_query: &mut Query<&mut Transform, T>,
) -> usize
where
    T: bevy::ecs::query::QueryFilter,
{
    let mut updated_count = 0;

    for (entity, new_position) in adjusted_points {
        if let Ok(mut transform) = transform_query.get_mut(*entity) {
            transform.translation.x = new_position.x;
            transform.translation.y = new_position.y;
            updated_count += 1;
            debug!(
                "Applied smooth constraint: moved entity {:?} to ({:.1}, {:.1})",
                entity, new_position.x, new_position.y
            );
        }
    }

    updated_count
}

/// System to automatically apply smooth curve constraints when a point becomes smooth
///
/// ## Automatic Smooth Point Enforcement:
///
/// This system is triggered when an on-curve point's `EnhancedPointType` changes to smooth
/// (typically through double-clicking the point). When this happens:
///
/// 1. **Find adjacent off-curve handles**: Locate any off-curve control points that are
///    directly next to the smooth point in the contour path (one before, one after)
///
/// 2. **Enforce collinearity**: Position the handles so they form a straight line through
///    the smooth point, ensuring smooth curve transitions without kinks
///
/// 3. **Preserve distances**: The handles maintain their distances from the smooth point,
///    only their angles are adjusted to be exactly opposite each other
///
/// This automatic enforcement ensures that smooth points behave correctly as soon as
/// they are marked as smooth, without requiring manual handle adjustment.
pub fn auto_apply_smooth_constraints(
    // Query for points that just became smooth
    changed_enhanced_points: Query<
        (Entity, &EnhancedPointType, &GlyphPointReference),
        Changed<EnhancedPointType>,
    >,
    // All points for finding handles and getting positions (mutable for updates)
    mut all_points_query: Query<(Entity, &mut Transform, &GlyphPointReference, &PointType)>,
) {
    for (smooth_entity, enhanced_type, point_ref) in changed_enhanced_points.iter() {
        // Only process if the point is now smooth and on-curve
        if !enhanced_type.is_smooth() || !enhanced_type.is_on_curve {
            continue;
        }

        debug!(
            "Auto-applying smooth constraints for newly smooth point: glyph='{}', contour={}, point={}",
            point_ref.glyph_name, point_ref.contour_index, point_ref.point_index
        );

        // Collect immutable data for constraint finding
        let point_data: Vec<_> = all_points_query
            .iter()
            .map(|(entity, transform, point_ref, point_type)| {
                (
                    entity,
                    transform.translation.truncate(),
                    point_ref.clone(),
                    *point_type,
                )
            })
            .collect();

        // Find smooth curve constraints for this point using proper curve segment analysis
        let constraint = find_curve_handles_for_smooth_point(smooth_entity, point_ref, &point_data);

        if let Some(constraint) = constraint {
            // Get current smooth point position
            let smooth_pos = point_data
                .iter()
                .find(|(entity, _, _, _)| *entity == smooth_entity)
                .map(|(_, pos, _, _)| *pos);

            let smooth_pos = if let Some(pos) = smooth_pos {
                pos
            } else {
                continue;
            };

            // If we have both handles, align them symmetrically around the smooth point
            if let (Some(left_handle), Some(right_handle)) =
                (constraint.left_handle, constraint.right_handle)
            {
                // Get handle positions from our collected data
                let left_pos = point_data
                    .iter()
                    .find(|(entity, _, _, _)| *entity == left_handle)
                    .map(|(_, pos, _, _)| *pos);
                let right_pos = point_data
                    .iter()
                    .find(|(entity, _, _, _)| *entity == right_handle)
                    .map(|(_, pos, _, _)| *pos);

                if let (Some(left_pos), Some(right_pos)) = (left_pos, right_pos) {
                    // Check if handles are already collinear - if so, don't move anything
                    let left_vector = left_pos - smooth_pos;
                    let right_vector = right_pos - smooth_pos;

                    // Check for collinearity using cross product (should be ~0 for collinear vectors)
                    let cross_product =
                        left_vector.x * right_vector.y - left_vector.y * right_vector.x;
                    let collinearity_threshold = 0.1; // Small threshold for floating point precision

                    if cross_product.abs() < collinearity_threshold {
                        debug!(
                            "Handles already collinear (cross product: {:.6}), skipping auto-smooth adjustment",
                            cross_product
                        );
                        continue;
                    }

                    debug!(
                        "Handles not collinear (cross product: {:.6}), applying auto-smooth constraint",
                        cross_product
                    );

                    // Calculate average direction and preserve both handle distances
                    let left_vector = left_pos - smooth_pos;
                    let right_vector = right_pos - smooth_pos;
                    let left_dist = left_vector.length();
                    let right_dist = right_vector.length();

                    // Use the average direction of both handles
                    let combined_vector = left_vector + right_vector;
                    if combined_vector.length() > 0.001 {
                        let avg_direction = combined_vector.normalize();

                        // Position both handles along the averaged direction, preserving their distances
                        let new_left_pos = smooth_pos + (avg_direction * left_dist);
                        let new_right_pos = smooth_pos + (-avg_direction * right_dist);

                        // Update both handles
                        if let Ok((_, mut left_transform, _, _)) =
                            all_points_query.get_mut(left_handle)
                        {
                            left_transform.translation.x = new_left_pos.x;
                            left_transform.translation.y = new_left_pos.y;
                        }

                        if let Ok((_, mut right_transform, _, _)) =
                            all_points_query.get_mut(right_handle)
                        {
                            right_transform.translation.x = new_right_pos.x;
                            right_transform.translation.y = new_right_pos.y;
                        }

                        debug!(
                            "Applied auto-smooth constraint: aligned both handles preserving distances ({:.1}, {:.1})",
                            left_dist, right_dist
                        );
                    }
                }
            }
            // If we only have one handle, that's fine - it defines the tangent direction
            else if constraint.left_handle.is_some() || constraint.right_handle.is_some() {
                debug!("Smooth point has only one handle - no alignment needed");
            }
        } else {
            debug!("No handles found for smooth point - no constraints to apply");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smooth_curve_constraint_creation() {
        // Test that we can create a smooth curve constraint
        let constraint = SmoothCurveConstraint {
            smooth_point: Entity::from_raw(1),
            smooth_point_ref: GlyphPointReference {
                glyph_name: "a".to_string(),
                contour_index: 0,
                point_index: 1,
            },
            left_handle: Some(Entity::from_raw(2)),
            right_handle: Some(Entity::from_raw(3)),
        };

        assert_eq!(constraint.smooth_point, Entity::from_raw(1));
        assert_eq!(constraint.left_handle, Some(Entity::from_raw(2)));
        assert_eq!(constraint.right_handle, Some(Entity::from_raw(3)));
    }

    #[test]
    fn test_smooth_curve_vector_calculation() {
        // Test the math for maintaining collinearity
        let smooth_point = Vec2::new(100.0, 200.0);
        let left_handle = Vec2::new(50.0, 150.0);

        // Calculate what the right handle should be
        let handle_vector = left_handle - smooth_point;
        let opposite_vector = -handle_vector;
        let expected_right = smooth_point + opposite_vector;

        // Should be symmetric around the smooth point
        assert_eq!(expected_right, Vec2::new(150.0, 250.0));

        // Verify collinearity: vectors should be opposite
        assert_eq!(handle_vector, -opposite_vector);
    }
}

/// Universal smooth constraint system that monitors ALL Transform changes
/// This ensures smooth constraints work regardless of how points are moved (drag, nudge, direct manipulation)
pub fn universal_smooth_constraints(
    // Use ParamSet to avoid query conflicts between immutable and mutable Transform access
    mut param_set: ParamSet<(
        // Query for points that have moved (immutable access)
        Query<(Entity, &Transform, &GlyphPointReference, &PointType), Changed<Transform>>,
        // Query for all points to analyze constraints (immutable access)
        Query<(
            Entity,
            &Transform,
            &GlyphPointReference,
            &PointType,
            Option<&EnhancedPointType>,
        )>,
        // Query for all points to modify (mutable access)
        Query<(
            Entity,
            &mut Transform,
            &GlyphPointReference,
            &PointType,
            Option<&EnhancedPointType>,
        )>,
    )>,
    // Track processed entities to avoid infinite loops
    mut processed: Local<std::collections::HashSet<Entity>>,
) {
    // Clear processed set each frame to allow continuous constraint updates during drag
    processed.clear();

    // First, collect changed points data (no processed filtering for now)
    let changed_data: Vec<_> = param_set
        .p0()
        .iter()
        .map(|(entity, transform, point_ref, point_type)| {
            (
                entity,
                transform.translation.truncate(),
                point_ref.clone(),
                *point_type,
            )
        })
        .collect();

    if changed_data.is_empty() {
        return;
    }

    info!(
        "[SMOOTH UNIVERSAL] System running - {} changed points total",
        changed_data.len()
    );

    // Build point data for constraint analysis
    let point_data: Vec<_> = param_set
        .p1()
        .iter()
        .map(|(entity, transform, point_ref, point_type, _)| {
            (
                entity,
                transform.translation.truncate(),
                point_ref.clone(),
                *point_type,
            )
        })
        .collect();

    // Collect constraint adjustments first to avoid borrow checker conflicts
    let mut constraint_adjustments = Vec::new();

    // For each changed point that's an off-curve handle
    for (moved_entity, moved_pos, moved_ref, moved_point_type) in &changed_data {
        if !moved_point_type.is_on_curve {
            info!(
                "[SMOOTH UNIVERSAL] Handle moved: glyph='{}', contour={}, point={}",
                moved_ref.glyph_name, moved_ref.contour_index, moved_ref.point_index
            );

            // Find smooth on-curve points in the same contour using p1 query
            for (_, smooth_transform, smooth_ref, smooth_point_type, smooth_enhanced) in
                param_set.p1().iter()
            {
                if smooth_ref.glyph_name == moved_ref.glyph_name
                    && smooth_ref.contour_index == moved_ref.contour_index
                    && smooth_point_type.is_on_curve
                    && smooth_enhanced.map_or(false, |e| e.is_smooth())
                {
                    // Use simplified neighbor detection
                    let (left_handle, right_handle) =
                        find_direct_neighbor_handles(smooth_ref, &point_data);

                    let moved_is_left = left_handle
                        .as_ref()
                        .map_or(false, |(_, _, ref_comp)| ref_comp == moved_ref);
                    let moved_is_right = right_handle
                        .as_ref()
                        .map_or(false, |(_, _, ref_comp)| ref_comp == moved_ref);

                    if moved_is_left || moved_is_right {
                        info!(
                            "[SMOOTH UNIVERSAL] Found constraint! smooth_point={}, moved_handle={}",
                            smooth_ref.point_index, moved_ref.point_index
                        );

                        let smooth_pos = smooth_transform.translation.truncate();

                        // Determine which handle to adjust
                        let other_handle = if moved_is_left {
                            right_handle
                        } else {
                            left_handle
                        };

                        if let Some((_, other_pos, other_ref)) = other_handle {
                            // Calculate collinear position preserving the opposite handle's original distance
                            let moved_vector = moved_pos - smooth_pos;
                            let other_distance = smooth_pos.distance(other_pos);

                            // Create a unit vector in the opposite direction of the moved handle
                            let moved_length = moved_vector.length();
                            if moved_length > 0.001 {
                                // Avoid division by zero
                                let moved_unit = moved_vector / moved_length;
                                let opposite_unit = -moved_unit;

                                // Position the opposite handle at its original distance but in the opposite direction
                                let new_other_pos = smooth_pos + (opposite_unit * other_distance);

                                info!(
                                    "[SMOOTH UNIVERSAL] Constraint calculation: moved_dist={:.1}, other_dist={:.1}, preserving other distance",
                                    moved_length, other_distance
                                );

                                constraint_adjustments.push((
                                    other_ref.clone(),
                                    new_other_pos,
                                    *moved_entity,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // Apply all constraint adjustments in a separate phase using mutable query
    for (other_ref, new_other_pos, _moved_entity) in constraint_adjustments {
        // Find and update the other handle using p2 (mutable query)
        for (_, mut other_transform, other_point_ref, _, _) in param_set.p2().iter_mut() {
            if other_point_ref == &other_ref {
                other_transform.translation = new_other_pos.extend(0.0);
                // DON'T mark constraint-adjusted points as processed - they should be able to trigger constraints again

                info!(
                    "[SMOOTH UNIVERSAL] Applied constraint: moved point {} to ({:.1}, {:.1})",
                    other_ref.point_index, new_other_pos.x, new_other_pos.y
                );
                break;
            }
        }
    }

    // Processed set is cleared each frame to allow continuous updates
}
