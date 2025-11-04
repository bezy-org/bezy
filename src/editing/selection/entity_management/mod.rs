//! Entity management for selection system

pub mod spawning;
pub mod sync;

// Explicit re-exports
pub use spawning::{cleanup_click_resource, despawn_inactive_sort_points, spawn_active_sort_points};
pub use sync::{
    EnhancedPointAttributes, sync_enhanced_point_attributes,
    update_glyph_data_from_selection,
};
