//! UFO file I/O operations

#[allow(unused_imports)]
use crate::core::cli::CliArgs;
use crate::core::state::AppState;
use anyhow::Result;
use bevy::prelude::Res;
use bevy::prelude::*;
use norad::Font;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Load a UFO font file from disk
#[allow(dead_code)]
pub fn load_ufo_from_path(path: impl AsRef<Path>) -> Result<Font> {
    let font = Font::load(path)?;
    Ok(font)
}

/// Load a UFO font file from disk (compatibility version)
/// This version matches the old API signature for compatibility
#[allow(dead_code)]
pub fn load_ufo_from_path_compat(path: &str) -> Result<Font, Box<dyn std::error::Error>> {
    let font_path = PathBuf::from(path);

    if !font_path.exists() {
        let error_msg = format!("File not found: {}", font_path.display());
        error!("{}", error_msg);
        return Err(error_msg.into());
    }

    let font = Font::load(font_path)?;
    Ok(font)
}

// Unicode Codepoint Mapping --------------------------------------------------

/// Convert a Unicode character to its hex codepoint string
fn char_to_hex_codepoint(unicode_char: char) -> String {
    format!("{:04X}", unicode_char as u32)
}

/// Add all codepoints from a glyph to the mapping
fn add_glyph_codepoints_to_map(map: &mut HashMap<String, String>, glyph: &norad::Glyph) {
    for &unicode_char in &glyph.codepoints {
        let codepoint_hex = char_to_hex_codepoint(unicode_char);
        map.insert(codepoint_hex, glyph.name().to_string());
    }
}

/// Build a map of Unicode codepoints to glyph names for efficient lookups
fn build_codepoint_glyph_map(font: &Font) -> HashMap<String, String> {
    let mut codepoint_to_glyph = HashMap::new();

    let layer = font.default_layer();

    for glyph in layer.iter() {
        add_glyph_codepoints_to_map(&mut codepoint_to_glyph, glyph);
    }
    codepoint_to_glyph
}

// Public Glyph Lookup API ---------------------------------------------------

/// Find a glyph by its Unicode codepoint (like "0041" for letter A)
#[allow(dead_code)]
pub fn find_glyph_by_unicode(font: &Font, codepoint_hex: &str) -> Option<String> {
    build_codepoint_glyph_map(font).get(codepoint_hex).cloned()
}

/// Get all Unicode codepoints that have glyphs in this font
#[allow(dead_code)]
pub fn get_all_codepoints(font: &Font) -> Vec<String> {
    let map = build_codepoint_glyph_map(font);
    let mut codepoints: Vec<String> = map.keys().cloned().collect();
    codepoints.sort_unstable();
    codepoints.dedup();

    debug!("Found {} codepoints in font", codepoints.len());
    codepoints
}

// Codepoint Navigation ------------------------------------------------------

/// Direction for cycling through codepoints
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum CycleDirection {
    Next,
    Previous,
}

/// Get the appropriate starting position based on direction
#[allow(dead_code)]
fn get_direction_default(codepoints: &[String], direction: CycleDirection) -> Option<String> {
    match direction {
        CycleDirection::Next => codepoints.first().cloned(),
        CycleDirection::Previous => codepoints.last().cloned(),
    }
}

/// Find the position of a codepoint in the list
#[allow(dead_code)]
fn find_codepoint_position(codepoints: &[String], target: &str) -> Option<usize> {
    codepoints.iter().position(|codepoint| codepoint == target)
}

/// Cycle to the next or previous codepoint in the font
#[allow(dead_code)]
pub fn cycle_codepoint_in_list(
    available_codepoints: &[String],
    current_codepoint: &str,
    direction: CycleDirection,
) -> Option<String> {
    if available_codepoints.is_empty() {
        return None;
    }

    // If no current codepoint, start at appropriate end based on direction
    if current_codepoint.is_empty() {
        return get_direction_default(available_codepoints, direction);
    }

    // Find where we are in the list
    let current_position = find_codepoint_position(available_codepoints, current_codepoint);

    let Some(current_position) = current_position else {
        // Current codepoint not found, start from the end based on direction
        return get_direction_default(available_codepoints, direction);
    };

    match direction {
        CycleDirection::Next => {
            let next_position = current_position + 1;
            if next_position < available_codepoints.len() {
                Some(available_codepoints[next_position].clone())
            } else {
                // Wrap around to beginning
                available_codepoints.first().cloned()
            }
        }
        CycleDirection::Previous => {
            if current_position > 0 {
                Some(available_codepoints[current_position - 1].clone())
            } else {
                // Wrap around to end
                available_codepoints.last().cloned()
            }
        }
    }
}

/// Move to the next codepoint in the font
#[allow(dead_code)]
pub fn find_next_codepoint_in_list(
    available_codepoints: &[String],
    current_codepoint: &str,
) -> Option<String> {
    cycle_codepoint_in_list(
        available_codepoints,
        current_codepoint,
        CycleDirection::Next,
    )
}

/// Move to the previous codepoint in the font
#[allow(dead_code)]
pub fn find_previous_codepoint_in_list(
    available_codepoints: &[String],
    current_codepoint: &str,
) -> Option<String> {
    cycle_codepoint_in_list(
        available_codepoints,
        current_codepoint,
        CycleDirection::Previous,
    )
}

// File Loading and Initialization (Compatibility Functions) ------------------

/// Set up the font when the app starts (compatibility function)
/// This provides the same API as the old version but works with the new architecture
#[allow(dead_code)]
pub fn initialize_font_state(mut commands: Commands, cli_args: Res<crate::core::cli::CliArgs>) {
    if let Some(font_path) = &cli_args.ufo_path {
        load_font_at_startup(&mut commands, font_path);
    } else {
        // No font specified, start with empty state
        commands.init_resource::<AppState>();
    }
}

/// Load and set up a font when the app starts (compatibility function)
/// This provides the same API as the old version but works with the new architecture
#[allow(dead_code)]
fn load_font_at_startup(commands: &mut Commands, font_path: &Path) {
    let path_string = font_path.to_str().unwrap_or_default();

    match load_ufo_from_path_compat(path_string) {
        Ok(_font) => {
            // Successfully loaded font - initialize AppState and let the main app system handle loading
            let mut app_state = AppState::default();
            match app_state.load_font_from_path(font_path.to_path_buf()) {
                Ok(_) => {
                    let font_name = app_state.get_font_display_name();
                    commands.insert_resource(app_state);
                    info!("Loaded font: {}", font_name);
                }
                Err(error) => {
                    error!("Failed to initialize font state: {}", error);
                    commands.init_resource::<AppState>();
                }
            }
        }
        Err(error) => {
            // Failed to load font
            error!("Failed to load UFO file: {}", error);
            commands.init_resource::<AppState>();
        }
    }
}
