//! Measure tool for measuring distances and angles
//!
//! The measure tool allows users to measure distances between points
//! and angles between segments.

use super::{EditTool, ToolInfo};
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};

/// Resource to track if measure mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct MeasureModeActive(pub bool);

/// State for the measure tool
#[derive(Resource, Default, Debug)]
pub struct MeasureToolState {
    pub measurement: MeasurementType,
    pub start_point: Option<Vec2>,
    pub end_point: Option<Vec2>,
    pub shift_locked: bool,
}

/// Type of measurement being performed
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum MeasurementType {
    #[default]
    None,
    Distance { start: Vec2, end: Vec2 },
    Angle { pivot: Vec2, p1: Vec2, p2: Vec2 },
}

impl MeasureToolState {
    /// Calculate the current measurement value
    pub fn get_measurement(&self) -> Option<MeasurementResult> {
        match self.measurement {
            MeasurementType::None => None,
            MeasurementType::Distance { start, end } => {
                let mut actual_end = end;
                if self.shift_locked {
                    // Constrain to axis
                    let delta = end - start;
                    actual_end = if delta.x.abs() > delta.y.abs() {
                        Vec2::new(end.x, start.y)  // Horizontal
                    } else {
                        Vec2::new(start.x, end.y)  // Vertical
                    };
                }
                let distance = (actual_end - start).length();
                Some(MeasurementResult::Distance(distance))
            }
            MeasurementType::Angle { pivot, p1, p2 } => {
                let v1 = (p1 - pivot).normalize();
                let v2 = (p2 - pivot).normalize();
                let angle = v1.angle_to(v2).to_degrees();
                Some(MeasurementResult::Angle(angle))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MeasurementResult {
    Distance(f32),
    Angle(f32),  // In degrees
}

/// The measure tool implementation
pub struct MeasureTool;

impl EditTool for MeasureTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "measure",
            display_name: "Measure",
            icon: "\u{E015}", // Ruler icon
            tooltip: "Measure distances and angles",
            shortcut: Some(KeyCode::KeyM),
        }
    }

    fn on_activate(&mut self, commands: &mut Commands) {
        commands.insert_resource(MeasureModeActive(true));
        commands.insert_resource(crate::io::input::InputMode::Measure);
        debug!("Measure tool activated");
    }

    fn on_deactivate(&mut self, commands: &mut Commands) {
        commands.insert_resource(MeasureModeActive(false));
        commands.insert_resource(crate::io::input::InputMode::Normal);
        debug!("Measure tool deactivated");
    }
}

/// Plugin for the measure tool
pub struct MeasureToolPlugin;

impl Plugin for MeasureToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeasureModeActive>()
            .init_resource::<MeasureToolState>()
            .add_systems(
                Update,
                (
                    handle_measure_direct_input,
                    render_measure_preview,
                    sync_measure_mode_with_tool_state,
                )
                .run_if(resource_exists::<crate::tools::ToolState>),
            );
    }
}

/// Handle direct input for the measure tool
fn handle_measure_direct_input(
    tool_state: Res<crate::tools::ToolState>,
    mut measure_state: ResMut<MeasureToolState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<bevy::core_pipeline::core_2d::Camera2d>>,
) {
    // Only process if measure tool is active
    if !tool_state.is_active(crate::tools::ToolId::Measure) {
        return;
    }

    // Update shift lock state
    measure_state.shift_locked = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

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

    // Handle mouse input for measurements
    if mouse_input.just_pressed(MouseButton::Left) {
        // Start a distance measurement
        measure_state.measurement = MeasurementType::Distance {
            start: world_position,
            end: world_position,
        };
        measure_state.start_point = Some(world_position);
        debug!("📏 MEASURE: Started measurement at {:?}", world_position);
    }

    if mouse_input.pressed(MouseButton::Left) {
        if let MeasurementType::Distance { start, .. } = measure_state.measurement {
            measure_state.measurement = MeasurementType::Distance {
                start,
                end: world_position,
            };
            measure_state.end_point = Some(world_position);
        }
    }

    if mouse_input.just_released(MouseButton::Left) {
        if let Some(result) = measure_state.get_measurement() {
            match result {
                MeasurementResult::Distance(dist) => {
                    info!("📏 MEASURE: Distance = {:.2} units", dist);
                }
                MeasurementResult::Angle(angle) => {
                    info!("📏 MEASURE: Angle = {:.2}°", angle);
                }
            }
        }
        // Keep the measurement visible until next click or escape
    }

    // Cancel measurement on Escape
    if keyboard.just_pressed(KeyCode::Escape) {
        measure_state.measurement = MeasurementType::None;
        measure_state.start_point = None;
        measure_state.end_point = None;
        debug!("📏 MEASURE: Cancelled measurement");
    }

    // Toggle to angle measurement with A key
    if keyboard.just_pressed(KeyCode::KeyA) {
        if let MeasurementType::Distance { start, end } = measure_state.measurement {
            // Convert to angle measurement with pivot at start
            measure_state.measurement = MeasurementType::Angle {
                pivot: start,
                p1: end,
                p2: world_position,
            };
            debug!("📏 MEASURE: Switched to angle mode");
        }
    }
}

