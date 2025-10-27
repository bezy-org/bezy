//! Shapes Tool - Geometric shape drawing tool
//!
//! This tool allows users to draw basic geometric shapes like rectangles,
//! ellipses, and rounded rectangles by clicking and dragging.

#![allow(dead_code)]

use crate::core::config::BezySettings;
use crate::core::state::{AppState, GlyphNavigation};
use crate::editing::selection::events::AppStateChanged;
use crate::rendering::zoom_aware_scaling::CameraResponsiveScale;
use crate::ui::edit_mode_toolbar::{EditTool, ToolRegistry};
use crate::ui::theme::*;
use crate::ui::themes::CurrentTheme;
use crate::utils::embedded_assets::{AssetServerFontExt, EmbeddedFonts};
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use kurbo::Shape;

/// Resource to track if shapes mode is currently active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct ShapesModeActive(pub bool);

/// Component to mark shape preview elements for cleanup
#[derive(Component)]
pub struct ShapePreviewElement;

pub struct ShapesTool;

impl EditTool for ShapesTool {
    fn id(&self) -> crate::ui::edit_mode_toolbar::ToolId {
        "shapes"
    }

    fn name(&self) -> &'static str {
        "Shapes"
    }

    fn icon(&self) -> &'static str {
        "\u{E016}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('r')
    }

    fn default_order(&self) -> i32 {
        30 // After pen, before text
    }

    fn description(&self) -> &'static str {
        "Draw geometric shapes"
    }

    fn update(&self, commands: &mut Commands) {
        // Activate shapes mode
        commands.insert_resource(ShapesModeActive(true));
        debug!("ShapesTool::update() called - activating shapes mode");
    }

    fn on_enter(&self) {
        debug!("✅ SHAPES TOOL: Entered Shapes tool");
    }

    fn on_exit(&self) {
        debug!("❌ SHAPES TOOL: Exited Shapes tool");
    }
}

/// Types of shapes that can be drawn
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Resource)]
pub enum ShapeType {
    #[default]
    Rectangle,
    Oval,
    RoundedRectangle,
}

impl ShapeType {
    /// Get the icon for this shape type
    pub fn get_icon(&self) -> &'static str {
        match self {
            ShapeType::Rectangle => "\u{E018}",        // Rectangle icon
            ShapeType::Oval => "\u{E019}",             // Oval icon
            ShapeType::RoundedRectangle => "\u{E020}", // Rounded Rectangle icon
        }
    }

    /// Get the display name for this shape type
    pub fn get_name(&self) -> &'static str {
        match self {
            ShapeType::Rectangle => "Rectangle",
            ShapeType::Oval => "Oval",
            ShapeType::RoundedRectangle => "Rounded Rectangle",
        }
    }
}

/// Component to mark buttons as part of the shapes submenu
#[derive(Component)]
pub struct ShapeSubMenuButton;

/// Component for individual shape mode buttons
#[derive(Component)]
pub struct ShapeModeButton {
    pub shape_type: ShapeType,
}

/// Resource to track the currently selected shape type
#[derive(Resource, Default)]
pub struct CurrentShapeType(pub ShapeType);

/// Active drawing state for shapes
#[derive(Resource, Default)]
pub struct ActiveShapeDrawing {
    pub is_drawing: bool,
    pub shape_type: ShapeType,
    pub start_position: Option<Vec2>,
    pub current_position: Option<Vec2>,
    pub needs_immediate_regeneration: bool,
}

impl ActiveShapeDrawing {
    /// Get the rectangle from the current drawing state
    pub fn get_rect(&self) -> Option<Rect> {
        if let (Some(start), Some(current)) = (self.start_position, self.current_position) {
            let min_x = start.x.min(current.x);
            let min_y = start.y.min(current.y);
            let max_x = start.x.max(current.x);
            let max_y = start.y.max(current.y);

            Some(Rect {
                min: Vec2::new(min_x, min_y),
                max: Vec2::new(max_x, max_y),
            })
        } else {
            None
        }
    }
}

/// Resource to store the current corner radius for rounded rectangles
#[derive(Resource)]
pub struct CurrentCornerRadius(pub f32);

impl Default for CurrentCornerRadius {
    fn default() -> Self {
        Self(10.0)
    }
}

/// Plugin for the shapes tool
pub struct ShapesToolPlugin;

impl Plugin for ShapesToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShapesModeActive>()
            .init_resource::<CurrentShapeType>()
            .init_resource::<ActiveShapeDrawing>()
            .init_resource::<CurrentCornerRadius>()
            .add_systems(Startup, register_shapes_tool)
            .add_systems(PostStartup, spawn_shapes_submenu)
            .add_systems(
                Update,
                (
                    // Input handling runs first
                    handle_shape_mouse_events
                        .in_set(crate::editing::FontEditorSets::Input),
                    // Force immediate update after shape creation
                    trigger_immediate_shape_update
                        .after(handle_shape_mouse_events)
                        .in_set(crate::editing::FontEditorSets::Input),
                    // Preview rendering runs in rendering phase
                    render_active_shape_drawing_with_dimensions
                        .in_set(crate::editing::FontEditorSets::Rendering),
                    // Mode management runs anytime
                    reset_shapes_mode_when_inactive,
                    toggle_shapes_submenu_visibility,
                    handle_shapes_submenu_selection,
                ),
            );
    }
}

