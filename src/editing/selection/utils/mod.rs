//! Utility functions for selection system

pub mod debug;
pub mod state;

// Re-export public utilities
pub use state::clear_selection_on_app_change;

// Debug utilities are internal (pub(crate)) and not re-exported
pub(crate) use debug::{debug_print_selection_rects, debug_validate_point_entity_uniqueness};
