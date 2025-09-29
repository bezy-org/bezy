//! Rendering and Visualization
//!
//! This module contains all rendering and visualization functionality:
//! - Drawing systems for glyphs, paths, and UI elements
//! - Camera management for viewport control
//! - Background patterns and visual aids
//! - Debug visualization tools
//! - Selection visualization (marquee, selected points, handles)


pub mod cameras;
pub mod checkerboard;
pub mod entity_pools;
pub mod glyph_renderer;
pub mod mesh_cache;
pub mod mesh_utils;
pub mod metrics;
pub mod outline_elements;
pub mod points;
pub mod post_editing_systems;
pub mod selection;
pub mod sort_renderer;
pub mod sort_visuals;
pub mod text_cursor;
pub mod zoom_aware_scaling;

// Re-export commonly used items
pub use checkerboard::{CheckerboardEnabled, CheckerboardPlugin};
pub use entity_pools::EntityPoolingPlugin;
pub use glyph_renderer::GlyphRenderingPlugin;
pub use mesh_cache::MeshCachingPlugin;
pub use metrics::MetricsRenderingPlugin;
pub use outline_elements::OutlineElementsPlugin;
pub use points::PointRenderingPlugin;
pub use post_editing_systems::{PostEditingRenderingPlugin, PostEditingRenderingSet};
pub use selection::{
    render_all_point_entities, render_selected_entities, render_selection_marquee,
};
pub use sort_visuals::SortHandleRenderingPlugin;
pub use text_cursor::{CursorRenderingState, TextEditorCursor};
pub use zoom_aware_scaling::{CameraResponsivePlugin, CameraResponsiveScale};
