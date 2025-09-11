//! Text buffer ECS components and systems
//!
//! This module implements buffer-level storage using ECS entities instead of
//! storing buffer metadata in individual sorts. Each text buffer becomes an
//! ECS entity with its own components for cursor position, layout mode, etc.

use crate::core::state::text_editor::buffer::BufferId;
use crate::core::state::text_editor::SortLayoutMode;
use bevy::prelude::*;

/// Component that marks an entity as a text buffer
#[derive(Component, Debug, Clone)]
pub struct TextBuffer {
    /// Unique buffer identifier
    pub id: BufferId,
    /// Text direction and layout mode
    pub layout_mode: SortLayoutMode,
    /// World position where this buffer starts (root position)
    pub root_position: Vec2,
    /// Whether this buffer is currently active for editing
    pub is_active: bool,
}

/// Component that stores cursor position for a text buffer
#[derive(Component, Debug, Clone)]
pub struct BufferCursor {
    /// Cursor position within the buffer (0 = before first character)
    pub position: usize,
}

/// Component that links a sort entity to its parent buffer entity
#[derive(Component, Debug, Clone, Copy)]
pub struct BufferMember {
    /// The buffer entity this sort belongs to
    pub buffer_entity: Entity,
    /// Index of this sort within the buffer (0 = first/root sort)
    pub buffer_index: usize,
}

/// Resource to track the currently active buffer for insert mode
#[derive(Resource, Default, Debug)]
pub struct ActiveTextBuffer {
    /// The entity of the currently active buffer, if any
    pub buffer_entity: Option<Entity>,
}

/// System set for buffer-related operations
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum BufferSystemSet {
    /// Update buffer state
    UpdateBuffers,
    /// Sync buffer membership
    SyncMembership,
    /// Render buffer elements
    RenderBuffers,
}

impl TextBuffer {
    /// Create a new text buffer with the given parameters
    pub fn new(id: BufferId, layout_mode: SortLayoutMode, root_position: Vec2) -> Self {
        Self {
            id,
            layout_mode,
            root_position,
            is_active: false,
        }
    }
}

impl BufferCursor {
    /// Create a new buffer cursor at the specified position
    pub fn new(position: usize) -> Self {
        Self { position }
    }

    /// Create a cursor at the end of a buffer with the given length
    pub fn at_end(buffer_length: usize) -> Self {
        Self {
            position: buffer_length,
        }
    }
}

impl BufferMember {
    /// Create a new buffer membership component
    pub fn new(buffer_entity: Entity, buffer_index: usize) -> Self {
        Self {
            buffer_entity,
            buffer_index,
        }
    }

    /// Check if this sort is the root sort (first in buffer)
    pub fn is_root(&self) -> bool {
        self.buffer_index == 0
    }
}
