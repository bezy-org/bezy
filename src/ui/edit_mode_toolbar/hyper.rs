//! Hyper Tool - Advanced curve drawing tool
//!
//! This tool allows users to draw smooth hyperbezier curves with automatic
//! control point calculation for smooth interpolation between points.

use crate::core::config::BezySettings;
use crate::core::state::{AppState, GlyphNavigation};
use crate::editing::selection::systems::AppStateChanged;
use crate::ui::edit_mode_toolbar::{EditTool, ToolRegistry};
use bevy::prelude::*;

pub struct HyperTool;

impl EditTool for HyperTool {
    fn id(&self) -> crate::ui::edit_mode_toolbar::ToolId {
        "hyper"
    }

    fn name(&self) -> &'static str {
        "Hyper"
    }

    fn icon(&self) -> &'static str {
        "\u{E012}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('h')
    }

    fn default_order(&self) -> i32 {
        100 // Advanced tool, later in toolbar
    }

    fn description(&self) -> &'static str {
        "Draw smooth hyperbezier curves"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(HyperModeActive(true));
    }
}

// Use HyperModeActive from tools::hyper and re-export it
pub use crate::tools::hyper::HyperModeActive;

/// The state of the hyper tool
#[derive(Resource, Default)]
pub struct HyperToolState {
    /// Whether currently drawing a path
    pub is_drawing: bool,
    /// Points already placed in the current path
    pub points: Vec<Vec2>,
    /// Whether each point is smooth or a corner
    pub is_smooth: Vec<bool>,
    /// The current cursor position
    pub cursor_position: Option<Vec2>,
    /// How close to the start point to close the path
    pub close_path_threshold: f32,
}

impl HyperToolState {
    pub fn new() -> Self {
        Self {
            is_drawing: false,
            points: Vec::new(),
            is_smooth: Vec::new(),
            cursor_position: None,
            close_path_threshold: 16.0,
        }
    }

    /// Check if we should close the path based on cursor proximity to start
    pub fn should_close_path(&self) -> bool {
        if self.points.len() < 3 {
            return false;
        }

        if let Some(cursor_pos) = self.cursor_position {
            if let Some(first_point) = self.points.first() {
                let distance = cursor_pos.distance(*first_point);
                return distance <= self.close_path_threshold;
            }
        }

        false
    }
}

/// Plugin for the hyper tool
pub struct HyperToolPlugin;

impl Plugin for HyperToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HyperModeActive>()
            .init_resource::<HyperToolState>()
            .add_systems(Startup, register_hyper_tool)
            .add_systems(
                Update,
                (
                    handle_hyper_mouse_events,
                    render_hyper_preview,
                    handle_hyper_keyboard_events,
                    reset_hyper_mode_when_inactive,
                ),
            );
    }
}

fn register_hyper_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(HyperTool));
}

