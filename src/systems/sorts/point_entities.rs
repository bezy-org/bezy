//! Point entity management for text editor sorts

use crate::core::state::font_data::PointTypeData;
use crate::core::state::{AppState, FontIRAppState};
use crate::editing::selection::components::{GlyphPointReference, PointType, Selectable};
use crate::editing::sort::manager::SortPointEntity;
use crate::editing::sort::{ActiveSort, InactiveSort, Sort};
use crate::geometry::point::EditPoint;
use crate::geometry::world_space::DPoint;
use bevy::prelude::*;
use kurbo::{PathEl, Point};
use std::collections::HashMap;

/// Component to track which sort entity a point belongs to
#[derive(Component)]
pub struct PointSortParent(pub Entity);

/// Spawn active sort points optimized
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_active_sort_points_optimized(
    mut commands: Commands,
    active_sort_query: Query<(Entity, &Sort, &Transform), With<ActiveSort>>,
    existing_points: Query<&PointSortParent>,
    // Debug: Check if existing points have SortPointEntity component (needed for rendering)
    existing_sort_points: Query<Entity, With<SortPointEntity>>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<FontIRAppState>>,
    // Debug: Check all buffer sorts
    all_buffer_sorts: Query<
        (Entity, &Sort, &Transform, Option<&ActiveSort>),
        With<crate::systems::sorts::sort_entities::BufferSortIndex>,
    >,
    // CRITICAL FIX: Trigger unified renderer update when points are spawned
    mut visual_update_tracker: ResMut<crate::rendering::glyph_renderer::SortVisualUpdateTracker>,
) {
    let active_sort_count = active_sort_query.iter().count();
    let total_buffer_sorts = all_buffer_sorts.iter().count();
    let sort_point_count = existing_sort_points.iter().count();

    // Always log for debugging the default sort issue
    debug!(
        "🔍 POINT SPAWN DEBUG: {} active sorts found, {} total buffer sorts, {} points with SortPointEntity",
        active_sort_count, total_buffer_sorts, sort_point_count
    );

    // Debug: Log all buffer sorts and their ActiveSort status
    for (entity, sort, _transform, maybe_active) in all_buffer_sorts.iter() {
        let is_active = maybe_active.is_some();
        debug!(
            "🔍 BUFFER SORT DEBUG: Entity {:?}, glyph '{}', has ActiveSort: {}",
            entity, sort.glyph_name, is_active
        );
    }

    if active_sort_count > 0 {
        debug!(
            "Point spawning system called: {} active sorts found",
            active_sort_count
        );
    }

    for (sort_entity, sort, transform) in active_sort_query.iter() {
        // Check if points already exist for this sort
        let existing_point_count = existing_points
            .iter()
            .filter(|parent| parent.0 == sort_entity)
            .count();
        let has_points = existing_point_count > 0;

        debug!(
            "🔍 POINT CHECK: Sort {:?} glyph '{}' has {} existing points, skipping: {}",
            sort_entity, sort.glyph_name, existing_point_count, has_points
        );

        // Debug: Check what components the existing points have
        for point_parent in existing_points.iter() {
            if point_parent.0 == sort_entity {
                debug!(
                    "🔍 Existing point has PointSortParent({:?})",
                    point_parent.0
                );
                break; // Just check one example
            }
        }

        if has_points {
            debug!(
                "Skipping point spawning for sort entity {:?} - {} points already exist",
                sort_entity, existing_point_count
            );
            continue; // Skip if points already exist
        }

        debug!(
            "Spawning points for active sort entity {:?}, glyph: {}",
            sort_entity, sort.glyph_name
        );
        let position = transform.translation.truncate();

        debug!(
            "Spawning points for ACTIVE sort '{}' at position ({:.1}, {:.1})",
            sort.glyph_name, position.x, position.y
        );

        // Try FontIR first, then fallback to AppState
        if let Some(fontir_state) = fontir_app_state.as_ref() {
            // CRITICAL FIX: Get FontIR BezPaths WITH pen tool edits for this glyph
            if let Some(paths) = fontir_state.get_glyph_paths_with_edits(&sort.glyph_name) {
                let mut point_count = 0;
                debug!(
                    "FontIR: Spawning points for active sort '{}' with {} paths",
                    sort.glyph_name,
                    paths.len()
                );

                // Spawn points for each BezPath (contour)
                for (contour_index, path) in paths.iter().enumerate() {
                    let elements: Vec<_> = path.elements().iter().collect();
                    let mut element_point_index = 0; // Track actual point index within the contour

                    for element in elements.iter() {
                        // Extract all points (on-curve and off-curve) from PathEl
                        let points = extract_points_from_path_element(element);

                        for (point_pos, point_type) in points {
                            // Calculate world position for this point
                            let world_pos =
                                position + Vec2::new(point_pos.x as f32, point_pos.y as f32);

                            if point_count < 10 {
                                debug!("FontIR Point {}: type={:?}, raw=({:.1}, {:.1}), sort_pos=({:.1}, {:.1}), world_pos=({:.1}, {:.1})", 
                                      point_count, point_type, point_pos.x, point_pos.y, position.x, position.y, world_pos.x, world_pos.y);
                            }

                            // Create the EditPoint
                            let edit_point = EditPoint {
                                position: Point::new(world_pos.x as f64, world_pos.y as f64),
                                point_type,
                            };

                            // Create PointType component
                            let point_type_component = PointType {
                                is_on_curve: matches!(
                                    point_type,
                                    PointTypeData::Move
                                        | PointTypeData::Line
                                        | PointTypeData::Curve
                                ),
                            };

                            // Spawn the point entity
                            commands.spawn((
                                Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                                edit_point,
                                point_type_component,
                                GlyphPointReference {
                                    glyph_name: sort.glyph_name.clone(),
                                    contour_index,
                                    point_index: element_point_index,
                                },
                                Selectable,
                                SortPointEntity { sort_entity },
                                PointSortParent(sort_entity),
                                Name::new(format!(
                                    "FontIR_Point[{},{},{}]",
                                    contour_index,
                                    element_point_index,
                                    if point_type == PointTypeData::OffCurve {
                                        "off"
                                    } else {
                                        "on"
                                    }
                                )),
                            ));

                            element_point_index += 1;
                            point_count += 1;
                            if point_count <= 10 {
                                debug!(
                                    "FontIR: Spawned {} point {} at world position ({:.1}, {:.1})",
                                    if point_type == PointTypeData::OffCurve {
                                        "off-curve"
                                    } else {
                                        "on-curve"
                                    },
                                    point_count,
                                    world_pos.x,
                                    world_pos.y
                                );
                            }
                        }
                    }
                }

                debug!(
                    "FontIR: Spawned {} points for active sort '{}'",
                    point_count, sort.glyph_name
                );

                // CRITICAL FIX: Trigger unified renderer update when points are spawned
                if point_count > 0 {
                    visual_update_tracker.needs_update = true;
                    debug!(
                        "🔄 TRIGGERED VISUAL UPDATE: Points spawned for sort '{}'",
                        sort.glyph_name
                    );
                }
            } else {
                warn!("FontIR: No paths found for glyph '{}'", sort.glyph_name);
            }
        } else if let Some(state) = app_state.as_ref() {
            // Fallback to UFO AppState logic
            if let Some(glyph_data) = state.workspace.font.get_glyph(&sort.glyph_name) {
                if let Some(outline) = &glyph_data.outline {
                    let mut point_count = 0;

                    for (contour_index, contour) in outline.contours.iter().enumerate() {
                        for (point_index, point) in contour.points.iter().enumerate() {
                            let world_pos = position + Vec2::new(point.x as f32, point.y as f32);

                            let edit_point = EditPoint {
                                position: Point::new(world_pos.x as f64, world_pos.y as f64),
                                point_type: point.point_type,
                            };

                            let point_type_component = PointType {
                                is_on_curve: matches!(
                                    point.point_type,
                                    PointTypeData::Move
                                        | PointTypeData::Line
                                        | PointTypeData::Curve
                                ),
                            };

                            commands.spawn((
                                Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                                edit_point,
                                point_type_component,
                                GlyphPointReference {
                                    glyph_name: sort.glyph_name.clone(),
                                    contour_index,
                                    point_index,
                                },
                                Selectable,
                                SortPointEntity { sort_entity },
                                PointSortParent(sort_entity),
                                Name::new(format!("UFO_Point[{contour_index},{point_index}]")),
                            ));

                            point_count += 1;
                        }
                    }

                    debug!(
                        "UFO: Spawned {} points for active sort '{}'",
                        point_count, sort.glyph_name
                    );

                    // CRITICAL FIX: Trigger unified renderer update when points are spawned
                    if point_count > 0 {
                        visual_update_tracker.needs_update = true;
                        debug!(
                            "🔄 TRIGGERED VISUAL UPDATE: Points spawned for sort '{}'",
                            sort.glyph_name
                        );
                    }
                } else {
                    warn!("UFO: No outline found for glyph '{}'", sort.glyph_name);
                }
            } else {
                warn!("UFO: No glyph data found for '{}'", sort.glyph_name);
            }
        } else {
            warn!("Point spawning failed - neither FontIR nor AppState available");
        }
    }
}

