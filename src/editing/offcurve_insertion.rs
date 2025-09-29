//! Off-curve point insertion for converting lines to curves
//!
//! This module handles the Alt+click functionality to insert off-curve points
//! on line segments, converting them to cubic Bézier curves.

use bevy::prelude::*;
use kurbo::{Line, Point};
use crate::editing::selection::events::AppStateChanged;
use crate::core::state::FontIRAppState;
use crate::rendering::glyph_renderer::SortVisualUpdateTracker;

/// Resource to track hover state over line segments
#[derive(Resource, Default)]
pub struct LineSegmentHoverState {
    /// The line segment currently being hovered over (in world space)
    pub hovered_segment: Option<(Vec2, Vec2)>,
    /// The indices of the segment in the contour (contour_index, start_point_index)
    pub segment_indices: Option<(usize, usize)>,
    /// Whether Alt is currently held
    pub alt_held: bool,
    /// The glyph name for the hovered segment
    pub glyph_name: Option<String>,
}

/// Component to mark preview elements for off-curve insertion
#[derive(Component)]
pub struct OffCurvePreviewElement;

/// System to detect hovering over line segments with Alt held
pub fn detect_line_segment_hover(
    mut hover_state: ResMut<LineSegmentHoverState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pointer_info: Res<crate::io::pointer::PointerInfo>,
    fontir_app_state: Option<Res<FontIRAppState>>,
    active_sort_query: Query<
        (Entity, &crate::editing::sort::Sort, &Transform),
        With<crate::editing::sort::ActiveSort>,
    >,
) {
    // Check if Alt is held
    let alt_held = keyboard_input.pressed(KeyCode::AltLeft) || keyboard_input.pressed(KeyCode::AltRight);
    hover_state.alt_held = alt_held;

    if !alt_held {
        // Clear hover state if Alt is not held
        hover_state.hovered_segment = None;
        hover_state.segment_indices = None;
        hover_state.glyph_name = None;
        return;
    }

    // Get the active sort
    let Ok((_, sort, sort_transform)) = active_sort_query.single() else {
        hover_state.hovered_segment = None;
        hover_state.segment_indices = None;
        hover_state.glyph_name = None;
        return;
    };

    let sort_position = sort_transform.translation.truncate();

    // Get FontIR data
    let Some(fontir_state) = fontir_app_state.as_ref() else {
        return;
    };

    // Get the paths for the current glyph
    let Some(paths) = fontir_state.get_glyph_paths_with_edits(&sort.glyph_name) else {
        return;
    };

    let mouse_world = pointer_info.design.to_raw();
    let mouse_relative = mouse_world - sort_position;
    let mouse_point = Point::new(mouse_relative.x as f64, mouse_relative.y as f64);

    // Find the closest line segment
    let mut closest_segment = None;
    let mut closest_distance = 20.0; // Threshold in pixels

    for (contour_index, path) in paths.iter().enumerate() {
        let elements: Vec<_> = path.elements().iter().cloned().collect();

        for (i, window) in elements.windows(2).enumerate() {
            // Check if this is a line segment
            if let (kurbo::PathEl::MoveTo(p1) | kurbo::PathEl::LineTo(p1),
                    kurbo::PathEl::LineTo(p2)) = (window[0], window[1]) {

                let line = Line::new(p1, p2);
                let distance = distance_to_line_segment(mouse_point, line);

                if distance < closest_distance {
                    closest_distance = distance;
                    // Convert to world coordinates
                    let world_p1 = Vec2::new(p1.x as f32, p1.y as f32) + sort_position;
                    let world_p2 = Vec2::new(p2.x as f32, p2.y as f32) + sort_position;
                    closest_segment = Some((world_p1, world_p2));
                    hover_state.segment_indices = Some((contour_index, i));
                    hover_state.glyph_name = Some(sort.glyph_name.clone());
                }
            }
        }
    }

    hover_state.hovered_segment = closest_segment;
}

/// Calculate distance from a point to a line segment
fn distance_to_line_segment(point: Point, line: Line) -> f64 {
    let v = line.p1 - line.p0;
    let w = point - line.p0;

    let c1 = w.dot(v);
    if c1 <= 0.0 {
        return point.distance(line.p0);
    }

    let c2 = v.dot(v);
    if c1 >= c2 {
        return point.distance(line.p1);
    }

    let b = c1 / c2;
    let pb = line.p0 + v * b;
    point.distance(pb)
}

/// System to render preview of off-curve points when hovering
pub fn render_offcurve_preview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    hover_state: Res<LineSegmentHoverState>,
    existing_preview: Query<Entity, With<OffCurvePreviewElement>>,
    camera_scale: Res<crate::rendering::zoom_aware_scaling::CameraResponsiveScale>,
    theme: Res<crate::ui::themes::CurrentTheme>,
) {
    // Clean up existing preview
    for entity in existing_preview.iter() {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }

    // Only render if we have a hovered segment and Alt is held
    let Some((p1, p2)) = hover_state.hovered_segment else {
        return;
    };

    if !hover_state.alt_held {
        return;
    }

    // Calculate where the off-curve points would be placed
    // Place them at 1/3 and 2/3 along the line
    let one_third = p1 + (p2 - p1) * 0.333;
    let two_thirds = p1 + (p2 - p1) * 0.667;

    // Create preview handles (orange dotted lines)
    let preview_color = theme.theme().action_color(); // Orange color
    let line_width = camera_scale.adjusted_line_width() * 1.5;

    // Draw dotted lines from on-curve to off-curve positions
    spawn_dashed_line(&mut commands, &mut meshes, &mut materials, p1, one_third, preview_color, line_width);
    spawn_dashed_line(&mut commands, &mut meshes, &mut materials, two_thirds, p2, preview_color, line_width);

    // Draw the preview off-curve points
    let point_size = camera_scale.adjusted_size(8.0); // Base size for off-curve points

    for pos in [one_third, two_thirds] {
        let mesh = create_diamond_mesh(point_size);
        commands.spawn((
            bevy::render::mesh::Mesh2d(meshes.add(mesh)),
            bevy::sprite::MeshMaterial2d(materials.add(ColorMaterial::from(preview_color))),
            Transform::from_translation(Vec3::new(pos.x, pos.y, 15.0)),
            OffCurvePreviewElement,
        ));
    }
}

