//! Shapes tool for creating geometric primitives
//!
//! The shapes tool allows users to create common geometric shapes
//! like rectangles, circles, and polygons.

use super::{EditTool, ToolInfo};
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};

/// Resource to track if shapes mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct ShapesModeActive(pub bool);

/// Types of shapes that can be drawn
#[derive(Debug, Clone, Copy, Default, PartialEq, Resource)]
pub enum ShapeType {
    #[default]
    Rectangle,
    Oval,
    RoundedRectangle,
}

/// Resource to track the currently selected shape type
#[derive(Resource, Default)]
pub struct CurrentShapeType(pub ShapeType);

/// State for shape drawing
#[derive(Resource, Default)]
pub struct ShapesToolState {
    pub is_drawing: bool,
    pub shape_type: ShapeType,
    pub start_position: Option<Vec2>,
    pub current_position: Option<Vec2>,
    pub corner_radius: f32,  // For rounded rectangles
}

impl ShapesToolState {
    pub fn reset(&mut self) {
        self.is_drawing = false;
        self.start_position = None;
        self.current_position = None;
    }

    pub fn get_bounds(&self) -> Option<(Vec2, Vec2)> {
        if let (Some(start), Some(current)) = (self.start_position, self.current_position) {
            let min = Vec2::new(start.x.min(current.x), start.y.min(current.y));
            let max = Vec2::new(start.x.max(current.x), start.y.max(current.y));
            Some((min, max))
        } else {
            None
        }
    }
}

/// Component to mark shape preview entities
#[derive(Component)]
pub struct ShapePreviewElement;

/// The shapes tool implementation
pub struct ShapesTool;

impl EditTool for ShapesTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "shapes",
            display_name: "Shapes",
            icon: "\u{E016}", // Shapes icon
            tooltip: "Create geometric shapes",
            shortcut: Some(KeyCode::KeyR), // R for Rectangle/shapes
        }
    }

    fn on_activate(&mut self, commands: &mut Commands) {
        commands.insert_resource(ShapesModeActive(true));
        commands.insert_resource(crate::io::input::InputMode::Shape);
        debug!("Shapes tool activated");
    }

    fn on_deactivate(&mut self, commands: &mut Commands) {
        commands.insert_resource(ShapesModeActive(false));
        commands.insert_resource(crate::io::input::InputMode::Normal);
        debug!("Shapes tool deactivated");
    }
}

/// Plugin for the shapes tool
pub struct ShapesToolPlugin;

impl Plugin for ShapesToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShapesModeActive>()
            .init_resource::<CurrentShapeType>()
            .init_resource::<ShapesToolState>()
            .add_systems(
                Update,
                (
                    handle_shapes_direct_input,
                    render_shapes_preview,
                    sync_shapes_mode_with_tool_state,
                )
                .run_if(resource_exists::<crate::tools::ToolState>),
            );
    }
}

/// Handle direct input for shapes tool
fn handle_shapes_direct_input(
    tool_state: Res<crate::tools::ToolState>,
    mut shapes_state: ResMut<ShapesToolState>,
    current_shape_type: Res<CurrentShapeType>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<bevy::core_pipeline::core_2d::Camera2d>>,
) {
    // Only process if shapes tool is active
    if !tool_state.is_active(crate::tools::ToolId::Shapes) {
        return;
    }

    // Update shape type from current selection
    shapes_state.shape_type = current_shape_type.0;

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    // Handle shape drawing
    if mouse_input.just_pressed(MouseButton::Left) {
        shapes_state.start_position = Some(world_position);
        shapes_state.current_position = Some(world_position);
        shapes_state.is_drawing = true;
        debug!("🔳 SHAPES: Started drawing {:?} at {:?}", shapes_state.shape_type, world_position);
    }

    if mouse_input.pressed(MouseButton::Left) && shapes_state.is_drawing {
        shapes_state.current_position = Some(world_position);
    }

    if mouse_input.just_released(MouseButton::Left) && shapes_state.is_drawing {
        if let Some((min, max)) = shapes_state.get_bounds() {
            let size = max - min;
            if size.length() > 5.0 {  // Minimum size threshold
                debug!("🔳 SHAPES: Completed {:?} from {:?} to {:?}",
                    shapes_state.shape_type, min, max);

                // TODO: Create actual shape in FontIR here
                create_shape_in_font(&shapes_state, min, max);
            }
        }
        shapes_state.reset();
    }

    // Cancel on Escape
    if keyboard.just_pressed(KeyCode::Escape) && shapes_state.is_drawing {
        shapes_state.reset();
        debug!("🔳 SHAPES: Cancelled shape drawing");
    }

    // Shape type switching with number keys
    if keyboard.just_pressed(KeyCode::Digit1) {
        shapes_state.shape_type = ShapeType::Rectangle;
        debug!("🔳 SHAPES: Switched to Rectangle");
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        shapes_state.shape_type = ShapeType::Oval;
        debug!("🔳 SHAPES: Switched to Oval");
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        shapes_state.shape_type = ShapeType::RoundedRectangle;
        debug!("🔳 SHAPES: Switched to RoundedRectangle");
    }
}