fn register_shapes_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(ShapesTool));
}

/// Handle mouse events for shape drawing
#[allow(clippy::too_many_arguments)]
pub fn handle_shape_mouse_events(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    current_shape_type: Res<CurrentShapeType>,
    mut active_drawing: ResMut<ActiveShapeDrawing>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut app_state_changed: EventWriter<AppStateChanged>,
    mut app_state: Option<ResMut<AppState>>,
    mut fontir_app_state: Option<ResMut<crate::core::state::FontIRAppState>>,
    glyph_navigation: Res<GlyphNavigation>,
    corner_radius: Res<CurrentCornerRadius>,
    shapes_mode: Option<Res<ShapesModeActive>>,
    current_tool: Option<Res<crate::ui::edit_mode_toolbar::CurrentTool>>,
    settings: Res<BezySettings>,
    // Query for active sort to get its position
    active_sort_query: Query<
        (Entity, &crate::editing::sort::Sort, &Transform),
        With<crate::editing::sort::ActiveSort>,
    >,
    mut visual_update_tracker: ResMut<crate::rendering::glyph_renderer::SortVisualUpdateTracker>,
) {
    // Check if shapes mode is active via multiple methods
    let shapes_is_active = shapes_mode.as_ref().is_some_and(|s| s.0)
        || (current_tool.as_ref().and_then(|t| t.get_current()) == Some("shapes"));

    // IMPORTANT: Shapes tool requires an active sort to work (like pen tool)
    let active_sort = if let Ok((sort_entity, sort, sort_transform)) = active_sort_query.single() {
        Some((sort_entity, sort, sort_transform))
    } else {
        None
    };

    // Debug: Always log when this system runs
    debug!(
        "SHAPES INPUT: handle_shape_tool_input called - shapes_is_active: {}, active_sort: {}, current_tool: {:?}",
        shapes_is_active,
        active_sort.is_some(),
        current_tool.as_ref().and_then(|t| t.get_current())
    );

    // Early exit if shapes tool is not active, no active sort, or other conditions
    let Some((_sort_entity, _sort, sort_transform)) = active_sort else {
        if shapes_is_active {
            // Only show this message when shapes tool is actually trying to be used
            if mouse_button_input.just_pressed(MouseButton::Left) {
                debug!("🔳 Shapes tool: Cannot draw without an active sort. Please select a glyph first.");
            }
        }
        debug!("SHAPES INPUT: Shapes not active or no active sort, exiting");
        return;
    };

    if !shapes_is_active {
        debug!("SHAPES INPUT: Shapes not active, exiting");
        return;
    }
    let sort_position = sort_transform.translation.truncate();

    let Ok(window) = windows.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Convert cursor position to world coordinates, then to sort-relative coordinates
    if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
        // Convert to sort-relative coordinates
        let sort_relative_position = world_position - sort_position;

        // Apply grid snapping to sort-relative coordinates
        let mut snapped_position = settings.apply_grid_snap(sort_relative_position);

        // Apply shift-key constraints for squares/circles
        if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight)
        {
            if let Some(start_pos) = active_drawing.start_position {
                snapped_position =
                    apply_shape_constraints(snapped_position, start_pos, current_shape_type.0);
            }
        }

        // Handle mouse button press
        if mouse_button_input.just_pressed(MouseButton::Left) {
            debug!(
                "SHAPES TOOL: Starting to draw {:?} at sort-relative ({:.1}, {:.1}), world ({:.1}, {:.1}), sort pos ({:.1}, {:.1})",
                current_shape_type.0, snapped_position.x, snapped_position.y, world_position.x, world_position.y, sort_position.x, sort_position.y
            );
            active_drawing.is_drawing = true;
            active_drawing.shape_type = current_shape_type.0;
            active_drawing.start_position = Some(snapped_position);
            active_drawing.current_position = Some(snapped_position);
        }

        // Handle mouse movement during drawing
        if active_drawing.is_drawing {
            active_drawing.current_position = Some(snapped_position);
            debug!(
                "SHAPES TOOL: Mouse drag update - sort-relative position: ({:.1}, {:.1})",
                snapped_position.x, snapped_position.y
            );
        }

        // Handle mouse button release
        if mouse_button_input.just_released(MouseButton::Left) && active_drawing.is_drawing {
            if let Some(rect) = active_drawing.get_rect() {
                debug!("SHAPES TOOL: Completing {:?} shape with rect: ({:.1}, {:.1}) to ({:.1}, {:.1})", 
                       active_drawing.shape_type, rect.min.x, rect.min.y, rect.max.x, rect.max.y);

                // Create the shape in the current glyph - try FontIR first, then legacy AppState
                if let Some(fontir_state) = fontir_app_state.as_mut() {
                    create_shape_fontir(
                        rect,
                        active_drawing.shape_type,
                        corner_radius.0,
                        fontir_state,
                        &mut app_state_changed,
                    );
                } else if let Some(state) = app_state.as_mut() {
                    create_shape(
                        rect,
                        active_drawing.shape_type,
                        corner_radius.0,
                        &glyph_navigation,
                        state,
                        &mut app_state_changed,
                    );
                }

                // Send multiple events to ensure all systems are triggered
                // This helps prevent the lag issue where the shape doesn't appear immediately
                app_state_changed.write(AppStateChanged);
                app_state_changed.write(AppStateChanged);

                // CRITICAL: Immediately trigger visual update to prevent lag
                visual_update_tracker.needs_update = true;
                debug!("🔄 SHAPES TOOL: Immediately triggered visual update after shape creation");

                // Mark that we need immediate regeneration
                active_drawing.needs_immediate_regeneration = true;
            }

            // Reset drawing state
            active_drawing.is_drawing = false;
            active_drawing.start_position = None;
            active_drawing.current_position = None;
        }
    }
}

