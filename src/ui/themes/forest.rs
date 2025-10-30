use crate::ui::theme_system::BezyTheme;
use bevy::prelude::*;

pub struct ForestTheme;

impl BezyTheme for ForestTheme {
    fn name(&self) -> &'static str {
        "Forest"
    }

    fn ui_text_primary(&self) -> Color {
        Color::srgb(0.82, 0.78, 0.66)
    }

    fn ui_text_secondary(&self) -> Color {
        Color::srgb(0.7, 0.65, 0.55)
    }

    fn ui_text_tertiary(&self) -> Color {
        Color::srgb(0.6, 0.55, 0.45)
    }

    fn ui_text_quaternary(&self) -> Color {
        Color::srgb(0.5, 0.45, 0.35)
    }

    fn background_color(&self) -> Color {
        Color::srgb(0.0, 0.0, 0.0)
    }

    fn widget_background_color(&self) -> Color {
        Color::srgba(0.1, 0.1, 0.1, 1.0)
    }

    fn widget_border_color(&self) -> Color {
        Color::srgb(0.3, 0.3, 0.3)
    }

    fn button_regular(&self) -> Color {
        Color::srgb(0.1, 0.1, 0.1)
    }

    fn button_regular_outline(&self) -> Color {
        Color::srgb(0.3, 0.3, 0.3)
    }

    fn button_hovered(&self) -> Color {
        Color::srgb(0.2, 0.2, 0.2)
    }

    fn button_pressed(&self) -> Color {
        Color::srgb(0.85, 0.45, 0.35)
    }

    fn button_hovered_outline(&self) -> Color {
        Color::srgb(0.65, 0.75, 0.5)
    }

    fn button_pressed_outline(&self) -> Color {
        Color::srgb(0.9, 0.6, 0.4)
    }

    fn button_regular_icon(&self) -> Color {
        Color::srgb(0.83, 0.78, 0.67)
    }

    fn button_hovered_icon(&self) -> Color {
        Color::srgb(0.9, 0.85, 0.75)
    }

    fn button_pressed_icon(&self) -> Color {
        Color::srgb(1.0, 1.0, 1.0)
    }

    fn focus_background_color(&self) -> Color {
        Color::srgb(1.0, 0.42, 0.0)
    }

    fn text_editor_background_color(&self) -> Color {
        Color::srgb(0.827, 0.776, 0.667)
    }

    fn on_curve_primary_color(&self) -> Color {
        Color::srgb(0.65, 0.75, 0.5)
    }

    fn on_curve_secondary_color(&self) -> Color {
        Color::srgb(0.4, 0.5, 0.3)
    }

    fn off_curve_primary_color(&self) -> Color {
        Color::srgb(0.839, 0.600, 0.714)
    }

    fn off_curve_secondary_color(&self) -> Color {
        Color::srgb(0.5, 0.35, 0.45)
    }

    fn path_line_color(&self) -> Color {
        Color::srgba(0.827, 0.776, 0.667, 1.0)
    }

    fn path_stroke_color(&self) -> Color {
        Color::srgb(0.827, 0.776, 0.667)
    }

    fn point_stroke_color(&self) -> Color {
        Color::srgba(0.176, 0.208, 0.231, 0.8)
    }

    fn handle_line_color(&self) -> Color {
        Color::srgba(0.6, 0.55, 0.45, 0.4)
    }

    fn error_color(&self) -> Color {
        Color::srgb(0.902, 0.494, 0.502)
    }

    fn action_color(&self) -> Color {
        Color::srgb(0.9, 0.4, 0.2)
    }

    fn selected_color(&self) -> Color {
        Color::srgb(0.859, 0.737, 0.498)
    }

    fn active_color(&self) -> Color {
        Color::srgb(0.65, 0.85, 0.5)
    }

    fn helper_color(&self) -> Color {
        Color::srgb(0.498, 0.733, 0.702)
    }

    fn special_color(&self) -> Color {
        Color::srgb(0.839, 0.600, 0.714)
    }

    fn selected_primary_color(&self) -> Color {
        Color::srgba(0.859, 0.737, 0.498, 1.0)
    }

    fn selected_secondary_color(&self) -> Color {
        Color::srgba(1.0, 0.42, 0.0, 1.0)
    }

    fn hover_point_color(&self) -> Color {
        Color::srgba(0.498, 0.733, 0.702, 0.8)
    }

    fn hover_orange_color(&self) -> Color {
        Color::srgb(1.0, 0.42, 0.0)
    }

    fn knife_line_color(&self) -> Color {
        Color::srgba(0.902, 0.494, 0.502, 0.9)
    }

    fn knife_intersection_color(&self) -> Color {
        Color::srgba(0.859, 0.737, 0.498, 1.0)
    }

    fn knife_start_point_color(&self) -> Color {
        Color::srgba(0.655, 0.753, 0.502, 1.0)
    }

    fn pen_point_color(&self) -> Color {
        Color::srgb(0.859, 0.737, 0.498)
    }

    fn pen_start_point_color(&self) -> Color {
        Color::srgb(0.655, 0.753, 0.502)
    }

    fn pen_line_color(&self) -> Color {
        Color::srgba(0.827, 0.776, 0.667, 0.9)
    }

    fn hyper_point_color(&self) -> Color {
        Color::srgba(0.655, 0.753, 0.502, 1.0)
    }

    fn hyper_line_color(&self) -> Color {
        Color::srgba(0.498, 0.733, 0.702, 0.8)
    }

    fn hyper_close_indicator_color(&self) -> Color {
        Color::srgba(0.859, 0.737, 0.498, 1.0)
    }

    fn shape_preview_color(&self) -> Color {
        Color::srgba(0.7, 0.65, 0.55, 0.6)
    }

    fn metaball_gizmo_color(&self) -> Color {
        Color::srgba(0.498, 0.733, 0.702, 0.6)
    }

    fn metaball_outline_color(&self) -> Color {
        Color::srgba(0.827, 0.776, 0.667, 1.0)
    }

    fn metaball_selected_color(&self) -> Color {
        Color::srgba(1.0, 0.42, 0.0, 0.8)
    }

    fn filled_glyph_color(&self) -> Color {
        Color::srgb(0.6, 0.55, 0.45)
    }

    fn checkerboard_color(&self) -> Color {
        Color::srgba(0.1, 0.1, 0.1, 0.25)
    }

    fn checkerboard_color_1(&self) -> Color {
        Color::srgb(0.2, 0.24, 0.27)
    }

    fn checkerboard_color_2(&self) -> Color {
        Color::srgb(0.24, 0.28, 0.31)
    }

    fn metrics_guide_color(&self) -> Color {
        Color::srgba(0.655, 0.753, 0.502, 0.5)
    }

    fn sort_active_metrics_color(&self) -> Color {
        self.active_color()
    }

    fn sort_inactive_metrics_color(&self) -> Color {
        Color::srgba(0.5, 0.45, 0.35, 0.5)
    }

    fn sort_active_outline_color(&self) -> Color {
        Color::srgb(1.0, 0.42, 0.0)
    }

    fn sort_inactive_outline_color(&self) -> Color {
        Color::srgb(0.6, 0.55, 0.45)
    }

    fn widget_border_radius(&self) -> f32 {
        8.0
    }

    fn toolbar_border_radius(&self) -> f32 {
        8.0
    }

    fn ui_border_radius(&self) -> f32 {
        8.0
    }
}