/// Render the shape preview using meshes
fn render_shapes_preview(
    tool_state: Res<crate::tools::ToolState>,
    shapes_state: Res<ShapesToolState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    existing_preview: Query<Entity, With<ShapePreviewElement>>,
) {
    // Clean up existing preview
    for entity in existing_preview.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Only render if shapes tool is active and drawing
    if !tool_state.is_active(crate::tools::ToolId::Shapes) || !shapes_state.is_drawing {
        return;
    }

    let Some((min, max)) = shapes_state.get_bounds() else {
        return;
    };

    let color = Color::srgba(0.3, 0.6, 1.0, 0.5); // Semi-transparent blue
    let outline_color = Color::srgb(0.3, 0.6, 1.0); // Solid blue for outline

    match shapes_state.shape_type {
        ShapeType::Rectangle => {
            // Draw rectangle outline using line meshes
            let width = 2.0;

            // Top line
            let top_mesh = crate::rendering::mesh_utils::create_line_mesh(
                Vec2::new(min.x, max.y), Vec2::new(max.x, max.y), width
            );
            // Right line
            let right_mesh = crate::rendering::mesh_utils::create_line_mesh(
                Vec2::new(max.x, max.y), Vec2::new(max.x, min.y), width
            );
            // Bottom line
            let bottom_mesh = crate::rendering::mesh_utils::create_line_mesh(
                Vec2::new(max.x, min.y), Vec2::new(min.x, min.y), width
            );
            // Left line
            let left_mesh = crate::rendering::mesh_utils::create_line_mesh(
                Vec2::new(min.x, min.y), Vec2::new(min.x, max.y), width
            );

            let material = materials.add(ColorMaterial::from(outline_color));

            // Spawn the lines
            for mesh in [top_mesh, right_mesh, bottom_mesh, left_mesh] {
                commands.spawn((
                    Mesh2d(meshes.add(mesh)),
                    MeshMaterial2d(material.clone()),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)),
                    ShapePreviewElement,
                ));
            }
        }
        ShapeType::Oval => {
            // Draw ellipse preview
            // For now, we'll approximate with a circle at the center
            let center = (min + max) * 0.5;
            let radius = ((max.x - min.x).abs().min((max.y - min.y).abs())) * 0.5;

            let circle_mesh = Mesh::from(Circle::new(radius));

            commands.spawn((
                Mesh2d(meshes.add(circle_mesh)),
                MeshMaterial2d(materials.add(ColorMaterial::from(color))),
                Transform::from_translation(center.extend(15.0)),
                ShapePreviewElement,
            ));
        }
        ShapeType::RoundedRectangle => {
            // Similar to rectangle but with corner indicators
            // For now, just draw a regular rectangle
            let width = 2.0;

            // Draw rectangle outline
            let lines = [
                (Vec2::new(min.x, max.y), Vec2::new(max.x, max.y)),
                (Vec2::new(max.x, max.y), Vec2::new(max.x, min.y)),
                (Vec2::new(max.x, min.y), Vec2::new(min.x, min.y)),
                (Vec2::new(min.x, min.y), Vec2::new(min.x, max.y)),
            ];

            let material = materials.add(ColorMaterial::from(outline_color));

            for (start, end) in lines {
                let mesh = crate::rendering::mesh_utils::create_line_mesh(start, end, width);
                commands.spawn((
                    Mesh2d(meshes.add(mesh)),
                    MeshMaterial2d(material.clone()),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)),
                    ShapePreviewElement,
                ));
            }

            // Add corner radius indicators (small circles at corners)
            let corner_radius = shapes_state.corner_radius.min(20.0);
            let corners = [
                Vec2::new(min.x + corner_radius, min.y + corner_radius),
                Vec2::new(max.x - corner_radius, min.y + corner_radius),
                Vec2::new(max.x - corner_radius, max.y - corner_radius),
                Vec2::new(min.x + corner_radius, max.y - corner_radius),
            ];

            for corner in corners {
                let circle_mesh = Mesh::from(Circle::new(3.0));
                commands.spawn((
                    Mesh2d(meshes.add(circle_mesh)),
                    MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
                    Transform::from_translation(corner.extend(16.0)),
                    ShapePreviewElement,
                ));
            }
        }
    }
}

/// Sync shapes mode with unified tool state
fn sync_shapes_mode_with_tool_state(
    tool_state: Res<crate::tools::ToolState>,
    mut shapes_mode: ResMut<ShapesModeActive>,
    mut shapes_state: ResMut<ShapesToolState>,
) {
    let should_be_active = tool_state.is_active(crate::tools::ToolId::Shapes);

    if shapes_mode.0 != should_be_active {
        shapes_mode.0 = should_be_active;

        // Reset shapes state when deactivating
        if !should_be_active {
            shapes_state.reset();
            debug!("🔳 SHAPES: Reset state on tool deactivation");
        }
    }
}

/// Create the actual shape in the font data
fn create_shape_in_font(shapes_state: &ShapesToolState, min: Vec2, max: Vec2) {
    // This would integrate with FontIR to actually create the shape
    // For now, just log what we would create
    match shapes_state.shape_type {
        ShapeType::Rectangle => {
            debug!("🔳 SHAPES: Would create rectangle from {:?} to {:?}", min, max);
        }
        ShapeType::Oval => {
            let center = (min + max) * 0.5;
            let rx = (max.x - min.x).abs() * 0.5;
            let ry = (max.y - min.y).abs() * 0.5;
            debug!("🔳 SHAPES: Would create oval at {:?} with radii ({}, {})", center, rx, ry);
        }
        ShapeType::RoundedRectangle => {
            debug!("🔳 SHAPES: Would create rounded rectangle from {:?} to {:?} with radius {}",
                min, max, shapes_state.corner_radius);
        }
    }
}
