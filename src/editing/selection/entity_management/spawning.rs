//! Entity spawning and despawning for selection system

use crate::core::state::font_data::PointTypeData;
use crate::core::state::AppState;
use crate::editing::selection::components::{
    FontIRPointReference, GlyphPointReference, PointType, Selectable, Selected, SelectionState,
};
use crate::editing::sort::manager::SortPointEntity;
use crate::editing::sort::{ActiveSortState, Sort};
use crate::geometry::bezpath_editing::{extract_editable_points, PathPointType};
use crate::geometry::point::EditPoint;
use bevy::prelude::*;

/// System to spawn point entities for the active sort using ECS as source of truth
pub fn spawn_active_sort_points(
    mut commands: Commands,
    active_sort_state: Res<ActiveSortState>,
    sort_query: Query<(Entity, &Sort, &Transform)>,
    point_entities: Query<Entity, With<SortPointEntity>>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    _selection_state: ResMut<SelectionState>,
) {
    // Only spawn points if there's an active sort
    if let Some(active_sort_entity) = active_sort_state.active_sort_entity {
        if let Ok((sort_entity, sort, transform)) = sort_query.get(active_sort_entity) {
            // Check if points already exist for this sort
            let existing_points = point_entities.iter().any(|_entity| {
                // Simplified check - in real implementation you'd check if points exist for this specific sort
                true
            });

            if !existing_points {
                let position = transform.translation.truncate();
                debug!("[spawn_active_sort_points] Spawning points for active sort: '{}' at position {:?}", 
                      sort.glyph_name, position);

                // Try FontIR first, then fallback to AppState
                if let Some(fontir_state) = fontir_app_state.as_ref() {
                    // Use FontIR spawning logic
                    spawn_fontir_points(
                        &mut commands,
                        sort_entity,
                        &sort.glyph_name,
                        position,
                        fontir_state,
                    );
                } else if let Some(app_state) = app_state.as_ref() {
                    // Use traditional AppState spawning logic
                    spawn_appstate_points(
                        &mut commands,
                        sort_entity,
                        &sort.glyph_name,
                        position,
                        app_state,
                    );
                } else {
                    warn!("[spawn_active_sort_points] No AppState or FontIR available for point spawning");
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
    active_sort_state: Res<ActiveSortState>,
    point_entities: Query<(Entity, &SortPointEntity)>,
    mut selection_state: ResMut<SelectionState>,
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

            commands.entity(entity).despawn();
            debug!(
                "[despawn_inactive_sort_points] Despawned point entity {:?} for inactive sort {:?}",
                entity, sort_point.sort_entity
            );
        }
    }
}

/// System to clean up the click resource
pub fn cleanup_click_resource(mut commands: Commands) {
    commands.remove_resource::<crate::editing::selection::events::ClickWorldPosition>();
}

/// Helper function to spawn points using FontIR data
fn spawn_fontir_points(
    commands: &mut Commands,
    sort_entity: Entity,
    glyph_name: &str,
    position: Vec2,
    fontir_state: &crate::core::state::FontIRAppState,
) {
    // Get FontIR glyph paths for the active sort
    if let Some(paths) = fontir_state.get_current_glyph_paths() {
        let mut point_count = 0;

        for (path_index, path) in paths.iter().enumerate() {
            let editable_points = extract_editable_points(path);

            for editable_point in editable_points {
                // Calculate world position: sort position + point offset
                let point_world_pos = position
                    + Vec2::new(
                        editable_point.position.x as f32,
                        editable_point.position.y as f32,
                    );
                point_count += 1;

                // Debug: Print first few point positions
                if point_count <= 5 {
                    debug!("[spawn_fontir_points] Point {}: local=({:.1}, {:.1}), world=({:.1}, {:.1})", 
                          point_count, editable_point.position.x, editable_point.position.y, point_world_pos.x, point_world_pos.y);
                }

                let fontir_point_ref = FontIRPointReference {
                    glyph_name: glyph_name.to_string(),
                    path_index,
                    point_ref: editable_point.reference,
                };

                let _entity = commands
                    .spawn((
                        EditPoint {
                            position: editable_point.position,
                            point_type: match editable_point.point_type {
                                PathPointType::OnCurve => PointTypeData::Line, // Simplified mapping
                                PathPointType::OffCurve => PointTypeData::OffCurve,
                            },
                        },
                        fontir_point_ref,
                        PointType {
                            is_on_curve: matches!(
                                editable_point.point_type,
                                PathPointType::OnCurve
                            ),
                        },
                        Transform::from_translation(point_world_pos.extend(0.0)),
                        Visibility::Visible,
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Selectable,
                        SortPointEntity { sort_entity },
                    ))
                    .id();
            }
        }
        debug!(
            "[spawn_fontir_points] Successfully spawned {} FontIR point entities",
            point_count
        );
    } else {
        warn!(
            "[spawn_fontir_points] No FontIR paths found for glyph '{}'",
            glyph_name
        );
    }
}

/// Helper function to spawn points using AppState data
fn spawn_appstate_points(
    commands: &mut Commands,
    sort_entity: Entity,
    glyph_name: &str,
    position: Vec2,
    app_state: &AppState,
) {
    // Get glyph data for the active sort
    if let Some(glyph_data) = app_state.workspace.font.get_glyph(glyph_name) {
        if let Some(outline) = &glyph_data.outline {
            let mut point_count = 0;

            for (contour_index, contour) in outline.contours.iter().enumerate() {
                for (point_index, point) in contour.points.iter().enumerate() {
                    // Calculate world position: sort position + point offset
                    let point_world_pos = position + Vec2::new(point.x as f32, point.y as f32);
                    point_count += 1;

                    // Debug: Print first few point positions
                    if point_count <= 5 {
                        debug!("[spawn_appstate_points] Point {}: local=({:.1}, {:.1}), world=({:.1}, {:.1})", 
                              point_count, point.x, point.y, point_world_pos.x, point_world_pos.y);
                    }

                    let glyph_point_ref = GlyphPointReference {
                        glyph_name: glyph_name.to_string(),
                        contour_index,
                        point_index,
                    };

                    let _entity = commands
                        .spawn((
                            EditPoint {
                                position: kurbo::Point::new(point.x, point.y),
                                point_type: point.point_type,
                            },
                            glyph_point_ref,
                            PointType {
                                is_on_curve: matches!(
                                    point.point_type,
                                    PointTypeData::Move
                                        | PointTypeData::Line
                                        | PointTypeData::Curve
                                ),
                            },
                            Transform::from_translation(point_world_pos.extend(0.0)),
                            Visibility::Visible,
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                            Selectable,
                            SortPointEntity { sort_entity },
                        ))
                        .id();
                }
            }
            debug!(
                "[spawn_appstate_points] Successfully spawned {} point entities",
                point_count
            );
        } else {
            warn!(
                "[spawn_appstate_points] No outline found for glyph '{}'",
                glyph_name
            );
        }
    } else {
        warn!(
            "[spawn_appstate_points] No glyph data found for '{}'",
            glyph_name
        );
    }
}
