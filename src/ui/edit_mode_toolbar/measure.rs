use crate::ui::edit_mode_toolbar::{EditTool, ToolRegistry};
use bevy::prelude::*;
use kurbo::ParamCurve;

/// Resource to track if measure mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct MeasureModeActive(pub bool);

pub struct MeasureTool;

impl EditTool for MeasureTool {
    fn id(&self) -> crate::ui::edit_mode_toolbar::ToolId {
        "measure"
    }

    fn name(&self) -> &'static str {
        "Measure"
    }

    fn icon(&self) -> &'static str {
        "\u{E015}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('m')
    }

    fn default_order(&self) -> i32 {
        60 // After text tool, before pan
    }

    fn description(&self) -> &'static str {
        "Measure distances and dimensions"
    }

    fn update(&self, commands: &mut Commands) {
        info!("📏 MEASURE_TOOL: update() called - setting measure mode active and input mode to Measure");
        commands.insert_resource(MeasureModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Measure);
    }

    fn on_enter(&self) {
        info!("Entered Measure tool");
    }

    fn on_exit(&self) {
        info!("Exited Measure tool");
    }
}

/// Plugin for the Measure tool
pub struct MeasureToolPlugin;

impl Plugin for MeasureToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeasureModeActive>()
            .add_systems(Startup, register_measure_tool)
            .add_systems(
                Update,
                (
                    manage_measure_mode_state,
                    update_measure_shift_state, // Add shift key detection
                    render_measure_preview.after(manage_measure_mode_state),
                ),
            );
    }
}

fn register_measure_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(MeasureTool));
}

/// System to manage measure mode activation/deactivation
pub fn manage_measure_mode_state(
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
    measure_mode: Option<Res<MeasureModeActive>>,
) {
    let is_measure_active = current_tool.get_current() == Some("measure");
    let current_mode = measure_mode.as_ref().map(|m| m.0).unwrap_or(false);

    if is_measure_active && !current_mode {
        // Measure tool is active but mode is not set - activate it
        commands.insert_resource(MeasureModeActive(true));
        info!("📏 MANAGE_MEASURE_MODE: Activating measure mode");
    } else if !is_measure_active && current_mode {
        // Measure tool is not active but mode is set - deactivate it
        commands.insert_resource(MeasureModeActive(false));
        info!("📏 MANAGE_MEASURE_MODE: Deactivating measure mode");
    }
}

/// System to update shift key state for axis-aligned measurements
pub fn update_measure_shift_state(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    mut measure_consumer: ResMut<crate::systems::input_consumer::MeasureInputConsumer>,
) {
    // Only update when measure tool is active
    if current_tool.get_current() == Some("measure") {
        let shift_pressed =
            keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

        // Only log when state changes to avoid spam
        if measure_consumer.shift_locked != shift_pressed {
            measure_consumer.shift_locked = shift_pressed;
            if shift_pressed {
                info!("📏 MEASURE: Shift constraint enabled - lines will be horizontal/vertical");
            } else {
                info!("📏 MEASURE: Shift constraint disabled - lines can be any angle");
            }
        }
    }
}

