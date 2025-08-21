//! UI Theme file
//!
//! This file provides the theme system interface for the Bezy font editor.
//! All visual styling constants are now provided through the theme system.
//!
//! For creating custom themes, see the themes/ directory and docs/THEME_CREATION_GUIDE.md

#![allow(unused_variables)]

use bevy::prelude::*;
#[allow(unused_imports)]
use bevy::ui::prelude::*;

// Re-export the theme system
pub use crate::ui::themes::*;

// Helper function to get current theme
pub fn get_current_theme<'a>(
    theme_res: &'a Res<CurrentTheme>,
) -> &'a dyn BezyTheme {
    theme_res.theme()
}

// =================================================================
// THEME-BASED FUNCTIONS
// These functions get values from the current theme.
// Use these instead of hardcoded constants!
// =================================================================

/// Get widget background color from current theme
pub fn get_widget_background_color(theme: &Res<CurrentTheme>) -> Color {
    theme.theme().widget_background_color()
}

/// Get widget border color from current theme  
pub fn get_widget_border_color(theme: &Res<CurrentTheme>) -> Color {
    theme.theme().widget_border_color()
}

/// Get normal text color from current theme
pub fn get_normal_text_color(theme: &Res<CurrentTheme>) -> Color {
    theme.theme().normal_text_color()
}

/// Get secondary text color from current theme
pub fn get_secondary_text_color(theme: &Res<CurrentTheme>) -> Color {
    theme.theme().secondary_text_color()
}

/// Get highlight text color from current theme
pub fn get_highlight_text_color(theme: &Res<CurrentTheme>) -> Color {
    theme.theme().highlight_text_color()
}

/// Get background color from current theme
pub fn get_background_color(theme: &Res<CurrentTheme>) -> Color {
    theme.theme().background_color()
}

// =================================================================
// COMPATIBILITY CONSTANTS FOR MIGRATION
// These provide the same names as before but get values from theme
// =================================================================

/// Checkerboard constants using theme values
pub fn get_checkerboard_constants(
    theme: &Res<CurrentTheme>,
) -> CheckerboardConstants {
    let t = theme.theme();
    CheckerboardConstants {
        color: t.checkerboard_color(),
        default_unit_size: t.checkerboard_default_unit_size(),
        scale_factor: t.checkerboard_scale_factor(),
        max_zoom_visible: t.checkerboard_max_zoom_visible(),
        enabled_by_default: t.checkerboard_enabled_by_default(),
    }
}

pub struct CheckerboardConstants {
    pub color: Color,
    pub default_unit_size: f32,
    pub scale_factor: f32,
    pub max_zoom_visible: f32,
    pub enabled_by_default: bool,
}

// Constants that can be accessed directly (don't change with theme)
pub const CHECKERBOARD_Z_LEVEL: f32 = 0.1;
pub const SELECTION_Z_DEPTH_OFFSET: f32 = 100.0;
pub const MIN_VISIBILITY_ZOOM: f32 = 0.01;
pub const GRID_SIZE_CHANGE_THRESHOLD: f32 = 1.25;
pub const VISIBLE_AREA_COVERAGE_MULTIPLIER: f32 = 1.2;
pub const MAX_SQUARES_PER_FRAME: usize = 2000;

// Window constants (these could be theme-aware in future)
pub const WINDOW_WIDTH: f32 = 1024.0;
pub const WINDOW_HEIGHT: f32 = 768.0;
pub const WINDOW_TITLE: &str = "Bezy";

// Rendering constants
pub const GIZMO_LINE_WIDTH: f32 = 4.0;
pub const DEBUG_SHOW_ORIGIN_CROSS: bool = false;

// Point rendering constants - these should use theme
pub const ON_CURVE_POINT_RADIUS: f32 = 4.0;
pub const OFF_CURVE_POINT_RADIUS: f32 = 4.0;
pub const ON_CURVE_SQUARE_ADJUSTMENT: f32 = 1.0;
pub const ON_CURVE_INNER_CIRCLE_RATIO: f32 = 0.25;
pub const OFF_CURVE_INNER_CIRCLE_RATIO: f32 = 0.25;
pub const USE_SQUARE_FOR_ON_CURVE: bool = true;

// Layout constants
pub const TOOLBAR_PADDING: f32 = 0.0;
pub const TOOLBAR_CONTAINER_MARGIN: f32 = 16.0;
pub const TOOLBAR_ITEM_SPACING: f32 = 4.0;
pub const TOOLBAR_BUTTON_SIZE: f32 = 64.0;
pub const BUTTON_ICON_SIZE: f32 = 48.0;

// Sort constants
pub const SORT_VERTICAL_PADDING: f32 = 256.0;
pub const SORT_HORIZONTAL_PADDING: f32 = 256.0;

// Selection constants
pub const SELECTION_MARGIN: f32 = 16.0;

// Camera constants
pub const MIN_ALLOWED_ZOOM_SCALE: f32 = 0.1;
pub const MAX_ALLOWED_ZOOM_SCALE: f32 = 64.0;
pub const INITIAL_ZOOM_SCALE: f32 = 1.0;

