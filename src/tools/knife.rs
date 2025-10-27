//! Knife tool for cutting contours
//!
//! The knife tool allows users to cut existing contours at specific points.

use super::{EditTool, ToolInfo};
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};

/// Resource to track if knife mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct KnifeModeActive(pub bool);

/// The knife tool implementation
pub struct KnifeTool;

impl EditTool for KnifeTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "knife",
            display_name: "Knife",
            icon: "\u{E012}", // Knife icon
            tooltip: "Cut contours at specific points",
            shortcut: Some(KeyCode::KeyK),
        }
    }

    fn on_activate(&mut self, commands: &mut Commands) {
        commands.insert_resource(KnifeModeActive(true));
        commands.insert_resource(crate::io::input::InputMode::Knife);
        debug!("Knife tool activated");
    }

    fn on_deactivate(&mut self, commands: &mut Commands) {
        commands.insert_resource(KnifeModeActive(false));
        commands.insert_resource(crate::io::input::InputMode::Normal);
        debug!("Knife tool deactivated");
    }
}

/// State for knife tool gesture
#[derive(Resource, Default, Debug)]
pub struct KnifeToolState {
    pub gesture: KnifeGestureState,
    pub shift_locked: bool,
}

/// The state of the knife cutting gesture
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum KnifeGestureState {
    #[default]
    Ready,
    Cutting { start: Vec2, current: Vec2 },
}

impl KnifeToolState {
    /// Get the cutting line with axis locking if shift is pressed
    pub fn get_cutting_line(&self) -> Option<(Vec2, Vec2)> {
        match self.gesture {
            KnifeGestureState::Cutting { start, current } => {
                let actual_end = if self.shift_locked {
                    // Apply axis constraint for shift key
                    let delta = current - start;
                    if delta.x.abs() > delta.y.abs() {
                        Vec2::new(current.x, start.y)  // Horizontal line
                    } else {
                        Vec2::new(start.x, current.y)  // Vertical line
                    }
                } else {
                    current
                };
                Some((start, actual_end))
            }
            KnifeGestureState::Ready => None,
        }
    }
}

/// Component to mark cut intersection points
#[derive(Component, Debug, Clone)]
pub struct CutIntersection {
    pub position: Vec2,
    pub t_value: f32,  // Parameter value along the segment
    pub segment_index: usize,
}

/// Plugin for the knife tool
pub struct KnifeToolPlugin;

impl Plugin for KnifeToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KnifeModeActive>()
            .init_resource::<KnifeToolState>()
            .add_systems(
                Update,
                (
                    handle_knife_direct_input,
                    render_knife_preview,
                    sync_knife_mode_with_tool_state,
                )
                .run_if(resource_exists::<crate::tools::ToolState>),
            );
    }
}

fn handle_knife_direct_input(
    tool_state: Res<crate::tools::ToolState>,
    mut knife_state: ResMut<KnifeToolState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<bevy::core_pipeline::core_2d::Camera2d>>,
) {
    // Only process if knife tool is active
    if !tool_state.is_active(crate::tools::ToolId::Knife) {
        return;
    }

    // Update shift lock state
    knife_state.shift_locked = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

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

    // Handle mouse input for cutting gesture
    if mouse_input.just_pressed(MouseButton::Left) {
        knife_state.gesture = KnifeGestureState::Cutting {
            start: world_position,
            current: world_position,
        };
        debug!("🔪 KNIFE: Started cut at {:?}", world_position);
    }

    if mouse_input.pressed(MouseButton::Left) {
        if let KnifeGestureState::Cutting { start, .. } = knife_state.gesture {
            knife_state.gesture = KnifeGestureState::Cutting {
                start,
                current: world_position,
            };
        }
    }

    if mouse_input.just_released(MouseButton::Left) {
        if let KnifeGestureState::Cutting { start, current } = knife_state.gesture {
            // Apply axis constraint if shift is held
            let end = if knife_state.shift_locked {
                let delta = current - start;
                if delta.x.abs() > delta.y.abs() {
                    Vec2::new(current.x, start.y)
                } else {
                    Vec2::new(start.x, current.y)
                }
            } else {
                current
            };

            debug!("🔪 KNIFE: Completed cut from {:?} to {:?}", start, end);

            // Execute the cut operation
            perform_knife_cut(start, end);

            knife_state.gesture = KnifeGestureState::Ready;
        }
    }

    // Cancel on Escape
    if keyboard.just_pressed(KeyCode::Escape) && matches!(knife_state.gesture, KnifeGestureState::Cutting { .. }) {
        knife_state.gesture = KnifeGestureState::Ready;
        debug!("🔪 KNIFE: Cancelled cut");
    }
}

/// Component to mark knife preview entities
#[derive(Component)]
struct KnifePreviewElement;

