//! Theme System Interface
//!
//! This file provides convenient accessor functions for getting values from the current theme.
//! Use these functions instead of accessing theme resources directly for better consistency.

use crate::ui::themes::{BezyTheme, CurrentTheme};
use bevy::prelude::*;

// =================================================================
// THEME ACCESSOR FUNCTIONS
// These functions get values from the current theme.
// Use these instead of hardcoded constants!
// =================================================================

/// Helper function to get current theme
pub fn get_current_theme<'a>(theme_res: &'a Res<CurrentTheme>) -> &'a dyn BezyTheme {
    theme_res.theme()
}

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
// THEME-DEPENDENT STRUCT ACCESSORS
// These provide structured access to theme values
// =================================================================

/// Checkerboard constants using theme values
pub fn get_checkerboard_constants(theme: &Res<CurrentTheme>) -> CheckerboardConstants {
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

pub fn get_theme_dependent_constants(theme: &Res<CurrentTheme>) -> ThemeDependentConstants {
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
