//! Debug utilities for selection system

use crate::editing::selection::components::SelectionRect;
use bevy::prelude::*;

/// Debug system to print selection rectangle information
pub fn debug_print_selection_rects(selection_rects: Query<&SelectionRect>) {
    for (i, rect) in selection_rects.iter().enumerate() {
        debug!(
            "SelectionRect {}: start=({:.1}, {:.1}), end=({:.1}, {:.1})",
            i, rect.start.x, rect.start.y, rect.end.x, rect.end.y
        );
    }
}

/// Debug system to validate point entity uniqueness
pub fn debug_validate_point_entity_uniqueness(
    point_entities: Query<
        (
            Entity,
            &crate::editing::selection::components::GlyphPointReference,
        ),
        With<crate::systems::sort_manager::SortPointEntity>,
    >,
) {
    let mut seen_points = std::collections::HashSet::new();
    let mut duplicates = 0;

    for (entity, glyph_ref) in point_entities.iter() {
        let key = (
            &glyph_ref.glyph_name,
            glyph_ref.contour_index,
            glyph_ref.point_index,
        );

        if !seen_points.insert(key) {
            warn!(
                "Duplicate point entity {:?} for glyph '{}' contour {} point {}",
                entity, glyph_ref.glyph_name, glyph_ref.contour_index, glyph_ref.point_index
            );
            duplicates += 1;
        }
    }

    if duplicates > 0 {
        error!("Found {} duplicate point entities!", duplicates);
    }
}

/// Debug utility to print selection state information
pub fn debug_selection_state(
    selection_state: Res<crate::editing::selection::components::SelectionState>,
) {
    if !selection_state.selected.is_empty() {
        debug!(
            "Current selection: {} entities",
            selection_state.selected.len()
        );
    }
}
