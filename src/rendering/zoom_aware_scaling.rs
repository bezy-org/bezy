//! Zoom-aware scaling system for mesh-based rendering
//!
//! This module provides automatic scaling for mesh-rendered elements
//! (points, lines, handles) to maintain visibility at all zoom levels.
//! Similar to how professional font editors keep UI elements visible
//! when zoomed out instead of letting them get teeny-tiny.

use crate::rendering::cameras::DesignCamera;
use bevy::prelude::*;

/// Zoom-responsive scaling behavior
#[derive(Debug, Clone)]
pub struct ZoomScaleConfig {
    /// Scale multiplier when at maximum zoom in
    pub zoom_in_max_factor: f32,
    /// Scale multiplier at default zoom (100%)
    pub zoom_default_factor: f32,
    /// Scale multiplier when at maximum zoom out
    pub zoom_out_max_factor: f32,
}

/// Configuration for zoom-responsive scaling behavior
impl Default for ZoomScaleConfig {
    fn default() -> Self {
        Self {
            // TODO(human): Adjust these values to tune point and line sizes at different zoom levels
            // These control how large points and lines appear relative to the glyph at different zooms:
            // - Lower values = smaller/thinner (more subtle)
            // - Higher values = larger/thicker (more prominent)
            // Test by zooming in/out and observing point/line visibility
            zoom_in_max_factor: 1.0,  // size when max zoomed in
            zoom_default_factor: 1.5, // size at default zoom (100%)
            zoom_out_max_factor: 8.0, // size when max zoomed out
        }
    }
}

/// Camera zoom ranges for interpolation
#[derive(Debug, Clone)]
pub struct ZoomRanges {
    /// Camera scale at maximum zoom in
    pub max_zoom_in: f32,
    /// Camera scale at default zoom (100%)
    pub default_zoom: f32,
    /// Camera scale at maximum zoom out
    pub max_zoom_out: f32,
}

impl Default for ZoomRanges {
    fn default() -> Self {
        Self {
            max_zoom_in: 0.2,   // Maximum zoom in
            default_zoom: 1.0,  // Default zoom level
            max_zoom_out: 16.0, // Maximum zoom out
        }
    }
}

/// Resource that tracks the current camera-responsive scale factor
#[derive(Resource, Default)]
pub struct CameraResponsiveScale {
    /// Current scale factor to apply to visual elements
    /// 1.0 = base size, >1.0 = bigger, <1.0 = smaller
    /// This field is private as it's automatically updated every frame
    scale_factor: f32,
    /// Base line width in world units at normal zoom
    pub base_line_width: f32,
    /// Configuration for scale factors at different zoom levels
    pub config: ZoomScaleConfig,
    /// Camera zoom ranges
    pub ranges: ZoomRanges,
}

impl CameraResponsiveScale {
    pub fn new() -> Self {
        Self {
            scale_factor: 1.0,
            base_line_width: 1.0,
            config: ZoomScaleConfig::default(),
            ranges: ZoomRanges::default(),
        }
    }

    /// Creates a new instance with custom configuration
    pub fn with_config(config: ZoomScaleConfig, ranges: ZoomRanges) -> Self {
        Self {
            scale_factor: 1.0,
            base_line_width: 1.0,
            config,
            ranges,
        }
    }

    /// Gets the current scale factor
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// Calculates scale factor based on camera zoom level
    pub fn calculate_scale_factor(&self, camera_scale: f32) -> f32 {
        let ranges = &self.ranges;
        let config = &self.config;

        if camera_scale <= ranges.default_zoom {
            // Interpolate between max zoom in and default
            lerp_scale(
                camera_scale,
                ranges.max_zoom_in,
                ranges.default_zoom,
                config.zoom_in_max_factor,
                config.zoom_default_factor,
            )
        } else {
            // Interpolate between default and max zoom out
            lerp_scale(
                camera_scale,
                ranges.default_zoom,
                ranges.max_zoom_out,
                config.zoom_default_factor,
                config.zoom_out_max_factor,
            )
        }
    }

    /// Get the adjusted line width based on camera zoom
    pub fn adjusted_line_width(&self) -> f32 {
        self.base_line_width * self.scale_factor
    }

    /// Get the adjusted size for any element
    pub fn adjusted_size(&self, base_size: f32) -> f32 {
        base_size * self.scale_factor
    }
}

/// Linear interpolation between two scale factors
fn lerp_scale(
    current: f32,
    range_start: f32,
    range_end: f32,
    factor_start: f32,
    factor_end: f32,
) -> f32 {
    let t = ((current - range_start) / (range_end - range_start)).clamp(0.0, 1.0);
    factor_start * (1.0 - t) + factor_end * t
}

/// Updates the camera-responsive scale based on current zoom
pub fn update_camera_responsive_scale(
    mut scale_res: ResMut<CameraResponsiveScale>,
    camera_q: Query<&Projection, With<DesignCamera>>,
) {
    let Ok(projection) = camera_q.single() else {
        return;
    };

    let camera_scale = match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => 1.0,
    };

    scale_res.scale_factor = scale_res.calculate_scale_factor(camera_scale);
}

/// Types of visual elements that respond to camera zoom
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResponsiveElementType {
    Line,
    Point,
    Handle,
}

/// Component marking entities that should scale with zoom
#[derive(Component)]
pub struct CameraResponsive {
    pub element_type: ResponsiveElementType,
    pub base_size: f32,
}

impl CameraResponsive {
    pub fn new(element_type: ResponsiveElementType, base_size: f32) -> Self {
        Self {
            element_type,
            base_size,
        }
    }

    pub fn line(base_width: f32) -> Self {
        Self::new(ResponsiveElementType::Line, base_width)
    }

    pub fn point(base_size: f32) -> Self {
        Self::new(ResponsiveElementType::Point, base_size)
    }

    pub fn handle(base_size: f32) -> Self {
        Self::new(ResponsiveElementType::Handle, base_size)
    }
}

/// Applies camera-responsive scaling to marked entities
pub fn apply_camera_responsive_scaling(
    scale_res: Res<CameraResponsiveScale>,
    mut query: Query<(&CameraResponsive, &mut Transform), Changed<CameraResponsive>>,
) {
    if !scale_res.is_changed() {
        return;
    }

    for (responsive, mut transform) in &mut query {
        let new_scale = match responsive.element_type {
            ResponsiveElementType::Line => scale_res.adjusted_line_width(),
            ResponsiveElementType::Point | ResponsiveElementType::Handle => {
                scale_res.adjusted_size(responsive.base_size)
            }
        };

        transform.scale = Vec3::splat(new_scale);
    }
}

/// Plugin for camera-responsive scaling
pub struct CameraResponsivePlugin;

impl Plugin for CameraResponsivePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraResponsiveScale::new())
            .add_systems(
                Update,
                (
                    update_camera_responsive_scale,
                    apply_camera_responsive_scaling,
                )
                    .chain(),
            );
    }
}
