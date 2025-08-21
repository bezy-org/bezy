//! Quadrant system for 2D positioning and selection handles
//!
//! This module provides a 9-point grid system (like a tic-tac-toe board) for
//! positioning elements and handling UI interactions like selection handles.

use bevy::prelude::*;

/// Nine positions in a 2D grid, used for selection handles and positioning
///
/// Think of this as a 3x3 grid:
///
/// ```text
/// TopLeft     Top     TopRight
/// Left        Center  Right  
/// BottomLeft  Bottom  BottomRight
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub enum Quadrant {
    #[default]
    Center,
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

#[allow(dead_code)]
static ALL_QUADRANTS: &[Quadrant] = &[
    Quadrant::TopLeft,
    Quadrant::Top,
    Quadrant::TopRight,
    Quadrant::Left,
    Quadrant::Center,
    Quadrant::Right,
    Quadrant::BottomLeft,
    Quadrant::Bottom,
    Quadrant::BottomRight,
];

impl Quadrant {
    /// Returns all quadrants for iteration
    #[allow(dead_code)]
    pub fn all() -> &'static [Quadrant] {
        ALL_QUADRANTS
    }

    /// Returns the opposite quadrant (TopRight becomes BottomLeft, etc.)
    ///
    /// Useful for anchoring resize handles - when dragging one corner,
    /// the opposite corner stays fixed.
    #[allow(dead_code)]
    pub fn inverse(self) -> Quadrant {
        self.invert_y().invert_x()
    }

    /// Flips the quadrant vertically (Top becomes Bottom, etc.)
    #[allow(dead_code)]
    fn invert_y(self) -> Quadrant {
        match self {
            Quadrant::TopRight => Quadrant::BottomRight,
            Quadrant::TopLeft => Quadrant::BottomLeft,
            Quadrant::Top => Quadrant::Bottom,
            Quadrant::BottomRight => Quadrant::TopRight,
            Quadrant::BottomLeft => Quadrant::TopLeft,
            Quadrant::Bottom => Quadrant::Top,
            other => other, // Center, Left, Right stay the same
        }
    }

    /// Flips the quadrant horizontally (Left becomes Right, etc.)
    #[allow(dead_code)]
    fn invert_x(self) -> Quadrant {
        match self {
            Quadrant::TopRight => Quadrant::TopLeft,
            Quadrant::TopLeft => Quadrant::TopRight,
            Quadrant::Left => Quadrant::Right,
            Quadrant::Right => Quadrant::Left,
            Quadrant::BottomRight => Quadrant::BottomLeft,
            Quadrant::BottomLeft => Quadrant::BottomRight,
            other => other, // Center, Top, Bottom stay the same
        }
    }

    /// Checks if this quadrant affects the X axis when resizing
    #[allow(dead_code)]
    pub fn modifies_x_axis(self) -> bool {
        !matches!(self, Quadrant::Top | Quadrant::Bottom | Quadrant::Center)
    }

    /// Checks if this quadrant affects the Y axis when resizing
    #[allow(dead_code)]
    pub fn modifies_y_axis(self) -> bool {
        !matches!(self, Quadrant::Left | Quadrant::Right | Quadrant::Center)
    }

    /// Determines which quadrant a point falls into within a rectangle
    #[allow(dead_code)]
    pub fn for_point_in_bounds(pt: Vec2, size: Vec2) -> Self {
        // Divide the rectangle into 3x3 zones
        let zone_x = size.x / 3.0;
        let zone_y = size.y / 3.0;

        // Determine which column (0, 1, or 2)
        let mouse_x = match pt.x {
            x if x < zone_x => 0,
            x if x >= zone_x && x < zone_x * 2.0 => 1,
            x if x >= zone_x * 2.0 => 2,
            _ => unreachable!(),
        };

        // Determine which row (0, 1, or 2)
        let mouse_y = match pt.y {
            y if y < zone_y => 0,
            y if y >= zone_y && y < zone_y * 2.0 => 1,
            y if y >= zone_y * 2.0 => 2,
            _ => unreachable!(),
        };

        // Map the 2D grid position to a quadrant
        match (mouse_x, mouse_y) {
            (0, 0) => Quadrant::TopLeft,
            (1, 0) => Quadrant::Top,
            (2, 0) => Quadrant::TopRight,
            (0, 1) => Quadrant::Left,
            (1, 1) => Quadrant::Center,
            (2, 1) => Quadrant::Right,
            (0, 2) => Quadrant::BottomLeft,
            (1, 2) => Quadrant::Bottom,
            (2, 2) => Quadrant::BottomRight,
            _ => unreachable!(),
        }
    }

    /// Gets the position of this quadrant within a rectangle
    #[allow(dead_code)]
    pub fn point_in_rect(self, bounds: Rect) -> Vec2 {
        let size = Vec2::new(bounds.width(), bounds.height());
        let origin = Vec2::new(bounds.min.x, bounds.min.y);

        let rel_point = match self {
            Quadrant::TopLeft => Vec2::new(0.0, 0.0),
            Quadrant::Top => Vec2::new(size.x / 2.0, 0.0),
            Quadrant::TopRight => Vec2::new(size.x, 0.0),
            Quadrant::Left => Vec2::new(0.0, size.y / 2.0),
            Quadrant::Center => Vec2::new(size.x / 2.0, size.y / 2.0),
            Quadrant::Right => Vec2::new(size.x, size.y / 2.0),
            Quadrant::BottomLeft => Vec2::new(0.0, size.y),
            Quadrant::Bottom => Vec2::new(size.x / 2.0, size.y),
            Quadrant::BottomRight => Vec2::new(size.x, size.y),
        };

        origin + rel_point
    }

    /// Gets the position in design space (where top-left is 0,0)
    ///
    /// Design space has origin at top-left, while Bevy uses bottom-left.
    /// This method handles the coordinate system conversion.
    #[allow(dead_code)]
    pub fn point_in_design_space_rect(self, rect: Rect) -> Vec2 {
        self.invert_y().point_in_rect(rect)
    }

    /// Calculates scaling for rectangle resize operations
    ///
    /// When dragging a resize handle, this determines how much to scale
    /// the rectangle while keeping the opposite corner anchored.
    #[allow(dead_code)]
    pub fn scale_design_space_rect(self, rect: Rect, drag: Vec2) -> Vec2 {
        // Ensure axis locking has already been applied
        assert_eq!(drag, self.lock_delta(drag));

        let start_point = self.point_in_design_space_rect(rect);
        let anchor_point = self.inverse().point_in_design_space_rect(rect);

        let original_delta = anchor_point - start_point;
        let new_delta = anchor_point - (start_point + drag);

        compute_scale(
            Vec2::new(original_delta.x.abs(), original_delta.y.abs()),
            Vec2::new(new_delta.x.abs(), new_delta.y.abs()),
        )
    }

    /// Constrains movement to appropriate axes for this quadrant
    ///
    /// For example, Top/Bottom quadrants only allow Y movement,
    /// Left/Right only allow X movement.
    #[allow(dead_code)]
    pub fn lock_delta(self, delta: Vec2) -> Vec2 {
        match self {
            // Vertical edges: only Y movement
            Quadrant::Top | Quadrant::Bottom => Vec2::new(0.0, delta.y),
            // Horizontal edges: only X movement
            Quadrant::Left | Quadrant::Right => Vec2::new(delta.x, 0.0),
            // Corners: allow both X and Y movement
            _ => delta,
        }
    }
}

/// Calculates scaling factors for rectangle resize operations
#[allow(dead_code)]
fn compute_scale(origin: Vec2, current: Vec2) -> Vec2 {
    let mut scale = Vec2::ONE;

    // Only scale if we have a non-zero dimension to work with
    if origin.x != 0.0 {
        scale.x = current.x / origin.x;
    }

    if origin.y != 0.0 {
        scale.y = current.y / origin.y;
    }

    scale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadrant_positioning() {
        let rect =
            Rect::from_corners(Vec2::new(10.0, 10.0), Vec2::new(100.0, 100.0));

        assert_eq!(
            Quadrant::BottomLeft.point_in_design_space_rect(rect),
            Vec2::new(10.0, 10.0)
        );

        assert_eq!(
            Quadrant::Center.point_in_design_space_rect(rect),
            Vec2::new(55.0, 55.0)
        );

        assert_eq!(
            Quadrant::TopRight.point_in_design_space_rect(rect),
            Vec2::new(100.0, 100.0)
        );

        assert_eq!(
            Quadrant::Top.point_in_design_space_rect(rect),
            Vec2::new(55.0, 100.0)
        );
    }
}