/// Render the shape being drawn with dimensions display using unified mesh-based system
#[allow(clippy::too_many_arguments)]
pub fn render_active_shape_drawing_with_dimensions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    active_drawing: Res<ActiveShapeDrawing>,
    shapes_mode: Option<Res<ShapesModeActive>>,
    current_tool: Option<Res<crate::ui::edit_mode_toolbar::CurrentTool>>,
    camera_scale: Res<CameraResponsiveScale>,
    existing_preview_query: Query<Entity, With<ShapePreviewElement>>,
    theme: Res<CurrentTheme>,
    asset_server: Res<AssetServer>,
    embedded_fonts: Res<EmbeddedFonts>,
    // Query for active sort to get its position for preview rendering
    active_sort_query: Query<
        (Entity, &crate::editing::sort::Sort, &Transform),
        With<crate::editing::sort::ActiveSort>,
    >,
) {
    // Clean up existing preview elements - ALWAYS do this first
    for entity in existing_preview_query.iter() {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }

    // Check if shapes mode is active via multiple methods (same as input handling)
    let shapes_is_active = shapes_mode.as_ref().is_some_and(|s| s.0)
        || (current_tool.as_ref().and_then(|t| t.get_current()) == Some("shapes"));

    // Early exit if shapes tool is not active
    if !shapes_is_active {
        debug!("SHAPES PREVIEW: Shapes mode not active, exiting after cleanup");
        return;
    }

    // Shapes tool requires an active sort to work
    let active_sort = if let Ok((sort_entity, sort, sort_transform)) = active_sort_query.single() {
        Some((sort_entity, sort, sort_transform))
    } else {
        None
    };

    // Debug: Always log when this system runs
    debug!("SHAPES PREVIEW: System running - shapes_mode_active: {:?}, active_sort: {}, current_tool: {:?}, is_drawing: {}",
           shapes_mode.as_ref().map(|s| s.0),
           active_sort.is_some(),
           current_tool.as_ref().and_then(|t| t.get_current()),
           active_drawing.is_drawing);

    // Only render if there's an active sort and we're drawing
    let Some((_sort_entity, _sort, sort_transform)) = active_sort else {
        debug!("SHAPES PREVIEW: No active sort, exiting after cleanup");
        return;
    };

    let sort_position = sort_transform.translation.truncate();

    if !active_drawing.is_drawing {
        debug!("SHAPES PREVIEW: Not drawing, exiting after cleanup");
        return;
    }

    if let Some(rect) = active_drawing.get_rect() {
        // Convert sort-relative rect to world coordinates for rendering
        let world_rect = Rect {
            min: rect.min + sort_position,
            max: rect.max + sort_position,
        };

        debug!("SHAPES PREVIEW: Drawing preview! Sort-relative rect: ({:.1}, {:.1}) to ({:.1}, {:.1}), world rect: ({:.1}, {:.1}) to ({:.1}, {:.1}), shape_type: {:?}", 
              rect.min.x, rect.min.y, rect.max.x, rect.max.y,
              world_rect.min.x, world_rect.min.y, world_rect.max.x, world_rect.max.y,
              active_drawing.shape_type);

        let preview_color = theme.theme().action_color(); // Orange action color like pen tool
        let line_width = camera_scale.adjusted_line_width() * 2.0;

        match active_drawing.shape_type {
            ShapeType::Rectangle | ShapeType::RoundedRectangle => {
                debug!("SHAPES PREVIEW: Drawing rectangle preview");
                draw_mesh_dashed_rectangle(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    world_rect, // Use world coordinates for rendering
                    preview_color,
                    line_width,
                );
            }
            ShapeType::Oval => {
                debug!("SHAPES PREVIEW: Drawing oval preview");
                draw_mesh_dashed_ellipse(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    world_rect, // Use world coordinates for rendering
                    preview_color,
                    line_width,
                );
            }
        }

        // Draw dimensions (width x height) similar to Glyphs app
        debug!("SHAPES PREVIEW: Drawing dimension lines");
        spawn_shape_dimension_lines(
            &mut commands,
            &mut meshes,
            &mut materials,
            world_rect, // Use world coordinates for rendering
            &camera_scale,
            &theme,
            &asset_server,
            &embedded_fonts,
        );
    } else {
        debug!("SHAPES PREVIEW: No rect available from active_drawing.get_rect()");
    }
}