// =================================================================
// THEME-DEPENDENT CONSTANTS
// These return values from the current theme
// =================================================================

pub fn get_theme_dependent_constants(
    theme: &Res<CurrentTheme>,
) -> ThemeDependentConstants {
    let t = theme.theme();
    ThemeDependentConstants {
        // Colors that change with theme
        checkerboard_color: t.checkerboard_color(),
        checkerboard_default_unit_size: t.checkerboard_default_unit_size(),
        checkerboard_scale_factor: t.checkerboard_scale_factor(),
        checkerboard_max_zoom_visible: t.checkerboard_max_zoom_visible(),
        checkerboard_enabled_by_default: t.checkerboard_enabled_by_default(),

        // Point colors (two-layer system)
        handle_line_color: t.handle_line_color(),
        on_curve_primary_color: t.on_curve_primary_color(),
        on_curve_secondary_color: t.on_curve_secondary_color(),
        off_curve_primary_color: t.off_curve_primary_color(),
        off_curve_secondary_color: t.off_curve_secondary_color(),
        selected_primary_color: t.selected_primary_color(),
        selected_secondary_color: t.selected_secondary_color(),
        path_stroke_color: t.path_stroke_color(),

        // UI colors
        metrics_guide_color: t.metrics_guide_color(),
        sort_active_metrics_color: t.sort_active_metrics_color(),
        sort_inactive_metrics_color: t.sort_inactive_metrics_color(),

        // Button colors
        normal_button_color: t.normal_button_color(),
        hovered_button_color: t.hovered_button_color(),
        pressed_button_color: t.pressed_button_color(),
        normal_button_outline_color: t.normal_button_outline_color(),
        hovered_button_outline_color: t.hovered_button_outline_color(),
        pressed_button_outline_color: t.pressed_button_outline_color(),

        // Toolbar colors
        toolbar_icon_color: t.toolbar_icon_color(),
    }
}

pub struct ThemeDependentConstants {
    pub checkerboard_color: Color,
    pub checkerboard_default_unit_size: f32,
    pub checkerboard_scale_factor: f32,
    pub checkerboard_max_zoom_visible: f32,
    pub checkerboard_enabled_by_default: bool,

    pub handle_line_color: Color,
    pub on_curve_primary_color: Color,
    pub on_curve_secondary_color: Color,
    pub off_curve_primary_color: Color,
    pub off_curve_secondary_color: Color,
    pub selected_primary_color: Color,
    pub selected_secondary_color: Color,
    pub path_stroke_color: Color,

    pub metrics_guide_color: Color,
    pub sort_active_metrics_color: Color,
    pub sort_inactive_metrics_color: Color,

    pub normal_button_color: Color,
    pub hovered_button_color: Color,
    pub pressed_button_color: Color,
    pub normal_button_outline_color: Color,
    pub hovered_button_outline_color: Color,
    pub pressed_button_outline_color: Color,

    pub toolbar_icon_color: Color,
}

// Legacy constants for immediate compatibility
// TODO: These should be removed as files are migrated to use the theme system
pub const CHECKERBOARD_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.5);
pub const CHECKERBOARD_DEFAULT_UNIT_SIZE: f32 = 32.0;
pub const CHECKERBOARD_SCALE_FACTOR: f32 = 2.0;
pub const CHECKERBOARD_MAX_ZOOM_VISIBLE: f32 = 32.0;
pub const CHECKERBOARD_ENABLED_BY_DEFAULT: bool = true;

pub const HANDLE_LINE_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 1.0);

// Legacy two-color point constants for compatibility
pub const ON_CURVE_PRIMARY_COLOR: Color = Color::srgb(0.3, 1.0, 0.5);
pub const ON_CURVE_SECONDARY_COLOR: Color = Color::srgb(0.1, 0.3, 0.15);
pub const OFF_CURVE_PRIMARY_COLOR: Color = Color::srgb(0.6, 0.4, 1.0);
pub const OFF_CURVE_SECONDARY_COLOR: Color = Color::srgb(0.2, 0.15, 0.35);
pub const SELECTED_PRIMARY_COLOR: Color = Color::srgba(1.0, 1.0, 0.0, 1.0);
pub const SELECTED_SECONDARY_COLOR: Color = Color::srgba(0.5, 0.5, 0.0, 1.0);

pub const PATH_STROKE_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
/// Color for inactive sort outlines (slightly dimmed)
pub const INACTIVE_OUTLINE_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);
/// Color for filled glyph rendering (inactive text sorts)
pub const FILLED_GLYPH_COLOR: Color = Color::srgb(0.7, 0.7, 0.7);
/// Z-layer for filled glyphs (below outlines but above background)
pub const FILLED_GLYPH_Z: f32 = 7.0;

pub const METRICS_GUIDE_COLOR: Color = Color::srgba(0.3, 1.0, 0.5, 0.5);
pub const SORT_ACTIVE_METRICS_COLOR: Color = Color::srgba(0.3, 1.0, 0.5, 0.5);
pub const SORT_INACTIVE_METRICS_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 0.5);