/// Render the knife cut preview line using meshes
fn render_knife_preview(
    tool_state: Res<crate::tools::ToolState>,
    knife_state: Res<KnifeToolState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    existing_preview: Query<Entity, With<KnifePreviewElement>>,
) {
    // Clean up existing preview
    for entity in existing_preview.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Only render if knife tool is active
    if !tool_state.is_active(crate::tools::ToolId::Knife) {
        return;
    }

    // Draw the cutting line if we're in cutting state
    if let Some((start, end)) = knife_state.get_cutting_line() {
        let color = Color::srgb(1.0, 0.3, 0.3); // Red for knife cut

        // Create line mesh for the cut preview
        let line_mesh = crate::rendering::mesh_utils::create_line_mesh(start, end, 2.0);

        commands.spawn((
            Mesh2d(meshes.add(line_mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from(color))),
            Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)), // High Z for visibility
            KnifePreviewElement,
        ));

        // Draw endpoints as circles
        let point_mesh = Mesh::from(Circle::new(3.0));
        let white_material = materials.add(ColorMaterial::from(Color::WHITE));

        // Start point
        commands.spawn((
            Mesh2d(meshes.add(point_mesh.clone())),
            MeshMaterial2d(white_material.clone()),
            Transform::from_translation(start.extend(16.0)),
            KnifePreviewElement,
        ));

        // End point
        commands.spawn((
            Mesh2d(meshes.add(point_mesh)),
            MeshMaterial2d(white_material),
            Transform::from_translation(end.extend(16.0)),
            KnifePreviewElement,
        ));

        // If shift is locked, show axis constraint hint
        if knife_state.shift_locked {
            let delta = end - start;
            let axis_color = Color::srgba(1.0, 1.0, 0.0, 0.3); // Yellow hint
            let axis_material = materials.add(ColorMaterial::from(axis_color));

            if delta.x.abs() > delta.y.abs() {
                // Horizontal constraint line
                let hint_start = Vec2::new(start.x - 50.0, start.y);
                let hint_end = Vec2::new(end.x + 50.0, end.y);
                let hint_mesh = crate::rendering::mesh_utils::create_line_mesh(hint_start, hint_end, 1.0);

                commands.spawn((
                    Mesh2d(meshes.add(hint_mesh)),
                    MeshMaterial2d(axis_material),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 14.0)),
                    KnifePreviewElement,
                ));
            } else {
                // Vertical constraint line
                let hint_start = Vec2::new(start.x, start.y - 50.0);
                let hint_end = Vec2::new(end.x, end.y + 50.0);
                let hint_mesh = crate::rendering::mesh_utils::create_line_mesh(hint_start, hint_end, 1.0);

                commands.spawn((
                    Mesh2d(meshes.add(hint_mesh)),
                    MeshMaterial2d(axis_material),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 14.0)),
                    KnifePreviewElement,
                ));
            }
        }
    }
}

/// Sync knife mode with unified tool state
fn sync_knife_mode_with_tool_state(
    tool_state: Res<crate::tools::ToolState>,
    mut knife_mode: ResMut<KnifeModeActive>,
    mut knife_state: ResMut<KnifeToolState>,
) {
    let should_be_active = tool_state.is_active(crate::tools::ToolId::Knife);

    if knife_mode.0 != should_be_active {
        knife_mode.0 = should_be_active;

        // Reset knife state when deactivating
        if !should_be_active {
            knife_state.gesture = KnifeGestureState::Ready;
            knife_state.shift_locked = false;
            debug!("🔪 KNIFE: Reset state on tool deactivation");
        }
    }
}

/// Perform the actual knife cut operation
fn perform_knife_cut(start: Vec2, end: Vec2) {
    // For now, just log the cut operation
    // Full implementation would require access to:
    // - Active glyph's contour data
    // - FontIR state for modification
    // - Sort entity management

    info!(
        "🔪 KNIFE: Cut performed from ({:.2}, {:.2}) to ({:.2}, {:.2})",
        start.x, start.y, end.x, end.y
    );

    // Future implementation steps:
    // 1. Get active glyph's contours from FontIR
    // 2. Find all intersection points between cut line and contours
    // 3. Split contours at intersection points
    // 4. Update FontIR with new contour data
    // 5. Trigger glyph re-render
}

/// Calculate intersection between a line segment and another line segment
/// Returns the parameter t (0-1) along the first segment if intersection exists
fn line_segment_intersection(p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2) -> Option<f32> {
    let s1 = p2 - p1;
    let s2 = p4 - p3;

    let denominator = -s2.x * s1.y + s1.x * s2.y;

    // Lines are parallel
    if denominator.abs() < 0.0001 {
        return None;
    }

    let s = (-s1.y * (p1.x - p3.x) + s1.x * (p1.y - p3.y)) / denominator;
    let t = (s2.x * (p1.y - p3.y) - s2.y * (p1.x - p3.x)) / denominator;

    // Check if intersection is within both segments
    if s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0 {
        Some(t)
    } else {
        None
    }
}
