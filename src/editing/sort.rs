//! Sort: Central concept representing a movable type piece
//!
//! A sort is a term from metal typesetting referring to the entire piece of metal
//! used in movable type. In Bezy, a sort mirrors the contents of a .glif file
//! from the UFO format and can contain compound glyphs with components and metadata.
//!
//! Sorts have two modes:
//! - Active: The sort being edited, shows editable outlines
//! - Inactive: Shows rendered outline, not editable
//!
//! Only one sort can be active at a time.

use crate::core::state::SortLayoutMode;
use bevy::prelude::*;

/// Represents a sort (glyph instance) in the design space
#[derive(Component, Debug, Clone)]
pub struct Sort {
    pub glyph_name: String,
    pub layout_mode: SortLayoutMode, // NEW: Distinguish buffer vs freeform sorts
}

/// Marker component for sorts that are currently active (editable)
#[derive(Component)]
pub struct ActiveSort;

/// Marker component for sorts that are inactive (rendered but not editable)
#[derive(Component)]
pub struct InactiveSort;

/// Resource to track the currently active sort
#[derive(Resource, Default)]
pub struct ActiveSortState {
    /// The entity ID of the currently active sort, if any
    pub active_sort_entity: Option<Entity>,
}

/// Bounds of a sort in design space coordinates
#[derive(Component, Debug, Clone)]
pub struct SortBounds {
    pub min: Vec2, // bottom-left corner
    pub max: Vec2, // top-right corner
}

impl SortBounds {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }
}

/// Events for sort management
#[derive(Event)]
pub enum SortEvent {
    CreateSort {
        glyph_name: String,
        position: Vec2,
        layout_mode: SortLayoutMode, // NEW: Specify layout mode when creating sorts
    },
    DeleteSort {
        entity: Entity,
    },
    ActivateSort {
        entity: Entity,
    },
    DeactivateSort {
        entity: Entity,
    },
}
