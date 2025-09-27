//! Geometric Primitives and Operations

pub mod bezpath_editing;
pub mod point;
pub mod quadrant;
pub mod utilities;
pub mod world_space;

// Re-export commonly used items
pub use utilities::{axis_lock_position, calculate_final_position_with_constraints};
pub use world_space::{DPoint, DVec2};
