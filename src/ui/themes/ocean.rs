//! Ocean Theme for Bezy
//!
//! A deep, calming theme inspired by ocean depths.
//! Features deep blues and teals with aqua accents for editing tools.

use super::BezyTheme;
use bevy::prelude::*;

/// Ocean theme implementation
///
/// This theme provides a deep, ocean-inspired appearance with:
/// - Deep blue background like ocean depths
/// - Teal and aqua accent colors
/// - Sea foam highlights
/// - Coral points for contrast
pub struct OceanTheme;

impl Default for OceanTheme {
    fn default() -> Self {
        Self
    }
}

impl BezyTheme for OceanTheme {
    fn name(&self) -> &'static str {
        "Ocean"
    }

    // =================================================================
    // TYPOGRAPHY
    // =================================================================

    fn normal_text_color(&self) -> Color {
        Color::srgb(0.85, 0.95, 1.0) // Light cyan
    }

    fn secondary_text_color(&self) -> Color {
        Color::srgb(0.6, 0.8, 0.9) // Muted cyan
    }

    fn highlight_text_color(&self) -> Color {
        Color::srgb(0.3, 1.0, 0.8) // Bright aqua
    }

    // =================================================================
    // COLORS - BACKGROUNDS & UI
    // =================================================================

    fn background_color(&self) -> Color {
        Color::srgb(0.02, 0.1, 0.15) // Deep ocean blue
    }

    fn widget_background_color(&self) -> Color {
        Color::srgba(0.05, 0.15, 0.22, 1.0) // Dark teal
    }

    fn widget_border_color(&self) -> Color {
        Color::srgba(0.2, 0.5, 0.6, 1.0) // Ocean blue
    }

    fn toolbar_background_color(&self) -> Color {
        Color::srgba(0.03, 0.12, 0.18, 1.0) // Darker ocean
    }

    fn toolbar_icon_color(&self) -> Color {
        Color::srgb(0.7, 0.9, 1.0) // Light blue
    }

    fn toolbar_border_color(&self) -> Color {
        Color::srgba(0.2, 0.5, 0.6, 1.0) // Ocean blue
    }

    fn panel_background_color(&self) -> Color {
        Color::srgb(0.08, 0.18, 0.25) // Medium ocean
    }

    fn normal_button_color(&self) -> Color {
        Color::srgb(0.1, 0.2, 0.28) // Dark teal
    }

    fn hovered_button_color(&self) -> Color {
        Color::srgb(0.15, 0.35, 0.45) // Lighter teal
    }

    fn pressed_button_color(&self) -> Color {
        Color::srgb(0.1, 0.7, 0.9) // Bright cyan
    }

    fn normal_button_outline_color(&self) -> Color {
        Color::srgb(0.3, 0.5, 0.6) // Ocean border
    }

    fn hovered_button_outline_color(&self) -> Color {
        Color::srgb(0.4, 0.7, 0.8) // Brighter ocean
    }

    fn pressed_button_outline_color(&self) -> Color {
        Color::srgb(0.0, 0.5, 0.7) // Deep cyan
    }

    fn pressed_button_icon_color(&self) -> Color {
        Color::srgb(1.0, 1.0, 1.0) // White
    }

    fn focus_background_color(&self) -> Color {
        Color::srgb(0.1, 0.7, 0.9) // Bright cyan
    }

    fn text_editor_background_color(&self) -> Color {
        Color::srgb(0.95, 0.98, 1.0) // Very light blue
    }

    // =================================================================
    // GLYPH RENDERING
    // =================================================================

    /// On-curve point colors (two-layer system)
    fn on_curve_primary_color(&self) -> Color {
        Color::srgb(1.0, 0.5, 0.4) // Coral
    }

    fn on_curve_secondary_color(&self) -> Color {
        Color::srgb(0.5, 0.25, 0.2) // Dark coral
    }

    /// Off-curve point colors (two-layer system)  
    fn off_curve_primary_color(&self) -> Color {
        Color::srgb(0.4, 0.8, 1.0) // Sky blue
    }

    fn off_curve_secondary_color(&self) -> Color {
        Color::srgb(0.2, 0.4, 0.5) // Dark ocean blue
    }

    fn path_line_color(&self) -> Color {
        Color::srgba(0.8, 0.95, 1.0, 1.0) // Light cyan
    }

    fn path_stroke_color(&self) -> Color {
        Color::srgb(0.7, 0.9, 1.0) // Pale blue
    }

    fn point_stroke_color(&self) -> Color {
        Color::srgba(0.05, 0.15, 0.22, 0.8) // Dark teal stroke
    }

    fn handle_line_color(&self) -> Color {
        Color::srgba(0.4, 0.7, 0.8, 0.4) // Translucent ocean
    }

    // =================================================================
    // SELECTION & INTERACTION
    // =================================================================

    /// Selected point colors (two-layer system for crosshairs)
    fn selected_primary_color(&self) -> Color {
        Color::srgba(1.0, 0.8, 0.1, 1.0) // Golden yellow (like sun on water)
    }

    fn selected_secondary_color(&self) -> Color {
        Color::srgba(0.5, 0.4, 0.05, 1.0) // Dark golden
    }

    fn hover_point_color(&self) -> Color {
        Color::srgba(0.2, 1.0, 0.8, 0.8) // Bright aqua
    }

    fn hover_orange_color(&self) -> Color {
        Color::srgb(1.0, 0.6, 0.2) // Coral orange
    }

    // =================================================================
    // EDITING TOOLS
    // =================================================================

    fn knife_line_color(&self) -> Color {
        Color::srgba(1.0, 0.3, 0.5, 0.9) // Pink coral
    }

    fn knife_intersection_color(&self) -> Color {
        Color::srgba(1.0, 0.8, 0.1, 1.0) // Golden
    }

    fn knife_start_point_color(&self) -> Color {
        Color::srgba(1.0, 0.5, 0.4, 1.0) // Coral
    }

    fn pen_point_color(&self) -> Color {
        Color::srgb(0.9, 0.7, 0.1) // Golden
    }

    fn pen_start_point_color(&self) -> Color {
        Color::srgb(0.2, 1.0, 0.6) // Sea green
    }

    fn pen_line_color(&self) -> Color {
        Color::srgba(0.8, 0.95, 1.0, 0.9) // Light cyan
    }

    fn hyper_point_color(&self) -> Color {
        Color::srgba(1.0, 0.5, 0.4, 1.0) // Coral
    }

    fn hyper_line_color(&self) -> Color {
        Color::srgba(0.3, 0.8, 1.0, 0.8) // Ocean blue
    }

    fn hyper_close_indicator_color(&self) -> Color {
        Color::srgba(0.9, 0.7, 0.1, 1.0) // Golden
    }

    fn shape_preview_color(&self) -> Color {
        Color::srgba(0.5, 0.8, 1.0, 0.6) // Translucent blue
    }

    // =================================================================
    // METABALLS
    // =================================================================

    fn metaball_gizmo_color(&self) -> Color {
        Color::srgba(0.2, 0.8, 1.0, 0.6) // Ocean blue
    }

    fn metaball_outline_color(&self) -> Color {
        Color::srgba(0.8, 0.95, 1.0, 1.0) // Light cyan
    }

    fn metaball_selected_color(&self) -> Color {
        Color::srgba(1.0, 0.8, 0.1, 0.8) // Golden
    }

    // =================================================================
    // GUIDES & GRIDS
    // =================================================================

    fn metrics_guide_color(&self) -> Color {
        Color::srgba(0.2, 1.0, 0.6, 0.6) // Sea green
    }

    fn checkerboard_color_1(&self) -> Color {
        Color::srgb(0.08, 0.16, 0.22) // Dark ocean
    }

    fn checkerboard_color_2(&self) -> Color {
        Color::srgb(0.1, 0.18, 0.25) // Slightly lighter
    }

    fn checkerboard_color(&self) -> Color {
        Color::srgba(0.15, 0.25, 0.32, 0.5) // Translucent ocean
    }

    // =================================================================
    // SORTING & LAYOUT
    // =================================================================

    fn sort_active_metrics_color(&self) -> Color {
        Color::srgba(0.2, 1.0, 0.6, 0.6) // Sea green
    }

    fn sort_inactive_metrics_color(&self) -> Color {
        Color::srgba(0.5, 0.7, 0.8, 0.5) // Muted ocean
    }

    fn sort_active_outline_color(&self) -> Color {
        Color::srgb(0.1, 0.7, 0.9) // Bright cyan
    }

    fn sort_inactive_outline_color(&self) -> Color {
        Color::srgb(0.4, 0.6, 0.7) // Muted blue
    }

    // =================================================================
    // INFO COLORS - SEMANTIC COLORS
    // =================================================================

    fn error_color(&self) -> Color {
        Color::srgb(1.0, 0.3, 0.3) // Coral red
    }

    fn action_color(&self) -> Color {
        Color::srgb(1.0, 0.7, 0.2) // Sunset orange
    }

    fn selected_color(&self) -> Color {
        Color::srgb(1.0, 1.0, 0.4) // Sandy yellow
    }

    fn active_color(&self) -> Color {
        Color::srgb(0.3, 1.0, 0.6) // Sea green
    }

    fn helper_color(&self) -> Color {
        Color::srgb(0.2, 0.8, 1.0) // Ocean blue
    }

    fn special_color(&self) -> Color {
        Color::srgb(0.7, 0.4, 1.0) // Deep sea purple
    }
}