/// Render the measure tool preview
#[allow(clippy::too_many_arguments)]
pub fn render_measure_preview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<bevy::render::mesh::Mesh>>,
    mut materials: ResMut<Assets<bevy::sprite::ColorMaterial>>,
    measure_consumer: Res<crate::systems::input_consumer::MeasureInputConsumer>,
    measure_mode: Option<Res<MeasureModeActive>>,
    camera_scale: Res<crate::rendering::zoom_aware_scaling::CameraResponsiveScale>,
    mut measure_entities: Local<Vec<Entity>>,
    theme: Res<crate::ui::themes::CurrentTheme>,
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    mut update_tracker: Local<Option<crate::systems::input_consumer::MeasureGestureState>>,
    fontir_state: Option<Res<crate::core::state::FontIRAppState>>,
) {
    // Check if tool is active
    let is_measure_active = current_tool.get_current() == Some("measure")
        && measure_mode.as_ref().map(|m| m.0).unwrap_or(false);

    // Debug current state
    info!(
        "📏 RENDER_MEASURE_PREVIEW: current_tool={:?}, measure_mode={:?}, is_measure_active={}",
        current_tool.get_current(),
        measure_mode.as_ref().map(|m| m.0),
        is_measure_active
    );
    info!(
        "📏 RENDER_MEASURE_PREVIEW: measure_consumer.gesture={:?}",
        measure_consumer.gesture
    );

    // Only update if gesture state has changed, measure tool became active, or cleanup needed
    let gesture_changed = update_tracker.as_ref() != Some(&measure_consumer.gesture);
    let cleanup_needed = !measure_entities.is_empty() && !is_measure_active;
    let tool_became_active = is_measure_active && measure_entities.is_empty();
    let needs_update = gesture_changed || cleanup_needed || tool_became_active;

    info!("📏 RENDER_MEASURE_PREVIEW: gesture_changed={}, cleanup_needed={}, tool_became_active={}, needs_update={}, measure_entities.len()={}", 
          gesture_changed, cleanup_needed, tool_became_active, needs_update, measure_entities.len());

    if !needs_update {
        return; // Early exit for performance
    }

    // Update tracking state
    *update_tracker = Some(measure_consumer.gesture);

    // Clean up previous measure entities
    for entity in measure_entities.drain(..) {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }

    // Check if measure tool is active
    if current_tool.get_current() != Some("measure") {
        return;
    }

    // Also check measure mode resource
    if let Some(measure_mode) = measure_mode {
        if !measure_mode.0 {
            return;
        }
    } else {
        return;
    }

    // Draw the measuring line
    if let Some((start, end)) = measure_consumer.get_measuring_line() {
        debug!(
            "📏 RENDER_MEASURE_PREVIEW: Drawing measuring line from {:?} to {:?}",
            start, end
        );
        let line_color = theme.theme().active_color(); // Use bright active green for measure line
        let line_width = camera_scale.adjusted_line_width();

        // Create dashed line for measuring (like knife tool)
        let dash_length = camera_scale.scale_factor * 8.0; // Match knife tool dash length
        let gap_length = camera_scale.scale_factor * 4.0; // Match knife tool gap length
        
        let dashed_line_entity = spawn_dashed_measure_line(
            &mut commands,
            &mut meshes,
            &mut materials,
            start,
            end,
            dash_length,
            gap_length,
            line_width,
            line_color,
            18.0, // z-order (below intersection points but above other elements)
        );
        measure_entities.push(dashed_line_entity);

        // Draw start point (yellow circle like intersection markers)
        let point_color = theme.theme().selected_color(); // Use yellow selection color
        let point_size = camera_scale.adjusted_point_size(4.0);
        let point_entity = spawn_measure_point_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            start,
            point_size,
            point_color,
            19.0, // z-order above line but below intersection points
        );
        measure_entities.push(point_entity);

        // Draw end point (yellow circle like intersection markers)
        let end_point_entity = spawn_measure_point_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            end,
            point_size,
            point_color,
            19.0, // z-order above line but below intersection points
        );
        measure_entities.push(end_point_entity);

        debug!(
            "📏 RENDER_MEASURE_PREVIEW: Created {} visual entities for measure preview",
            measure_entities.len()
        );
    } else {
        // Log when we're not drawing
        if matches!(
            measure_consumer.gesture,
            crate::systems::input_consumer::MeasureGestureState::Ready
        ) {
            debug!("📏 RENDER_MEASURE_PREVIEW: No measuring line to draw (Ready state)");
        }
    }

    // Calculate and draw intersection points from actual glyph data
    if let Some((start, end)) = measure_consumer.get_measuring_line() {
        let intersections = calculate_measure_intersections(start, end, &fontir_state);

        let intersection_color = theme.theme().selected_color(); // Use yellow selection color for intersection points

        for &intersection in &intersections {
            // Create yellow filled circles for intersection points (doubled size)
            let intersection_size = camera_scale.adjusted_point_size(6.0); // Doubled from 3.0 to 6.0
            let intersection_entity = spawn_measure_point_mesh(
                &mut commands,
                &mut meshes,
                &mut materials,
                intersection,
                intersection_size,
                intersection_color,
                20.0, // z-order above everything else
            );
            measure_entities.push(intersection_entity);
        }

        // Calculate and display distance measurements for ALL consecutive pairs
        if intersections.len() >= 2 {
            info!(
                "📏 MEASURE: Found {} intersection points, calculating {} segment distances",
                intersections.len(),
                intersections.len() - 1
            );
            
            // Sort intersections by position along the measuring line
            let mut sorted_intersections = intersections.clone();
            let measuring_line = kurbo::Line::new(
                kurbo::Point::new(start.x as f64, start.y as f64),
                kurbo::Point::new(end.x as f64, end.y as f64),
            );
            
            sorted_intersections.sort_by(|a, b| {
                let cutting_dir = (measuring_line.p1 - measuring_line.p0).normalize();
                let a_proj = (kurbo::Point::new(a.x as f64, a.y as f64) - measuring_line.p0).dot(cutting_dir);
                let b_proj = (kurbo::Point::new(b.x as f64, b.y as f64) - measuring_line.p0).dot(cutting_dir);
                a_proj.partial_cmp(&b_proj).unwrap_or(std::cmp::Ordering::Equal)
            });
            
            // Create distance measurements for each consecutive pair
            for i in 0..(sorted_intersections.len() - 1) {
                let point1 = sorted_intersections[i];
                let point2 = sorted_intersections[i + 1];
                let distance = point1.distance(point2);
                let midpoint = (point1 + point2) * 0.5;

                // Format distance value appropriately - show integers without decimals
                let distance_text = if distance.fract().abs() < 1e-6 {
                    // It's essentially a whole number
                    format!("{distance:.0}")
                } else {
                    // Show one decimal place
                    format!("{distance:.1}")
                };

                // Spawn pill-shaped background for text with higher z-orders
                let pill_entities = spawn_measure_text_with_pill_background(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    midpoint,
                    &distance_text,
                    &theme,
                    &camera_scale,
                );
                measure_entities.extend(pill_entities);

                info!(
                    "📏 MEASURE: Segment {}: Distance between points {:?} and {:?} = {} units",
                    i + 1, point1, point2, distance_text
                );
            }
        }
    }
}