/// System to trigger immediate update when shape is created
pub fn trigger_immediate_shape_update(
    mut active_drawing: ResMut<ActiveShapeDrawing>,
    mut app_state_changed: EventWriter<AppStateChanged>,
    mut visual_update_tracker: ResMut<crate::rendering::glyph_renderer::SortVisualUpdateTracker>,
) {
    if active_drawing.needs_immediate_regeneration {
        // Send more events to ensure all systems process them
        for _ in 0..3 {
            app_state_changed.write(AppStateChanged);
        }
        visual_update_tracker.needs_update = true;
        active_drawing.needs_immediate_regeneration = false;
        debug!("🚀 SHAPES TOOL: Forced immediate update pipeline for shape rendering");
    }
}

/// Reset shapes mode when another tool is selected
pub fn reset_shapes_mode_when_inactive(
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
    mut active_drawing: ResMut<ActiveShapeDrawing>,
) {
    if current_tool.get_current() != Some("shapes") {
        // Cancel any active drawing
        if active_drawing.is_drawing {
            debug!("SHAPES TOOL: Cancelling active drawing - switching tools");
            active_drawing.is_drawing = false;
            active_drawing.start_position = None;
            active_drawing.current_position = None;
        }

        // Mark shapes mode as inactive
        commands.insert_resource(ShapesModeActive(false));
    }
}

/// Create a shape in the current glyph
fn create_shape(
    rect: Rect,
    shape_type: ShapeType,
    corner_radius: f32,
    glyph_navigation: &GlyphNavigation,
    app_state: &mut AppState,
    app_state_changed: &mut EventWriter<AppStateChanged>,
) {
    let Some(glyph_name) = glyph_navigation.find_glyph(app_state) else {
        warn!("No current glyph selected for shape creation");
        return;
    };

    // Create contour points based on shape type
    let points = match shape_type {
        ShapeType::Rectangle => create_rectangle_points(rect),
        ShapeType::Oval => create_ellipse_points(rect),
        ShapeType::RoundedRectangle => create_rounded_rectangle_points(rect, corner_radius),
    };

    // Add the contour to the glyph
    if let Some(glyph_data) = app_state.workspace.font.glyphs.get_mut(&glyph_name) {
        if glyph_data.outline.is_none() {
            glyph_data.outline = Some(crate::core::state::OutlineData {
                contours: Vec::new(),
            });
        }

        if let Some(outline) = &mut glyph_data.outline {
            outline
                .contours
                .push(crate::core::state::ContourData { points });

            debug!(
                "Created {} shape in glyph '{}'",
                match shape_type {
                    ShapeType::Rectangle => "rectangle",
                    ShapeType::Oval => "oval",
                    ShapeType::RoundedRectangle => "rounded rectangle",
                },
                glyph_name
            );

            app_state_changed.write(AppStateChanged);
        }
    }
}

/// Create shape using FontIR (preferred method)
fn create_shape_fontir(
    rect: Rect,
    shape_type: ShapeType,
    corner_radius: f32,
    fontir_app_state: &mut crate::core::state::FontIRAppState,
    app_state_changed: &mut EventWriter<AppStateChanged>,
) {
    let Some(current_glyph_name) = fontir_app_state.current_glyph.clone() else {
        warn!("No current glyph selected for FontIR shape creation");
        return;
    };

    // Create BezPath from shape type using Kurbo primitives
    let bez_path = match shape_type {
        ShapeType::Rectangle => create_rectangle_bezpath(rect),
        ShapeType::Oval => create_ellipse_bezpath(rect),
        ShapeType::RoundedRectangle => create_rounded_rectangle_bezpath(rect, corner_radius),
    };

    // Get or create a working copy using the proper method (like pen tool)
    if let Some(working_copy) = fontir_app_state.get_or_create_working_copy(&current_glyph_name) {
        working_copy.contours.push(bez_path.clone());
        working_copy.is_dirty = true;
        app_state_changed.write(AppStateChanged);

        debug!(
            "Created {} shape with FontIR in glyph '{}'. Total contours: {}",
            match shape_type {
                ShapeType::Rectangle => "rectangle",
                ShapeType::Oval => "oval",
                ShapeType::RoundedRectangle => "rounded rectangle",
            },
            current_glyph_name,
            working_copy.contours.len()
        );
    } else {
        warn!(
            "Could not create working copy for FontIR shape in glyph '{}'",
            current_glyph_name
        );
    }
}

/// Create points for a rectangle
fn create_rectangle_points(rect: Rect) -> Vec<crate::core::state::PointData> {
    vec![
        crate::core::state::PointData {
            x: rect.min.x as f64,
            y: rect.min.y as f64,
            point_type: crate::core::state::PointTypeData::Move,
        },
        crate::core::state::PointData {
            x: rect.max.x as f64,
            y: rect.min.y as f64,
            point_type: crate::core::state::PointTypeData::Line,
        },
        crate::core::state::PointData {
            x: rect.max.x as f64,
            y: rect.max.y as f64,
            point_type: crate::core::state::PointTypeData::Line,
        },
        crate::core::state::PointData {
            x: rect.min.x as f64,
            y: rect.max.y as f64,
            point_type: crate::core::state::PointTypeData::Line,
        },
    ]
}