/// Component to mark measure preview entities
#[derive(Component)]
struct MeasurePreviewElement;

/// Render the measurement preview using meshes
fn render_measure_preview(
    tool_state: Res<crate::tools::ToolState>,
    measure_state: Res<MeasureToolState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    existing_preview: Query<Entity, With<MeasurePreviewElement>>,
) {
    // Clean up existing preview
    for entity in existing_preview.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // Only render if measure tool is active
    if !tool_state.is_active(crate::tools::ToolId::Measure) {
        return;
    }

    match measure_state.measurement {
        MeasurementType::Distance { start, end } => {
            let mut actual_end = end;
            if measure_state.shift_locked {
                // Apply axis constraint
                let delta = end - start;
                actual_end = if delta.x.abs() > delta.y.abs() {
                    Vec2::new(end.x, start.y)
                } else {
                    Vec2::new(start.x, end.y)
                };
            }

            // Draw measurement line using mesh
            let color = Color::srgb(0.3, 0.8, 1.0); // Cyan for measurements
            let line_mesh = crate::rendering::mesh_utils::create_line_mesh(start, actual_end, 2.0);

            commands.spawn((
                Mesh2d(meshes.add(line_mesh)),
                MeshMaterial2d(materials.add(ColorMaterial::from(color))),
                Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)),
                MeasurePreviewElement,
            ));

            // Draw endpoints
            let point_mesh = Mesh::from(Circle::new(4.0));
            let white_material = materials.add(ColorMaterial::from(Color::WHITE));

            commands.spawn((
                Mesh2d(meshes.add(point_mesh.clone())),
                MeshMaterial2d(white_material.clone()),
                Transform::from_translation(start.extend(16.0)),
                MeasurePreviewElement,
            ));

            commands.spawn((
                Mesh2d(meshes.add(point_mesh)),
                MeshMaterial2d(white_material),
                Transform::from_translation(actual_end.extend(16.0)),
                MeasurePreviewElement,
            ));
        }
        MeasurementType::Angle { pivot, p1, p2 } => {
            let color = Color::srgb(1.0, 0.8, 0.3); // Yellow for angles

            // Draw the angle lines
            let line1_mesh = crate::rendering::mesh_utils::create_line_mesh(pivot, p1, 2.0);
            let line2_mesh = crate::rendering::mesh_utils::create_line_mesh(pivot, p2, 2.0);
            let angle_material = materials.add(ColorMaterial::from(color));

            commands.spawn((
                Mesh2d(meshes.add(line1_mesh)),
                MeshMaterial2d(angle_material.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)),
                MeasurePreviewElement,
            ));

            commands.spawn((
                Mesh2d(meshes.add(line2_mesh)),
                MeshMaterial2d(angle_material.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)),
                MeasurePreviewElement,
            ));

            // Draw pivot point
            let pivot_mesh = Mesh::from(Circle::new(5.0));
            commands.spawn((
                Mesh2d(meshes.add(pivot_mesh)),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
                Transform::from_translation(pivot.extend(16.0)),
                MeasurePreviewElement,
            ));

            // Draw p1 and p2 points
            let point_mesh = Mesh::from(Circle::new(3.0));
            commands.spawn((
                Mesh2d(meshes.add(point_mesh.clone())),
                MeshMaterial2d(angle_material.clone()),
                Transform::from_translation(p1.extend(16.0)),
                MeasurePreviewElement,
            ));

            commands.spawn((
                Mesh2d(meshes.add(point_mesh)),
                MeshMaterial2d(angle_material),
                Transform::from_translation(p2.extend(16.0)),
                MeasurePreviewElement,
            ));
        }
        MeasurementType::None => {}
    }
}

/// Sync measure mode with unified tool state
fn sync_measure_mode_with_tool_state(
    tool_state: Res<crate::tools::ToolState>,
    mut measure_mode: ResMut<MeasureModeActive>,
    mut measure_state: ResMut<MeasureToolState>,
) {
    let should_be_active = tool_state.is_active(crate::tools::ToolId::Measure);

    if measure_mode.0 != should_be_active {
        measure_mode.0 = should_be_active;

        // Reset measure state when deactivating
        if !should_be_active {
            measure_state.measurement = MeasurementType::None;
            measure_state.start_point = None;
            measure_state.end_point = None;
            measure_state.shift_locked = false;
            debug!("📏 MEASURE: Reset state on tool deactivation");
        }
    }
}
