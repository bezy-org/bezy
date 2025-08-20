//! Types of state modifications, for the purposes of undo.

use bevy::prelude::*;

/// Types of state modifications, for the purposes of undo.
///
/// Certain state modifications group together in undo; for instance when dragging
/// a point, each individual edit (each time we receive a mouse moved event)
/// is combined into a single edit representing the entire drag.
///
/// When a tool handles a modification to the state, it returns an `EditType` that describes
/// what sort of modification it made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum EditType {
    /// Any change that always gets its own undo group
    Normal,
    /// Nudge left using the left arrow key
    NudgeLeft,
    /// Nudge right using the right arrow key
    NudgeRight,
    /// Nudge up using the up arrow key
    NudgeUp,
    /// Nudge down using the down arrow key
    NudgeDown,
    /// An edit where a drag of some kind is in progress
    Drag,
    /// An edit that finishes a drag; it combines with the previous undo
    /// group, but not with any subsequent event
    DragUp,
}

impl EditType {
    /// Check if two edit types require a new undo group
    /// Different edit types or long pauses between edits should start new groups
    #[allow(dead_code)]
    pub fn needs_new_undo_group(self, other: EditType) -> bool {
        match (self, other) {
            // Make each nudge operation its own undo group
            (EditType::NudgeDown, EditType::NudgeDown) => true,
            (EditType::NudgeUp, EditType::NudgeUp) => true,
            (EditType::NudgeLeft, EditType::NudgeLeft) => true,
            (EditType::NudgeRight, EditType::NudgeRight) => true,

            // A drag and its completion are combined
            (EditType::Drag, EditType::Drag) => false,
            (EditType::Drag, EditType::DragUp) => false,
            // All other edit combinations create a new undo group
            _ => true,
        }
    }
}
