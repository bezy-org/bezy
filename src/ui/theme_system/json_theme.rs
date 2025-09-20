//! JSON-based theme system

#![allow(clippy::let_and_return)]
#![allow(clippy::unnecessary_operation)]
#![allow(clippy::type_complexity)]
#![allow(unused_must_use)]
//!
//! This replaces the Rust trait-based themes with JSON files that can be edited
//! live and reloaded without recompilation.

use super::{embedded_themes, BezyTheme};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// Marker components for UI elements that need border radius updates
#[derive(Component)]
pub struct WidgetBorderRadius;

#[derive(Component)]
pub struct ToolbarBorderRadius;

#[derive(Component)]
pub struct UiBorderRadius;

/// Complete theme definition in JSON format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonTheme {
    pub name: String,

    // Unified UI text colors
    pub ui_text_primary: [f32; 3],
    pub ui_text_secondary: [f32; 3],
    pub ui_text_tertiary: [f32; 3],
    pub ui_text_quaternary: [f32; 3],

    // Background colors
    pub background: [f32; 3],
    pub widget_background: [f32; 4],
    pub widget_border: [f32; 3],
    pub toolbar_background: [f32; 4],
    pub toolbar_icon: [f32; 3],
    pub toolbar_border: [f32; 4],
    pub panel_background: [f32; 3],

    // Button colors
    pub normal_button: [f32; 3],
    pub hovered_button: [f32; 3],
    pub pressed_button: [f32; 3],
    pub normal_button_outline: [f32; 3],
    pub hovered_button_outline: [f32; 3],
    pub pressed_button_outline: [f32; 3],
    pub pressed_button_icon: [f32; 3],

    // Special backgrounds
    pub focus_background: [f32; 3],
    pub text_editor_background: [f32; 3],

    // Point colors (two-layer system)
    pub on_curve_primary: [f32; 3],
    pub on_curve_secondary: [f32; 3],
    pub off_curve_primary: [f32; 3],
    pub off_curve_secondary: [f32; 3],

    // Path colors
    pub path_line: [f32; 4],
    pub path_stroke: [f32; 3],
    pub point_stroke: [f32; 4],
    pub handle_line: [f32; 4],

    // Semantic colors
    pub error: [f32; 3],
    pub action: [f32; 3],
    pub selected: [f32; 3],
    pub active: [f32; 3],
    pub helper: [f32; 3],
    pub special: [f32; 3],

    // Selection colors (two-layer system)
    pub selected_primary: [f32; 4],
    pub selected_secondary: [f32; 4],
    pub hover_point: [f32; 4],
    pub hover_orange: [f32; 3],

    // Tool colors
    pub knife_line: [f32; 4],
    pub knife_intersection: [f32; 4],
    pub knife_start_point: [f32; 4],
    pub pen_point: [f32; 3],
    pub pen_start_point: [f32; 3],
    pub pen_line: [f32; 4],
    pub hyper_point: [f32; 4],
    pub hyper_line: [f32; 4],
    pub hyper_close_indicator: [f32; 4],
    pub shape_preview: [f32; 4],

    // Metaballs
    pub metaball_gizmo: [f32; 4],
    pub metaball_outline: [f32; 4],
    pub metaball_selected: [f32; 4],

    // Guides and grids
    pub metrics_guide: [f32; 4],
    pub checkerboard_color_1: [f32; 3],
    pub checkerboard_color_2: [f32; 3],
    pub checkerboard: [f32; 4],

    // Sort colors
    pub sort_active_metrics: [f32; 4],
    pub sort_inactive_metrics: [f32; 4],
    pub sort_active_outline: [f32; 3],
    pub sort_inactive_outline: [f32; 3],
    pub filled_glyph: [f32; 3],

    // Border radius properties
    pub widget_border_radius: f32,
    pub toolbar_border_radius: f32,
    pub ui_border_radius: f32,
}

