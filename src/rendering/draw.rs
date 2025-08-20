//! Drawing algorithms and helpers

#![allow(deprecated)]

use crate::core::state::{AppState, FontMetrics, GlyphNavigation};

use crate::rendering::cameras::DesignCamera;
use crate::ui::theme::DEBUG_SHOW_ORIGIN_CROSS;
use bevy::prelude::*;

/// System that draws the debug origin cross and square
pub fn draw_origin_cross(mut gizmos: Gizmos) {
    // Only draw the debug cross if enabled in theme settings
    if DEBUG_SHOW_ORIGIN_CROSS {
        let red = Color::srgb(1.0, 0.0, 0.0);

        // Draw a simple test cross at the origin using 2D gizmos to render on top of sorts
        gizmos.line_2d(Vec2::new(-64.0, 0.0), Vec2::new(64.0, 0.0), red);
        gizmos.line_2d(Vec2::new(0.0, -64.0), Vec2::new(0.0, 64.0), red);

        // Draw a 32x32 red square centered at origin
        gizmos.rect_2d(
            Vec2::ZERO,            // position (center)
            Vec2::new(32.0, 32.0), // size
            red,
        );
    }
}
