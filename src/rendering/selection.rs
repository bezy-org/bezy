//! Selection rendering systems
//!
//! This module handles visual rendering for selection-related features:
//! - Selection marquee/rectangle (drag selection) - mesh-based dashed rectangle
//!
//! Selected point highlighting is handled by the mesh-based point rendering
//! system in src/rendering/points.rs which already supports selected state.

use crate::editing::selection::components::SelectionRect;
use crate::editing::selection::DragSelectionState;
use crate::rendering::zoom_aware_scaling::CameraResponsiveScale;
use crate::ui::themes::CurrentTheme;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};

#[derive(Component)]
pub struct MarqueeMesh;

/// Renders the selection marquee rectangle during drag selection using mesh-based dashed lines
pub fn render_selection_marquee(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    drag_state: Res<DragSelectionState>,
    marquee_query: Query<(Entity, &SelectionRect)>,
    existing_marquee_meshes: Query<Entity, With<MarqueeMesh>>,
    theme: Res<CurrentTheme>,
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    camera_scale: Res<CameraResponsiveScale>,
) {
    // Clean up existing marquee meshes
    for entity in existing_marquee_meshes.iter() {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }

    // Only render marquee when in select mode
    if current_tool.get_current() != Some("select") {
        return;
    }

    // Only render marquee when dragging
    if !drag_state.is_dragging {
        return;
    }

    // Try to find the selection rect from the query
    if let Some((_, rect)) = marquee_query.iter().next() {
        debug!(
            "[render_selection_marquee] Drawing marquee: start={:?}, end={:?}",
            rect.start, rect.end
        );
        let start = rect.start;
        let end = rect.end;
        let color = theme.action_color();

        // Four corners
        let p1 = Vec2::new(start.x, start.y);
        let p2 = Vec2::new(end.x, start.y);
        let p3 = Vec2::new(end.x, end.y);
        let p4 = Vec2::new(start.x, end.y);

        let line_width = camera_scale.adjusted_line_width();
        let dash_length = 8.0;
        let gap_length = 4.0;
        let z = 15.0; // Above points

        // Create dashed lines for each edge
        create_dashed_line_meshes(
            &mut commands,
            &mut meshes,
            &mut materials,
            p1,
            p2,
            line_width,
            dash_length,
            gap_length,
            color,
            z,
        );
        create_dashed_line_meshes(
            &mut commands,
            &mut meshes,
            &mut materials,
            p2,
            p3,
            line_width,
            dash_length,
            gap_length,
            color,
            z,
        );
        create_dashed_line_meshes(
            &mut commands,
            &mut meshes,
            &mut materials,
            p3,
            p4,
            line_width,
            dash_length,
            gap_length,
            color,
            z,
        );
        create_dashed_line_meshes(
            &mut commands,
            &mut meshes,
            &mut materials,
            p4,
            p1,
            line_width,
            dash_length,
            gap_length,
            color,
            z,
        );
    }
}

/// Create mesh-based dashed line segments
fn create_dashed_line_meshes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    line_width: f32,
    dash_length: f32,
    gap_length: f32,
    color: Color,
    z: f32,
) {
    let direction = (end - start).normalize();
    let total_length = start.distance(end);
    let segment_length = dash_length + gap_length;

    let mut current_pos = 0.0;
    while current_pos < total_length {
        let dash_start_pos = current_pos;
        let dash_end_pos = (current_pos + dash_length).min(total_length);

        let dash_start = start + direction * dash_start_pos;
        let dash_end = start + direction * dash_end_pos;

        let line_mesh =
            crate::rendering::mesh_utils::create_line_mesh(dash_start, dash_end, line_width);

        commands.spawn((
            MarqueeMesh,
            Mesh2d(meshes.add(line_mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_xyz(
                (dash_start.x + dash_end.x) * 0.5,
                (dash_start.y + dash_end.y) * 0.5,
                z,
            ),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));

        current_pos += segment_length;
    }
}
