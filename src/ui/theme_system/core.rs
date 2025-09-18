//! Core theming infrastructure
//!
//! This module contains the fundamental building blocks of the theming system:
//! - BezyTheme trait that defines all visual constants
//! - ThemeRegistry for theme discovery and management
//! - CurrentTheme resource for runtime theme access
//! - ThemeVariant for theme identification and CLI integration

use bevy::prelude::*;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::OnceLock;

use super::{embedded_themes, json_theme};
use crate::ui::themes::{campfire, darkmode, lightmode, strawberry};

// =================================================================
// THEME REGISTRY - AUTOMATIC THEME DISCOVERY
// =================================================================

/// Theme registry that loads themes from JSON files
pub struct ThemeRegistry {
    themes: HashMap<String, Box<dyn BezyTheme>>,
}

impl ThemeRegistry {
    /// Create a new registry and load all JSON themes
    pub fn new() -> Self {
        let mut registry = Self {
            themes: HashMap::new(),
        };

        // Load themes from JSON files
        registry.load_json_themes();

        // Fallback to built-in themes if no JSON themes found
        if registry.themes.is_empty() {
            registry.load_builtin_themes();
        }

        registry
    }

    /// Load themes from JSON files
    fn load_json_themes(&mut self) {
        // Check if user themes directory exists
        let user_themes_dir = embedded_themes::get_user_themes_dir();

        if user_themes_dir.exists() {
            // User has custom themes directory, load from there
            info!("Loading themes from user directory: {:?}", user_themes_dir);

            if let Ok(entries) = std::fs::read_dir(&user_themes_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            match json_theme::JsonTheme::load_from_file(&path) {
                                Ok(theme) => {
                                    info!("Loaded user theme: {}", theme.name);
                                    self.themes.insert(stem.to_string(), Box::new(theme));
                                }
                                Err(e) => {
                                    error!("Failed to load user theme from {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // No user directory, load embedded themes
            info!("No user themes directory found, loading embedded themes");

            for (name, content) in embedded_themes::get_embedded_themes() {
                match embedded_themes::load_theme_from_string(content) {
                    Ok(theme) => {
                        info!("Loaded embedded theme: {}", theme.name);
                        self.themes.insert(name, Box::new(theme));
                    }
                    Err(e) => {
                        error!("Failed to load embedded theme {}: {}", name, e);
                    }
                }
            }
        }
    }

    /// Fallback to built-in Rust themes
    fn load_builtin_themes(&mut self) {
        warn!("No JSON themes found, using built-in themes");

        self.themes
            .insert("darkmode".to_string(), Box::new(darkmode::DarkModeTheme));
        self.themes
            .insert("lightmode".to_string(), Box::new(lightmode::LightModeTheme));
        self.themes.insert(
            "strawberry".to_string(),
            Box::new(strawberry::StrawberryTheme),
        );
        self.themes
            .insert("campfire".to_string(), Box::new(campfire::CampfireTheme));
    }

    /// Get all available theme names
    pub fn get_theme_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.themes.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get a theme by name (returns a reference)
    pub fn get_theme(&self, name: &str) -> Option<&dyn BezyTheme> {
        self.themes.get(name).map(|theme| theme.as_ref())
    }

    /// Create a theme instance by name (clones the theme)
    pub fn create_theme(&self, name: &str) -> Option<Box<dyn BezyTheme>> {
        // For JSON themes, we need to reload them to get fresh data
        if let Some(_theme) = self.themes.get(name) {
            // Check if user themes directory exists and has this theme
            let user_themes_dir = embedded_themes::get_user_themes_dir();
            let user_json_path = user_themes_dir.join(format!("{}.json", name));

            if user_json_path.exists() {
                if let Ok(json_theme) = json_theme::JsonTheme::load_from_file(&user_json_path) {
                    return Some(Box::new(json_theme));
                }
            }

            // Fallback to embedded theme
            if let Some(content) = embedded_themes::get_embedded_themes().get(name) {
                if let Ok(json_theme) = embedded_themes::load_theme_from_string(content) {
                    return Some(Box::new(json_theme));
                }
            }

            // If all else fails, return None
            None
        } else {
            None
        }
    }

    /// Check if a theme exists
    pub fn has_theme(&self, name: &str) -> bool {
        self.themes.contains_key(name)
    }

    /// Register a new theme (for dynamic registration)
    pub fn register_theme<T: BezyTheme + Default + 'static>(&mut self, name: String) {
        self.themes.insert(name, Box::new(T::default()));
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global theme registry instance
static GLOBAL_REGISTRY: OnceLock<ThemeRegistry> = OnceLock::new();

/// Get the global theme registry
pub fn get_theme_registry() -> &'static ThemeRegistry {
    GLOBAL_REGISTRY.get_or_init(ThemeRegistry::new)
}

// =================================================================
// THEME VARIANT - CLI INTEGRATION
// =================================================================

/// Legacy ThemeVariant for backward compatibility
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeVariant {
    name: String,
}

impl ThemeVariant {
    /// Create a new theme variant
    pub fn new(name: String) -> Self {
        Self { name }
    }

    /// Get all available theme names for CLI help
    pub fn all_names() -> Vec<String> {
        get_theme_registry().get_theme_names()
    }

    /// Parse theme name from string (case-insensitive)
    pub fn parse(s: &str) -> Option<Self> {
        let registry = get_theme_registry();
        let name = s.to_lowercase();

        if registry.has_theme(&name) {
            Some(Self::new(name))
        } else {
            None
        }
    }

    /// Get the display name of this theme
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Default for ThemeVariant {
    fn default() -> Self {
        Self::new("darkmode".to_string())
    }
}

impl FromStr for ThemeVariant {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| format!("Unknown theme: '{s}'"))
    }
}

// =================================================================
// BEZY THEME TRAIT - CORE INTERFACE
// =================================================================

/// Main theme trait that defines all visual styling constants
///
/// Every theme must implement this trait to provide all visual constants
/// used throughout the application. This ensures themes are complete and
/// prevents any hardcoded visual constants from remaining in the codebase.
pub trait BezyTheme: Send + Sync + 'static {
    /// Get the display name of this theme
    fn name(&self) -> &'static str;

    // =================================================================
    // TYPOGRAPHY
    // =================================================================

    /// Font file paths
    fn grotesk_font_path(&self) -> &'static str {
        "fonts/bezy-grotesk-regular.ttf"
    }
    fn mono_font_path(&self) -> &'static str {
        "fonts/HasubiMono-Regular.ttf"
    }