/// Create points for an ellipse using proper Kurbo curves
fn create_ellipse_points(rect: Rect) -> Vec<crate::core::state::PointData> {
    let center_x = (rect.min.x + rect.max.x) / 2.0;
    let center_y = (rect.min.y + rect.max.y) / 2.0;
    let radius_x = (rect.max.x - rect.min.x) / 2.0;
    let radius_y = (rect.max.y - rect.min.y) / 2.0;

    // Create a Kurbo ellipse and convert to BezPath
    let center = kurbo::Point::new(center_x as f64, center_y as f64);
    let radii = kurbo::Vec2::new(radius_x as f64, radius_y as f64);
    let ellipse = kurbo::Ellipse::new(center, radii, 0.0); // No rotation

    // Convert to BezPath to get the actual Bézier curves
    let bez_path = ellipse.to_path(1e-3); // Tolerance for conversion

    // Convert BezPath elements to PointData
    let mut points = Vec::new();

    for element in bez_path.elements() {
        match element {
            kurbo::PathEl::MoveTo(pt) => {
                points.push(crate::core::state::PointData {
                    x: pt.x,
                    y: pt.y,
                    point_type: crate::core::state::PointTypeData::Move,
                });
            }
            kurbo::PathEl::LineTo(pt) => {
                points.push(crate::core::state::PointData {
                    x: pt.x,
                    y: pt.y,
                    point_type: crate::core::state::PointTypeData::Line,
                });
            }
            kurbo::PathEl::CurveTo(pt1, pt2, pt3) => {
                // Add the first control point as off-curve
                points.push(crate::core::state::PointData {
                    x: pt1.x,
                    y: pt1.y,
                    point_type: crate::core::state::PointTypeData::OffCurve,
                });
                // Add the second control point as off-curve
                points.push(crate::core::state::PointData {
                    x: pt2.x,
                    y: pt2.y,
                    point_type: crate::core::state::PointTypeData::OffCurve,
                });
                // Add the end point as on-curve
                points.push(crate::core::state::PointData {
                    x: pt3.x,
                    y: pt3.y,
                    point_type: crate::core::state::PointTypeData::Curve,
                });
            }
            kurbo::PathEl::QuadTo(pt1, pt2) => {
                // Convert quadratic to cubic for consistency
                // Add control point as off-curve
                points.push(crate::core::state::PointData {
                    x: pt1.x,
                    y: pt1.y,
                    point_type: crate::core::state::PointTypeData::OffCurve,
                });
                // Add end point as on-curve
                points.push(crate::core::state::PointData {
                    x: pt2.x,
                    y: pt2.y,
                    point_type: crate::core::state::PointTypeData::Curve,
                });
            }
            kurbo::PathEl::ClosePath => {
                // Already handled by the ellipse generation
            }
        }
    }

    points
}

/// Create points for a rounded rectangle using Kurbo
fn create_rounded_rectangle_points(rect: Rect, radius: f32) -> Vec<crate::core::state::PointData> {
    // Create rounded rectangle with the specified radius
    let rounded_rect = kurbo::RoundedRect::new(
        rect.min.x as f64,
        rect.min.y as f64,
        rect.max.x as f64,
        rect.max.y as f64,
        radius as f64,
    );

    // Convert to BezPath to get the actual Bézier curves
    let bez_path = rounded_rect.to_path(1e-3); // Tolerance for conversion

    // Convert BezPath elements to PointData
    let mut points = Vec::new();

    for element in bez_path.elements() {
        match element {
            kurbo::PathEl::MoveTo(pt) => {
                points.push(crate::core::state::PointData {
                    x: pt.x,
                    y: pt.y,
                    point_type: crate::core::state::PointTypeData::Move,
                });
            }
            kurbo::PathEl::LineTo(pt) => {
                points.push(crate::core::state::PointData {
                    x: pt.x,
                    y: pt.y,
                    point_type: crate::core::state::PointTypeData::Line,
                });
            }
            kurbo::PathEl::CurveTo(pt1, pt2, pt3) => {
                // Add the first control point as off-curve
                points.push(crate::core::state::PointData {
                    x: pt1.x,
                    y: pt1.y,
                    point_type: crate::core::state::PointTypeData::OffCurve,
                });
                // Add the second control point as off-curve
                points.push(crate::core::state::PointData {
                    x: pt2.x,
                    y: pt2.y,
                    point_type: crate::core::state::PointTypeData::OffCurve,
                });
                // Add the end point as on-curve
                points.push(crate::core::state::PointData {
                    x: pt3.x,
                    y: pt3.y,
                    point_type: crate::core::state::PointTypeData::Curve,
                });
            }
            kurbo::PathEl::QuadTo(pt1, pt2) => {
                // Convert quadratic to cubic for consistency
                // Add control point as off-curve
                points.push(crate::core::state::PointData {
                    x: pt1.x,
                    y: pt1.y,
                    point_type: crate::core::state::PointTypeData::OffCurve,
                });
                // Add end point as on-curve
                points.push(crate::core::state::PointData {
                    x: pt2.x,
                    y: pt2.y,
                    point_type: crate::core::state::PointTypeData::Curve,
                });
            }
            kurbo::PathEl::ClosePath => {
                // Already handled by the shape generation
            }
        }
    }

    points
}

// ================================================================
// BEZPATH CREATION FUNCTIONS (for FontIR integration)
// ================================================================