impl JsonTheme {
    /// Load theme from JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let theme: JsonTheme = serde_json::from_str(&contents)?;
        Ok(theme)
    }

    /// Save theme to JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

impl BezyTheme for JsonTheme {
    fn name(&self) -> &'static str {
        // This is a bit of a hack since we need a static str but have a String
        // In practice, theme names are known at compile time
        match self.name.as_str() {
            "Dark Mode" => "Dark Mode",
            "Light Mode" => "Light Mode",
            "Campfire" => "Campfire",
            "Ocean" => "Ocean",
            "Strawberry" => "Strawberry",
            _ => "Custom",
        }
    }

    // Unified UI text colors
    fn ui_text_primary(&self) -> Color {
        Color::srgb(
            self.ui_text_primary[0],
            self.ui_text_primary[1],
            self.ui_text_primary[2],
        )
    }

    fn ui_text_secondary(&self) -> Color {
        Color::srgb(
            self.ui_text_secondary[0],
            self.ui_text_secondary[1],
            self.ui_text_secondary[2],
        )
    }

    fn ui_text_tertiary(&self) -> Color {
        Color::srgb(
            self.ui_text_tertiary[0],
            self.ui_text_tertiary[1],
            self.ui_text_tertiary[2],
        )
    }

    fn ui_text_quaternary(&self) -> Color {
        Color::srgb(
            self.ui_text_quaternary[0],
            self.ui_text_quaternary[1],
            self.ui_text_quaternary[2],
        )
    }

    // Backgrounds
    fn background_color(&self) -> Color {
        Color::srgb(self.background[0], self.background[1], self.background[2])
    }

    fn widget_background_color(&self) -> Color {
        Color::srgba(
            self.widget_background[0],
            self.widget_background[1],
            self.widget_background[2],
            self.widget_background[3],
        )
    }

    fn widget_border_color(&self) -> Color {
        Color::srgb(
            self.widget_border[0],
            self.widget_border[1],
            self.widget_border[2],
        )
    }

    fn toolbar_background_color(&self) -> Color {
        Color::srgba(
            self.toolbar_background[0],
            self.toolbar_background[1],
            self.toolbar_background[2],
            self.toolbar_background[3],
        )
    }

    fn toolbar_icon_color(&self) -> Color {
        Color::srgb(
            self.toolbar_icon[0],
            self.toolbar_icon[1],
            self.toolbar_icon[2],
        )
    }

    fn toolbar_border_color(&self) -> Color {
        Color::srgba(
            self.toolbar_border[0],
            self.toolbar_border[1],
            self.toolbar_border[2],
            self.toolbar_border[3],
        )
    }

    fn panel_background_color(&self) -> Color {
        Color::srgb(
            self.panel_background[0],
            self.panel_background[1],
            self.panel_background[2],
        )
    }

    // Buttons
    fn normal_button_color(&self) -> Color {
        Color::srgb(
            self.normal_button[0],
            self.normal_button[1],
            self.normal_button[2],
        )
    }

    fn hovered_button_color(&self) -> Color {
        Color::srgb(
            self.hovered_button[0],
            self.hovered_button[1],
            self.hovered_button[2],
        )
    }

    fn pressed_button_color(&self) -> Color {
        Color::srgb(
            self.pressed_button[0],
            self.pressed_button[1],
            self.pressed_button[2],
        )
    }

    fn normal_button_outline_color(&self) -> Color {
        Color::srgb(
            self.normal_button_outline[0],
            self.normal_button_outline[1],
            self.normal_button_outline[2],
        )
    }

    fn hovered_button_outline_color(&self) -> Color {
        Color::srgb(
            self.hovered_button_outline[0],
            self.hovered_button_outline[1],
            self.hovered_button_outline[2],
        )
    }

    fn pressed_button_outline_color(&self) -> Color {
        Color::srgb(
            self.pressed_button_outline[0],
            self.pressed_button_outline[1],
            self.pressed_button_outline[2],
        )
    }

    fn pressed_button_icon_color(&self) -> Color {
        Color::srgb(
            self.pressed_button_icon[0],
            self.pressed_button_icon[1],
            self.pressed_button_icon[2],
        )
    }

    fn focus_background_color(&self) -> Color {
        Color::srgb(
            self.focus_background[0],
            self.focus_background[1],
            self.focus_background[2],
        )
    }

    fn text_editor_background_color(&self) -> Color {
        Color::srgb(
            self.text_editor_background[0],
            self.text_editor_background[1],
            self.text_editor_background[2],
        )
    }

    // Point colors
    fn on_curve_primary_color(&self) -> Color {
        Color::srgb(
            self.on_curve_primary[0],
            self.on_curve_primary[1],
            self.on_curve_primary[2],
        )
    }

    fn on_curve_secondary_color(&self) -> Color {
        Color::srgb(
            self.on_curve_secondary[0],
            self.on_curve_secondary[1],
            self.on_curve_secondary[2],
        )
    }

    fn off_curve_primary_color(&self) -> Color {
        Color::srgb(
            self.off_curve_primary[0],
            self.off_curve_primary[1],
            self.off_curve_primary[2],
        )
    }

    fn off_curve_secondary_color(&self) -> Color {
        Color::srgb(
            self.off_curve_secondary[0],
            self.off_curve_secondary[1],
            self.off_curve_secondary[2],
        )
    }

    // Path colors
    fn path_line_color(&self) -> Color {
        Color::srgba(
            self.path_line[0],
            self.path_line[1],
            self.path_line[2],
            self.path_line[3],
        )
    }

    fn path_stroke_color(&self) -> Color {
        Color::srgb(
            self.path_stroke[0],
            self.path_stroke[1],
            self.path_stroke[2],
        )
    }

    fn point_stroke_color(&self) -> Color {
        Color::srgba(
            self.point_stroke[0],
            self.point_stroke[1],
            self.point_stroke[2],
            self.point_stroke[3],
        )
    }

    fn handle_line_color(&self) -> Color {
        Color::srgba(
            self.handle_line[0],
            self.handle_line[1],
            self.handle_line[2],
            self.handle_line[3],
        )
    }

    // Semantic colors
    fn error_color(&self) -> Color {
        Color::srgb(self.error[0], self.error[1], self.error[2])
    }

    fn action_color(&self) -> Color {
        Color::srgb(self.action[0], self.action[1], self.action[2])
    }

    fn selected_color(&self) -> Color {
        Color::srgb(self.selected[0], self.selected[1], self.selected[2])
    }

    fn active_color(&self) -> Color {
        Color::srgb(self.active[0], self.active[1], self.active[2])
    }

    fn helper_color(&self) -> Color {
        Color::srgb(self.helper[0], self.helper[1], self.helper[2])
    }

    fn special_color(&self) -> Color {
        Color::srgb(self.special[0], self.special[1], self.special[2])
    }

    // Selection colors
    fn selected_primary_color(&self) -> Color {
        Color::srgba(
            self.selected_primary[0],
            self.selected_primary[1],
            self.selected_primary[2],
            self.selected_primary[3],
        )
    }

    fn selected_secondary_color(&self) -> Color {
        Color::srgba(
            self.selected_secondary[0],
            self.selected_secondary[1],
            self.selected_secondary[2],
            self.selected_secondary[3],
        )
    }

    fn hover_point_color(&self) -> Color {
        Color::srgba(
            self.hover_point[0],
            self.hover_point[1],
            self.hover_point[2],
            self.hover_point[3],
        )
    }

    fn hover_orange_color(&self) -> Color {
        Color::srgb(
            self.hover_orange[0],
            self.hover_orange[1],
            self.hover_orange[2],
        )
    }

    // Tool colors
    fn knife_line_color(&self) -> Color {
        Color::srgba(
            self.knife_line[0],
            self.knife_line[1],
            self.knife_line[2],
            self.knife_line[3],
        )
    }

    fn knife_intersection_color(&self) -> Color {
        Color::srgba(
            self.knife_intersection[0],
            self.knife_intersection[1],
            self.knife_intersection[2],
            self.knife_intersection[3],
        )
    }

    fn knife_start_point_color(&self) -> Color {
        Color::srgba(
            self.knife_start_point[0],
            self.knife_start_point[1],
            self.knife_start_point[2],
            self.knife_start_point[3],
        )
    }

    fn pen_point_color(&self) -> Color {
        Color::srgb(self.pen_point[0], self.pen_point[1], self.pen_point[2])
    }

    fn pen_start_point_color(&self) -> Color {
        Color::srgb(
            self.pen_start_point[0],
            self.pen_start_point[1],
            self.pen_start_point[2],
        )
    }

    fn pen_line_color(&self) -> Color {
        Color::srgba(
            self.pen_line[0],
            self.pen_line[1],
            self.pen_line[2],
            self.pen_line[3],
        )
    }

    fn hyper_point_color(&self) -> Color {
        Color::srgba(
            self.hyper_point[0],
            self.hyper_point[1],
            self.hyper_point[2],
            self.hyper_point[3],
        )
    }

    fn hyper_line_color(&self) -> Color {
        Color::srgba(
            self.hyper_line[0],
            self.hyper_line[1],
            self.hyper_line[2],
            self.hyper_line[3],
        )
    }

    fn hyper_close_indicator_color(&self) -> Color {
        Color::srgba(
            self.hyper_close_indicator[0],
            self.hyper_close_indicator[1],
            self.hyper_close_indicator[2],
            self.hyper_close_indicator[3],
        )
    }

    fn shape_preview_color(&self) -> Color {
        Color::srgba(
            self.shape_preview[0],
            self.shape_preview[1],
            self.shape_preview[2],
            self.shape_preview[3],
        )
    }

    // Metaballs
    fn metaball_gizmo_color(&self) -> Color {
        Color::srgba(
            self.metaball_gizmo[0],
            self.metaball_gizmo[1],
            self.metaball_gizmo[2],
            self.metaball_gizmo[3],
        )
    }

    fn metaball_outline_color(&self) -> Color {
        Color::srgba(
            self.metaball_outline[0],
            self.metaball_outline[1],
            self.metaball_outline[2],
            self.metaball_outline[3],
        )
    }

    fn metaball_selected_color(&self) -> Color {
        Color::srgba(
            self.metaball_selected[0],
            self.metaball_selected[1],
            self.metaball_selected[2],
            self.metaball_selected[3],
        )
    }

    // Guides
    fn metrics_guide_color(&self) -> Color {
        Color::srgba(
            self.metrics_guide[0],
            self.metrics_guide[1],
            self.metrics_guide[2],
            self.metrics_guide[3],
        )
    }

    fn checkerboard_color_1(&self) -> Color {
        Color::srgb(
            self.checkerboard_color_1[0],
            self.checkerboard_color_1[1],
            self.checkerboard_color_1[2],
        )
    }

    fn checkerboard_color_2(&self) -> Color {
        Color::srgb(
            self.checkerboard_color_2[0],
            self.checkerboard_color_2[1],
            self.checkerboard_color_2[2],
        )
    }

    fn checkerboard_color(&self) -> Color {
        Color::srgba(
            self.checkerboard[0],
            self.checkerboard[1],
            self.checkerboard[2],
            self.checkerboard[3],
        )
    }

    // Sort colors
    fn sort_active_metrics_color(&self) -> Color {
        Color::srgba(
            self.sort_active_metrics[0],
            self.sort_active_metrics[1],
            self.sort_active_metrics[2],
            self.sort_active_metrics[3],
        )
    }

    fn sort_inactive_metrics_color(&self) -> Color {
        Color::srgba(
            self.sort_inactive_metrics[0],
            self.sort_inactive_metrics[1],
            self.sort_inactive_metrics[2],
            self.sort_inactive_metrics[3],
        )
    }

    fn sort_active_outline_color(&self) -> Color {
        Color::srgb(
            self.sort_active_outline[0],
            self.sort_active_outline[1],
            self.sort_active_outline[2],
        )
    }

    fn sort_inactive_outline_color(&self) -> Color {
        Color::srgb(
            self.sort_inactive_outline[0],
            self.sort_inactive_outline[1],
            self.sort_inactive_outline[2],
        )
    }

    fn filled_glyph_color(&self) -> Color {
        Color::srgb(
            self.filled_glyph[0],
            self.filled_glyph[1],
            self.filled_glyph[2],
        )
    }

    // Border radius properties
    fn widget_border_radius(&self) -> f32 {
        self.widget_border_radius
    }

    fn toolbar_border_radius(&self) -> f32 {
        self.toolbar_border_radius
    }

    fn ui_border_radius(&self) -> f32 {
        self.ui_border_radius
    }
}

