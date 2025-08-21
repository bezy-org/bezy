//! Editing Functionality
//!
//! This module contains all editing-related functionality:
//! - Edit sessions for managing editing state
//! - Edit types and mode definitions
//! - Selection management for points, paths, and objects
//! - Undo/redo system for reversible operations
//! - Sort system for movable type placement and editing

#![allow(unused_imports)]

pub mod edit_session;
pub mod edit_type;
pub mod selection;
pub mod sort;
pub mod sort_plugin;
pub mod system_sets;
pub mod text_editor_plugin;
pub mod undo;
pub mod undo_plugin;

// Re-export commonly used items
pub use edit_session::EditSessionPlugin;
pub use selection::SelectionPlugin;
pub use sort_plugin::SortPlugin;
pub use system_sets::{FontEditorSets, FontEditorSystemSetsPlugin};
pub use text_editor_plugin::TextEditorPlugin;
pub use undo_plugin::UndoPlugin;
