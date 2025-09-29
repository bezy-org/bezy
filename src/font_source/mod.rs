//! Font source data structures and state management
//!
//! This module contains everything related to the font files being edited
//! (UFO, designspace, etc.), as opposed to UI fonts used by the editor.

pub mod data;
pub mod fontir_state;
pub mod metrics;
pub mod ufo_point;

#[cfg(test)]
mod tests;

// Explicit re-exports for public API
// Data structures
pub use data::{ComponentData, ContourData, FontData, GlyphData, OutlineData, PointData, PointTypeData};
// FontIR state
pub use fontir_state::{EditableGlyphInstance, FontIRAppState, FontIRMetrics};
// Metrics
pub use metrics::{FontInfo, FontMetrics};
// UFO point types
pub use ufo_point::{UfoPoint, UfoPointComponent, UfoPointType};