/// Create BezPath for rectangle using Kurbo
fn create_rectangle_bezpath(rect: Rect) -> kurbo::BezPath {
    kurbo::Rect::new(
        rect.min.x as f64,
        rect.min.y as f64,
        rect.max.x as f64,
        rect.max.y as f64,
    )
    .to_path(1e-3)
}

/// Create BezPath for ellipse using Kurbo
fn create_ellipse_bezpath(rect: Rect) -> kurbo::BezPath {
    let center_x = (rect.min.x + rect.max.x) / 2.0;
    let center_y = (rect.min.y + rect.max.y) / 2.0;
    let radius_x = (rect.max.x - rect.min.x) / 2.0;
    let radius_y = (rect.max.y - rect.min.y) / 2.0;

    let center = kurbo::Point::new(center_x as f64, center_y as f64);
    let radii = kurbo::Vec2::new(radius_x as f64, radius_y as f64);
    let ellipse = kurbo::Ellipse::new(center, radii, 0.0);
    ellipse.to_path(1e-3)
}

/// Create BezPath for rounded rectangle using Kurbo  
fn create_rounded_rectangle_bezpath(rect: Rect, corner_radius: f32) -> kurbo::BezPath {
    kurbo::RoundedRect::new(
        rect.min.x as f64,
        rect.min.y as f64,
        rect.max.x as f64,
        rect.max.y as f64,
        corner_radius as f64,
    )
    .to_path(1e-3)
}

// ================================================================
// MESH-BASED PREVIEW SYSTEM (Replaces Gizmos)
// ================================================================

/// Spawn a mesh-based dashed line for shape preview
fn spawn_shape_preview_dashed_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    color: Color,
    width: f32,
) {
    let dash_length = 8.0;
    let gap_length = 4.0;

    let direction = (end - start).normalize();
    let total_length = start.distance(end);
    let segment_length = dash_length + gap_length;

    let mut current_pos = 0.0;

    while current_pos < total_length {
        let dash_start = start + direction * current_pos;
        let dash_end_pos = (current_pos + dash_length).min(total_length);
        let dash_end = start + direction * dash_end_pos;

        // Create line mesh for this dash segment
        let line_mesh = crate::rendering::mesh_utils::create_line_mesh(dash_start, dash_end, width);

        // Calculate midpoint for proper positioning
        let midpoint = (dash_start + dash_end) * 0.5;

        commands.spawn((
            Mesh2d(meshes.add(line_mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from(color))),
            Transform::from_translation(Vec3::new(midpoint.x, midpoint.y, 10.0)), // Position at midpoint
            ShapePreviewElement,
        ));

        current_pos += segment_length;
    }
}

/// Spawn mesh-based dimension lines with camera-responsive scaling
#[allow(clippy::too_many_arguments)]
fn spawn_shape_dimension_lines(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    rect: Rect,
    camera_scale: &CameraResponsiveScale,
    theme: &CurrentTheme,
    asset_server: &AssetServer,
    embedded_fonts: &EmbeddedFonts,
) {
    let width = (rect.max.x - rect.min.x).abs();
    let height = (rect.max.y - rect.min.y).abs();

    let dimension_color = theme.theme().action_color(); // Use orange action color
    let line_width = camera_scale.adjusted_line_width() * 1.0;

    // Width dimension (horizontal line below shape)
    let width_y = rect.min.y - 20.0;
    let width_start = Vec2::new(rect.min.x, width_y);
    let width_end = Vec2::new(rect.max.x, width_y);

    let width_line_mesh =
        crate::rendering::mesh_utils::create_line_mesh(width_start, width_end, line_width);

    // Calculate midpoint for width line positioning
    let width_midpoint = (width_start + width_end) * 0.5;

    commands.spawn((
        Mesh2d(meshes.add(width_line_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from(dimension_color))),
        Transform::from_translation(Vec3::new(width_midpoint.x, width_midpoint.y, 11.0)), // Position at midpoint
        ShapePreviewElement,
    ));

    // Add arrows at width line ends
    spawn_arrow(
        commands,
        meshes,
        materials,
        width_start,
        Vec2::new(-1.0, 0.0),
        dimension_color,
        camera_scale,
    );
    spawn_arrow(
        commands,
        meshes,
        materials,
        width_end,
        Vec2::new(1.0, 0.0),
        dimension_color,
        camera_scale,
    );

    // Width measurement text
    commands.spawn((
        Text2d(format!("{width:.0}")),
        TextFont {
            font: asset_server
                .load_font_with_fallback(theme.theme().mono_font_path(), embedded_fonts),
            font_size: 14.0,
            ..default()
        },
        TextColor(dimension_color),
        bevy::sprite::Anchor::Center,
        bevy::text::TextBounds::UNBOUNDED,
        Transform::from_translation(Vec3::new(width_midpoint.x, width_y - 12.0, 12.0)),
        ShapePreviewElement,
    ));

    // Height dimension (vertical line to the right of shape)
    let height_x = rect.max.x + 20.0;
    let height_start = Vec2::new(height_x, rect.min.y);
    let height_end = Vec2::new(height_x, rect.max.y);

    let height_line_mesh =
        crate::rendering::mesh_utils::create_line_mesh(height_start, height_end, line_width);

    // Calculate midpoint for height line positioning
    let height_midpoint = (height_start + height_end) * 0.5;

    commands.spawn((
        Mesh2d(meshes.add(height_line_mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from(dimension_color))),
        Transform::from_translation(Vec3::new(height_midpoint.x, height_midpoint.y, 11.0)), // Position at midpoint
        ShapePreviewElement,
    ));

    // Add arrows at height line ends
    spawn_arrow(
        commands,
        meshes,
        materials,
        height_start,
        Vec2::new(0.0, -1.0),
        dimension_color,
        camera_scale,
    );
    spawn_arrow(
        commands,
        meshes,
        materials,
        height_end,
        Vec2::new(0.0, 1.0),
        dimension_color,
        camera_scale,
    );

    // Height measurement text
    commands.spawn((
        Text2d(format!("{height:.0}")),
        TextFont {
            font: asset_server
                .load_font_with_fallback(theme.theme().mono_font_path(), embedded_fonts),
            font_size: 14.0,
            ..default()
        },
        TextColor(dimension_color),
        bevy::sprite::Anchor::Center,
        bevy::text::TextBounds::UNBOUNDED,
        Transform::from_translation(Vec3::new(height_x + 12.0, height_midpoint.y, 12.0)),
        ShapePreviewElement,
    ));
}

