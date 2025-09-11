//! Mouse and trackpad managment

use crate::geometry::world_space::DPoint;
use crate::rendering::cameras::DesignCamera;
use bevy::prelude::*;

/// Single source of truth for pointer (mouse/trackpad) position
/// This replaces the old CursorInfo to avoid confusion with text editor cursor
#[derive(Resource)]
pub struct PointerInfo {
    /// Screen space coordinates (pixels)
    pub screen: Vec2,
    /// Design space coordinates (canonical font coordinates)
    pub design: DPoint,
    /// World space coordinates (for debugging)
    pub world: Vec2,
}

impl Default for PointerInfo {
    fn default() -> Self {
        Self {
            screen: Vec2::ZERO,
            design: DPoint::new(0.0, 0.0),
            world: Vec2::ZERO,
        }
    }
}

/// Plugin that centrally manages pointer position conversions
pub struct PointerPlugin;

impl Plugin for PointerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PointerInfo>()
            .add_systems(Update, update_pointer_position);
    }
}

/// System that updates pointer position once per frame
/// This is the ONLY place coordinate conversions should happen
fn update_pointer_position(
    mut pointer_info: ResMut<PointerInfo>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<DesignCamera>>,
) {
    if let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), camera_query.single())
    {
        if let Some(screen_pos) = window.cursor_position() {
            pointer_info.screen = screen_pos;

            // Convert to world space
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) {
                pointer_info.world = world_pos;

                // Convert to design space
                pointer_info.design = DPoint::from_raw(world_pos);
            }
        }
    }
}