/// Handle mouse events for the hyper tool
#[allow(clippy::too_many_arguments)]
pub fn handle_hyper_mouse_events(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut hyper_state: ResMut<HyperToolState>,
    hyper_mode: Option<Res<HyperModeActive>>,
    glyph_navigation: Res<GlyphNavigation>,
    mut app_state: ResMut<AppState>,
    mut app_state_changed: EventWriter<AppStateChanged>,
) {
    // Only handle events when in hyper mode
    if let Some(hyper_mode) = hyper_mode {
        if !hyper_mode.0 {
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

        // Update cursor position for preview
        hyper_state.cursor_position = Some(snapped_position);

        // Handle left mouse button press
        if mouse_button_input.just_pressed(MouseButton::Left) {
            let is_smooth =
                !keyboard.pressed(KeyCode::AltLeft) && !keyboard.pressed(KeyCode::AltRight);

            // Check if we should close the path
            if hyper_state.should_close_path() && hyper_state.is_drawing {
                // Close the path
                if hyper_state.points.len() >= 3 {
                    create_hyper_contour(
                        &hyper_state.points,
                        &hyper_state.is_smooth,
                        true, // closed path
                        &glyph_navigation,
                        &mut app_state,
                        &mut app_state_changed,
                    );
                }

                // Reset state
                hyper_state.is_drawing = false;
                hyper_state.points.clear();
                hyper_state.is_smooth.clear();
            } else {
                // Add a new point
                hyper_state.points.push(snapped_position);
                hyper_state.is_smooth.push(is_smooth);
                hyper_state.is_drawing = true;
            }
        }

        // Handle right mouse button press (finish path)
        if mouse_button_input.just_pressed(MouseButton::Right) && hyper_state.is_drawing {
            if hyper_state.points.len() >= 2 {
                create_hyper_contour(
                    &hyper_state.points,
                    &hyper_state.is_smooth,
                    false, // open path
                    &glyph_navigation,
                    &mut app_state,
                    &mut app_state_changed,
                );
            }

            // Reset state
            hyper_state.is_drawing = false;
            hyper_state.points.clear();
            hyper_state.is_smooth.clear();
        }
    }
}

/// Handle keyboard events for the hyper tool
pub fn handle_hyper_keyboard_events(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut hyper_state: ResMut<HyperToolState>,
    hyper_mode: Option<Res<HyperModeActive>>,
) {
    // Only handle events when in hyper mode
    if let Some(hyper_mode) = hyper_mode {
        if !hyper_mode.0 {
            return;
        }
    } else {
        return;
    }

    // Handle Escape key to cancel current path
    if keyboard.just_pressed(KeyCode::Escape) && hyper_state.is_drawing {
        hyper_state.is_drawing = false;
        hyper_state.points.clear();
        hyper_state.is_smooth.clear();
        debug!("Cancelled hyperbezier path");
    }
}

/// Reset hyper mode when tool is inactive
pub fn reset_hyper_mode_when_inactive(
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
    mut hyper_state: ResMut<HyperToolState>,
) {
    if current_tool.get_current() != Some("hyper") {
        // Reset state and mark inactive
        *hyper_state = HyperToolState::new();
        commands.insert_resource(HyperModeActive(false));
    }
}

/// Render the hyper tool preview
pub fn render_hyper_preview(
    mut gizmos: Gizmos,
    hyper_state: Res<HyperToolState>,
    hyper_mode: Option<Res<HyperModeActive>>,
) {
    // Only render when in hyper mode
    if let Some(hyper_mode) = hyper_mode {
        if !hyper_mode.0 {
            return;
        }
    } else {
        return;
    }

    if !hyper_state.is_drawing {
        return;
    }

    let point_color = Color::srgba(0.3, 1.0, 0.5, 1.0);
    let line_color = Color::srgba(0.5, 0.8, 1.0, 0.8);
    let close_indicator_color = Color::srgba(1.0, 1.0, 0.0, 1.0);

    // Draw existing points
    for (i, &point) in hyper_state.points.iter().enumerate() {
        let radius = if *hyper_state.is_smooth.get(i).unwrap_or(&true) {
            4.0
        } else {
            3.0
        };
        gizmos.circle_2d(point, radius, point_color);
    }

    // Draw lines between points
    for i in 0..hyper_state.points.len().saturating_sub(1) {
        gizmos.line_2d(hyper_state.points[i], hyper_state.points[i + 1], line_color);
    }

    // Draw preview line to cursor
    if let Some(cursor_pos) = hyper_state.cursor_position {
        if let Some(&last_point) = hyper_state.points.last() {
            gizmos.line_2d(last_point, cursor_pos, line_color);
        }

        // Draw close indicator if near start point
        if hyper_state.should_close_path() {
            if let Some(&first_point) = hyper_state.points.first() {
                gizmos.circle_2d(first_point, 8.0, close_indicator_color);
            }
        }
    }
}

/// Create a hyperbezier contour from points
fn create_hyper_contour(
    points: &[Vec2],
    is_smooth: &[bool],
    closed: bool,
    glyph_navigation: &GlyphNavigation,
    app_state: &mut AppState,
    app_state_changed: &mut EventWriter<AppStateChanged>,
) {
    let Some(glyph_name) = glyph_navigation.find_glyph(app_state) else {
        warn!("No current glyph selected for hyperbezier creation");
        return;
    };

    if points.len() < 2 {
        return;
    }

    // Create contour points with smooth curves
    let mut contour_points = Vec::new();

    for (i, &point) in points.iter().enumerate() {
        let point_type = if i == 0 {
            crate::core::state::PointTypeData::Move
        } else if *is_smooth.get(i).unwrap_or(&true) {
            crate::core::state::PointTypeData::Curve
        } else {
            crate::core::state::PointTypeData::Line
        };

        contour_points.push(crate::core::state::PointData {
            x: point.x as f64,
            y: point.y as f64,
            point_type,
        });
    }

    // If closed, add a line back to start if needed
    if closed && points.len() > 2 {
        // The path will automatically close in UFO format
    }

    // Add the contour to the glyph
    if let Some(glyph_data) = app_state.workspace.font.glyphs.get_mut(&glyph_name) {
        if glyph_data.outline.is_none() {
            glyph_data.outline = Some(crate::core::state::OutlineData {
                contours: Vec::new(),
            });
        }

        if let Some(outline) = &mut glyph_data.outline {
            outline.contours.push(crate::core::state::ContourData {
                points: contour_points,
            });

            debug!(
                "Created hyperbezier {} contour with {} points in glyph '{}'",
                if closed { "closed" } else { "open" },
                points.len(),
                glyph_name
            );

            app_state_changed.write(AppStateChanged);
        }
    }
}
