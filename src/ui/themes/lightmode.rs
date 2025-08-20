//! Light Mode Theme for Bezy
//!
//! A clean, bright theme with excellent contrast and modern aesthetics.
//! Perfect for well-lit environments and users who prefer light interfaces.

use super::BezyTheme;
use bevy::prelude::*;

/// Light mode theme implementation
///
/// This theme provides a bright, clean appearance with:
/// - Light gray background
/// - Dark text for readability
/// - Saturated colors for editing tools
/// - High contrast for precise editing
pub struct LightModeTheme;

impl BezyTheme for LightModeTheme {
    fn name(&self) -> &'static str {
        "Light Mode"
    }

    // =================================================================
    // TYPOGRAPHY
    // =================================================================

    fn normal_text_color(&self) -> Color {
        Color::srgb(0.1, 0.1, 0.1)
    }

    fn secondary_text_color(&self) -> Color {
        Color::srgb(0.4, 0.4, 0.4)
    }

    fn highlight_text_color(&self) -> Color {
        Color::srgb(0.8, 0.5, 0.0)
    }

    // =================================================================
    // COLORS - BACKGROUNDS & UI
    // =================================================================

    fn background_color(&self) -> Color {
        Color::srgb(0.95, 0.95, 0.95)
    }

    fn widget_background_color(&self) -> Color {
        Color::srgba(1.0, 1.0, 1.0, 1.0)
    }

    fn widget_border_color(&self) -> Color {
        Color::srgba(0.3, 0.3, 0.3, 1.0)
    }

    fn toolbar_background_color(&self) -> Color {
        Color::srgba(0.9, 0.9, 0.9, 1.0)
    }

    fn toolbar_icon_color(&self) -> Color {
        Color::srgb(0.25, 0.25, 0.25)
    }

    fn toolbar_border_color(&self) -> Color {
        Color::srgba(0.3, 0.3, 0.3, 1.0)
    }

    fn panel_background_color(&self) -> Color {
        Color::srgb(0.92, 0.92, 0.92)
    }

    fn normal_button_color(&self) -> Color {
        Color::srgb(0.85, 0.85, 0.85)
    }

    fn hovered_button_color(&self) -> Color {
        Color::srgb(0.75, 0.75, 0.75)
    }

    fn pressed_button_color(&self) -> Color {
        Color::srgb(0.2, 0.6, 1.0)
    }

    fn normal_button_outline_color(&self) -> Color {
        Color::srgb(0.5, 0.5, 0.5)
    }

    fn hovered_button_outline_color(&self) -> Color {
        Color::srgb(0.25, 0.25, 0.25)
    }

    fn pressed_button_outline_color(&self) -> Color {
        Color::srgb(0.1, 0.4, 0.8)
    }

    fn pressed_button_icon_color(&self) -> Color {
        Color::srgb(1.0, 1.0, 1.0)
    }

    fn focus_background_color(&self) -> Color {
        Color::srgb(0.2, 0.6, 1.0)
    }

    fn text_editor_background_color(&self) -> Color {
        Color::srgb(0.05, 0.05, 0.05)
    }

    // =================================================================
    // GLYPH RENDERING
    // =================================================================

    /// On-curve point colors (two-layer system)
    fn on_curve_primary_color(&self) -> Color {
        Color::srgb(0.1, 0.7, 0.3) // Bright green
    }

    fn on_curve_secondary_color(&self) -> Color {
        Color::srgb(0.05, 0.35, 0.15) // Dark green
    }

    /// Off-curve point colors (two-layer system)  
    fn off_curve_primary_color(&self) -> Color {
        Color::srgb(0.4, 0.2, 0.8) // Bright purple
    }

    fn off_curve_secondary_color(&self) -> Color {
        Color::srgb(0.2, 0.1, 0.4) // Dark purple
    }

    fn path_line_color(&self) -> Color {
        Color::srgba(0.0, 0.0, 0.0, 1.0)
    }

    fn path_stroke_color(&self) -> Color {
        Color::srgb(0.1, 0.1, 0.1)
    }

    fn point_stroke_color(&self) -> Color {
        Color::srgba(0.9, 0.9, 0.9, 0.8)
    }

    fn handle_line_color(&self) -> Color {
        Color::srgba(0.4, 0.4, 0.4, 0.6)
    }

    // =================================================================
    // SELECTION & INTERACTION
    // =================================================================

    /// Selected point colors (two-layer system for crosshairs)
    fn selected_primary_color(&self) -> Color {
        Color::srgba(1.0, 0.6, 0.0, 1.0) // Bright orange
    }

    fn selected_secondary_color(&self) -> Color {
        Color::srgba(0.5, 0.3, 0.0, 1.0) // Dark orange
    }

    fn hover_point_color(&self) -> Color {
        Color::srgba(0.2, 0.6, 1.0, 0.8)
    }

    fn hover_orange_color(&self) -> Color {
        Color::srgb(0.8, 0.4, 0.0)
    }

    // =================================================================
    // EDITING TOOLS
    // =================================================================

    fn knife_line_color(&self) -> Color {
        Color::srgba(0.8, 0.2, 0.2, 0.9)
    }

    fn knife_intersection_color(&self) -> Color {
        Color::srgba(1.0, 0.8, 0.0, 1.0)
    }

    fn knife_start_point_color(&self) -> Color {
        Color::srgba(0.1, 0.7, 0.3, 1.0)
    }

    fn pen_point_color(&self) -> Color {
        Color::srgb(0.8, 0.6, 0.0)
    }

    fn pen_start_point_color(&self) -> Color {
        Color::srgb(0.0, 0.7, 0.4)
    }

    fn pen_line_color(&self) -> Color {
        Color::srgba(0.0, 0.0, 0.0, 0.9)
    }

    fn hyper_point_color(&self) -> Color {
        Color::srgba(0.1, 0.7, 0.3, 1.0)
    }

    fn hyper_line_color(&self) -> Color {
        Color::srgba(0.2, 0.5, 0.8, 0.8)
    }

    fn hyper_close_indicator_color(&self) -> Color {
        Color::srgba(0.8, 0.6, 0.0, 1.0)
    }

    fn shape_preview_color(&self) -> Color {
        Color::srgba(0.3, 0.3, 0.3, 0.6)
    }

    // =================================================================
    // METABALLS
    // =================================================================

    fn metaball_gizmo_color(&self) -> Color {
        Color::srgba(0.2, 0.5, 0.8, 0.6)
    }

    fn metaball_outline_color(&self) -> Color {
        Color::srgba(0.0, 0.0, 0.0, 1.0)
    }

    fn metaball_selected_color(&self) -> Color {
        Color::srgba(0.8, 0.5, 0.0, 0.8)
    }

    // =================================================================
    // GUIDES & GRIDS
    // =================================================================

    fn metrics_guide_color(&self) -> Color {
        Color::srgba(0.1, 0.7, 0.3, 0.6)
    }

    fn checkerboard_color_1(&self) -> Color {
        Color::srgb(0.88, 0.88, 0.88)
    }

    fn checkerboard_color_2(&self) -> Color {
        Color::srgb(0.92, 0.92, 0.92)
    }

    fn checkerboard_color(&self) -> Color {
        Color::srgba(0.85, 0.85, 0.85, 0.5)
    }

    // =================================================================
    // SORTING & LAYOUT
    // =================================================================

    fn sort_active_metrics_color(&self) -> Color {
        Color::srgba(0.1, 0.7, 0.3, 0.6)
    }

    fn sort_inactive_metrics_color(&self) -> Color {
        Color::srgba(0.5, 0.5, 0.5, 0.5)
    }

    fn sort_active_outline_color(&self) -> Color {
        Color::srgb(0.2, 0.6, 1.0)
    }

    fn sort_inactive_outline_color(&self) -> Color {
        Color::srgb(0.4, 0.4, 0.4)
    }

    // =================================================================
    // INFO COLORS - SEMANTIC COLORS
    // =================================================================

    fn error_color(&self) -> Color {
        Color::srgb(0.8, 0.0, 0.0) // Dark red
    }

    fn action_color(&self) -> Color {
        Color::srgb(0.9, 0.4, 0.0) // Dark orange
    }

    fn selected_color(&self) -> Color {
        Color::srgb(0.8, 0.7, 0.0) // Dark yellow
    }

    fn active_color(&self) -> Color {
        Color::srgb(0.0, 0.6, 0.0) // Dark green
    }

    fn helper_color(&self) -> Color {
        Color::srgb(0.0, 0.3, 0.8) // Dark blue
    }

    fn special_color(&self) -> Color {
        Color::srgb(0.6, 0.0, 0.8) // Dark purple
    }
}