/// Spawn a small arrow at the given position pointing in the given direction
fn spawn_arrow(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec2,
    direction: Vec2,
    color: Color,
    camera_scale: &CameraResponsiveScale,
) {
    let arrow_size = camera_scale.adjusted_line_width() * 5.0;
    let direction = direction.normalize();

    // Create arrow vertices (triangle pointing in direction)
    let perpendicular = Vec2::new(-direction.y, direction.x);
    let tip = position + direction * arrow_size;
    let base_left = position - direction * arrow_size * 0.5 + perpendicular * arrow_size * 0.5;
    let base_right = position - direction * arrow_size * 0.5 - perpendicular * arrow_size * 0.5;

    // Create triangle mesh
    let vertices = vec![
        [tip.x, tip.y, 0.0],
        [base_left.x, base_left.y, 0.0],
        [base_right.x, base_right.y, 0.0],
    ];

    let indices = vec![0, 1, 2];
    let normals = vec![[0.0, 0.0, 1.0]; 3];
    let uvs = vec![[0.5, 1.0], [0.0, 0.0], [1.0, 0.0]];

    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    commands.spawn((
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from(color))),
        Transform::from_translation(Vec3::new(0.0, 0.0, 11.0)), // Same z as lines
        ShapePreviewElement,
    ));
}

/// Draw a mesh-based dashed rectangle preview
fn draw_mesh_dashed_rectangle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    rect: Rect,
    color: Color,
    width: f32,
) {
    let corners = [
        Vec2::new(rect.min.x, rect.min.y),
        Vec2::new(rect.max.x, rect.min.y),
        Vec2::new(rect.max.x, rect.max.y),
        Vec2::new(rect.min.x, rect.max.y),
    ];

    for i in 0..4 {
        let start = corners[i];
        let end = corners[(i + 1) % 4];
        spawn_shape_preview_dashed_line(commands, meshes, materials, start, end, color, width);
    }
}

/// Draw a mesh-based dashed ellipse preview
fn draw_mesh_dashed_ellipse(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    rect: Rect,
    color: Color,
    width: f32,
) {
    let center_x = (rect.min.x + rect.max.x) / 2.0;
    let center_y = (rect.min.y + rect.max.y) / 2.0;
    let radius_x = (rect.max.x - rect.min.x) / 2.0;
    let radius_y = (rect.max.y - rect.min.y) / 2.0;

    let mut points = Vec::new();

    // Create 32 points for smooth ellipse preview (more than the 16 in old version)
    for i in 0..32 {
        let angle = (i as f32) * std::f32::consts::PI * 2.0 / 32.0;
        let x = center_x + radius_x * angle.cos();
        let y = center_y + radius_y * angle.sin();
        points.push(Vec2::new(x, y));
    }

    for i in 0..32 {
        let start = points[i];
        let end = points[(i + 1) % 32];
        spawn_shape_preview_dashed_line(commands, meshes, materials, start, end, color, width);
    }
}

/// Apply shape constraints when shift is held (square, circle, rounded square)
fn apply_shape_constraints(cursor_pos: Vec2, start_pos: Vec2, shape_type: ShapeType) -> Vec2 {
    let delta = cursor_pos - start_pos;

    match shape_type {
        ShapeType::Rectangle | ShapeType::RoundedRectangle => {
            // For rectangles, make it a square by using the larger dimension
            let size = delta.x.abs().max(delta.y.abs());
            let sign_x = if delta.x >= 0.0 { 1.0 } else { -1.0 };
            let sign_y = if delta.y >= 0.0 { 1.0 } else { -1.0 };
            start_pos + Vec2::new(size * sign_x, size * sign_y)
        }
        ShapeType::Oval => {
            // For ovals, make it a circle by using the larger dimension
            let size = delta.x.abs().max(delta.y.abs());
            let sign_x = if delta.x >= 0.0 { 1.0 } else { -1.0 };
            let sign_y = if delta.y >= 0.0 { 1.0 } else { -1.0 };
            start_pos + Vec2::new(size * sign_x, size * sign_y)
        }
    }
}

