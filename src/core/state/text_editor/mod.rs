//! Text editor module with gap buffer implementation
//!
//! This module provides text editing functionality for font editing operations.
//! It's split into multiple files for better organization:
//! - `buffer.rs`: Gap buffer implementation and data types
//! - `editor.rs`: Text editing operations and state management

pub mod buffer;
pub mod editor;
pub mod text_buffer;

// Re-export main types for public API compatibility
pub use buffer::{
    ActiveSortEntity, GridConfig, SortBuffer, SortData, SortKind, SortLayoutMode, TextEditorState,
    TextModeConfig,
};

// Re-export new buffer-level types
pub use text_buffer::{ActiveTextBuffer, BufferCursor, BufferMember, BufferSystemSet, TextBuffer};