/// Calculate intersections between measure line and current glyph contours
fn calculate_measure_intersections(
    start: Vec2,
    end: Vec2,
    fontir_state: &Option<Res<crate::core::state::FontIRAppState>>,
) -> Vec<Vec2> {
    let mut intersections = Vec::new();

    // Convert measuring line to kurbo Line for intersection testing
    let measuring_line = kurbo::Line::new(
        kurbo::Point::new(start.x as f64, start.y as f64),
        kurbo::Point::new(end.x as f64, end.y as f64),
    );

    // Try FontIR state first (preferred)
    if let Some(fontir_state) = fontir_state {
        if let Some(ref current_glyph) = fontir_state.current_glyph {
            if let Some(paths) = fontir_state.get_glyph_paths_with_edits(current_glyph) {
                debug!(
                    "📏 CALCULATE_MEASURE_INTERSECTIONS: Found {} paths for glyph '{}'",
                    paths.len(),
                    current_glyph
                );
                for path in &paths {
                    let path_intersections = find_measure_path_intersections(path, &measuring_line);
                    for intersection in path_intersections {
                        intersections.push(Vec2::new(intersection.x as f32, intersection.y as f32));
                    }
                }
                debug!(
                    "📏 CALCULATE_MEASURE_INTERSECTIONS: Total intersections found: {}",
                    intersections.len()
                );
                return intersections;
            } else {
                info!(
                    "📏 CALCULATE_MEASURE_INTERSECTIONS: No paths found for glyph '{}'",
                    current_glyph
                );
            }
        } else {
            info!("📏 CALCULATE_MEASURE_INTERSECTIONS: No current glyph selected");
        }
    } else {
        info!("📏 CALCULATE_MEASURE_INTERSECTIONS: No FontIR state available");
    }

    intersections
}

