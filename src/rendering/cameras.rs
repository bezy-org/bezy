//! Camera system for the Bezy font editor

#![allow(clippy::uninlined_format_args)]

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::OrthographicProjection;
use bevy_pancam::{PanCam, PanCamPlugin};

// Component to mark the main design camera
#[derive(Component)]
pub struct DesignCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanCamPlugin)
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (zoom_camera, toggle_camera_controls));
    }
}

pub fn setup_camera(mut commands: Commands) {
    // Create a 2D camera with proper configuration positioned to view font design space
    // Center the camera at (0,0) in design space for simplicity
    // This makes the design space origin appear at the center of the screen
    let camera_center_y = 0.0; // Center on design space origin

    debug!(
        "Setting up camera at y={} to center on design space origin",
        camera_center_y
    );

    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        // Position camera to center on glyph area instead of design space origin
        Transform::from_xyz(0.0, camera_center_y, 1000.0),
        // Set initial orthographic projection scale for 25% zoom (temporary for testing)
        Projection::Orthographic(OrthographicProjection {
            scale: 4.0, // 4.0 = 25% zoom (larger scale = more zoomed out)
            ..OrthographicProjection::default_2d()
        }),
        PanCam {
            grab_buttons: vec![MouseButton::Left, MouseButton::Middle],
            enabled: true,
            zoom_to_cursor: true,
            min_scale: crate::ui::theme::MIN_ALLOWED_ZOOM_SCALE,
            max_scale: crate::ui::theme::MAX_ALLOWED_ZOOM_SCALE,
            ..default()
        },
        DesignCamera,
    ));
}

// Handle camera zooming with mouse wheel
fn zoom_camera(
    mut scroll_events: EventReader<MouseWheel>,
    _cameras: Query<&mut Transform, With<DesignCamera>>,
) {
    let scroll = scroll_events.read().fold(0.0, |scroll, event| {
        scroll
            + match event.unit {
                MouseScrollUnit::Line => event.y,
                MouseScrollUnit::Pixel => event.y / 20.0,
            }
    });

    if scroll != 0.0 {
        // We don't need to manually handle zooming here,
        // as PanCam will handle it automatically
        // This system is kept in place for future custom zoom behavior if needed
    }
}

/// Handles keyboard shortcuts for camera control
fn toggle_camera_controls(mut query: Query<&mut PanCam>, keys: Res<ButtonInput<KeyCode>>) {
    // Handle zoom to cursor toggle
    if keys.just_pressed(KeyCode::KeyT) {
        for mut pancam in query.iter_mut() {
            pancam.zoom_to_cursor = !pancam.zoom_to_cursor;
            let status = if pancam.zoom_to_cursor {
                "enabled"
            } else {
                "disabled"
            };
            debug!("Camera zoom to cursor {}", status);
        }
    }
}

#[cfg(test)]
mod camera_and_pointer_tests {
    use super::*;
    use crate::editing::selection::coordinate_system::SelectionCoordinateSystem;
    use crate::geometry::world_space::DPoint;
    use bevy::prelude::*;

    /// Simulate a camera at a given position and scale, and a point at a given design space position.
    /// Returns true if a marquee at the visible area would select the point.
    fn is_point_selectable_by_marquee(
        camera_center: Vec2,
        camera_scale: f32,
        window_size: Vec2,
        point_pos: Vec2,
        marquee_size: Vec2,
    ) -> bool {
        // Compute visible area in design space (for future use)
        let half_win = window_size / 2.0 * camera_scale;
        let _visible_min = camera_center - half_win;
        let _visible_max = camera_center + half_win;

        // Place marquee at the center of the visible area
        let marquee_center = camera_center;
        let marquee_start = DPoint::from_raw(marquee_center - marquee_size / 2.0);
        let marquee_end = DPoint::from_raw(marquee_center + marquee_size / 2.0);

        // Is the point inside the marquee?
        SelectionCoordinateSystem::is_point_in_rectangle(&point_pos, &marquee_start, &marquee_end)
    }

    #[test]
    fn test_camera_at_origin_point_at_negative_y() {
        // Camera at (0,0), point at (-900, 0), window 1000x1000, marquee 100x100
        let camera_center = Vec2::new(0.0, 0.0);
        let camera_scale = 1.0;
        let window_size = Vec2::new(1000.0, 1000.0);
        let point_pos = Vec2::new(0.0, -900.0);
        let marquee_size = Vec2::new(100.0, 100.0);
        let selectable = is_point_selectable_by_marquee(
            camera_center,
            camera_scale,
            window_size,
            point_pos,
            marquee_size,
        );
        println!(
            "[test_camera_at_origin_point_at_negative_y] selectable={}",
            selectable
        );
        assert!(
            !selectable,
            "Point at -900 should NOT be selectable when camera is at 0"
        );
    }

    #[test]
    fn test_camera_centered_on_glyph() {
        // Camera at (0, -900), point at (0, -900), window 1000x1000, marquee 100x100
        let camera_center = Vec2::new(0.0, -900.0);
        let camera_scale = 1.0;
        let window_size = Vec2::new(1000.0, 1000.0);
        let point_pos = Vec2::new(0.0, -900.0);
        let marquee_size = Vec2::new(100.0, 100.0);
        let selectable = is_point_selectable_by_marquee(
            camera_center,
            camera_scale,
            window_size,
            point_pos,
            marquee_size,
        );
        println!("[test_camera_centered_on_glyph] selectable={}", selectable);
        assert!(
            selectable,
            "Point at -900 should be selectable when camera is centered on it"
        );
    }

    #[test]
    fn test_camera_centered_on_glyph_with_zoom() {
        // Camera at (0, -900), point at (0, -900), window 1000x1000, marquee 100x100, zoomed in (scale=0.5)
        let camera_center = Vec2::new(0.0, -900.0);
        let camera_scale = 0.5; // Zoomed in (see less world space)
        let window_size = Vec2::new(1000.0, 1000.0);
        let point_pos = Vec2::new(0.0, -900.0);
        let marquee_size = Vec2::new(100.0, 100.0);
        let selectable = is_point_selectable_by_marquee(
            camera_center,
            camera_scale,
            window_size,
            point_pos,
            marquee_size,
        );
        println!(
            "[test_camera_centered_on_glyph_with_zoom] selectable={}",
            selectable
        );
        assert!(
            selectable,
            "Point at -900 should be selectable when camera is centered and zoomed"
        );
    }

    #[test]
    fn test_screen_to_design_space_conversion() {
        // Simulate a screen position and camera, and check design space conversion
        let camera_center = Vec2::new(0.0, -900.0);
        let camera_scale = 1.0;
        let window_size = Vec2::new(1000.0, 1000.0);
        let screen_pos = Vec2::new(500.0, 500.0); // Center of window
                                                  // Convert screen to design space
        let design_space_pos = camera_center + (screen_pos - window_size / 2.0) * camera_scale;
        println!(
            "[test_screen_to_design_space_conversion] screen_pos={:?}, design_space_pos={:?}",
            screen_pos, design_space_pos
        );
        assert_eq!(
            design_space_pos, camera_center,
            "Screen center should map to camera center in design space"
        );
    }
}