    /// Font sizes
    fn widget_title_font_size(&self) -> f32 {
        24.0
    }
    fn widget_text_font_size(&self) -> f32 {
        24.0
    }
    fn default_font_size(&self) -> f32 {
        16.0
    }
    fn small_font_size(&self) -> f32 {
        12.0
    }
    fn large_font_size(&self) -> f32 {
        20.0
    }
    fn button_icon_font_size(&self) -> f32 {
        48.0
    }
    fn sort_text_font_size(&self) -> f32 {
        24.0
    }

    /// Text colors
    fn normal_text_color(&self) -> Color;
    fn secondary_text_color(&self) -> Color;
    fn highlight_text_color(&self) -> Color;

    // =================================================================
    // LAYOUT & SPACING
    // =================================================================

    /// Widget layout
    fn widget_padding(&self) -> f32 {
        16.0
    }
    fn widget_margin(&self) -> f32 {
        24.0
    }
    fn widget_row_gap(&self) -> f32 {
        0.0
    }
    fn widget_border_width(&self) -> f32 {
        2.0
    }
    fn widget_border_radius(&self) -> f32 {
        0.0
    }

    /// Toolbar layout
    fn toolbar_padding(&self) -> f32 {
        0.0
    }
    fn toolbar_container_margin(&self) -> f32 {
        16.0
    }
    fn toolbar_item_spacing(&self) -> f32 {
        4.0
    }
    fn toolbar_border_width(&self) -> f32 {
        2.0
    }
    fn toolbar_border_radius(&self) -> f32 {
        0.0
    }

