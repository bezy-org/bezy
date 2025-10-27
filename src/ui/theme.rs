//! UI Theme Interface
//!
//! This file serves as the main entry point for the theme system.
//! All theme-related constants, functions, and utilities are now organized
//! in the theme_system directory and re-exported here for compatibility.
//!
//! For creating custom themes, see the themes/ directory and docs/THEME_CREATION_GUIDE.md

// Re-export the main theme system components
pub use crate::ui::theme_system::{
    get_theme_registry, BezyTheme, CurrentTheme, ThemeRegistry, ThemeVariant,
    ToolbarBorderRadius, UiBorderRadius, WidgetBorderRadius,
    // Re-export all 22+ layout constants used throughout the UI
    CHECKERBOARD_Z_LEVEL, DEBUG_SHOW_ORIGIN_CROSS, FILLED_GLYPH_Z, GIZMO_LINE_WIDTH,
    GRID_SIZE_CHANGE_THRESHOLD, INITIAL_ZOOM_SCALE, LINE_LEADING, MAX_ALLOWED_ZOOM_SCALE,
    MAX_SQUARES_PER_FRAME, MIN_ALLOWED_ZOOM_SCALE, MIN_VISIBILITY_ZOOM, SELECTION_MARGIN,
    SELECTION_Z_DEPTH_OFFSET, TOOLBAR_BORDER_RADIUS, TOOLBAR_BORDER_WIDTH,
    TOOLBAR_CONTAINER_MARGIN, TOOLBAR_GRID_SPACING, TOOLBAR_PADDING,
    VISIBLE_AREA_COVERAGE_MULTIPLIER, WIDGET_ROW_LEADING, WIDGET_TEXT_FONT_SIZE,
    WIDGET_TITLE_FONT_SIZE, create_widget_style, toolbar_submenu_top_position,
};
