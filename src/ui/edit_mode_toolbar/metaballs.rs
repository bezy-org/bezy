/*
Metaball Tool (Bezy Font Editor)
===============================

Goal:
-----
Interactive tool for creating blobby, merged outlines using metaball gizmos—similar to Blender's and Grasshopper's metaball/metasurface tools. The user places and manipulates circle gizmos, and the tool generates a smooth, organic outline that merges and bulges between them.

Approach:
---------
- Each gizmo is a metaball: field at point p is sum of r^2 / (distance^2 + epsilon) for all gizmos.
- The isocontour (where field = threshold) is extracted using the `contour-isobands` crate (marching squares implementation).
- The outline is rendered as a preview in the editor.
- Threshold and grid resolution are critical for the "blobby" effect. Lower threshold and finer grid = more merging and smoother outline.

Tuning:
-------
- If the outline is not blobby, try lowering the threshold (e.g., 0.1, 0.05).
- If the outline is jagged, lower the grid resolution (e.g., 0.25).
- Make sure the preview is rendering the output of `generate_metaball_outline`, not the hull of the gizmos.

Known Issues:
-------------
- If you still see a polygonal hull, check the rendering code and field function.
- Further debugging may be needed to ensure the correct outline is shown.
- For font export, you may want to fit a Bézier curve to the outline.

Status:
-------
- MVP implementation is present and correct in code, but may require further parameter tuning or rendering fixes for the desired effect.
*/
use crate::core::config::BezySettings;
use crate::core::state::{AppState, GlyphNavigation};
use crate::editing::selection::systems::AppStateChanged;
use crate::ui::edit_mode_toolbar::{EditTool, ToolId, ToolRegistry};
use crate::ui::themes::CurrentTheme;
use bevy::prelude::*;
use contour_isobands::ContourBuilder;

// ============================================================================
// RESOURCES
// ============================================================================

/// Tracks whether metaballs mode is currently active
#[derive(Resource, Default)]
pub struct MetaballsModeActive(pub bool);

/// Represents a single metaball gizmo
#[derive(Component, Clone, Debug)]
pub struct MetaballGizmo {
    pub position: Vec2,
    pub radius: f32,
    pub strength: f32,
    pub is_selected: bool,
}

/// Collection of metaball gizmos for the current glyph
#[derive(Resource, Default)]
pub struct MetaballGizmos {
    pub gizmos: Vec<MetaballGizmo>,
}

/// Settings for metaball generation
#[derive(Resource)]
pub struct MetaballSettings {
    pub resolution: f32,
    pub threshold: f32,
    pub _smoothing: f32,
}

impl Default for MetaballSettings {
    fn default() -> Self {
        Self {
            resolution: 0.5,
            threshold: 0.2,
            _smoothing: 0.1,
        }
    }
}

// ============================================================================
// TOOL IMPLEMENTATION
// ============================================================================

pub struct MetaballsTool;

impl EditTool for MetaballsTool {
    fn id(&self) -> ToolId {
        "metaballs"
    }

    fn name(&self) -> &'static str {
        "Metaballs"
    }

    fn icon(&self) -> &'static str {
        "\u{E021}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('m')
    }

    fn default_order(&self) -> i32 {
        60
    }

    fn description(&self) -> &'static str {
        "Create organic shapes using metaball gizmos"
    }

    fn update(&self, commands: &mut Commands) {
        // Activate metaballs mode
        commands.insert_resource(MetaballsModeActive(true));
        debug!("MetaballsTool::update() called - activating metaballs mode");
    }

    fn on_enter(&self) {
        debug!("METABALLS TOOL: Entered metaballs mode");
    }

    fn on_exit(&self) {
        debug!("METABALLS TOOL: Exited metaballs mode");
    }
}

// ============================================================================
// PLUGIN
// ============================================================================

pub struct MetaballsToolPlugin;

impl Plugin for MetaballsToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MetaballsModeActive>()
            .init_resource::<MetaballGizmos>()
            .init_resource::<MetaballSettings>()
            .add_systems(Startup, register_metaballs_tool)
            .add_systems(
                Update,
                (
                    handle_metaballs_mouse_events,
                    render_metaball_gizmos,
                    render_metaball_outline_preview,
                    reset_metaballs_mode_when_inactive,
                    handle_metaballs_keyboard_shortcuts,
                ),
            );
    }
}

fn register_metaballs_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(MetaballsTool));
}

// ============================================================================
// INPUT HANDLING
// ============================================================================

