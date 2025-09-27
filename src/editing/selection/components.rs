use bevy::prelude::*;
use std::collections::BTreeSet;

//-----------------------------------------------------------------------
// Selection System Components
//-----------------------------------------------------------------------
// This file defines the core components and state used by the selection
// system, which handles user interactions with on-screen elements such as
// glyph points, curves, and other UI elements. The system allows users to:
// - Select individual entities
// - Multi-select using modifier keys
// - Perform rectangular selection via click and drag
// - Track hover state for improved UX feedback
//
// ARCHITECTURE OVERVIEW:
// The selection system uses a component-based approach where:
// 1. `Selectable` marks entities that can be interacted with
// 2. `Selected` and `Hovered` are added/removed dynamically to track state
// 3. `SelectionState` resource manages the global selection state
// 4. `SelectionRect` represents an in-progress rectangular selection
//
// Typical workflow:
// - Mouse hover -> Add `Hovered` component
// - Mouse click on entity -> Add `Selected` component + update `SelectionState`
// - Shift+click -> Toggle selection state (multi-select)
// - Click and drag -> Create `SelectionRect` and select all contained entities
// - Click on empty space -> Clear selection

/// Marker component for entities that can be selected
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Selectable;

/// Marker component for entities that are currently selected
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Selected;


/// Component for the selection rectangle during drag operations
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SelectionRect {
    /// The start position of the selection rectangle
    pub start: Vec2,
    /// The end position of the selection rectangle
    pub end: Vec2,
}

/// The type of point (on-curve or off-curve)
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct PointType {
    /// Whether this is an on-curve point
    pub is_on_curve: bool,
}

/// Component that links an entity to a specific point in a glyph outline
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct GlyphPointReference {
    /// Name of the glyph this point belongs to
    pub glyph_name: String,
    /// Index of the contour within the glyph outline
    pub contour_index: usize,
    /// Index of the point within the contour
    pub point_index: usize,
}

/// Component that links an entity to a specific point in a FontIR BezPath
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct FontIRPointReference {
    /// Name of the glyph this point belongs to
    pub glyph_name: String,
    /// Which path (contour) this point belongs to
    pub path_index: usize,
    /// Reference to the specific point in the BezPath
    pub point_ref: crate::geometry::bezpath_editing::PathPointRef,
}

impl Default for PointType {
    fn default() -> Self {
        Self { is_on_curve: true }
    }
}

/// Resource to track the global selection state
#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct SelectionState {
    /// Whether multi-select mode is active (e.g., shift key is held)
    pub multi_select: bool,
    /// The currently selected entities
    /// Using a public field without reflect(ignore) to make it more accessible to other systems
    #[reflect(ignore)]
    pub selected: BTreeSet<Entity>,
}

