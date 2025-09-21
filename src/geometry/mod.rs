//! Geometric Primitives and Operations

pub mod bezpath_editing;
pub mod point;
pub mod quadrant;
pub mod world_space;

// Re-export commonly used items
pub use world_space::{DPoint, DVec2};
