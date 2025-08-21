//! Camera-responsive scaling system
//!
//! This module provides a system to adjust visual element sizes (line widths, point sizes, etc.)
//! based on camera zoom level to maintain visual consistency across different zoom levels.
//!
//! This addresses the issue where mesh-based rendering elements become too small when zoomed out,
//! unlike gizmos which maintain screen-space size automatically.

use crate::rendering::cameras::DesignCamera;
use bevy::prelude::*;

/// Resource that tracks the current camera-responsive scale factor
#[derive(Resource, Default)]
pub struct CameraResponsiveScale {
    /// The current scale factor to apply to visual elements
    /// 1.0 = normal size, >1.0 = bigger, <1.0 = smaller
    pub scale_factor: f32,
    /// The base line width in world units at normal zoom (1.0)
    pub base_line_width: f32,
    /// Scale factor when zoomed in to maximum
    pub zoom_in_max_factor: f32,
    /// Scale factor at default zoom level
    pub default_factor: f32,
    /// Scale factor when zoomed out to maximum
    pub zoom_out_max_factor: f32,
    /// Camera scale value that represents maximum zoom in
    pub zoom_in_max_camera_scale: f32,
    /// Camera scale value that represents default zoom
    pub default_camera_scale: f32,
    /// Camera scale value that represents maximum zoom out
    pub zoom_out_max_camera_scale: f32,
}

impl CameraResponsiveScale {
    pub fn new() -> Self {
        Self {
            scale_factor: 1.0,
            base_line_width: 1.0,
            // EASY TO TUNE: Adjust these three scale factors
            zoom_in_max_factor: 1.0, // Keep current size when zoomed in
            default_factor: 1.0,     // Keep current size at default zoom
            zoom_out_max_factor: 12.0, // Make 12x bigger when zoomed out (was 3.0)
            // Camera scale ranges (you can adjust these if needed)
            zoom_in_max_camera_scale: 0.2, // Maximum zoom in
            default_camera_scale: 1.0,     // Default zoom level
            zoom_out_max_camera_scale: 16.0, // Maximum zoom out
        }
    }

    /// Get the adjusted line width based on camera zoom
    pub fn adjusted_line_width(&self) -> f32 {
        self.base_line_width * self.scale_factor
    }

    /// Get the adjusted point size based on camera zoom  
    pub fn adjusted_point_size(&self, base_size: f32) -> f32 {
        base_size * self.scale_factor
    }

    /// Get the adjusted handle size based on camera zoom
    pub fn adjusted_handle_size(&self, base_size: f32) -> f32 {
        base_size * self.scale_factor
    }
}

/// System that updates the camera-responsive scale based on current camera zoom
pub fn update_camera_responsive_scale(
    mut scale_resource: ResMut<CameraResponsiveScale>,
    camera_query: Query<(&Transform, &Projection), With<DesignCamera>>,
) {
    if let Ok((camera_transform, projection)) = camera_query.single() {
        // Get camera scale from OrthographicProjection (this is what PanCam modifies for zoom)
        let projection_scale = match projection {
            Projection::Orthographic(ortho) => ortho.scale,
            _ => 1.0, // Default for non-orthographic projections
        };
        let _transform_scale = camera_transform.scale.x;

        // Use projection scale for responsive scaling (this is the real zoom level)
        let camera_scale = projection_scale;

        // Calculate responsive scale factor
        // When camera_scale is large (zoomed in), we want smaller visual elements
        // When camera_scale is small (zoomed out), we want larger visual elements
        // Use inverse relationship to make the effect very obvious

        // Simple interpolation between three scale factors
        let responsive_factor = if camera_scale
            <= scale_resource.default_camera_scale
        {
            // Interpolate between zoom_in_max and default
            let t = (camera_scale - scale_resource.zoom_in_max_camera_scale)
                / (scale_resource.default_camera_scale
                    - scale_resource.zoom_in_max_camera_scale);
            let t = t.clamp(0.0, 1.0);
            scale_resource.zoom_in_max_factor * (1.0 - t)
                + scale_resource.default_factor * t
        } else {
            // Interpolate between default and zoom_out_max
            let t = (camera_scale - scale_resource.default_camera_scale)
                / (scale_resource.zoom_out_max_camera_scale
                    - scale_resource.default_camera_scale);
            let t = t.clamp(0.0, 1.0);
            scale_resource.default_factor * (1.0 - t)
                + scale_resource.zoom_out_max_factor * t
        };

        // Store the interpolated factor directly (no additional clamping needed)
        scale_resource.scale_factor = responsive_factor;
    }
}

/// Component to mark entities that should respond to camera zoom
#[derive(Component)]
pub struct CameraResponsive {
    /// The type of visual element this entity represents
    pub element_type: ResponsiveElementType,
    /// The base size when scale_factor = 1.0
    pub base_size: f32,
}

/// Types of visual elements that can respond to camera zoom
#[derive(Debug, Clone)]
pub enum ResponsiveElementType {
    LineWidth,
    PointSize,
    HandleSize,
}

/// System that applies camera-responsive scaling to marked entities
pub fn apply_camera_responsive_scaling(
    scale_resource: Res<CameraResponsiveScale>,
    mut responsive_query: Query<
        (&CameraResponsive, &mut Transform),
        Changed<CameraResponsive>,
    >,
) {
    if scale_resource.is_changed() {
        for (responsive, mut transform) in responsive_query.iter_mut() {
            let new_scale = match responsive.element_type {
                ResponsiveElementType::LineWidth => {
                    scale_resource.adjusted_line_width()
                }
                ResponsiveElementType::PointSize => {
                    scale_resource.adjusted_point_size(responsive.base_size)
                }
                ResponsiveElementType::HandleSize => {
                    scale_resource.adjusted_handle_size(responsive.base_size)
                }
            };

            // Update the transform scale while preserving position
            transform.scale = Vec3::new(new_scale, new_scale, 1.0);
        }
    }
}

/// Plugin for camera-responsive scaling
pub struct CameraResponsivePlugin;

impl Plugin for CameraResponsivePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraResponsiveScale::new())
            .add_systems(Update, update_camera_responsive_scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_responsive_scale_calculation() {
        let mut scale = CameraResponsiveScale::new();

        // Test zoomed in (small camera scale)
        scale.scale_factor = 0.5_f32.sqrt(); // camera_scale = 0.5
        assert!(
            scale.scale_factor < 1.0,
            "Zoomed in should have smaller scale factor"
        );

        // Test zoomed out (large camera scale)
        scale.scale_factor = 4.0_f32.sqrt(); // camera_scale = 4.0
        assert!(
            scale.scale_factor > 1.0,
            "Zoomed out should have larger scale factor"
        );

        // Test normal zoom
        scale.scale_factor = 1.0_f32.sqrt(); // camera_scale = 1.0
        assert_eq!(
            scale.scale_factor, 1.0,
            "Normal zoom should have scale factor of 1.0"
        );
    }

    #[test]
    fn test_adjusted_sizes() {
        let mut scale = CameraResponsiveScale::new();
        scale.scale_factor = 2.0;

        assert_eq!(scale.adjusted_line_width(), 2.0);
        assert_eq!(scale.adjusted_point_size(10.0), 20.0);
        assert_eq!(scale.adjusted_handle_size(16.0), 32.0);
    }
}
