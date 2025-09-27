//! Point and entity management for glyph editing
//!
//! This module provides the core structures for working with individual points
//! and entities within a glyph's outline.

use crate::core::state::font_data::PointTypeData;
use bevy::prelude::*;
use kurbo::Point;

/// A point in a glyph's outline that can be edited
#[derive(Component, Debug, Clone, PartialEq)]
pub struct EditPoint {
    pub position: Point,           // Position in glyph coordinate space
    pub point_type: PointTypeData, // Point type (move, line, curve, etc.)
}


/// Unique identifier for entities in a glyph (points, guides, components)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub struct EntityId {
    parent: u32,      // The parent path or component ID
    index: u16,       // The index within the parent
    kind: EntityKind, // The type of entity this ID refers to
}

/// The different types of entities that can exist in a glyph
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub enum EntityKind {
    Point,     // A point in a contour path
    Guide,     // A guide line for alignment
    Component, // A component reference to another glyph
}