pub const NORMAL_BUTTON_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);
pub const HOVERED_BUTTON_COLOR: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON_COLOR: Color = Color::srgb(1.0, 0.4, 0.0);
pub const NORMAL_BUTTON_OUTLINE_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
pub const HOVERED_BUTTON_OUTLINE_COLOR: Color = Color::srgb(0.75, 0.75, 0.75);
pub const PRESSED_BUTTON_OUTLINE_COLOR: Color = Color::srgb(1.0, 0.8, 0.3);

pub const TOOLBAR_ICON_COLOR: Color = Color::srgb(0.75, 0.75, 0.75);

// Additional missing constants
// Removed SORT_*_OUTLINE_COLOR constants - outline rendering disabled per user request
pub const METABALL_GIZMO_COLOR: Color = Color::srgba(0.3, 0.7, 1.0, 0.6);
pub const METABALL_SELECTED_COLOR: Color = Color::srgba(1.0, 0.8, 0.0, 0.8);
pub const METABALL_OUTLINE_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);
pub const PRESSED_BUTTON_ICON_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
pub const TOOLBAR_BORDER_WIDTH: f32 = 2.0;
pub const TOOLBAR_BORDER_RADIUS: f32 = 0.0;

// Selection constants - using primary color for compatibility
pub const SELECTED_POINT_COLOR: Color = SELECTED_PRIMARY_COLOR;
pub const SELECTION_POINT_RADIUS: f32 = 4.0;
pub const SELECTED_CIRCLE_RADIUS_MULTIPLIER: f32 = 1.0;

// Text constants
pub const LINE_LEADING: f32 = 0.0;
pub const WIDGET_TEXT_FONT_SIZE: f32 = 20.0;
pub const WIDGET_TITLE_FONT_SIZE: f32 = 20.0;
pub const WIDGET_MARGIN: f32 = 24.0;
pub const WIDGET_PADDING: f32 = 16.0;
pub const WIDGET_BORDER_WIDTH: f32 = 2.0;
pub const WIDGET_BORDER_RADIUS: f32 = 0.0;
pub const WIDGET_ROW_LEADING: f32 = 0.4; // Vertical spacing between rows in panes (negative for tighter spacing)

// Widget colors
pub const WIDGET_BACKGROUND_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 1.0);
pub const WIDGET_BORDER_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 1.0);
pub const NORMAL_TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
pub const SECONDARY_TEXT_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);
pub const PANEL_BACKGROUND_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);
pub const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

// Hover constants
pub const HOVER_POINT_COLOR: Color = Color::srgba(0.3, 0.8, 1.0, 0.7);
pub const HOVER_CIRCLE_RADIUS_MULTIPLIER: f32 = 1.0;

// =================================================================
// MINIMAL COMPATIBILITY CONSTANTS
// Font paths remain constants as they don't change per theme
// =================================================================

pub const GROTESK_FONT_PATH: &str = "fonts/BezyGrotesk-Regular.ttf";
pub const MONO_FONT_PATH: &str = "fonts/HasubiMono-Regular.ttf";

// =================================================================
// WIDGET CREATION FUNCTIONS
// These now use theme-aware functions where possible
// =================================================================

/// Creates a consistent styled container for UI widgets/panes
/// Uses the current theme for colors
pub fn create_widget_style<T: Component + Default>(
    _asset_server: &Res<AssetServer>,
    theme: &Res<CurrentTheme>,
    position: PositionType,
    position_props: UiRect,
    marker: T,
    name: &str,
) -> impl Bundle {
    use super::themes::WidgetBorderRadius;

    (
        Node {
            position_type: position,
            left: position_props.left,
            right: position_props.right,
            top: position_props.top,
            bottom: position_props.bottom,
            padding: UiRect::all(Val::Px(16.0)), // WIDGET_PADDING
            margin: UiRect::all(Val::Px(0.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(WIDGET_ROW_LEADING), // Use theme constant
            border: UiRect::all(Val::Px(2.0)), // WIDGET_BORDER_WIDTH
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

/// Creates a text component with standard styling
#[allow(dead_code)]
pub fn create_widget_text(
    asset_server: &Res<AssetServer>,
    text: &str,
    font_size: f32,
    color: Color,
) -> Text {
    Text(text.to_string())
}

/// Creates a label (dim) and value (bright) text pair for a widget row
#[allow(dead_code)]
pub fn create_widget_label_value_pair(
    asset_server: &Res<AssetServer>,
    label: &str,
    value: &str,
) -> (Node, Text) {
    (
        Node {
            ..Default::default()
        },
        Text(format!("{label} {value}")),
    )
}

/// Create a default text style with the given font
#[allow(dead_code)]
pub fn get_default_text_style(asset_server: &Res<AssetServer>) -> Text {
    Text::new("")
}

/// Create a text style with custom content and font size
#[allow(dead_code)]
pub fn create_text_style(text: &str, font_size: f32) -> Text {
    Text::new(text)
}