/// System to regenerate point entities when FontIR data changes (e.g., pen tool adds contours)
pub fn regenerate_points_on_fontir_change(
    mut commands: Commands,
    mut app_state_events: EventReader<crate::editing::selection::events::AppStateChanged>,
    active_sort_query: Query<
        (Entity, &crate::editing::sort::Sort, &Transform),
        With<crate::editing::sort::ActiveSort>,
    >,
    existing_point_query: Query<(Entity, &PointSortParent)>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    _app_state: Option<Res<crate::core::state::AppState>>,
    mut selection_state: ResMut<crate::editing::selection::SelectionState>,
) {
    // Only run when FontIR data changes
    if app_state_events.read().count() == 0 {
        return;
    }

    // For each active sort, regenerate its point entities to include new contours
    for (sort_entity, sort, _sort_transform) in active_sort_query.iter() {
        debug!(
            "Regenerating point entities for active sort '{}' due to FontIR change",
            sort.glyph_name
        );

        // Despawn existing point entities for this sort
        let mut despawn_count = 0;
        for (point_entity, parent) in existing_point_query.iter() {
            if parent.0 == sort_entity {
                // Remove from selection if selected
                selection_state.selected.remove(&point_entity);
                if let Ok(mut entity_commands) = commands.get_entity(point_entity) {
                    entity_commands.despawn();
                    despawn_count += 1;
                }
            }
        }

        if despawn_count > 0 {
            debug!(
                "Despawned {} old point entities for sort '{}'",
                despawn_count, sort.glyph_name
            );
        }

        // Respawn point entities with updated FontIR data using FontIR paths
        if let Some(fontir_state) = fontir_app_state.as_ref() {
            if let Some(paths) = fontir_state.get_glyph_paths_with_edits(&sort.glyph_name) {
                let mut point_count = 0;
                debug!(
                    "FontIR: Regenerating {} paths for active sort '{}'",
                    paths.len(),
                    sort.glyph_name
                );

                // Spawn points for each BezPath (contour) - including new pen tool contours
                for (contour_index, path) in paths.iter().enumerate() {
                    let elements: Vec<_> = path.elements().iter().collect();
                    let mut element_point_index = 0;

                    for element in elements.iter() {
                        let points = extract_points_from_path_element(element);

                        for (point_pos, point_type_data) in points {
                            // All FontIR coordinates are now consistently relative to sort position
                            let sort_position = active_sort_query
                                .get(sort_entity)
                                .map(|(_, _, transform)| transform.translation.truncate())
                                .unwrap_or(Vec2::ZERO);

                            let world_pos =
                                sort_position + Vec2::new(point_pos.x as f32, point_pos.y as f32);

                            // COORDINATE DEBUG: Log transformation details
                            if point_count < 5 {
                                debug!("🔍 COORD DEBUG [{}]: relative=({:.1}, {:.1}), sort_pos=({:.1}, {:.1}), final_world=({:.1}, {:.1})", 
                                      point_count, point_pos.x, point_pos.y, sort_position.x, sort_position.y, world_pos.x, world_pos.y);
                            }

                            let point_index = element_point_index;

                            // Create EditPoint component
                            let edit_point = EditPoint {
                                position: Point::new(world_pos.x as f64, world_pos.y as f64),
                                point_type: point_type_data,
                            };

                            // Create PointType component
                            let point_type_component = PointType {
                                is_on_curve: matches!(
                                    point_type_data,
                                    PointTypeData::Move
                                        | PointTypeData::Line
                                        | PointTypeData::Curve
                                ),
                            };

                            commands.spawn((
                                Transform::from_xyz(world_pos.x, world_pos.y, 10.0),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                                edit_point,
                                point_type_component,
                                GlyphPointReference {
                                    glyph_name: sort.glyph_name.clone(),
                                    contour_index,
                                    point_index,
                                },
                                Selectable,
                                SortPointEntity { sort_entity },
                                PointSortParent(sort_entity),
                                Name::new(format!(
                                    "FontIR_Regen_Point[{contour_index},{point_index}]"
                                )),
                            ));

                            point_count += 1;
                            element_point_index += 1;
                        }
                    }
                }

                debug!(
                    "FontIR: Regenerated {} points for active sort '{}'",
                    point_count, sort.glyph_name
                );
            }
        }
    }
}

