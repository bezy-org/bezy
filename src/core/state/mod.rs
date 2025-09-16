//! Application state management.
//!
//! This module defines thread-safe data structures optimized for our font editor.
//! We use norad only for loading/saving UFO files, not as runtime storage.
//!
//! The main AppState resource contains all font data in a format optimized for
//! real-time editing operations. Changes are batched and synchronized with the
//! UFO format only when saving.

// Sub-modules
pub mod app_state;
pub mod font_data;
pub mod font_metrics;
pub mod fontir_app_state;
pub mod navigation;
pub mod text_editor;
pub mod ufo_point;

#[cfg(test)]
mod test_components;

// Re-export all public items to maintain the existing API
pub use app_state::*;
pub use font_data::*;
pub use font_metrics::*;
pub use fontir_app_state::*;
pub use navigation::*;
pub use text_editor::*;
pub use ufo_point::*;