/// Create a diamond mesh for off-curve points
fn create_diamond_mesh(size: f32) -> Mesh {
    let half_size = size * 0.5;

    let vertices = vec![
        [0.0, half_size, 0.0],      // Top
        [half_size, 0.0, 0.0],       // Right
        [0.0, -half_size, 0.0],      // Bottom
        [-half_size, 0.0, 0.0],      // Left
    ];

    let indices = vec![0, 1, 2, 2, 3, 0];
    let normals = vec![[0.0, 0.0, 1.0]; 4];
    let uvs = vec![[0.5, 1.0], [1.0, 0.5], [0.5, 0.0], [0.0, 0.5]];

    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    mesh
}

/// Spawn a dashed line for preview
fn spawn_dashed_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    color: Color,
    width: f32,
) {
    let dash_length = 4.0;
    let gap_length = 4.0;

    let direction = (end - start).normalize();
    let total_length = start.distance(end);
    let segment_length = dash_length + gap_length;

    let mut current_pos = 0.0;

    while current_pos < total_length {
        let dash_start = start + direction * current_pos;
        let dash_end_pos = (current_pos + dash_length).min(total_length);
        let dash_end = start + direction * dash_end_pos;

        let line_mesh = crate::rendering::mesh_utils::create_line_mesh(dash_start, dash_end, width);
        let midpoint = (dash_start + dash_end) * 0.5;

        commands.spawn((
            bevy::render::mesh::Mesh2d(meshes.add(line_mesh)),
            bevy::sprite::MeshMaterial2d(materials.add(ColorMaterial::from(color))),
            Transform::from_translation(Vec3::new(midpoint.x, midpoint.y, 14.0)),
            OffCurvePreviewElement,
        ));

        current_pos += segment_length;
    }
}

/// System to handle Alt+click to insert off-curve points
pub fn handle_offcurve_insertion(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    hover_state: Res<LineSegmentHoverState>,
    mut fontir_app_state: Option<ResMut<FontIRAppState>>,
    mut app_state_changed: EventWriter<AppStateChanged>,
    mut visual_update_tracker: ResMut<SortVisualUpdateTracker>,
) {
    // Check for left click with Alt held
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    if !hover_state.alt_held {
        return;
    }

    let Some((p1, p2)) = hover_state.hovered_segment else {
        return;
    };

    let Some((contour_index, segment_index)) = hover_state.segment_indices else {
        return;
    };

    let Some(glyph_name) = hover_state.glyph_name.as_ref() else {
        return;
    };

    let Some(fontir_state) = fontir_app_state.as_mut() else {
        return;
    };

    // Get or create working copy
    let Some(working_copy) = fontir_state.get_or_create_working_copy(glyph_name) else {
        return;
    };

    // Get the contour
    if contour_index >= working_copy.contours.len() {
        return;
    }

    // Convert the line segment to a cubic curve
    let contour = &mut working_copy.contours[contour_index];
    let mut new_elements = Vec::new();

    for (i, element) in contour.elements().iter().enumerate() {
        new_elements.push(element.clone());

        // If this is the line segment we're converting
        if i == segment_index + 1 {
            if let kurbo::PathEl::LineTo(end_point) = element {
                // Get the start point
                let start_point = if i > 0 {
                    match new_elements[i - 1] {
                        kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) | kurbo::PathEl::CurveTo(_, _, p) => p,
                        _ => continue,
                    }
                } else {
                    continue;
                };

                // Calculate off-curve control points
                let ctrl1 = start_point + (*end_point - start_point) * 0.333;
                let ctrl2 = start_point + (*end_point - start_point) * 0.667;

                // Replace the LineTo with CurveTo
                new_elements.pop(); // Remove the LineTo we just added
                new_elements.push(kurbo::PathEl::CurveTo(ctrl1, ctrl2, *end_point));

                debug!("Converted line segment to curve with control points at {:?} and {:?}", ctrl1, ctrl2);
            }
        }
    }

    // Create new path with converted elements
    let mut new_path = kurbo::BezPath::new();
    for element in new_elements {
        match element {
            kurbo::PathEl::MoveTo(p) => new_path.move_to(p),
            kurbo::PathEl::LineTo(p) => new_path.line_to(p),
            kurbo::PathEl::CurveTo(p1, p2, p3) => new_path.curve_to(p1, p2, p3),
            kurbo::PathEl::QuadTo(p1, p2) => new_path.quad_to(p1, p2),
            kurbo::PathEl::ClosePath => new_path.close_path(),
        }
    }

    working_copy.contours[contour_index] = new_path;
    working_copy.is_dirty = true;

    // Trigger updates
    app_state_changed.write(AppStateChanged);
    visual_update_tracker.needs_update = true;

    info!("Inserted off-curve points on line segment in glyph '{}'", glyph_name);
}

/// Plugin to add off-curve insertion functionality
pub struct OffCurveInsertionPlugin;

impl Plugin for OffCurveInsertionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LineSegmentHoverState>()
            .add_systems(
                Update,
                (
                    detect_line_segment_hover,
                    render_offcurve_preview.after(detect_line_segment_hover),
                    handle_offcurve_insertion,
                )
            );
    }
}