/// JSON theme manager that watches for file changes
#[derive(Resource)]
pub struct JsonThemeManager {
    themes_dir: PathBuf,
    loaded_themes: HashMap<String, JsonTheme>,
    file_timestamps: HashMap<String, SystemTime>,
    check_timer: Timer,
}

impl Default for JsonThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonThemeManager {
    pub fn new() -> Self {
        // Use user themes directory if it exists, otherwise use empty path
        let themes_dir = if embedded_themes::user_themes_dir_exists() {
            embedded_themes::get_user_themes_dir()
        } else {
            PathBuf::new()
        };

        Self {
            themes_dir,
            loaded_themes: HashMap::new(),
            file_timestamps: HashMap::new(),
            check_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        }
    }

    /// Load all JSON theme files from the themes directory
    pub fn load_all_themes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // First try to load from user directory if it exists
        if self.themes_dir.exists() && !self.themes_dir.as_os_str().is_empty() {
            for entry in fs::read_dir(&self.themes_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        match JsonTheme::load_from_file(&path) {
                            Ok(theme) => {
                                info!("Loaded user theme: {}", theme.name);
                                self.loaded_themes.insert(stem.to_string(), theme);

                                if let Ok(metadata) = fs::metadata(&path) {
                                    if let Ok(modified) = metadata.modified() {
                                        self.file_timestamps.insert(stem.to_string(), modified);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to load theme from {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        // If no themes loaded from user dir, load embedded themes
        if self.loaded_themes.is_empty() {
            for (name, content) in embedded_themes::get_embedded_themes() {
                match embedded_themes::load_theme_from_string(content) {
                    Ok(theme) => {
                        info!("Loaded embedded theme: {}", theme.name);
                        self.loaded_themes.insert(name, theme);
                    }
                    Err(e) => {
                        error!("Failed to load embedded theme: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Check for theme file changes and reload if needed
    pub fn check_for_changes(&mut self) -> Vec<String> {
        let mut changed_themes = Vec::new();

        if !self.themes_dir.exists() {
            return changed_themes;
        }

        if let Ok(entries) = fs::read_dir(&self.themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(metadata) = fs::metadata(&path) {
                            if let Ok(modified) = metadata.modified() {
                                let should_reload = match self.file_timestamps.get(stem) {
                                    Some(&last_modified) => {
                                        let is_newer = modified > last_modified;
                                        is_newer
                                    }
                                    None => true,
                                };

                                if should_reload {
                                    match JsonTheme::load_from_file(&path) {
                                        Ok(theme) => {
                                            self.loaded_themes.insert(stem.to_string(), theme);
                                            self.file_timestamps.insert(stem.to_string(), modified);
                                            changed_themes.push(stem.to_string());
                                        }
                                        Err(e) => {
                                            println!(
                                                "❌ Failed to reload theme from {path:?}: {e}"
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        changed_themes
    }

    /// Get a theme by name
    pub fn get_theme(&self, name: &str) -> Option<&JsonTheme> {
        self.loaded_themes.get(name)
    }

    /// Get all available theme names
    pub fn get_theme_names(&self) -> Vec<String> {
        self.loaded_themes.keys().cloned().collect()
    }
}

/// System to check for theme file changes
pub fn check_json_theme_changes(
    theme_manager: Option<ResMut<JsonThemeManager>>,
    mut current_theme: ResMut<CurrentTheme>,
    mut clear_color: ResMut<ClearColor>,
    time: Res<Time>,
) {
    let Some(mut theme_manager) = theme_manager else {
        return;
    };
    theme_manager.check_timer.tick(time.delta());

    if theme_manager.check_timer.just_finished() {
        let changed_themes = theme_manager.check_for_changes();

        !changed_themes.is_empty();

        // If the current theme was changed, reload it
        let current_name = current_theme.variant.name().to_string();
        if changed_themes.contains(&current_name) {
            // Force reload from user directory if it exists
            let user_themes_dir = embedded_themes::get_user_themes_dir();
            let user_json_path = user_themes_dir.join(format!("{current_name}.json"));

            if user_json_path.exists() {
                if let Ok(json_theme) = JsonTheme::load_from_file(&user_json_path) {
                    current_theme.set_theme(Box::new(json_theme));

                    // Update the background color immediately
                    clear_color.0 = current_theme.theme().background_color();
                }
            }
        }
    }
}

use super::CurrentTheme;

/// System to update all theme properties when theme changes
pub fn update_border_radius_on_theme_change(
    theme: Res<CurrentTheme>,
    mut widget_query: Query<
        (&mut BorderRadius, &mut BackgroundColor, &mut BorderColor),
        With<WidgetBorderRadius>,
    >,
    mut toolbar_query: Query<
        (&mut BorderRadius, &mut BackgroundColor, &mut BorderColor),
        (With<ToolbarBorderRadius>, Without<WidgetBorderRadius>),
    >,
    mut ui_query: Query<
        (&mut BorderRadius, &mut BackgroundColor, &mut BorderColor),
        (
            With<UiBorderRadius>,
            Without<WidgetBorderRadius>,
            Without<ToolbarBorderRadius>,
        ),
    >,
    mut clear_color: ResMut<ClearColor>,
) {
    // Only update in debug mode for hot-reload development
    #[cfg(debug_assertions)]
    if theme.is_changed() {
        // Update background color
        clear_color.0 = theme.theme().background_color();

        // Update widget properties
        for (mut border_radius, mut bg_color, mut border_color) in widget_query.iter_mut() {
            *border_radius = BorderRadius::all(Val::Px(theme.theme().widget_border_radius()));
            *bg_color = BackgroundColor(theme.theme().widget_background_color());
            *border_color = BorderColor(theme.theme().widget_border_color());
        }

        // Update toolbar properties
        for (mut border_radius, mut bg_color, mut border_color) in toolbar_query.iter_mut() {
            *border_radius = BorderRadius::all(Val::Px(theme.theme().toolbar_border_radius()));
            *bg_color = BackgroundColor(theme.theme().toolbar_background_color());
            *border_color = BorderColor(theme.theme().toolbar_border_color());
        }

        // Update UI properties
        for (mut border_radius, mut bg_color, mut border_color) in ui_query.iter_mut() {
            *border_radius = BorderRadius::all(Val::Px(theme.theme().ui_border_radius()));
            *bg_color = BackgroundColor(theme.theme().widget_background_color());
            *border_color = BorderColor(theme.theme().widget_border_color());
        }
    }
}

/// System to update text colors and other UI elements when theme changes
///
/// Note: This system is disabled because it was overriding specific UI text colors
/// (like ui_text_label and ui_text_value) with normal_text_color, preventing
/// proper color differentiation in panes.
pub fn update_ui_colors_on_theme_change(
    _theme: Res<CurrentTheme>,
    _text_query: Query<&mut TextColor>,
) {
    // This system is intentionally disabled to preserve specific text colors
    // set by individual UI components (like pane labels vs values)
}