/// Find intersections between a measuring line and a path (reuse knife tool logic)
fn find_measure_path_intersections(
    path: &kurbo::BezPath,
    measuring_line: &kurbo::Line,
) -> Vec<kurbo::Point> {
    let mut intersections = Vec::new();
    let mut current_point = kurbo::Point::ZERO;

    for element in path.elements() {
        match element {
            kurbo::PathEl::MoveTo(pt) => {
                current_point = *pt;
            }
            kurbo::PathEl::LineTo(end) => {
                let segment = kurbo::Line::new(current_point, *end);
                if let Some(intersection_point) =
                    line_line_intersection_simple(measuring_line, &segment)
                {
                    intersections.push(intersection_point);
                }
                current_point = *end;
            }
            kurbo::PathEl::CurveTo(c1, c2, end) => {
                let curve = kurbo::CubicBez::new(current_point, *c1, *c2, *end);
                let curve_intersections = curve_line_intersections_simple(&curve, measuring_line);
                intersections.extend(curve_intersections);
                current_point = *end;
            }
            kurbo::PathEl::QuadTo(c, end) => {
                let curve = kurbo::QuadBez::new(current_point, *c, *end);
                let curve_intersections = quad_line_intersections_simple(&curve, measuring_line);
                intersections.extend(curve_intersections);
                current_point = *end;
            }
            kurbo::PathEl::ClosePath => {
                if let Some(start_point) = get_path_start_point(path) {
                    let segment = kurbo::Line::new(current_point, start_point);
                    if let Some(intersection_point) =
                        line_line_intersection_simple(measuring_line, &segment)
                    {
                        intersections.push(intersection_point);
                    }
                }
            }
        }
    }

    intersections.dedup_by(|a, b| a.distance(*b) < 5.0);
    intersections
}

/// Helper functions from knife tool (reused for measure tool)
fn line_line_intersection_simple(line1: &kurbo::Line, line2: &kurbo::Line) -> Option<kurbo::Point> {
    let p1 = line1.p0;
    let p2 = line1.p1;
    let p3 = line2.p0;
    let p4 = line2.p1;

    let denom = (p1.x - p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x - p4.x);

    if denom.abs() < 1e-10 {
        return None;
    }

    let t = ((p1.x - p3.x) * (p3.y - p4.y) - (p1.y - p3.y) * (p3.x - p4.x)) / denom;
    let u = -((p1.x - p2.x) * (p1.y - p3.y) - (p1.y - p2.y) * (p1.x - p3.x)) / denom;

    // IMPORTANT: Check that both t and u are within [0,1] for line segment intersection
    // t is the parameter for the measuring line, u is for the path segment
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some(kurbo::Point::new(
            p1.x + t * (p2.x - p1.x),
            p1.y + t * (p2.y - p1.y),
        ))
    } else {
        None
    }
}

fn curve_line_intersections_simple(
    curve: &kurbo::CubicBez,
    line: &kurbo::Line,
) -> Vec<kurbo::Point> {
    let mut intersections = Vec::new();
    let curve_seg = kurbo::PathSeg::Cubic(*curve);
    let curve_intersections = curve_seg.intersect_line(*line);

    for intersection in curve_intersections {
        let point = curve.eval(intersection.segment_t);
        
        // IMPORTANT: Check if intersection point lies within the measuring line segment
        // kurbo finds intersections with infinite line, but we want line segment only
        if point_lies_on_line_segment(point, line) {
            intersections.push(point);
        }
    }

    intersections.dedup_by(|a, b| a.distance(*b) < 1.0);
    intersections
}

fn quad_line_intersections_simple(curve: &kurbo::QuadBez, line: &kurbo::Line) -> Vec<kurbo::Point> {
    let mut intersections = Vec::new();
    let curve_seg = kurbo::PathSeg::Quad(*curve);
    let curve_intersections = curve_seg.intersect_line(*line);

    for intersection in curve_intersections {
        let point = curve.eval(intersection.segment_t);
        
        // IMPORTANT: Check if intersection point lies within the measuring line segment
        // kurbo finds intersections with infinite line, but we want line segment only
        if point_lies_on_line_segment(point, line) {
            intersections.push(point);
        }
    }

    intersections.dedup_by(|a, b| a.distance(*b) < 1.0);
    intersections
}

fn get_path_start_point(path: &kurbo::BezPath) -> Option<kurbo::Point> {
    for element in path.elements() {
        if let kurbo::PathEl::MoveTo(pt) = element {
            return Some(*pt);
        }
    }
    None
}

