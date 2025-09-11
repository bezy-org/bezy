//! Sort Plugin
//!
//! Bevy plugin that registers all sort-related systems, resources, and events.

use crate::core::state::{AppState, GlyphNavigation};
use crate::editing::selection::components::{
    GlyphPointReference, PointType, Selectable, Selected, SelectionState,
};
use crate::editing::selection::nudge::PointCoordinates;
use crate::editing::sort::{ActiveSort, ActiveSortState, InactiveSort, Sort, SortEvent};
use crate::rendering::cameras::DesignCamera;
use crate::rendering::sort_renderer::{
    manage_sort_labels, update_sort_label_colors, update_sort_label_positions,
};
use crate::systems::sort_manager::{
    handle_sort_events, respawn_sort_points_on_glyph_change, spawn_current_glyph_sort,
};
use bevy::prelude::*;

/// System sets for Sort management to ensure proper ordering
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SortSystemSet {
    Input,
    Management,
    PointSpawning,
    Rendering,
}

pub struct SortPlugin;

impl Plugin for SortPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins(SelectionPlugin)
            .init_resource::<ActiveSortState>()
            .add_event::<SortEvent>()
            // Configure system sets to run in proper order
            .configure_sets(
                Update,
                (
                    SortSystemSet::Management,
                    SortSystemSet::PointSpawning,
                    SortSystemSet::Rendering,
                )
                    .chain(), // Ensure they run in order
            )
            // Management systems (events, basic sort operations)
            .add_systems(
                Update,
                (handle_sort_events,).in_set(SortSystemSet::Management),
            )
            // Point spawning systems
            .add_systems(
                Update,
                (respawn_sort_points_on_glyph_change,).in_set(SortSystemSet::PointSpawning),
            )
            // Label management systems (text labels for sorts)
            .add_systems(
                Update,
                (
                    manage_sort_labels,
                    update_sort_label_positions,
                    update_sort_label_colors,
                )
                    .chain()
                    .in_set(SortSystemSet::Rendering),
            );
    }
}
