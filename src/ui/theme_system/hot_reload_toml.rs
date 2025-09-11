//! Hot reloading system using external TOML config files
//!
//! This approach allows editing colors in TOML files that are loaded at runtime,
//! making it easier to tweak colors without recompiling.

use super::{BezyTheme, CurrentTheme};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Color configuration that can be loaded from TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    // Typography colors
    pub normal_text: [f32; 3],
    pub secondary_text: [f32; 3],
    pub highlight_text: [f32; 3],
    
    // Background colors
    pub background: [f32; 3],
    pub widget_background: [f32; 4],
    
    // Point colors
    pub on_curve_primary: [f32; 3],
    pub on_curve_secondary: [f32; 3],
    pub off_curve_primary: [f32; 3],
    pub off_curve_secondary: [f32; 3],
    
    // Selection colors
    pub selected_primary: [f32; 4],
    pub selected_secondary: [f32; 4],
    
    // Path colors
    pub path_stroke: [f32; 3],
    pub handle_line: [f32; 4],
}

impl ThemeConfig {
    /// Load theme config from a TOML file
    pub fn load_from_file(path: &PathBuf) -> Option<Self> {
        match fs::read_to_string(path) {
            Ok(contents) => {
                match toml::from_str(&contents) {
                    Ok(config) => Some(config),
                    Err(e) => {
                        error!("Failed to parse theme TOML: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Failed to read theme file: {}", e);
                None
            }
        }
    }
    
    /// Save current theme config to a TOML file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;
        Ok(())
    }
}

/// Creates a default theme config from the current theme
pub fn create_theme_config(theme: &dyn BezyTheme) -> ThemeConfig {
    ThemeConfig {
        normal_text: color_to_array(theme.normal_text_color()),
        secondary_text: color_to_array(theme.secondary_text_color()),
        highlight_text: color_to_array(theme.highlight_text_color()),
        
        background: color_to_array(theme.background_color()),
        widget_background: color_to_array_with_alpha(theme.widget_background_color()),
        
        on_curve_primary: color_to_array(theme.on_curve_primary_color()),
        on_curve_secondary: color_to_array(theme.on_curve_secondary_color()),
        off_curve_primary: color_to_array(theme.off_curve_primary_color()),
        off_curve_secondary: color_to_array(theme.off_curve_secondary_color()),
        
        selected_primary: color_to_array_with_alpha(theme.selected_primary_color()),
        selected_secondary: color_to_array_with_alpha(theme.selected_secondary_color()),
        
        path_stroke: color_to_array(theme.path_stroke_color()),
        handle_line: color_to_array_with_alpha(theme.handle_line_color()),
    }
}

fn color_to_array(color: Color) -> [f32; 3] {
    let rgba = color.to_srgba();
    [rgba.red, rgba.green, rgba.blue]
}

fn color_to_array_with_alpha(color: Color) -> [f32; 4] {
    let rgba = color.to_srgba();
    [rgba.red, rgba.green, rgba.blue, rgba.alpha]
}

/// System to export current theme to TOML for editing
pub fn export_theme_to_toml(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_theme: Res<CurrentTheme>,
) {
    // Theme export functionality disabled - using Cmd+E for TTF export instead
    // If theme export is needed in the future, use a different shortcut like Cmd+Shift+T
}