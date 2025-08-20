//! Campfire Theme for Bezy
//!
//! A warm, cozy theme inspired by campfire colors.
//! Features deep oranges, reds, and golden yellows with dark charcoal backgrounds.

use super::BezyTheme;
use bevy::prelude::*;

/// Campfire theme implementation
///
/// This theme provides a warm, cozy appearance with:
/// - Dark charcoal background
/// - Warm orange and red accents
/// - Golden yellow highlights
/// - Embers and flame-inspired colors
pub struct CampfireTheme;

impl BezyTheme for CampfireTheme {
    fn name(&self) -> &'static str {
        "Campfire"
    }

    // =================================================================
    // TYPOGRAPHY
    // =================================================================

    fn normal_text_color(&self) -> Color {
        Color::srgb(0.9, 0.8, 0.6)
    }

    fn secondary_text_color(&self) -> Color {
        Color::srgb(0.7, 0.6, 0.4)
    }

    fn highlight_text_color(&self) -> Color {
        Color::srgb(1.0, 0.9, 0.3)
    }

    // =================================================================
    // COLORS - BACKGROUNDS & UI
    // =================================================================

    fn background_color(&self) -> Color {
        Color::srgb(0.08, 0.06, 0.04)
    }

    fn widget_background_color(&self) -> Color {
        Color::srgba(0.15, 0.1, 0.08, 1.0)
    }

    fn widget_border_color(&self) -> Color {
        Color::srgba(0.6, 0.4, 0.2, 1.0)
    }

    fn toolbar_background_color(&self) -> Color {
        Color::srgba(0.12, 0.08, 0.06, 1.0)
    }

    fn toolbar_icon_color(&self) -> Color {
        Color::srgb(0.8, 0.6, 0.4)
    }

    fn toolbar_border_color(&self) -> Color {
        Color::srgba(0.6, 0.4, 0.2, 1.0)
    }

    fn panel_background_color(&self) -> Color {
        Color::srgb(0.18, 0.12, 0.08)
    }

    fn normal_button_color(&self) -> Color {
        Color::srgb(0.2, 0.15, 0.1)
    }

    fn hovered_button_color(&self) -> Color {
        Color::srgb(0.4, 0.25, 0.15)
    }

    fn pressed_button_color(&self) -> Color {
        Color::srgb(1.0, 0.5, 0.1)
    }

    fn normal_button_outline_color(&self) -> Color {
        Color::srgb(0.5, 0.35, 0.2)
    }

    fn hovered_button_outline_color(&self) -> Color {
        Color::srgb(0.7, 0.45, 0.25)
    }

    fn pressed_button_outline_color(&self) -> Color {
        Color::srgb(1.0, 0.7, 0.3)
    }

    fn pressed_button_icon_color(&self) -> Color {
        Color::srgb(1.0, 1.0, 0.9)
    }

    fn focus_background_color(&self) -> Color {
        Color::srgb(1.0, 0.5, 0.1)
    }

    fn text_editor_background_color(&self) -> Color {
        Color::srgb(0.95, 0.9, 0.8)
    }

    // =================================================================
    // GLYPH RENDERING
    // =================================================================

    /// On-curve point colors (two-layer system)
    fn on_curve_primary_color(&self) -> Color {
        Color::srgb(0.3, 0.8, 0.2) // Bright green
    }

    fn on_curve_secondary_color(&self) -> Color {
        Color::srgb(0.15, 0.4, 0.1) // Dark green
    }

    /// Off-curve point colors (two-layer system)  
    fn off_curve_primary_color(&self) -> Color {
        Color::srgb(1.0, 0.6, 0.2) // Bright orange
    }

    fn off_curve_secondary_color(&self) -> Color {
        Color::srgb(0.5, 0.3, 0.1) // Dark orange/brown
    }

    fn path_line_color(&self) -> Color {
        Color::srgba(0.9, 0.8, 0.6, 1.0)
    }

    fn path_stroke_color(&self) -> Color {
        Color::srgb(0.8, 0.7, 0.5)
    }

    fn point_stroke_color(&self) -> Color {
        Color::srgba(0.1, 0.08, 0.06, 0.8)
    }

    fn handle_line_color(&self) -> Color {
        Color::srgba(0.5, 0.4, 0.3, 0.4)
    }

    // =================================================================
    // SELECTION & INTERACTION
    // =================================================================

    /// Selected point colors (two-layer system for crosshairs)
    fn selected_primary_color(&self) -> Color {
        Color::srgba(1.0, 0.9, 0.3, 1.0) // Bright golden yellow
    }

    fn selected_secondary_color(&self) -> Color {
        Color::srgba(0.5, 0.45, 0.15, 1.0) // Dark golden
    }

    fn hover_point_color(&self) -> Color {
        Color::srgba(1.0, 0.7, 0.3, 0.8)
    }

    fn hover_orange_color(&self) -> Color {
        Color::srgb(1.0, 0.6, 0.2)
    }

    // =================================================================
    // EDITING TOOLS
    // =================================================================

    fn knife_line_color(&self) -> Color {
        Color::srgba(1.0, 0.3, 0.1, 0.9)
    }

    fn knife_intersection_color(&self) -> Color {
        Color::srgba(1.0, 0.9, 0.3, 1.0)
    }

    fn knife_start_point_color(&self) -> Color {
        Color::srgba(0.3, 0.8, 0.2, 1.0)
    }

    fn pen_point_color(&self) -> Color {
        Color::srgb(1.0, 0.8, 0.3)
    }

    fn pen_start_point_color(&self) -> Color {
        Color::srgb(0.4, 0.8, 0.3)
    }

    fn pen_line_color(&self) -> Color {
        Color::srgba(0.9, 0.8, 0.6, 0.9)
    }

    fn hyper_point_color(&self) -> Color {
        Color::srgba(0.3, 0.8, 0.2, 1.0)
    }

    fn hyper_line_color(&self) -> Color {
        Color::srgba(1.0, 0.6, 0.3, 0.8)
    }

    fn hyper_close_indicator_color(&self) -> Color {
        Color::srgba(1.0, 0.8, 0.3, 1.0)
    }

    fn shape_preview_color(&self) -> Color {
        Color::srgba(0.7, 0.5, 0.3, 0.6)
    }

    // =================================================================
    // METABALLS
    // =================================================================

    fn metaball_gizmo_color(&self) -> Color {
        Color::srgba(1.0, 0.6, 0.3, 0.6)
    }

    fn metaball_outline_color(&self) -> Color {
        Color::srgba(0.9, 0.8, 0.6, 1.0)
    }

    fn metaball_selected_color(&self) -> Color {
        Color::srgba(1.0, 0.8, 0.3, 0.8)
    }

    // =================================================================
    // GUIDES & GRIDS
    // =================================================================

    fn metrics_guide_color(&self) -> Color {
        Color::srgba(0.3, 0.8, 0.2, 0.6)
    }

    fn checkerboard_color_1(&self) -> Color {
        Color::srgb(0.12, 0.08, 0.06)
    }

    fn checkerboard_color_2(&self) -> Color {
        Color::srgb(0.15, 0.1, 0.08)
    }

    fn checkerboard_color(&self) -> Color {
        Color::srgba(0.2, 0.15, 0.1, 0.5)
    }

    // =================================================================
    // SORTING & LAYOUT
    // =================================================================

    fn sort_active_metrics_color(&self) -> Color {
        Color::srgba(0.3, 0.8, 0.2, 0.6)
    }

    fn sort_inactive_metrics_color(&self) -> Color {
        Color::srgba(0.5, 0.4, 0.3, 0.5)
    }

    fn sort_active_outline_color(&self) -> Color {
        Color::srgb(1.0, 0.5, 0.1)
    }

    fn sort_inactive_outline_color(&self) -> Color {
        Color::srgb(0.6, 0.4, 0.3)
    }

    // =================================================================
    // INFO COLORS - SEMANTIC COLORS
    // =================================================================

    fn error_color(&self) -> Color {
        Color::srgb(1.0, 0.2, 0.1) // Bright red-orange
    }

    fn action_color(&self) -> Color {
        Color::srgb(1.0, 0.6, 0.1) // Warm orange
    }

    fn selected_color(&self) -> Color {
        Color::srgb(1.0, 0.9, 0.3) // Golden yellow
    }

    fn active_color(&self) -> Color {
        Color::srgb(0.6, 1.0, 0.3) // Bright green
    }

    fn helper_color(&self) -> Color {
        Color::srgb(0.3, 0.7, 1.0) // Sky blue
    }

    fn special_color(&self) -> Color {
        Color::srgb(0.9, 0.4, 1.0) // Purple
    }
}
