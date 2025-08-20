//! Strawberry Theme for Bezy
//!
//! A warm, playful theme inspired by strawberry colors.
//! Features soft pinks and reds with complementary greens for contrast.

use super::BezyTheme;
use bevy::prelude::*;

/// Strawberry theme implementation
///
/// This theme provides a warm, inviting appearance with:
/// - Soft pink background tones
/// - Rich strawberry accent colors
/// - Green highlights for contrast
/// - Cream and warm neutrals
pub struct StrawberryTheme;

impl BezyTheme for StrawberryTheme {
    fn name(&self) -> &'static str {
        "Strawberry"
    }

    // =================================================================
    // TYPOGRAPHY
    // =================================================================

    fn normal_text_color(&self) -> Color {
        Color::srgb(0.2, 0.1, 0.1)
    }

    fn secondary_text_color(&self) -> Color {
        Color::srgb(0.5, 0.3, 0.3)
    }

    fn highlight_text_color(&self) -> Color {
        Color::srgb(0.8, 0.2, 0.4)
    }

    // =================================================================
    // COLORS - BACKGROUNDS & UI
    // =================================================================

    fn background_color(&self) -> Color {
        Color::srgb(0.98, 0.94, 0.95)
    }

    fn widget_background_color(&self) -> Color {
        Color::srgba(0.98, 0.92, 0.94, 1.0)
    }

    fn widget_border_color(&self) -> Color {
        Color::srgba(0.8, 0.4, 0.5, 1.0)
    }

    fn toolbar_background_color(&self) -> Color {
        Color::srgba(0.95, 0.88, 0.90, 1.0)
    }

    fn toolbar_icon_color(&self) -> Color {
        Color::srgb(0.6, 0.3, 0.4)
    }

    fn toolbar_border_color(&self) -> Color {
        Color::srgba(0.8, 0.4, 0.5, 1.0)
    }

    fn panel_background_color(&self) -> Color {
        Color::srgb(0.96, 0.90, 0.92)
    }

    fn normal_button_color(&self) -> Color {
        Color::srgb(0.92, 0.85, 0.87)
    }

    fn hovered_button_color(&self) -> Color {
        Color::srgb(0.88, 0.75, 0.78)
    }

    fn pressed_button_color(&self) -> Color {
        Color::srgb(0.8, 0.2, 0.4)
    }

    fn normal_button_outline_color(&self) -> Color {
        Color::srgb(0.7, 0.4, 0.5)
    }

    fn hovered_button_outline_color(&self) -> Color {
        Color::srgb(0.6, 0.3, 0.4)
    }

    fn pressed_button_outline_color(&self) -> Color {
        Color::srgb(0.6, 0.1, 0.3)
    }

    fn pressed_button_icon_color(&self) -> Color {
        Color::srgb(1.0, 0.95, 0.96)
    }

    fn focus_background_color(&self) -> Color {
        Color::srgb(0.8, 0.2, 0.4)
    }

    fn text_editor_background_color(&self) -> Color {
        Color::srgb(0.1, 0.05, 0.06)
    }

    // =================================================================
    // GLYPH RENDERING
    // =================================================================

    /// On-curve point colors (two-layer system)
    fn on_curve_primary_color(&self) -> Color {
        Color::srgb(0.2, 0.7, 0.3) // Fresh green
    }

    fn on_curve_secondary_color(&self) -> Color {
        Color::srgb(0.1, 0.35, 0.15) // Dark green
    }

    /// Off-curve point colors (two-layer system)  
    fn off_curve_primary_color(&self) -> Color {
        Color::srgb(0.8, 0.3, 0.5) // Strawberry pink
    }

    fn off_curve_secondary_color(&self) -> Color {
        Color::srgb(0.4, 0.15, 0.25) // Dark berry
    }

    fn path_line_color(&self) -> Color {
        Color::srgba(0.3, 0.1, 0.1, 1.0)
    }

    fn path_stroke_color(&self) -> Color {
        Color::srgb(0.2, 0.1, 0.1)
    }

    fn point_stroke_color(&self) -> Color {
        Color::srgba(0.95, 0.90, 0.92, 0.8)
    }

    fn handle_line_color(&self) -> Color {
        Color::srgba(0.6, 0.4, 0.4, 0.5)
    }

    // =================================================================
    // SELECTION & INTERACTION
    // =================================================================

    /// Selected point colors (two-layer system for crosshairs)
    fn selected_primary_color(&self) -> Color {
        Color::srgba(0.9, 0.8, 0.2, 1.0) // Bright yellow
    }

    fn selected_secondary_color(&self) -> Color {
        Color::srgba(0.45, 0.4, 0.1, 1.0) // Dark yellow
    }

    fn hover_point_color(&self) -> Color {
        Color::srgba(0.2, 0.7, 0.5, 0.8)
    }

    fn hover_orange_color(&self) -> Color {
        Color::srgb(0.9, 0.5, 0.2)
    }

    // =================================================================
    // EDITING TOOLS
    // =================================================================

    fn knife_line_color(&self) -> Color {
        Color::srgba(0.9, 0.2, 0.3, 0.9)
    }

    fn knife_intersection_color(&self) -> Color {
        Color::srgba(0.9, 0.8, 0.2, 1.0)
    }

    fn knife_start_point_color(&self) -> Color {
        Color::srgba(0.2, 0.7, 0.3, 1.0)
    }

    fn pen_point_color(&self) -> Color {
        Color::srgb(0.9, 0.7, 0.2)
    }

    fn pen_start_point_color(&self) -> Color {
        Color::srgb(0.2, 0.7, 0.4)
    }

    fn pen_line_color(&self) -> Color {
        Color::srgba(0.3, 0.1, 0.1, 0.9)
    }

    fn hyper_point_color(&self) -> Color {
        Color::srgba(0.2, 0.7, 0.3, 1.0)
    }

    fn hyper_line_color(&self) -> Color {
        Color::srgba(0.8, 0.4, 0.5, 0.8)
    }

    fn hyper_close_indicator_color(&self) -> Color {
        Color::srgba(0.9, 0.7, 0.2, 1.0)
    }

    fn shape_preview_color(&self) -> Color {
        Color::srgba(0.6, 0.4, 0.4, 0.6)
    }

    // =================================================================
    // METABALLS
    // =================================================================

    fn metaball_gizmo_color(&self) -> Color {
        Color::srgba(0.8, 0.4, 0.5, 0.6)
    }

    fn metaball_outline_color(&self) -> Color {
        Color::srgba(0.3, 0.1, 0.1, 1.0)
    }

    fn metaball_selected_color(&self) -> Color {
        Color::srgba(0.9, 0.7, 0.2, 0.8)
    }

    // =================================================================
    // GUIDES & GRIDS
    // =================================================================

    fn metrics_guide_color(&self) -> Color {
        Color::srgba(0.2, 0.7, 0.3, 0.6)
    }

    fn checkerboard_color_1(&self) -> Color {
        Color::srgb(0.96, 0.92, 0.93)
    }

    fn checkerboard_color_2(&self) -> Color {
        Color::srgb(0.98, 0.94, 0.95)
    }

    fn checkerboard_color(&self) -> Color {
        Color::srgba(0.94, 0.88, 0.90, 0.5)
    }

    // =================================================================
    // SORTING & LAYOUT
    // =================================================================

    fn sort_active_metrics_color(&self) -> Color {
        Color::srgba(0.2, 0.7, 0.3, 0.6)
    }

    fn sort_inactive_metrics_color(&self) -> Color {
        Color::srgba(0.6, 0.4, 0.4, 0.5)
    }

    fn sort_active_outline_color(&self) -> Color {
        Color::srgb(0.8, 0.2, 0.4)
    }

    fn sort_inactive_outline_color(&self) -> Color {
        Color::srgb(0.6, 0.4, 0.4)
    }

    // =================================================================
    // INFO COLORS - SEMANTIC COLORS
    // =================================================================

    fn error_color(&self) -> Color {
        Color::srgb(1.0, 0.1, 0.3) // Deep strawberry red
    }

    fn action_color(&self) -> Color {
        Color::srgb(1.0, 0.5, 0.2) // Peach orange
    }

    fn selected_color(&self) -> Color {
        Color::srgb(1.0, 0.9, 0.4) // Cream yellow
    }

    fn active_color(&self) -> Color {
        Color::srgb(0.4, 1.0, 0.5) // Fresh green
    }

    fn helper_color(&self) -> Color {
        Color::srgb(0.3, 0.6, 1.0) // Berry blue
    }

    fn special_color(&self) -> Color {
        Color::srgb(0.8, 0.3, 1.0) // Berry purple
    }
}