    /// Button sizes
    fn toolbar_button_size(&self) -> f32 {
        64.0
    }
    fn button_icon_size(&self) -> f32 {
        48.0
    }

    /// UI margins and spacing (from scattered constants)
    fn selection_margin(&self) -> f32 {
        16.0
    }
    fn text_margin(&self) -> f32 {
        16.0
    }
    fn ui_small_margin(&self) -> f32 {
        4.0
    }
    fn ui_medium_margin(&self) -> f32 {
        8.0
    }
    fn ui_large_margin(&self) -> f32 {
        24.0
    }
    fn ui_xlarge_margin(&self) -> f32 {
        96.0
    }
    fn metaballs_padding(&self) -> f32 {
        20.0
    }

    /// Border radii
    fn ui_border_radius(&self) -> f32 {
        12.0
    }

    // =================================================================
    // COLORS - BACKGROUNDS & UI
    // =================================================================

    /// Main background
    fn background_color(&self) -> Color;

    /// Widget colors  
    fn widget_background_color(&self) -> Color;
    fn widget_border_color(&self) -> Color;

    /// Toolbar colors
    fn toolbar_background_color(&self) -> Color;
    fn toolbar_icon_color(&self) -> Color;
    fn toolbar_border_color(&self) -> Color;

    /// Panel colors
    fn panel_background_color(&self) -> Color;

    /// Button colors
    fn normal_button_color(&self) -> Color;
    fn hovered_button_color(&self) -> Color;
    fn pressed_button_color(&self) -> Color;

    /// Button outline colors
    fn normal_button_outline_color(&self) -> Color;
    fn hovered_button_outline_color(&self) -> Color;
    fn pressed_button_outline_color(&self) -> Color;
    fn pressed_button_icon_color(&self) -> Color;

    /// Focus and special states
    fn focus_background_color(&self) -> Color;
    fn text_editor_background_color(&self) -> Color;

    // =================================================================
    // GLYPH RENDERING
    // =================================================================

    /// Point rendering
    fn on_curve_point_radius(&self) -> f32 {
        4.0
    }
    fn off_curve_point_radius(&self) -> f32 {
        4.0
    }

    /// On-curve point colors (two-layer system)
    fn on_curve_primary_color(&self) -> Color;
    fn on_curve_secondary_color(&self) -> Color;

    /// Off-curve point colors (two-layer system)  
    fn off_curve_primary_color(&self) -> Color;
    fn off_curve_secondary_color(&self) -> Color;

    /// Point layout details
    fn on_curve_square_adjustment(&self) -> f32 {
        1.0
    }
    fn on_curve_inner_circle_ratio(&self) -> f32 {
        0.5
    }
    fn off_curve_inner_circle_ratio(&self) -> f32 {
        0.5
    }
    fn use_square_for_on_curve(&self) -> bool {
        true
    }

    /// Path rendering
    fn path_line_color(&self) -> Color;
    fn path_line_width(&self) -> f32 {
        2.0
    }
    fn path_stroke_color(&self) -> Color;
    fn point_stroke_color(&self) -> Color;

    /// Handle lines
    fn handle_line_color(&self) -> Color;

    // =================================================================
    // INFO COLORS - SEMANTIC COLORS
    // =================================================================

    /// Error color - used for error states and validation failures
    fn error_color(&self) -> Color;

    /// Action color - used for active actions and interactions (like selection marquee)
    fn action_color(&self) -> Color;

    /// Selected color - used for selected items and highlighted elements
    fn selected_color(&self) -> Color;

    /// Active color - used for active/current states
    fn active_color(&self) -> Color;

