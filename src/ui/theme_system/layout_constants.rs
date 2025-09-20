//! Layout and UI Constants
//!
//! This file contains UI layout constants, sizing values, and other non-theme-dependent
//! constants used throughout the application.

use bevy::prelude::*;

// =================================================================
// RENDERING AND VISUAL CONSTANTS
// =================================================================

pub const CHECKERBOARD_Z_LEVEL: f32 = 0.1;
pub const SELECTION_Z_DEPTH_OFFSET: f32 = 100.0;
pub const MIN_VISIBILITY_ZOOM: f32 = 0.01;
pub const GRID_SIZE_CHANGE_THRESHOLD: f32 = 1.25;
pub const VISIBLE_AREA_COVERAGE_MULTIPLIER: f32 = 1.2;
pub const MAX_SQUARES_PER_FRAME: usize = 2000;

/// Z-layer for filled glyphs (below outlines but above background)
pub const FILLED_GLYPH_Z: f32 = 7.0;

// Rendering constants
pub const GIZMO_LINE_WIDTH: f32 = 4.0;
pub const DEBUG_SHOW_ORIGIN_CROSS: bool = false;

// Point rendering constants - now moved to theme trait

// =================================================================
// WINDOW AND APPLICATION CONSTANTS
// =================================================================
// Window constants now moved to theme trait

// =================================================================
// TOOLBAR AND UI LAYOUT CONSTANTS
// =================================================================

pub const TOOLBAR_PADDING: f32 = 0.0;
pub const TOOLBAR_CONTAINER_MARGIN: f32 = 16.0;
// TOOLBAR_BUTTON_SIZE moved to theme trait
pub const TOOLBAR_BORDER_WIDTH: f32 = 2.0;
pub const TOOLBAR_BORDER_RADIUS: f32 = 0.0;

/// Grid-based spacing between buttons - scales with button size
pub const TOOLBAR_GRID_SPACING: f32 = 64.0 * 0.0625; // 4px at 64px button size
/// Legacy constant for compatibility - use TOOLBAR_GRID_SPACING instead
pub const TOOLBAR_ITEM_SPACING: f32 = TOOLBAR_GRID_SPACING;
// BUTTON_ICON_SIZE moved to theme trait

/// Helper function to calculate submenu position below main toolbar
pub fn toolbar_submenu_top_position() -> f32 {
    TOOLBAR_CONTAINER_MARGIN + 64.0 + TOOLBAR_GRID_SPACING * 2.0
}

// =================================================================
// WIDGET AND PANE CONSTANTS
// =================================================================

pub const WIDGET_TEXT_FONT_SIZE: f32 = 20.0;
pub const WIDGET_TITLE_FONT_SIZE: f32 = 20.0;
// Widget constants moved to theme trait
pub const WIDGET_ROW_LEADING: f32 = 0.4; // Vertical spacing between rows in panes (negative for tighter spacing)

pub const LINE_LEADING: f32 = 0.0;

// =================================================================
// SORT AND LAYOUT CONSTANTS
// =================================================================

// Sort padding moved to theme trait

// Selection constants
pub const SELECTION_MARGIN: f32 = 16.0;

// =================================================================
// CAMERA AND ZOOM CONSTANTS
// =================================================================

pub const MIN_ALLOWED_ZOOM_SCALE: f32 = 0.1;
pub const MAX_ALLOWED_ZOOM_SCALE: f32 = 64.0;
pub const INITIAL_ZOOM_SCALE: f32 = 1.0;

// =================================================================
// FONT AND ASSET CONSTANTS
// =================================================================

// Font paths moved to theme trait

// =================================================================
// WIDGET CREATION FUNCTIONS
// =================================================================

/// Creates a consistent styled container for UI widgets/panes
/// Uses the current theme for colors
pub fn create_widget_style<T: Component + Default>(
    _asset_server: &Res<AssetServer>,
    theme: &Res<crate::ui::themes::CurrentTheme>,
    position: PositionType,
    position_props: UiRect,
    marker: T,
    name: &str,
) -> impl Bundle {
    use crate::ui::themes::WidgetBorderRadius;

    (
        Node {
            position_type: position,
            left: position_props.left,
            right: position_props.right,
            top: position_props.top,
            bottom: position_props.bottom,
            padding: UiRect::all(Val::Px(theme.theme().widget_padding())),
            margin: UiRect::all(Val::Px(0.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(WIDGET_ROW_LEADING),
            border: UiRect::all(Val::Px(theme.theme().widget_border_width())),
            width: Val::Auto,
            height: Val::Auto,
            min_width: Val::Auto,
            min_height: Val::Auto,
            max_width: Val::Px(256.0),
            max_height: Val::Percent(50.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            ..default()
        },
        BackgroundColor(theme.theme().widget_background_color()),
        BorderColor(theme.theme().widget_border_color()),
        BorderRadius::all(Val::Px(theme.theme().widget_border_radius())),
        WidgetBorderRadius,
        marker,
        Name::new(name.to_string()),
    )
}
