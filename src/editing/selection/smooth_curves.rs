//! Smooth curve handling for UFO-compliant point editing
//!
//! This module implements the logic for maintaining smooth curve tangents
//! when editing points with the smooth=true flag.

use crate::core::state::ufo_point::{UfoPoint, UfoPointType};
use crate::editing::selection::components::{GlyphPointReference, PointType};
use crate::editing::selection::enhanced_point_component::EnhancedPointType;
use bevy::log::debug;
use bevy::prelude::*;
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
    let smooth_point_pos = if let Ok((_, transform, _, _)) = all_points_query.get(constraint.smooth_point) {
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
pub fn is_point_smooth(
    entity: Entity,
    enhanced_points: &Query<&EnhancedPointType>,
) -> bool {
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
        if point_ref.glyph_name == glyph_name
            && enhanced.is_on_curve
            && enhanced.is_smooth()
        {
            if let Some(constraint) = find_smooth_curve_constraints(
                entity,
                point_ref,
                all_points_query
            ) {
                debug!(
                    "Found smooth constraint for point {:?} with {} handles",
                    entity,
                    constraint.left_handle.is_some() as usize + constraint.right_handle.is_some() as usize
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
/// This is triggered when the Enhanced Point Type changes to smooth
pub fn auto_apply_smooth_constraints(
    // Query for points that just became smooth
    changed_enhanced_points: Query<(Entity, &EnhancedPointType, &GlyphPointReference), Changed<EnhancedPointType>>,
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
        let point_data: Vec<_> = all_points_query.iter()
            .map(|(entity, transform, point_ref, point_type)| (entity, transform.translation.truncate(), point_ref.clone(), *point_type))
            .collect();

        // Find smooth curve constraints for this point
        let constraint = {
            let mut left_handle = None;
            let mut right_handle = None;

            // Find adjacent off-curve points
            for (entity, _pos, p_ref, p_type) in &point_data {
                if p_ref.glyph_name == point_ref.glyph_name
                    && p_ref.contour_index == point_ref.contour_index
                    && !p_type.is_on_curve
                {
                    // Check if this is the previous off-curve (left handle)
                    if point_ref.point_index > 0
                        && p_ref.point_index == point_ref.point_index - 1
                    {
                        left_handle = Some(*entity);
                    }
                    // Check if this is the next off-curve (right handle)
                    else if p_ref.point_index == point_ref.point_index + 1 {
                        right_handle = Some(*entity);
                    }
                }
            }

            if left_handle.is_some() || right_handle.is_some() {
                Some(SmoothCurveConstraint {
                    smooth_point: smooth_entity,
                    smooth_point_ref: point_ref.clone(),
                    left_handle,
                    right_handle,
                })
            } else {
                None
            }
        };

        if let Some(constraint) = constraint {
            // Get current smooth point position
            let smooth_pos = point_data.iter()
                .find(|(entity, _, _, _)| *entity == smooth_entity)
                .map(|(_, pos, _, _)| *pos);

            let smooth_pos = if let Some(pos) = smooth_pos {
                pos
            } else {
                continue;
            };

            // If we have both handles, align them symmetrically around the smooth point
            if let (Some(left_handle), Some(right_handle)) = (constraint.left_handle, constraint.right_handle) {
                // Get handle positions from our collected data
                let left_pos = point_data.iter()
                    .find(|(entity, _, _, _)| *entity == left_handle)
                    .map(|(_, pos, _, _)| *pos);
                let right_pos = point_data.iter()
                    .find(|(entity, _, _, _)| *entity == right_handle)
                    .map(|(_, pos, _, _)| *pos);

                if let (Some(left_pos), Some(right_pos)) = (left_pos, right_pos) {
                    // Use the handle that's further from the smooth point as the reference
                    let left_dist = smooth_pos.distance(left_pos);
                    let right_dist = smooth_pos.distance(right_pos);

                    let (target_handle, reference_pos) = if left_dist >= right_dist {
                        (right_handle, left_pos)
                    } else {
                        (left_handle, right_pos)
                    };

                    // Calculate the collinear position for the target handle
                    let reference_vector = reference_pos - smooth_pos;
                    let target_vector = -reference_vector;
                    let target_position = smooth_pos + target_vector;

                    // Update the target handle position
                    if let Ok((_, mut target_transform, _, _)) = all_points_query.get_mut(target_handle) {
                        target_transform.translation.x = target_position.x;
                        target_transform.translation.y = target_position.y;

                        debug!(
                            "Applied auto-smooth constraint: reference handle at ({:.1}, {:.1}), moved target handle to ({:.1}, {:.1})",
                            reference_pos.x, reference_pos.y,
                            target_position.x, target_position.y
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