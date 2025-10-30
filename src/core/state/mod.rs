//! Application state management.
//!
//! This module contains core application state. Font source editing data
//! has been moved to the font_source module for clarity.

// Core state modules that remain here
pub mod app_state;
pub mod navigation;
pub mod text_editor;

// Re-export core state
pub use app_state::*;
pub use navigation::*;
pub use text_editor::*;

// TEMPORARY: Re-export font_source items for backward compatibility
// TODO: Update all imports to use font_source directly, then remove these
pub use crate::font_source::{
    ComponentData, ContourData, FontData,
    FontInfo, FontMetrics, GlyphData, OutlineData, PointData, PointTypeData, UfoPoint,
    UfoPointComponent, UfoPointType,
};

// Keep old module names for now to avoid breaking imports
pub mod font_data {
    pub use crate::font_source::{
        ComponentData, ContourData, FontData, GlyphData, OutlineData, PointData, PointTypeData,
    };
}
pub mod font_metrics {
    pub use crate::font_source::{FontInfo, FontMetrics};
}
pub mod ufo_point {
    pub use crate::font_source::{UfoPoint, UfoPointComponent, UfoPointType};
}
