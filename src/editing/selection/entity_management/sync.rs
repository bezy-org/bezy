//! Data synchronization between ECS entities and UFO font data

use crate::core::state::AppState;
use crate::editing::selection::components::{GlyphPointReference, Selected};
use crate::editing::selection::nudge::NudgeState;
use crate::editing::sort::Sort;
use crate::systems::sort_manager::SortPointEntity;
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;
use std::collections::HashMap;

/// System to update the actual glyph data when a point is moved
#[allow(clippy::type_complexity)]
pub fn update_glyph_data_from_selection(
    query: Query<
        (&Transform, &GlyphPointReference, Option<&SortPointEntity>),
        (With<Selected>, Changed<Transform>),
    >,
    sort_query: Query<(&Sort, &Transform)>,
    mut app_state: ResMut<AppState>,
    _nudge_state: Res<NudgeState>,
    knife_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>>,
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

    info!(
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

        info!(
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
        info!(
            "[update_glyph_data_from_selection] Successfully updated {} outline points",
            query.iter().count()
        );
    } else {
        info!("[update_glyph_data_from_selection] No outline updates needed");
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
