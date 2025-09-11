//! Legacy Constants for Backward Compatibility
//!
//! This file contains hardcoded constants that were used before the theme system.
//! These exist purely for backward compatibility during migration.
//!
//! TODO: This entire file should be deleted once all code is migrated to use the theme system.

use bevy::prelude::*;

// =================================================================
// LEGACY COMPATIBILITY CONSTANTS
// These provide hardcoded values from before the theme system
// DELETE THIS ENTIRE FILE once migration to theme system is complete
// =================================================================

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
pub const METABALL_GIZMO_COLOR: Color = Color::srgba(0.3, 0.7, 1.0, 0.6);
pub const METABALL_SELECTED_COLOR: Color = Color::srgba(1.0, 0.8, 0.0, 0.8);
pub const METABALL_OUTLINE_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);
pub const PRESSED_BUTTON_ICON_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);

// Selection constants - using primary color for compatibility
pub const SELECTED_POINT_COLOR: Color = SELECTED_PRIMARY_COLOR;

// Widget colors
pub const WIDGET_BACKGROUND_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 1.0);
pub const WIDGET_BORDER_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 1.0);
pub const NORMAL_TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
pub const SECONDARY_TEXT_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);
pub const PANEL_BACKGROUND_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);
pub const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

// Hover constants
pub const HOVER_POINT_COLOR: Color = Color::srgba(0.3, 0.8, 1.0, 0.7);
