//! Text editor sorts system
//!
//! This module manages text sorts in the font editor, handling their placement,
//! rendering, input processing, and entity lifecycle management.

pub mod cursor;
pub mod input_utilities;
pub mod keyboard_input;
pub mod point_entities;
pub mod rtl_shaping;
pub mod sort_entities;
pub mod sort_placement;
pub mod unicode_input;

// Re-export all sort system functionality
// TODO: Phase 3 - These modules have mixed pub/pub(crate) visibility that needs careful refactoring
// Many of these are tightly coupled internal systems that shouldn't be public
// For now, keeping wildcards to avoid breaking the text editor plugin
pub use cursor::*;
pub use input_utilities::*;
pub use keyboard_input::*;
pub use point_entities::*;
pub use rtl_shaping::*;
pub use sort_entities::*;
pub use sort_placement::*;
pub use unicode_input::*;