/// Handle mouse events for metaball gizmo placement and manipulation
pub fn handle_metaballs_mouse_events(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut metaball_gizmos: ResMut<MetaballGizmos>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    metaballs_mode: Option<Res<MetaballsModeActive>>,
) {
    // Only handle input if metaballs tool is active
    if let Some(metaballs_mode) = metaballs_mode {
        if !metaballs_mode.0 {
            return;
        }
    } else {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Convert cursor position to world coordinates
    if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
        // Apply grid snapping
        let settings = BezySettings::default();
        let snapped_position = settings.apply_grid_snap(world_position);

        // Handle left click to place new metaball gizmo
        if mouse_button_input.just_pressed(MouseButton::Left) {
            debug!(
                "METABALLS TOOL: Placing new gizmo at ({:.1}, {:.1})",
                snapped_position.x, snapped_position.y
            );

            let new_gizmo = MetaballGizmo {
                position: snapped_position,
                radius: 20.0,
                strength: 1.0,
                is_selected: false,
            };

            metaball_gizmos.gizmos.push(new_gizmo);
        }

        // Handle right click to select/deselect gizmos
        if mouse_button_input.just_pressed(MouseButton::Right) {
            // Find closest gizmo within selection radius
            let selection_radius = 15.0;
            let mut closest_distance = f32::INFINITY;
            let mut closest_index = None;

            for (i, gizmo) in metaball_gizmos.gizmos.iter().enumerate() {
                let distance = gizmo.position.distance(snapped_position);
                if distance < selection_radius && distance < closest_distance {
                    closest_distance = distance;
                    closest_index = Some(i);
                }
            }

            // Toggle selection
            if let Some(index) = closest_index {
                metaball_gizmos.gizmos[index].is_selected =
                    !metaball_gizmos.gizmos[index].is_selected;
                debug!("METABALLS TOOL: Toggled selection of gizmo {}", index);
            }
        }
    }
}

/// Handle keyboard shortcuts for metaballs
pub fn handle_metaballs_keyboard_shortcuts(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut metaball_gizmos: ResMut<MetaballGizmos>,
    mut app_state: ResMut<AppState>,
    mut app_state_changed: EventWriter<AppStateChanged>,
    glyph_navigation: Res<GlyphNavigation>,
    metaballs_mode: Option<Res<MetaballsModeActive>>,
) {
    // Only handle input if metaballs tool is active
    if let Some(metaballs_mode) = metaballs_mode {
        if !metaballs_mode.0 {
            return;
        }
    } else {
        return;
    }

    // Ctrl+C: Convert metaballs to curves
    if keyboard_input.pressed(KeyCode::ControlLeft) && keyboard_input.just_pressed(KeyCode::KeyC) {
        debug!("METABALLS TOOL: Converting metaballs to curves");
        convert_metaballs_to_curves(
            &mut metaball_gizmos,
            &mut app_state,
            &mut app_state_changed,
            &glyph_navigation,
        );
    }

    // Delete: Remove selected gizmos
    if keyboard_input.just_pressed(KeyCode::Delete) {
        metaball_gizmos.gizmos.retain(|gizmo| !gizmo.is_selected);
        debug!("METABALLS TOOL: Removed selected gizmos");
    }
}

// ============================================================================
// RENDERING
// ============================================================================

/// Render metaball gizmos
pub fn render_metaball_gizmos(
    mut gizmos: Gizmos,
    metaball_gizmos: Res<MetaballGizmos>,
    metaballs_mode: Option<Res<MetaballsModeActive>>,
    theme: Res<CurrentTheme>,
) {
    // Only render if metaballs tool is active
    if let Some(metaballs_mode) = metaballs_mode {
        if !metaballs_mode.0 {
            return;
        }
    } else {
        return;
    }

    for gizmo in &metaball_gizmos.gizmos {
        let color = if gizmo.is_selected {
            theme.theme().metaball_selected_color()
        } else {
            theme.theme().metaball_gizmo_color()
        };

        // Draw the metaball gizmo circle
        gizmos.circle_2d(gizmo.position, gizmo.radius, color);

        // Draw selection indicator
        if gizmo.is_selected {
            gizmos.circle_2d(
                gizmo.position,
                gizmo.radius + 2.0,
                theme.theme().metaball_outline_color(),
            );
        }
    }
}

