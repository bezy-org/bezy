//! Sort management module
//!
//! Contains all sort-related editing functionality including components,
//! management systems, and plugin registration.

pub mod components;
pub mod manager;
pub mod plugin;

// Explicit re-exports for public API
// Components
pub use components::{ActiveSort, ActiveSortState, InactiveSort, Sort, SortBounds, SortEvent};
// Manager functionality
pub use manager::{
    NewlySpawnedCrosshair, SortCrosshair, SortPointEntity, auto_activate_first_sort,
    handle_glyph_navigation_changes, handle_sort_events, respawn_sort_points_on_glyph_change,
    spawn_current_glyph_sort, spawn_point_entities_for_sort, spawn_sort_point_entities,
};
// Plugin
pub use plugin::{SortPlugin, SortSystemSet};
