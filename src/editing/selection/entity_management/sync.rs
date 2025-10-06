//! Data synchronization between ECS entities and UFO font data

use crate::core::state::AppState;
use crate::editing::selection::components::{GlyphPointReference, PointType};
use crate::editing::selection::enhanced_point_component::EnhancedPointType;
use crate::editing::selection::nudge::NudgeState;
use crate::editing::smooth_curves::find_direct_neighbor_handles;
use crate::editing::sort::manager::SortPointEntity;
use crate::editing::sort::Sort;
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;
use std::collections::HashMap;

/// Resource to track enhanced point attributes for UFO saving
/// Maps (glyph_name, contour_index, point_index) to enhanced attributes
#[derive(Resource, Default)]
pub struct EnhancedPointAttributes {
    pub attributes: HashMap<(String, usize, usize), crate::core::state::ufo_point::UfoPoint>,
}

/// System to update the actual glyph data when a point is moved
#[allow(clippy::type_complexity)]
pub fn update_glyph_data_from_selection(
    query: Query<(&Transform, &GlyphPointReference, Option<&SortPointEntity>), Changed<Transform>>,
    sort_query: Query<(&Sort, &Transform)>,
    all_points_query: Query<(
        Entity,
        &Transform,
        &GlyphPointReference,
        &PointType,
        Option<&EnhancedPointType>,
    )>,
    mut app_state: ResMut<AppState>,
    _nudge_state: Res<NudgeState>,
    knife_mode: Option<Res<crate::ui::edit_mode_toolbar::knife::KnifeModeActive>>,
    mut commands: Commands,
) {
    debug!("[SMOOTH] update_glyph_data_from_selection CALLED");

    // Skip processing if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            debug!("[SMOOTH] Skipping - knife mode active");
            return;
        }
    }

    // Early return if no points were moved
    if query.is_empty() {
        debug!("[SMOOTH] No points with Changed<Transform>");
        return;
    }

    debug!("[SMOOTH] Processing {} moved points", query.iter().count());

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

    // Apply smooth constraints when handles are moved to maintain collinearity.
    // When an off-curve point is moved, find any smooth on-curve points it's connected to,
    // and automatically adjust the opposite handle to maintain collinearity.

    // Struct to hold constraint update data
    struct HandleAdjustment {
        handle_ref: GlyphPointReference,
        new_position: Vec2,
    }

    // Collect adjustments first to avoid borrow checker conflicts
    let mut handle_adjustments = Vec::new();

    // Build point data for constraint analysis
    let point_data: Vec<_> = all_points_query
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

    // Check each moved point for smooth constraint implications
    for (moved_transform, moved_ref, _) in query.iter() {
        let moved_pos = moved_transform.translation.truncate();

        debug!(
            "[SMOOTH] Checking moved point: glyph='{}', contour={}, point={}",
            moved_ref.glyph_name, moved_ref.contour_index, moved_ref.point_index
        );

        // Check if this is an off-curve handle that was moved
        if let Some((_moved_entity, _, _, moved_point_type)) = point_data
            .iter()
            .find(|(_, _, ref_comp, _)| ref_comp == moved_ref)
        {
            debug!(
                "[SMOOTH] Point type: is_on_curve={}",
                moved_point_type.is_on_curve
            );

            if !moved_point_type.is_on_curve {
                // This is an off-curve handle that was moved
                debug!(
                    "[SMOOTH] OFF-CURVE handle moved: glyph='{}', contour={}, point={}",
                    moved_ref.glyph_name, moved_ref.contour_index, moved_ref.point_index
                );

                // Find smooth on-curve points in the same contour that could be adjacent
                for (_, smooth_transform, smooth_ref, smooth_point_type, smooth_enhanced) in
                    all_points_query.iter()
                {
                    if smooth_ref.glyph_name == moved_ref.glyph_name
                        && smooth_ref.contour_index == moved_ref.contour_index
                        && smooth_point_type.is_on_curve
                        && smooth_enhanced.is_some_and(|e| e.is_smooth())
                    {
                        // Use simplified neighbor check
                        let (left_handle, right_handle) =
                            find_direct_neighbor_handles(smooth_ref, &point_data);

                        debug!(
                            "[SMOOTH] Checking smooth point {} - left_handle: {:?}, right_handle: {:?}",
                            smooth_ref.point_index,
                            left_handle.as_ref().map(|(_, _, r)| r.point_index),
                            right_handle.as_ref().map(|(_, _, r)| r.point_index)
                        );

                        let smooth_pos = smooth_transform.translation.truncate();
                        let moved_is_left = left_handle
                            .as_ref()
                            .is_some_and(|(_, _, ref_comp)| ref_comp == moved_ref);
                        let moved_is_right = right_handle
                            .as_ref()
                            .is_some_and(|(_, _, ref_comp)| ref_comp == moved_ref);

                        if moved_is_left || moved_is_right {
                            debug!(
                                "[SMOOTH] MATCH! Found smooth point {} adjacent to moved handle {}",
                                smooth_ref.point_index, moved_ref.point_index
                            );

                            // Determine which handle to adjust
                            let other_handle = if moved_is_left {
                                right_handle
                            } else {
                                left_handle
                            };

                            if let Some((_, _, other_ref)) = other_handle {
                                // Calculate collinear position for the opposite handle
                                let handle_vector = moved_pos - smooth_pos;
                                let opposite_vector = -handle_vector;
                                let new_other_pos = smooth_pos + opposite_vector;

                                handle_adjustments.push(HandleAdjustment {
                                    handle_ref: other_ref.clone(),
                                    new_position: new_other_pos,
                                });

                                debug!(
                                    "[SMOOTH] QUEUED adjustment: will move point {} to ({:.1}, {:.1}) to stay collinear",
                                    other_ref.point_index,
                                    new_other_pos.x,
                                    new_other_pos.y
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Apply all handle adjustments (separate phase to avoid borrow conflicts)
    for adjustment in handle_adjustments {
        // Update transform for visual feedback
        for (handle_entity, _, handle_ref, _, _) in all_points_query.iter() {
            if handle_ref == &adjustment.handle_ref {
                commands
                    .entity(handle_entity)
                    .insert(Transform::from_translation(
                        adjustment.new_position.extend(0.0),
                    ));
                break;
            }
        }

        // Update UFO data for persistence
        app_state.set_point_position(
            &adjustment.handle_ref.glyph_name,
            adjustment.handle_ref.contour_index,
            adjustment.handle_ref.point_index,
            adjustment.new_position.x as f64,
            adjustment.new_position.y as f64,
        );

        debug!(
            "[update_glyph_data_from_selection] Applied smooth constraint adjustment: glyph='{}', point={}, new_pos=({:.1}, {:.1})",
            adjustment.handle_ref.glyph_name,
            adjustment.handle_ref.point_index,
            adjustment.new_position.x,
            adjustment.new_position.y
        );
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

/// System to update point positions when sort position changes
#[allow(clippy::type_complexity)]
pub fn sync_point_positions_to_sort(
    mut param_set: ParamSet<(
        Query<(Entity, &Sort, &Transform), Changed<Sort>>,
        Query<(&mut Transform, &SortPointEntity, &GlyphPointReference)>,
    )>,
    app_state: Res<AppState>,
) {
    // First, collect all the sort positions that have changed
    let mut sort_positions = HashMap::new();

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

/// System to sync enhanced point attributes (like smooth) to a resource for UFO saving
/// This ensures that when points are modified with enhanced attributes,
/// those changes are preserved when saving to UFO files
#[allow(clippy::type_complexity)]
pub fn sync_enhanced_point_attributes(
    enhanced_query: Query<(&EnhancedPointType, &GlyphPointReference), Changed<EnhancedPointType>>,
    mut enhanced_attributes: ResMut<EnhancedPointAttributes>,
) {
    if enhanced_query.is_empty() {
        return;
    }

    debug!(
        "[sync_enhanced_point_attributes] Processing {} changed enhanced points",
        enhanced_query.iter().count()
    );

    for (enhanced_point, point_ref) in enhanced_query.iter() {
        let key = (
            point_ref.glyph_name.clone(),
            point_ref.contour_index,
            point_ref.point_index,
        );

        // Store the enhanced point attributes for later UFO saving
        enhanced_attributes
            .attributes
            .insert(key.clone(), enhanced_point.ufo_point.clone());

        debug!(
            "Stored enhanced point attributes: glyph='{}', contour={}, point={}, smooth={}",
            point_ref.glyph_name,
            point_ref.contour_index,
            point_ref.point_index,
            enhanced_point.ufo_point.smooth.unwrap_or(false)
        );
    }
}
