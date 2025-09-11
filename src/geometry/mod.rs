//! Geometric Primitives and Operations

pub mod bezpath_editing;
pub mod world_space;
pub mod point;
pub mod quadrant;

// Re-export commonly used items
pub use world_space::{DPoint, DVec2};
