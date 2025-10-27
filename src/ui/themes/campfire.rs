use crate::ui::theme_system::BezyTheme;
use bevy::prelude::*;

pub struct CampfireTheme;

impl BezyTheme for CampfireTheme {
    fn name(&self) -> &'static str { "Campfire" }
    fn ui_text_primary(&self) -> Color { Color::srgb(0.95, 0.85, 0.7) }
    fn ui_text_secondary(&self) -> Color { Color::srgb(0.7, 0.6, 0.5) }
    fn ui_text_tertiary(&self) -> Color { Color::srgb(0.6, 0.5, 0.4) }
    fn ui_text_quaternary(&self) -> Color { Color::srgb(0.5, 0.4, 0.3) }
    fn background_color(&self) -> Color { Color::srgb(0.1, 0.05, 0.0) }
    fn widget_background_color(&self) -> Color { Color::srgba(0.2, 0.1, 0.05, 1.0) }
    fn widget_border_color(&self) -> Color { Color::srgb(0.6, 0.4, 0.2) }
    fn button_regular(&self) -> Color { Color::srgb(0.2, 0.1, 0.05) }
    fn button_hovered(&self) -> Color { Color::srgb(0.3, 0.2, 0.1) }
    fn button_pressed(&self) -> Color { Color::srgb(1.0, 0.5, 0.1) }
    fn button_regular_outline(&self) -> Color { Color::srgb(0.5, 0.3, 0.1) }
    fn button_hovered_outline(&self) -> Color { Color::srgb(0.7, 0.4, 0.2) }
    fn button_pressed_outline(&self) -> Color { Color::srgb(1.0, 0.7, 0.3) }
    fn button_regular_icon(&self) -> Color { Color::srgb(0.9, 0.7, 0.4) }
    fn button_hovered_icon(&self) -> Color { Color::srgb(0.9, 0.7, 0.4) }
    fn button_pressed_icon(&self) -> Color { Color::srgb(1.0, 1.0, 1.0) }
    fn focus_background_color(&self) -> Color { Color::srgb(1.0, 0.5, 0.1) }
    fn text_editor_background_color(&self) -> Color { Color::srgb(0.95, 0.85, 0.7) }
    fn on_curve_primary_color(&self) -> Color { Color::srgb(0.3, 0.8, 0.2) }
    fn on_curve_secondary_color(&self) -> Color { Color::srgb(0.15, 0.4, 0.1) }
    fn off_curve_primary_color(&self) -> Color { Color::srgb(1.0, 0.6, 0.2) }
    fn off_curve_secondary_color(&self) -> Color { Color::srgb(0.5, 0.3, 0.1) }
    fn path_line_color(&self) -> Color { Color::srgba(0.9, 0.7, 0.4, 1.0) }
    fn path_stroke_color(&self) -> Color { Color::srgb(0.9, 0.7, 0.4) }
    fn point_stroke_color(&self) -> Color { Color::srgba(0.1, 0.05, 0.0, 0.8) }
    fn handle_line_color(&self) -> Color { Color::srgba(0.7, 0.5, 0.3, 0.4) }
    fn error_color(&self) -> Color { Color::srgb(1.0, 0.2, 0.0) }
    fn action_color(&self) -> Color { Color::srgb(1.0, 0.6, 0.0) }
    fn selected_color(&self) -> Color { Color::srgb(1.0, 0.9, 0.3) }
    fn active_color(&self) -> Color { Color::srgb(0.5, 1.0, 0.2) }
    fn helper_color(&self) -> Color { Color::srgb(0.3, 0.7, 1.0) }
    fn special_color(&self) -> Color { Color::srgb(0.8, 0.3, 1.0) }
    fn selected_primary_color(&self) -> Color { Color::srgba(1.0, 0.9, 0.3, 1.0) }
    fn selected_secondary_color(&self) -> Color { Color::srgba(0.5, 0.45, 0.15, 1.0) }
    fn hover_point_color(&self) -> Color { Color::srgba(1.0, 0.7, 0.0, 0.8) }
    fn hover_orange_color(&self) -> Color { Color::srgb(1.0, 0.5, 0.0) }
    fn knife_line_color(&self) -> Color { Color::srgba(1.0, 0.4, 0.2, 0.9) }
    fn knife_intersection_color(&self) -> Color { Color::srgba(1.0, 0.9, 0.3, 1.0) }
    fn knife_start_point_color(&self) -> Color { Color::srgba(0.5, 1.0, 0.2, 1.0) }
    fn pen_point_color(&self) -> Color { Color::srgb(1.0, 0.9, 0.3) }
    fn pen_start_point_color(&self) -> Color { Color::srgb(0.5, 1.0, 0.2) }
    fn pen_line_color(&self) -> Color { Color::srgba(0.9, 0.7, 0.4, 0.9) }
    fn hyper_point_color(&self) -> Color { Color::srgba(0.5, 1.0, 0.2, 1.0) }
    fn hyper_line_color(&self) -> Color { Color::srgba(0.7, 0.9, 1.0, 0.8) }
    fn hyper_close_indicator_color(&self) -> Color { Color::srgba(1.0, 0.9, 0.3, 1.0) }
    fn shape_preview_color(&self) -> Color { Color::srgba(0.7, 0.5, 0.3, 0.6) }
    fn metaball_gizmo_color(&self) -> Color { Color::srgba(1.0, 0.6, 0.2, 0.6) }
    fn metaball_outline_color(&self) -> Color { Color::srgba(0.9, 0.7, 0.4, 1.0) }
    fn metaball_selected_color(&self) -> Color { Color::srgba(1.0, 0.9, 0.3, 0.8) }
    fn filled_glyph_color(&self) -> Color { Color::srgb(0.5, 0.4, 0.35) }
    fn checkerboard_color(&self) -> Color { Color::srgba(0.2, 0.1, 0.05, 0.5) }
    fn checkerboard_color_1(&self) -> Color { Color::srgb(0.15, 0.08, 0.03) }
    fn checkerboard_color_2(&self) -> Color { Color::srgb(0.18, 0.1, 0.05) }
    fn metrics_guide_color(&self) -> Color { Color::srgba(0.3, 0.8, 0.2, 0.5) }
    fn sort_active_metrics_color(&self) -> Color { self.active_color() }
    fn sort_inactive_metrics_color(&self) -> Color { Color::srgba(0.5, 0.3, 0.1, 0.5) }
    fn sort_active_outline_color(&self) -> Color { Color::srgb(1.0, 0.5, 0.1) }
    fn sort_inactive_outline_color(&self) -> Color { Color::srgb(0.6, 0.4, 0.2) }
    fn widget_border_radius(&self) -> f32 { 0.0 }
    fn toolbar_border_radius(&self) -> f32 { 0.0 }
    fn ui_border_radius(&self) -> f32 { 12.0 }
}