/// Despawn inactive sort points optimized
pub fn despawn_inactive_sort_points_optimized(
    mut commands: Commands,
    inactive_sort_query: Query<Entity, (With<InactiveSort>, Changed<InactiveSort>)>,
    point_query: Query<(Entity, &PointSortParent)>,
) {
    for inactive_sort_entity in inactive_sort_query.iter() {
        // Find and despawn all points belonging to this sort
        let mut despawn_count = 0;
        for (point_entity, parent) in point_query.iter() {
            if parent.0 == inactive_sort_entity {
                // Use get_entity to check if entity exists before despawning
                if let Ok(mut entity_commands) = commands.get_entity(point_entity) {
                    entity_commands.despawn();
                    despawn_count += 1;
                }
            }
        }

        if despawn_count > 0 {
            debug!("Despawned {} points for inactive sort", despawn_count);
        }
    }
}

/// Detect when active sorts change glyph and force point regeneration
#[allow(clippy::type_complexity)]
pub(crate) fn detect_sort_glyph_changes(
    mut commands: Commands,
    changed_sorts: Query<(Entity, &Sort), (With<ActiveSort>, Changed<Sort>)>,
    existing_point_query: Query<(Entity, &SortPointEntity)>,
    mut visual_update_tracker: ResMut<crate::rendering::glyph_renderer::SortVisualUpdateTracker>,
    mut local_previous_glyphs: Local<HashMap<Entity, String>>,
) {
    for (sort_entity, sort) in changed_sorts.iter() {
        let current_glyph_name = &sort.glyph_name;

        // Check if the glyph name actually changed
        let glyph_changed = local_previous_glyphs.get(&sort_entity) != Some(current_glyph_name);

        if glyph_changed {
            debug!(
                "Detected glyph change for sort {:?}: switching to '{}'",
                sort_entity, current_glyph_name
            );

            // Despawn all existing point entities for this sort
            let mut despawn_count = 0;
            for (point_entity, sort_point) in existing_point_query.iter() {
                if sort_point.sort_entity == sort_entity {
                    // Use get_entity to check if entity exists before despawning
                    if let Ok(mut entity_commands) = commands.get_entity(point_entity) {
                        entity_commands.despawn();
                        despawn_count += 1;
                    }
                }
            }

            if despawn_count > 0 {
                debug!(
                    "Despawned {} existing point entities for sort {:?}",
                    despawn_count, sort_entity
                );
            }

            // Update the previous glyph name
            local_previous_glyphs.insert(sort_entity, current_glyph_name.clone());

            // Trigger visual update to ensure new points are rendered
            visual_update_tracker.needs_update = true;

            debug!(
                "Sort {:?} glyph changed to '{}', triggered point regeneration",
                sort_entity, current_glyph_name
            );
        }
    }
}

/// Extract all points (on-curve and off-curve) from a kurbo PathEl
fn extract_points_from_path_element(element: &PathEl) -> Vec<(Point, PointTypeData)> {
    match element {
        PathEl::MoveTo(pt) => vec![(*pt, PointTypeData::Move)],
        PathEl::LineTo(pt) => vec![(*pt, PointTypeData::Line)],
        PathEl::CurveTo(c1, c2, pt) => vec![
            (*c1, PointTypeData::OffCurve), // First control point
            (*c2, PointTypeData::OffCurve), // Second control point
            (*pt, PointTypeData::Curve),    // End point
        ],
        PathEl::QuadTo(c, pt) => vec![
            (*c, PointTypeData::OffCurve), // Control point
            (*pt, PointTypeData::Curve),   // End point
        ],
        PathEl::ClosePath => vec![], // ClosePath creates implicit line back to start - handled by unified rendering
    }
}
