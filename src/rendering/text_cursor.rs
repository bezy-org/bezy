//! Text cursor rendering for the text editor
//!
//! This module handles the visual rendering of text cursors in Insert mode,
//! including mesh generation and visual updates.

use crate::rendering::entity_pools::{update_cursor_entity, EntityPools, PooledEntityType};
use bevy::prelude::*;
use bevy::render::mesh::Mesh;
use bevy::sprite::ColorMaterial;

/// Component to mark text editor cursor entities
#[derive(Component)]
pub struct TextEditorCursor;

/// Resource to track cursor state for change detection
#[derive(Resource, Default)]
pub struct CursorRenderingState {
    pub last_cursor_position: Option<Vec2>,
    pub last_tool: Option<String>,
    pub last_placement_mode: Option<crate::ui::edit_mode_toolbar::text::TextPlacementMode>,
    pub last_buffer_cursor_position: Option<usize>,
    pub last_camera_scale: Option<f32>,
}

/// Render a text cursor at the specified world position (internal)
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_cursor_at_position(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    entity_pools: &mut ResMut<EntityPools>,
    cursor_world_pos: Vec2,
    upm: f32,
    descender: f32,
    camera_scale: &crate::rendering::zoom_aware_scaling::CameraResponsiveScale,
) {
    warn!(
        "🎨 RENDERING CURSOR at world_pos=({:.1}, {:.1})",
        cursor_world_pos.x, cursor_world_pos.y
    );

    // Calculate cursor bounds based on font metrics
    let cursor_top = cursor_world_pos.y + upm; // UPM top
    let cursor_bottom = cursor_world_pos.y + descender; // Descender bottom

    // Bright orange cursor color
    let cursor_color = Color::srgb(1.0, 0.5, 0.0);

    // Create zoom-aware mesh-based cursor
    create_mesh_cursor(
        commands,
        meshes,
        materials,
        entity_pools,
        cursor_world_pos,
        cursor_top,
        cursor_bottom,
        cursor_color,
        camera_scale,
    );
}

/// Clear all cursor entities from the screen (internal)
pub(crate) fn clear_cursor_entities(
    commands: &mut Commands,
    entity_pools: &mut ResMut<EntityPools>,
) {
    entity_pools.return_cursor_entities(commands);
}

/// Create a mesh-based cursor with triangular ends
#[allow(clippy::too_many_arguments)]
fn create_mesh_cursor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    entity_pools: &mut ResMut<EntityPools>,
    cursor_pos: Vec2,
    cursor_top: f32,
    cursor_bottom: f32,
    cursor_color: Color,
    camera_scale: &crate::rendering::zoom_aware_scaling::CameraResponsiveScale,
) {
    let outline_width = camera_scale.adjusted_line_width();
    let cursor_width = outline_width * 2.0; // 2x the outline width
    let circle_size = cursor_width * 4.0;

    // Create main vertical line mesh
    let line_mesh = create_cursor_line_mesh(
        Vec2::new(cursor_pos.x, cursor_bottom),
        Vec2::new(cursor_pos.x, cursor_top),
        cursor_width,
    );

    // Create circle meshes for top and bottom
    let top_circle_mesh = create_circle_mesh(circle_size);
    let bottom_circle_mesh = create_circle_mesh(circle_size);

    let cursor_material = materials.add(ColorMaterial::from(cursor_color));
    let cursor_z = 15.0; // Above everything else

    // Get cursor line entity from pool
    let line_entity = entity_pools.get_cursor_entity(commands, PooledEntityType::Cursor);

    update_cursor_entity(
        commands,
        line_entity,
        meshes.add(line_mesh),
        cursor_material.clone(),
        Transform::from_xyz(cursor_pos.x, (cursor_top + cursor_bottom) * 0.5, cursor_z),
        TextEditorCursor,
    );

    // Get top circle entity from pool
    let top_circle_entity = entity_pools.get_cursor_entity(commands, PooledEntityType::Cursor);

    update_cursor_entity(
        commands,
        top_circle_entity,
        meshes.add(top_circle_mesh),
        cursor_material.clone(),
        Transform::from_xyz(cursor_pos.x, cursor_top, cursor_z),
        TextEditorCursor,
    );

    // Get bottom circle entity from pool
    let bottom_circle_entity = entity_pools.get_cursor_entity(commands, PooledEntityType::Cursor);

    update_cursor_entity(
        commands,
        bottom_circle_entity,
        meshes.add(bottom_circle_mesh),
        cursor_material,
        Transform::from_xyz(cursor_pos.x, cursor_bottom, cursor_z),
        TextEditorCursor,
    );
}

/// Create a vertical line mesh for the cursor
fn create_cursor_line_mesh(start: Vec2, end: Vec2, width: f32) -> Mesh {
    let direction = (end - start).normalize();
    let perpendicular = Vec2::new(-direction.y, direction.x) * width * 0.5;
    let midpoint = (start + end) * 0.5;

    // Make coordinates relative to midpoint
    let start_rel = start - midpoint;
    let end_rel = end - midpoint;

    let vertices = vec![
        [
            start_rel.x - perpendicular.x,
            start_rel.y - perpendicular.y,
            0.0,
        ], // Bottom left
        [
            start_rel.x + perpendicular.x,
            start_rel.y + perpendicular.y,
            0.0,
        ], // Top left
        [
            end_rel.x + perpendicular.x,
            end_rel.y + perpendicular.y,
            0.0,
        ], // Top right
        [
            end_rel.x - perpendicular.x,
            end_rel.y - perpendicular.y,
            0.0,
        ], // Bottom right
    ];

    let indices = vec![0, 1, 2, 0, 2, 3]; // Two triangles forming a rectangle
    let uvs = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
    let normals = vec![[0.0, 0.0, 1.0]; 4];

    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    mesh
}

/// Create a circular mesh for cursor ends
fn create_circle_mesh(diameter: f32) -> Mesh {
    let radius = diameter * 0.5;
    let segments = 32; // Number of segments for circle smoothness

    let mut vertices = vec![[0.0, 0.0, 0.0]]; // Center vertex
    let mut uvs = vec![[0.5, 0.5]]; // Center UV
    let mut indices = Vec::new();

    // Create circle vertices around the perimeter
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        let x = radius * angle.cos();
        let y = radius * angle.sin();

        vertices.push([x, y, 0.0]);

        // UV coordinates mapped from -1,1 to 0,1
        let u = (x / radius + 1.0) * 0.5;
        let v = (y / radius + 1.0) * 0.5;
        uvs.push([u, v]);

        // Create triangle indices (center, current, next)
        let next_i = (i + 1) % segments;
        indices.push(0); // Center
        indices.push((i + 1) as u32); // Current vertex
        indices.push((next_i + 1) as u32); // Next vertex
    }

    let normals = vec![[0.0, 0.0, 1.0]; vertices.len()];

    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    mesh
}