    /// Helper color - used for helper text and guidance
    fn helper_color(&self) -> Color;

    /// Special color - used for special states and emphasis
    fn special_color(&self) -> Color;

    // =================================================================
    // SELECTION & INTERACTION
    // =================================================================

    /// Selection styling
    fn selection_point_radius(&self) -> f32 {
        4.0
    }
    fn selected_circle_radius_multiplier(&self) -> f32 {
        1.0
    }
    fn selected_cross_size_multiplier(&self) -> f32 {
        1.0
    }
    /// Selected point colors (two-layer system for crosshairs)
    fn selected_primary_color(&self) -> Color;
    fn selected_secondary_color(&self) -> Color;

    /// Hover states
    fn hover_circle_radius_multiplier(&self) -> f32 {
        1.0
    }
    fn hover_point_color(&self) -> Color;
    fn hover_orange_color(&self) -> Color;

    /// Tool feedback
    fn text_cursor_radius(&self) -> f32 {
        12.0
    }

    // =================================================================
    // EDITING TOOLS
    // =================================================================

    /// Knife tool
    fn knife_line_color(&self) -> Color;
    fn knife_intersection_color(&self) -> Color;
    fn knife_start_point_color(&self) -> Color;
    fn knife_dash_length(&self) -> f32 {
        8.0
    }
    fn knife_gap_length(&self) -> f32 {
        4.0
    }
    fn knife_cross_size(&self) -> f32 {
        8.0
    }

    /// Pen tool
    fn pen_point_color(&self) -> Color;
    fn pen_start_point_color(&self) -> Color;
    fn pen_line_color(&self) -> Color;

    /// Hyper tool
    fn hyper_point_color(&self) -> Color;
    fn hyper_line_color(&self) -> Color;
    fn hyper_close_indicator_color(&self) -> Color;

    /// Shape tool
    fn shape_preview_color(&self) -> Color;

    // =================================================================
    // METABALLS
    // =================================================================

    fn metaball_gizmo_color(&self) -> Color;
    fn metaball_outline_color(&self) -> Color;
    fn metaball_selected_color(&self) -> Color;

    // =================================================================
    // GUIDES & GRIDS
    // =================================================================

    /// Metrics guides
    fn metrics_guide_color(&self) -> Color;

    /// Checkerboard grid
    fn checkerboard_enabled_by_default(&self) -> bool {
        true
    }
    fn checkerboard_default_unit_size(&self) -> f32 {
        32.0
    }
    fn checkerboard_color_1(&self) -> Color;
    fn checkerboard_color_2(&self) -> Color;
    fn checkerboard_color(&self) -> Color;
    fn checkerboard_scale_factor(&self) -> f32 {
        2.0
    }
    fn checkerboard_max_zoom_visible(&self) -> f32 {
        32.0
    }

    // =================================================================
    // SORTING & LAYOUT
    // =================================================================

    /// Sort visualization
    fn sort_active_metrics_color(&self) -> Color;
    fn sort_inactive_metrics_color(&self) -> Color;
    fn sort_active_outline_color(&self) -> Color;
    fn sort_inactive_outline_color(&self) -> Color;

    /// Filled glyph color for inactive sorts
    fn filled_glyph_color(&self) -> Color;

    /// Sort spacing
    fn sort_horizontal_padding(&self) -> f32 {
        256.0
    }
    fn sort_vertical_padding(&self) -> f32 {
        256.0
    }

    // =================================================================
    // RENDERING PERFORMANCE
    // =================================================================

    /// Checkerboard performance
    fn checkerboard_z_level(&self) -> f32 {
        0.1
    }
    fn min_visibility_zoom(&self) -> f32 {
        0.01
    }
    fn grid_size_change_threshold(&self) -> f32 {
        1.25
    }
    fn visible_area_coverage_multiplier(&self) -> f32 {
        1.2
    }
    fn max_squares_per_frame(&self) -> usize {
        2000
    }

