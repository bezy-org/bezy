//! Font source data structures and state management
//!
//! This module contains everything related to the font files being edited
//! (UFO, designspace, etc.), as opposed to UI fonts used by the editor.

pub mod data;
pub mod fontir_state;
pub mod metrics;
pub mod ufo_point;

// Keep re-exports simple and explicit for clarity
pub use data::*; // Exports FontData and all related types
pub use fontir_state::{EditableGlyphInstance, FontIRAppState, FontIRMetrics};
pub use metrics::{FontInfo, FontMetrics};
pub use ufo_point::{UfoPoint, UfoPointComponent, UfoPointType};