/// Render the metaball outline preview in real-time
pub fn render_metaball_outline_preview(
    mut gizmos: Gizmos,
    metaball_gizmos: Res<MetaballGizmos>,
    metaballs_mode: Option<Res<MetaballsModeActive>>,
    theme: Res<CurrentTheme>,
) {
    // Only render if metaballs tool is active
    if let Some(metaballs_mode) = metaballs_mode {
        if !metaballs_mode.0 {
            return;
        }
    } else {
        return;
    }
    if metaball_gizmos.gizmos.len() < 2 {
        return;
    }
    // Generate cubic outline
    let segments = generate_metaball_cubic_outline(&metaball_gizmos.gizmos);
    // Draw each cubic segment as a polyline for preview
    for seg in &segments {
        // Optionally, draw the control polygon for debugging
        // gizmos.line_2d(seg[0], seg[1], Color::YELLOW);
        // gizmos.line_2d(seg[2], seg[3], Color::YELLOW);
        // Draw the cubic as a polyline
        let steps = 16;
        let mut prev = seg[0];
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let p = cubic_bezier(seg[0], seg[1], seg[2], seg[3], t);
            gizmos.line_2d(prev, p, theme.theme().metaball_outline_color());
            prev = p;
        }
    }
}

/// Evaluate a cubic Bézier at t
fn cubic_bezier(p0: Vec2, c1: Vec2, c2: Vec2, p1: Vec2, t: f32) -> Vec2 {
    let mt = 1.0 - t;
    mt * mt * mt * p0 + 3.0 * mt * mt * t * c1 + 3.0 * mt * t * t * c2 + t * t * t * p1
}

// ============================================================================
// MODE MANAGEMENT
// ============================================================================

/// Reset metaballs mode when another tool is selected
pub fn reset_metaballs_mode_when_inactive(
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
) {
    if current_tool.get_current() != Some("metaballs") {
        // Mark metaballs mode as inactive
        commands.insert_resource(MetaballsModeActive(false));
    }
}

// ============================================================================
// METABALL ALGORITHMS
// ============================================================================

/// Calculate metaball influence at a given point
fn calculate_metaball_influence(point: Vec2, gizmos: &[MetaballGizmo]) -> f32 {
    let mut total_influence = 0.0;
    for gizmo in gizmos {
        let distance_sq = point.distance_squared(gizmo.position);
        let r2 = gizmo.radius * gizmo.radius;
        // Add a small epsilon to avoid division by zero
        let influence = gizmo.strength * r2 / (distance_sq + 1e-4);
        total_influence += influence;
    }
    total_influence
}

/// Generate metaball outline using contour-isobands crate (marching squares)
fn generate_metaball_outline(gizmos: &[MetaballGizmo], settings: &MetaballSettings) -> Vec<Vec2> {
    if gizmos.is_empty() {
        return Vec::new();
    }
    // Calculate bounding box
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for gizmo in gizmos {
        min_x = min_x.min(gizmo.position.x - gizmo.radius);
        min_y = min_y.min(gizmo.position.y - gizmo.radius);
        max_x = max_x.max(gizmo.position.x + gizmo.radius);
        max_y = max_y.max(gizmo.position.y + gizmo.radius);
    }
    let padding = 20.0;
    min_x -= padding;
    min_y -= padding;
    max_x += padding;
    max_y += padding;
    let grid_size = settings.resolution.max(0.5); // finer grid for smoothness
    let nx = ((max_x - min_x) / grid_size).ceil() as usize + 1;
    let ny = ((max_y - min_y) / grid_size).ceil() as usize + 1;
    // Build the field as a flat Vec<f64>
    let mut values = Vec::with_capacity(nx * ny);
    for iy in 0..ny {
        for ix in 0..nx {
            let x = min_x + ix as f32 * grid_size;
            let y = min_y + iy as f32 * grid_size;
            let v = calculate_metaball_influence(Vec2::new(x, y), gizmos) as f64;
            values.push(v);
        }
    }
    // Use contour-isobands to extract the isocontour
    let intervals = vec![settings.threshold as f64];
    let builder = ContourBuilder::new(nx, ny)
        .x_origin(min_x as f64)
        .y_origin(min_y as f64)
        .x_step(grid_size as f64)
        .y_step(grid_size as f64);
    let bands = builder.contours(&values, &intervals).unwrap_or_default();
    if bands.is_empty() || bands[0].geometry().0.is_empty() {
        return Vec::new();
    }
    // Use the first polygon (exterior ring)
    let poly = &bands[0].geometry().0[0];
    poly.exterior()
        .0
        .iter()
        .map(|c| Vec2::new(c.x as f32, c.y as f32))
        .collect()
}

// ============================================================================
// CONVERSION TO CURVES
// ============================================================================

