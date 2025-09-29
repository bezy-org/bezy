//! Geometry utility functions
//!
//! Shared geometry functions to avoid code duplication across the codebase.
//! Functions for coordinate conversion, position calculation, and transformations.

use bevy::prelude::*;

/// Lock a position to horizontal or vertical axis relative to another point
/// (used when shift is held to constrain movement)
pub fn axis_lock_position(pos: Vec2, relative_to: Vec2) -> Vec2 {
    let dxy = pos - relative_to;
    if dxy.x.abs() > dxy.y.abs() {
        Vec2::new(pos.x, relative_to.y)
    } else {
        Vec2::new(relative_to.x, pos.y)
    }
}

/// Apply grid snap and axis locking to a position
///
/// This is the common logic used by various tools for position calculation
pub fn calculate_final_position_with_constraints(
    cursor_pos: Vec2,
    snap_to_grid: bool,
    grid_size: f32,
    axis_lock: Option<Vec2>,
) -> Vec2 {
    // Apply snap to grid first
    let snapped_pos = if snap_to_grid {
        Vec2::new(
            (cursor_pos.x / grid_size).round() * grid_size,
            (cursor_pos.y / grid_size).round() * grid_size,
        )
    } else {
        cursor_pos
    };

    // Apply axis locking if requested
    if let Some(relative_to) = axis_lock {
        axis_lock_position(snapped_pos, relative_to)
    } else {
        snapped_pos
    }
}
