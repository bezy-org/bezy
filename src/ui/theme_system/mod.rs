//! Theme system infrastructure
//!
//! This module contains the core theming infrastructure:
//! - BezyTheme trait defining all theme properties
//! - ThemeRegistry for managing available themes
//! - CurrentTheme resource for runtime theme access
//! - Layout constants used throughout the UI
//!
//! Actual theme definitions live in ../themes/

pub mod core;
pub mod layout_constants;

// Re-export commonly used items
pub use core::{get_theme_registry, BezyTheme, CurrentTheme, ThemeRegistry, ThemeVariant};

// Re-export ALL layout constants - these are genuine public constants used throughout UI
// There are 22+ constants that are widely used across the codebase
pub use layout_constants::{
    CHECKERBOARD_Z_LEVEL, DEBUG_SHOW_ORIGIN_CROSS, FILLED_GLYPH_Z, GIZMO_LINE_WIDTH,
    GRID_SIZE_CHANGE_THRESHOLD, INITIAL_ZOOM_SCALE, LINE_LEADING, MAX_ALLOWED_ZOOM_SCALE,
    MAX_SQUARES_PER_FRAME, MIN_ALLOWED_ZOOM_SCALE, MIN_VISIBILITY_ZOOM, SELECTION_MARGIN,
    SELECTION_Z_DEPTH_OFFSET, TOOLBAR_BORDER_RADIUS, TOOLBAR_BORDER_WIDTH,
    TOOLBAR_CONTAINER_MARGIN, TOOLBAR_GRID_SPACING, TOOLBAR_PADDING,
    VISIBLE_AREA_COVERAGE_MULTIPLIER, WIDGET_ROW_LEADING, WIDGET_TEXT_FONT_SIZE,
    WIDGET_TITLE_FONT_SIZE, ToolbarBorderRadius, UiBorderRadius, WidgetBorderRadius,
    create_widget_style, toolbar_submenu_top_position,
};