    /// Camera limits
    fn min_allowed_zoom_scale(&self) -> f32 {
        0.1
    }
    fn max_allowed_zoom_scale(&self) -> f32 {
        64.0
    }
    fn initial_zoom_scale(&self) -> f32 {
        1.0
    }
    fn keyboard_zoom_step(&self) -> f32 {
        0.9
    }

    // =================================================================
    // WINDOW & GIZMOS
    // =================================================================

    fn window_title(&self) -> &'static str {
        "Bezy"
    }
    fn window_width(&self) -> f32 {
        1024.0
    }
    fn window_height(&self) -> f32 {
        768.0
    }

    fn gizmo_line_width(&self) -> f32 {
        3.0
    }
    fn line_leading(&self) -> f32 {
        0.0
    }

    // =================================================================
    // DEBUG
    // =================================================================

    fn debug_show_origin_cross(&self) -> bool {
        false
    }
}

// =================================================================
// CURRENT THEME RESOURCE - RUNTIME ACCESS
// =================================================================

/// Bevy resource that holds the current theme
///
/// This resource provides access to the current theme from any Bevy system.
/// The theme can be changed at runtime by updating this resource.
#[derive(Resource)]
pub struct CurrentTheme {
    pub variant: ThemeVariant,
    theme: Box<dyn BezyTheme>,
}

impl CurrentTheme {
    /// Create a new CurrentTheme with the specified variant
    pub fn new(variant: ThemeVariant) -> Self {
        let registry = get_theme_registry();

        // Try to load theme through the registry (which handles user dir and embedded themes)
        let theme = registry
            .create_theme(variant.name())
            .unwrap_or_else(|| Box::new(darkmode::DarkModeTheme));

        Self { variant, theme }
    }

    /// Get the current theme implementation
    pub fn theme(&self) -> &dyn BezyTheme {
        self.theme.as_ref()
    }

    /// Switch to a different theme variant
    pub fn switch_to(&mut self, variant: ThemeVariant) {
        if self.variant != variant {
            let registry = get_theme_registry();
            if let Some(theme) = registry.create_theme(variant.name()) {
                self.variant = variant;
                self.theme = theme;
            }
        }
    }

    /// Set the theme directly (for hot reloading)
    pub fn set_theme(&mut self, theme: Box<dyn BezyTheme>) {
        self.theme = theme;
    }
}

impl Default for CurrentTheme {
    fn default() -> Self {
        Self::new(ThemeVariant::default())
    }
}

/// Convenience functions for accessing theme values
///
/// These functions provide a simple way to access theme values from systems
/// without having to write out the full resource access pattern every time.
impl CurrentTheme {
    /// Get the normal text color for the current theme
    pub fn get_normal_text_color(&self) -> Color {
        self.theme().normal_text_color()
    }

    /// Get the secondary text color for the current theme
    pub fn get_secondary_text_color(&self) -> Color {
        self.theme().secondary_text_color()
    }

    /// Get the highlight text color for the current theme
    pub fn get_highlight_text_color(&self) -> Color {
        self.theme().highlight_text_color()
    }

    // =================================================================
    // INFO COLORS - SEMANTIC COLORS
    // =================================================================

    /// Get the error color for the current theme
    pub fn error_color(&self) -> Color {
        self.theme().error_color()
    }

    /// Get the action color for the current theme
    pub fn action_color(&self) -> Color {
        self.theme().action_color()
    }

    /// Get the selected color for the current theme
    pub fn selected_color(&self) -> Color {
        self.theme().selected_color()
    }

    /// Get the active color for the current theme
    pub fn active_color(&self) -> Color {
        self.theme().active_color()
    }

    /// Get the helper color for the current theme
    pub fn helper_color(&self) -> Color {
        self.theme().helper_color()
    }

    /// Get the special color for the current theme
    pub fn special_color(&self) -> Color {
        self.theme().special_color()
    }
}
