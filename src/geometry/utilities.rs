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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axis_lock_horizontal() {
        let pos = Vec2::new(100.0, 80.0);
        let relative_to = Vec2::new(50.0, 60.0);
        let result = axis_lock_position(pos, relative_to);
        // X diff is 50, Y diff is 20, so should lock to horizontal (Y)
        assert_eq!(result, Vec2::new(100.0, 60.0));
    }

    #[test]
    fn test_axis_lock_vertical() {
        let pos = Vec2::new(60.0, 120.0);
        let relative_to = Vec2::new(50.0, 60.0);
        let result = axis_lock_position(pos, relative_to);
        // X diff is 10, Y diff is 60, so should lock to vertical (X)
        assert_eq!(result, Vec2::new(50.0, 120.0));
    }

    #[test]
    fn test_position_with_grid_snap() {
        let cursor_pos = Vec2::new(123.4, 567.8);
        let result = calculate_final_position_with_constraints(
            cursor_pos,
            true,  // snap to grid
            10.0,  // grid size
            None,  // no axis lock
        );
        assert_eq!(result, Vec2::new(120.0, 570.0));
    }

    #[test]
    fn test_position_with_axis_lock() {
        let cursor_pos = Vec2::new(100.0, 80.0);
        let relative_to = Vec2::new(50.0, 60.0);
        let result = calculate_final_position_with_constraints(
            cursor_pos,
            false, // no grid snap
            1.0,   // grid size (unused)
            Some(relative_to), // axis lock
        );
        assert_eq!(result, Vec2::new(100.0, 60.0)); // locked horizontally
    }
}