// ================================================================
// SHAPES SUBMENU SYSTEMS (following pen tool pattern)
// ================================================================

/// Helper function to spawn a single shape mode button using the unified system
fn spawn_shape_mode_button(
    parent: &mut ChildSpawnerCommands,
    shape_type: ShapeType,
    asset_server: &Res<AssetServer>,
    embedded_fonts: &Res<EmbeddedFonts>,
    theme: &Res<CurrentTheme>,
) {
    // Use the unified toolbar button creation system for consistent styling with hover text
    crate::ui::edit_mode_toolbar::ui::create_toolbar_button_with_hover_text(
        parent,
        shape_type.get_icon(),
        Some(shape_type.get_name()), // Show the shape name on hover
        (ShapeSubMenuButton, ShapeModeButton { shape_type }),
        asset_server,
        embedded_fonts,
        theme,
    );
}

pub fn spawn_shapes_submenu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    embedded_fonts: Res<EmbeddedFonts>,
    theme: Res<CurrentTheme>,
) {
    debug!("🔳 Spawning shapes submenu with Rectangle, Oval, and Rounded Rectangle");
    debug!("🔳 Default shape type is: {:?}", ShapeType::default());

    let shapes = [
        ShapeType::Rectangle,
        ShapeType::Oval,
        ShapeType::RoundedRectangle,
    ];

    // Create the parent submenu node (left-aligned to match main toolbar)
    // Position below main toolbar using consistent helper
    let submenu_node = Node {
        position_type: PositionType::Absolute,
        top: Val::Px(toolbar_submenu_top_position()),
        left: Val::Px(TOOLBAR_CONTAINER_MARGIN), // Left-aligned to match toolbar
        flex_direction: FlexDirection::Row,
        padding: UiRect::all(Val::Px(TOOLBAR_PADDING)),
        margin: UiRect::all(Val::ZERO),
        row_gap: Val::Px(TOOLBAR_PADDING),
        display: Display::None, // Hidden by default
        ..default()
    };

    // Spawn the submenu with all buttons
    commands
        .spawn((submenu_node, Name::new("ShapesSubMenu")))
        .with_children(|parent| {
            for shape_type in shapes {
                spawn_shape_mode_button(parent, shape_type, &asset_server, &embedded_fonts, &theme);
            }
        });

    debug!("🔳 Shapes submenu spawned successfully");
}

/// Auto-show shapes submenu when shapes tool is active (like pen tool)
pub fn toggle_shapes_submenu_visibility(
    current_tool: Option<Res<crate::ui::edit_mode_toolbar::CurrentTool>>,
    mut submenu_query: Query<(&mut Node, &Name)>,
) {
    let is_shapes_tool_active =
        current_tool.as_ref().and_then(|tool| tool.get_current()) == Some("shapes");

    for (mut node, name) in submenu_query.iter_mut() {
        if name.as_str() == "ShapesSubMenu" {
            let new_display = if is_shapes_tool_active {
                Display::Flex
            } else {
                Display::None
            };

            if node.display != new_display {
                node.display = new_display;
                debug!(
                    "🔳 Shapes submenu visibility changed: tool_active={}, display={:?}",
                    is_shapes_tool_active, new_display
                );
            }
        }
    }
}

/// Handle shapes submenu selection and visual feedback (following pen tool pattern)
pub fn handle_shapes_submenu_selection(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &ShapeModeButton,
            Entity,
        ),
        With<ShapeSubMenuButton>,
    >,
    mut current_shape_type: ResMut<CurrentShapeType>,
    children_query: Query<&Children>,
    mut text_query: Query<&mut TextColor>,
    theme: Res<CurrentTheme>,
) {
    // Debug: Log if we find any submenu buttons
    let button_count = interaction_query.iter().len();
    if button_count > 0 {
        static mut LAST_LOG: f32 = 0.0;
        unsafe {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f32();
            if current_time - LAST_LOG > 2.0 {
                LAST_LOG = current_time;
                debug!(
                    "🔳 Shapes submenu selection system: found {} buttons",
                    button_count
                );
            }
        }
    }

    for (interaction, mut color, mut border_color, shape_button, entity) in &mut interaction_query {
        let is_current_shape = current_shape_type.0 == shape_button.shape_type;

        // Debug: Log interactions for debugging
        if *interaction != Interaction::None {
            debug!(
                "🔳 Button interaction: {:?} for shape {:?} (current: {:?})",
                interaction, shape_button.shape_type, current_shape_type.0
            );
        }

        if *interaction == Interaction::Pressed && !is_current_shape {
            current_shape_type.0 = shape_button.shape_type;
            debug!("🔳 Switched to shape type: {:?}", shape_button.shape_type);
        }

        // Use the unified button color system for consistent appearance with main toolbar
        crate::ui::edit_mode_toolbar::ui::update_toolbar_button_colors(
            *interaction,
            is_current_shape,
            &mut color,
            &mut border_color,
            &theme,
        );

        // Use the unified text color system for consistent icon colors with main toolbar
        crate::ui::edit_mode_toolbar::ui::update_toolbar_button_text_colors(
            entity,
            is_current_shape,
            &children_query,
            &mut text_query,
            &theme,
        );
    }
}
