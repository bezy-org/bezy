//! Editing Functionality
//!
//! This module contains all editing-related functionality:
//! - Edit sessions for managing editing state
//! - Selection management for points, paths, and objects
//! - Undo/redo system for reversible operations
//! - Sort system for movable type placement and editing


pub mod edit_session;
pub mod selection;
pub mod smooth_curves;
pub mod sort;
pub mod system_sets;
pub mod text_editor_plugin;

// Re-export commonly used items
pub use edit_session::EditSessionPlugin;
pub use selection::SelectionPlugin;
pub use sort::SortPlugin;
pub use system_sets::{FontEditorSets, FontEditorSystemSetsPlugin};
pub use text_editor_plugin::TextEditorPlugin;