/// Check if a point lies on a line segment (not just the infinite line)
fn point_lies_on_line_segment(point: kurbo::Point, line: &kurbo::Line) -> bool {
    // Calculate the parameter t for the point on the line
    let dx = line.p1.x - line.p0.x;
    let dy = line.p1.y - line.p0.y;
    
    // Handle near-vertical and near-horizontal lines appropriately
    let t = if dx.abs() > dy.abs() {
        // Use x coordinate for parameter calculation
        if dx.abs() < 1e-10 {
            // Vertical line
            return (point.x - line.p0.x).abs() < 1e-6 
                && point.y >= line.p0.y.min(line.p1.y) - 1e-6
                && point.y <= line.p0.y.max(line.p1.y) + 1e-6;
        }
        (point.x - line.p0.x) / dx
    } else {
        // Use y coordinate for parameter calculation
        if dy.abs() < 1e-10 {
            // Horizontal line
            return (point.y - line.p0.y).abs() < 1e-6 
                && point.x >= line.p0.x.min(line.p1.x) - 1e-6
                && point.x <= line.p0.x.max(line.p1.x) + 1e-6;
        }
        (point.y - line.p0.y) / dy
    };
    
    // Check if t is within [0, 1] (point lies on line segment)
    (0.0..=1.0).contains(&t)
}

/// Spawn a dashed line mesh for the measure tool (similar to knife tool)
#[allow(clippy::too_many_arguments)]
fn spawn_dashed_measure_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<bevy::render::mesh::Mesh>>,
    materials: &mut ResMut<Assets<bevy::sprite::ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    dash_length: f32,
    gap_length: f32,
    width: f32,
    color: bevy::color::Color,
    z: f32,
) -> Entity {
    use bevy::render::mesh::{Indices, PrimitiveTopology};

    let direction = (end - start).normalize();
    let perpendicular = Vec2::new(-direction.y, direction.x) * width * 0.5;
    let total_length = start.distance(end);
    let segment_length = dash_length + gap_length;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut vertex_count = 0u32;

    // Generate all dash segments in a single mesh
    let mut current_pos = 0.0;
    while current_pos < total_length {
        let dash_start = start + direction * current_pos;
        let dash_end_pos = (current_pos + dash_length).min(total_length);
        let dash_end = start + direction * dash_end_pos;

        // Add vertices for this dash segment
        vertices.push([
            dash_start.x - perpendicular.x,
            dash_start.y - perpendicular.y,
            z,
        ]);
        vertices.push([
            dash_start.x + perpendicular.x,
            dash_start.y + perpendicular.y,
            z,
        ]);
        vertices.push([
            dash_end.x + perpendicular.x,
            dash_end.y + perpendicular.y,
            z,
        ]);
        vertices.push([
            dash_end.x - perpendicular.x,
            dash_end.y - perpendicular.y,
            z,
        ]);

        // Add indices for this dash segment
        indices.extend_from_slice(&[
            vertex_count,
            vertex_count + 1,
            vertex_count + 2,
            vertex_count,
            vertex_count + 2,
            vertex_count + 3,
        ]);

        vertex_count += 4;
        current_pos += segment_length;
    }

    if vertices.is_empty() {
        // Create a dummy entity if no dashes were created
        return commands.spawn_empty().id();
    }

    let mut mesh = bevy::render::mesh::Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(bevy::sprite::ColorMaterial::from(color));

    commands
        .spawn((
            bevy::render::mesh::Mesh2d(mesh_handle),
            bevy::sprite::MeshMaterial2d(material_handle),
            Transform::from_translation(Vec3::new(0.0, 0.0, z)),
        ))
        .id()
}


/// Spawn a point (circle) mesh for the measure tool
fn spawn_measure_point_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<bevy::render::mesh::Mesh>>,
    materials: &mut ResMut<Assets<bevy::sprite::ColorMaterial>>,
    position: Vec2,
    radius: f32,
    color: bevy::color::Color,
    z: f32,
) -> Entity {
    use bevy::render::mesh::{Indices, PrimitiveTopology};

    // Create circle using triangle fan
    let segments = 16;
    let mut vertices = vec![[position.x, position.y, z]]; // Center vertex
    let mut indices = Vec::new();

    // Create vertices around the circle
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        let x = position.x + radius * angle.cos();
        let y = position.y + radius * angle.sin();
        vertices.push([x, y, z]);
    }

    // Create triangle indices
    for i in 0..segments {
        indices.extend_from_slice(&[0, i + 1, i + 2]);
    }

    let mut mesh = bevy::render::mesh::Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(bevy::sprite::ColorMaterial::from(color));

    commands
        .spawn((
            bevy::render::mesh::Mesh2d(mesh_handle),
            bevy::sprite::MeshMaterial2d(material_handle),
            Transform::from_translation(Vec3::new(0.0, 0.0, z)),
        ))
        .id()
}

