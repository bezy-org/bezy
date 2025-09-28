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

impl Quadrant {}