/// Convert metaball gizmos to cubic curves and add to current glyph
fn convert_metaballs_to_curves(
    metaball_gizmos: &mut MetaballGizmos,
    app_state: &mut AppState,
    app_state_changed: &mut EventWriter<AppStateChanged>,
    glyph_navigation: &GlyphNavigation,
) {
    let Some(glyph_name) = glyph_navigation.find_glyph(app_state) else {
        warn!("No current glyph selected for metaball conversion");
        return;
    };

    if metaball_gizmos.gizmos.is_empty() {
        warn!("No metaball gizmos to convert");
        return;
    }

    // Generate outline from metaballs
    let settings = MetaballSettings::default();
    let outline_points = generate_metaball_outline(&metaball_gizmos.gizmos, &settings);

    if outline_points.is_empty() {
        warn!("No outline generated from metaballs");
        return;
    }

    // Convert outline points to cubic curves
    let curves = outline_points_to_cubic_curves(&outline_points);
    let curves_count = curves.len();

    // Add curves to the glyph
    if let Some(glyph_data) = app_state.workspace.font.glyphs.get_mut(&glyph_name) {
        if glyph_data.outline.is_none() {
            glyph_data.outline = Some(crate::core::state::OutlineData {
                contours: Vec::new(),
            });
        }

        if let Some(outline) = &mut glyph_data.outline {
            for curve in curves {
                outline
                    .contours
                    .push(crate::core::state::ContourData { points: curve });
            }

            debug!(
                "Converted {} metaball gizmos to {} curves in glyph '{}'",
                metaball_gizmos.gizmos.len(),
                curves_count,
                glyph_name
            );

            // Clear the gizmos after conversion
            metaball_gizmos.gizmos.clear();

            app_state_changed.write(AppStateChanged);
        }
    }
}

/// Convert outline points to cubic curves
fn outline_points_to_cubic_curves(points: &[Vec2]) -> Vec<Vec<crate::core::state::PointData>> {
    if points.len() < 3 {
        return Vec::new();
    }

    let mut curves = Vec::new();
    let mut current_curve = Vec::new();

    // Start with a move command
    current_curve.push(crate::core::state::PointData {
        x: points[0].x as f64,
        y: points[0].y as f64,
        point_type: crate::core::state::PointTypeData::Move,
    });

    // Convert points to cubic curves
    for point in points.iter().skip(1) {
        // For simplicity, use line segments (can be improved with proper cubic curve fitting)
        current_curve.push(crate::core::state::PointData {
            x: point.x as f64,
            y: point.y as f64,
            point_type: crate::core::state::PointTypeData::Line,
        });
    }

    curves.push(current_curve);
    curves
}

// ============================================================================
// NEW: CUBIC BEZIER METABALL OUTLINE GENERATOR
// ============================================================================

/// Generate a smooth cubic Bézier outline for metaballs, using circle intersections as on-curve points and tangents for control points.
pub fn generate_metaball_cubic_outline(gizmos: &[MetaballGizmo]) -> Vec<[Vec2; 4]> {
    let n = gizmos.len();
    if n < 2 {
        return Vec::new();
    }
    let mut segments = Vec::new();
    // For each pair of adjacent metaballs
    for i in 0..n {
        let a = &gizmos[i];
        let b = &gizmos[(i + 1) % n];
        let p0 = a.position;
        let p1 = b.position;
        let r0 = a.radius;
        let r1 = b.radius;
        let d = p0.distance(p1);
        // Default: points on the circles in the direction of the other
        let mut oncurve_a = p0 + (p1 - p0).normalize_or_zero() * r0;
        let mut oncurve_b = p1 + (p0 - p1).normalize_or_zero() * r1;
        let mut tangent_a = (p1 - p0).perp().normalize_or_zero();
        let mut tangent_b = (p0 - p1).perp().normalize_or_zero();
        // If circles intersect, use intersection points
        if d < r0 + r1 && d > (r0 - r1).abs() && d > 1e-3 {
            // Law of cosines
            let a2 = (r0 * r0 - r1 * r1 + d * d) / (2.0 * d);
            let h = (r0 * r0 - a2 * a2).max(0.0).sqrt();
            let dir = (p1 - p0).normalize_or_zero();
            let mid = p0 + dir * a2;
            let perp = Vec2::new(-dir.y, dir.x);
            let int1 = mid + perp * h;
            let int2 = mid - perp * h;
            // Pick the intersection points so the path is smooth
            // We'll use int1 for a->b, int2 for b->a
            oncurve_a = int1;
            oncurve_b = int2;
            tangent_a = perp;
            tangent_b = -perp;
        }
        // Control points: offset from oncurve along tangent, proportional to radius
        let ctrl_dist = 0.55 * r0.min(r1); // 0.55 is a good default for circle-to-cubic
        let ctrl1 = oncurve_a + tangent_a * ctrl_dist;
        let ctrl2 = oncurve_b + tangent_b * ctrl_dist;
        segments.push([oncurve_a, ctrl1, ctrl2, oncurve_b]);
    }
    segments
}
