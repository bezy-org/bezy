use crate::core::state::AppState;
use crate::editing::selection::components::{GlyphPointReference, PointType};
use crate::editing::sort::manager::SortPointEntity;
use crate::editing::sort::{ActiveSort, Sort};
use crate::ui::themes::CurrentTheme;
use bevy::prelude::*;

/// Component to mark handle line entities
#[derive(Component)]
pub struct HandleLine {
    pub start_entity: Entity,
    pub end_entity: Entity,
}

/// Plugin for rendering various outline elements (handles, tangents, etc.)
pub struct OutlineElementsPlugin;

impl Plugin for OutlineElementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_handle_lines, cleanup_orphaned_handles)
                .chain()
                .in_set(crate::rendering::PostEditingRenderingSet),
        );
    }
}

/// Updates handle lines between on-curve and off-curve points
#[allow(clippy::too_many_arguments)]
fn update_handle_lines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    _app_state: Option<Res<AppState>>,
    _active_sort_query: Query<&Sort, With<ActiveSort>>,
    point_query: Query<
        (Entity, &Transform, &GlyphPointReference, &PointType),
        With<SortPointEntity>,
    >,
    existing_handles: Query<(Entity, &HandleLine)>,
    theme: Res<CurrentTheme>,
) {
    let point_count = point_query.iter().count();

    // Early exit if no points
    if point_count == 0 {
        return;
    }

    // First clear existing handles
    for (entity, _) in existing_handles.iter() {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }

    // Check if we have any points to work with
    if point_query.is_empty() {
        return;
    }

    // Get glyph name from the first point (they should all be from the same glyph)
    let Some(first_point) = point_query.iter().next() else {
        return; // No points to render
    };
    let glyph_name = &first_point.2.glyph_name;

    create_handles_from_point_entities(
        &mut commands,
        &mut meshes,
        &mut materials,
        &point_query,
        glyph_name,
        &theme,
    );
}

/// Create handles from point entities
fn create_handles_from_point_entities(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    point_query: &Query<
        (Entity, &Transform, &GlyphPointReference, &PointType),
        With<SortPointEntity>,
    >,
    glyph_name: &str,
    theme: &CurrentTheme,
) {
    // Create material for handles
    let handle_material = materials.add(ColorMaterial::from(theme.theme().handle_line_color()));
    let mut _handles_created = 0;

    // Group existing point entities by contour and index for mapping
    let mut point_entities: std::collections::HashMap<(usize, usize), Entity> =
        std::collections::HashMap::new();
    let mut point_positions: std::collections::HashMap<Entity, Vec2> =
        std::collections::HashMap::new();

    for (entity, transform, glyph_ref, _) in point_query.iter() {
        if glyph_ref.glyph_name == glyph_name {
            let position = transform.translation.truncate();
            point_entities.insert((glyph_ref.contour_index, glyph_ref.point_index), entity);
            point_positions.insert(entity, position);
        }
    }

    // Create handles based on point connectivity (adjacent on/off-curve points)
    // Use the same logic as UFO system - connect consecutive points where one is on-curve and other is off-curve
    for (entity, transform, glyph_ref, point_type) in point_query.iter() {
        if glyph_ref.glyph_name != glyph_name {
            continue;
        }

        let current_pos = transform.translation.truncate();
        let current_is_on_curve = point_type.is_on_curve;

        // Find the next point in the same contour
        let next_index = glyph_ref.point_index + 1;
        if let Some(next_entity) = point_entities.get(&(glyph_ref.contour_index, next_index)) {
            if let Some(next_pos) = point_positions.get(next_entity) {
                // Get the next point's type
                if let Ok((_, _, _, next_point_type)) = point_query.get(*next_entity) {
                    let next_is_on_curve = next_point_type.is_on_curve;

                    // Create handle if one is on-curve and the other is off-curve
                    if current_is_on_curve != next_is_on_curve {
                        let direction = *next_pos - current_pos;
                        let length = direction.length();
                        let angle = direction.y.atan2(direction.x);
                        let midpoint = (current_pos + *next_pos) / 2.0;

                        // Create a rectangle mesh for the line
                        let line_thickness = 1.0; // 1px width for subtle handles
                        let line_mesh = meshes.add(Rectangle::new(length, line_thickness));

                        commands.spawn((
                            Mesh2d(line_mesh),
                            MeshMaterial2d(handle_material.clone()),
                            Transform::from_translation(midpoint.extend(10.0))
                                .with_rotation(Quat::from_rotation_z(angle)),
                            HandleLine {
                                start_entity: entity,
                                end_entity: *next_entity,
                            },
                        ));

                        _handles_created += 1;
                    }
                }
            }
        }
    }
}

/// Cleans up handle lines when their connected points are removed
fn cleanup_orphaned_handles(
    mut commands: Commands,
    handles: Query<(Entity, &HandleLine)>,
    points: Query<Entity, With<SortPointEntity>>,
) {
    for (handle_entity, handle_line) in handles.iter() {
        // Check if both connected points still exist
        if points.get(handle_line.start_entity).is_err()
            || points.get(handle_line.end_entity).is_err()
        {
            if let Ok(mut entity_commands) = commands.get_entity(handle_entity) {
                entity_commands.despawn();
            }
        }
    }
}
