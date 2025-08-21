//! Text editor sorts system
//!
//! This module manages text sorts in the font editor, handling their placement,
//! rendering, input processing, and entity lifecycle management.

pub mod input_utilities;
pub mod keyboard_input;
pub mod point_entities;
pub mod sort_entities;
pub mod sort_placement;
pub mod sort_rendering;
pub mod sort_state;
pub mod unicode_input;

// Re-export commonly used functions
pub use input_utilities::*;
pub use keyboard_input::*;
pub use point_entities::*;
pub use sort_entities::*;
pub use sort_placement::*;
pub use sort_rendering::*;
pub use sort_state::*;
pub use unicode_input::*;
