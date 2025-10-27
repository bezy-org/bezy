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

// =================================================================
// THEME REGISTRY - AUTOMATIC THEME DISCOVERY
// =================================================================

/// Simple theme registry with Rust-based themes
pub struct ThemeRegistry {
    themes: HashMap<String, Box<dyn BezyTheme>>,
}

impl ThemeRegistry {
    /// Create a new registry with built-in themes
    pub fn new() -> Self {
        use crate::ui::themes::*;

        let mut themes: HashMap<String, Box<dyn BezyTheme>> = HashMap::new();
        themes.insert("dark".to_string(), Box::new(DarkTheme));
        themes.insert("light".to_string(), Box::new(LightTheme));
        themes.insert("strawberry".to_string(), Box::new(StrawberryTheme));
        themes.insert("campfire".to_string(), Box::new(CampfireTheme));

        Self { themes }
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

    /// Create a theme instance by name
    pub fn create_theme(&self, name: &str) -> Option<Box<dyn BezyTheme>> {
        use crate::ui::themes::*;

        match name {
            "dark" => Some(Box::new(DarkTheme)),
            "light" => Some(Box::new(LightTheme)),
            "strawberry" => Some(Box::new(StrawberryTheme)),
            "campfire" => Some(Box::new(CampfireTheme)),
            _ => None,
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
        Self::new("dark".to_string())
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
        "fonts/BezyGrotesk-Regular.ttf"
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

    /// Unified UI text colors
    fn ui_text_primary(&self) -> Color;
    fn ui_text_secondary(&self) -> Color;
    fn ui_text_tertiary(&self) -> Color;
    fn ui_text_quaternary(&self) -> Color;

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

    /// Button colors - covers buttons, toolbars, and interactive elements
    fn button_regular(&self) -> Color;
    fn button_hovered(&self) -> Color;
    fn button_pressed(&self) -> Color;
    fn button_regular_outline(&self) -> Color;
    fn button_hovered_outline(&self) -> Color;
    fn button_pressed_outline(&self) -> Color;
    fn button_regular_icon(&self) -> Color;
    fn button_hovered_icon(&self) -> Color;
    fn button_pressed_icon(&self) -> Color;

    /// Focus and special states
    fn focus_background_color(&self) -> Color;
    fn text_editor_background_color(&self) -> Color;

    // =================================================================
    // GLYPH RENDERING
    // =================================================================

    /// Point rendering
    // These values are multiplied by zoom-aware scaling factors (see zoom_aware_scaling.rs)
    // Larger values = bigger points at all zoom levels
    fn on_curve_point_radius(&self) -> f32 {
        3.0
    }
    fn off_curve_point_radius(&self) -> f32 {
        3.0
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
        0.25
    }
    fn off_curve_inner_circle_ratio(&self) -> f32 {
        0.25
    }
    fn use_square_for_on_curve(&self) -> bool {
        true
    }

    /// Path rendering
    fn path_line_color(&self) -> Color;
    // Larger values = thicker lines at all zoom levels
    fn path_line_width(&self) -> f32 {
        1.0
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

        // Load theme through the registry
        let theme = registry.create_theme(variant.name()).unwrap_or_else(|| {
            // Fallback to dark theme
            use crate::ui::themes::DarkTheme;
            warn!("Theme '{}' not found, falling back to dark theme", variant.name());
            Box::new(DarkTheme)
        });

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
    /// Get the primary UI text color (most prominent text)
    pub fn get_ui_text_primary(&self) -> Color {
        self.theme().ui_text_primary()
    }

    /// Get the secondary UI text color (labels, less prominent text)
    pub fn get_ui_text_secondary(&self) -> Color {
        self.theme().ui_text_secondary()
    }

    /// Get the tertiary UI text color (supporting text, hints)
    pub fn get_ui_text_tertiary(&self) -> Color {
        self.theme().ui_text_tertiary()
    }

    /// Get the quaternary UI text color (subtle text, placeholders)
    pub fn get_ui_text_quaternary(&self) -> Color {
        self.theme().ui_text_quaternary()
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