/// Spawn a text label with pill-shaped background for distance measurement
fn spawn_measure_text_with_pill_background(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<bevy::render::mesh::Mesh>>,
    materials: &mut ResMut<Assets<bevy::sprite::ColorMaterial>>,
    position: Vec2,
    text_content: &str,
    theme: &Res<crate::ui::themes::CurrentTheme>,
    camera_scale: &Res<crate::rendering::zoom_aware_scaling::CameraResponsiveScale>,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    // Calculate camera-responsive font size
    let base_font_size = 14.0;
    let scaled_font_size = base_font_size * camera_scale.scale_factor;

    // Estimate text dimensions for background pill
    let text_width = text_content.len() as f32 * scaled_font_size * 0.6; // Rough estimation
    let text_height = scaled_font_size;
    let pill_width = text_width + (8.0 * camera_scale.scale_factor); // Padding
    let pill_height = text_height + (4.0 * camera_scale.scale_factor); // Padding

    // Create pill-shaped background (rounded rectangle) using orange color
    let background_color = theme.theme().action_color(); // Same orange as hit points
    let pill_entity = spawn_pill_background_mesh(
        commands,
        meshes,
        materials,
        position,
        pill_width,
        pill_height,
        background_color,
        100.0, // Much higher z-order below text
    );
    entities.push(pill_entity);

    // Create text label on top using black color
    let text_color = Color::BLACK; // Black text for good contrast on orange
    let text_entity = commands
        .spawn((
            Text2d::new(text_content),
            TextFont {
                font_size: scaled_font_size,
                ..default()
            },
            TextColor(text_color),
            Transform::from_translation(Vec3::new(position.x, position.y, 200.0)), // Much higher z-order above background
        ))
        .id();
    entities.push(text_entity);

    entities
}

/// Spawn a pill-shaped background mesh
#[allow(clippy::too_many_arguments)]
fn spawn_pill_background_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<bevy::render::mesh::Mesh>>,
    materials: &mut ResMut<Assets<bevy::sprite::ColorMaterial>>,
    position: Vec2,
    width: f32,
    height: f32,
    color: bevy::color::Color,
    z: f32,
) -> Entity {
    use bevy::render::mesh::{Indices, PrimitiveTopology};

    // Create rounded rectangle (pill shape) using multiple segments
    let radius = height * 0.5;
    let rect_width = width - height; // Width of the rectangular part
    let segments = 8; // Number of segments for each rounded end

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Center vertex for triangle fan
    vertices.push([position.x, position.y, z]);
    let center_index = 0;

    // Left semicircle
    let left_center = position.x - rect_width * 0.5;
    for i in 0..=segments {
        let angle =
            std::f32::consts::PI * 0.5 + (i as f32 / segments as f32) * std::f32::consts::PI;
        let x = left_center + radius * angle.cos();
        let y = position.y + radius * angle.sin();
        vertices.push([x, y, z]);
    }

    // Right semicircle
    let right_center = position.x + rect_width * 0.5;
    for i in 0..=segments {
        let angle =
            -std::f32::consts::PI * 0.5 + (i as f32 / segments as f32) * std::f32::consts::PI;
        let x = right_center + radius * angle.cos();
        let y = position.y + radius * angle.sin();
        vertices.push([x, y, z]);
    }

    // Create triangle fan indices
    let total_vertices = vertices.len() as u32;
    for i in 1..total_vertices {
        let next_i = if i == total_vertices - 1 { 1 } else { i + 1 };
        indices.extend_from_slice(&[center_index, i, next_i]);
    }

    let mut mesh = bevy::render::mesh::Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(bevy::sprite::ColorMaterial::from(color));

    commands
        .spawn((
            bevy::render::mesh::Mesh2d(mesh_handle),
            bevy::sprite::MeshMaterial2d(material_handle),
            Transform::from_translation(Vec3::new(0.0, 0.0, z)),
        ))
        .id()
}
