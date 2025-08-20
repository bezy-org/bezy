//! Glyph navigation and codepoint utilities
//!
//! This module provides functionality for navigating between glyphs
//! using Unicode codepoints and cycling through available glyphs.

use crate::core::state::app_state::AppState;
use bevy::prelude::*;

/// Glyph navigation state
#[derive(Resource)]
pub struct GlyphNavigation {
    /// The current Unicode codepoint being viewed (like "0061" for 'a')
    pub current_codepoint: Option<String>,
    /// Whether we found this codepoint in the loaded font
    pub codepoint_found: bool,
    /// Legacy fields for compatibility
    pub current_glyph: Option<String>,
}

impl Default for GlyphNavigation {
    fn default() -> Self {
        Self {
            current_codepoint: None,
            codepoint_found: false,
            current_glyph: Some("a".to_string()),
        }
    }
}

/// Navigation direction for cycling through codepoints
#[derive(Clone, Debug)]
pub enum CycleDirection {
    Next,
    Previous,
}

impl GlyphNavigation {
    /// Change to a different codepoint
    pub fn set_codepoint(&mut self, new_codepoint: String) {
        self.current_codepoint = Some(new_codepoint);
        self.codepoint_found = false; // We'll need to check if this exists
    }

    /// Get the current codepoint as a string for display
    pub fn get_codepoint_string(&self) -> String {
        self.current_codepoint.clone().unwrap_or_default()
    }

    /// Find the glyph name for the current codepoint
    pub fn find_glyph(&self, app_state: &AppState) -> Option<String> {
        self.current_codepoint.as_ref().and_then(|codepoint| {
            find_glyph_by_unicode_codepoint(app_state, codepoint)
        })
    }
}

/// Find a glyph by Unicode codepoint in the app state
pub fn find_glyph_by_unicode_codepoint(
    app_state: &AppState,
    codepoint: &str,
) -> Option<String> {
    // Parse the codepoint string to a character
    if let Ok(codepoint_num) = u32::from_str_radix(codepoint, 16) {
        if let Some(ch) = char::from_u32(codepoint_num) {
            // Search through all glyphs for one with this unicode value
            for (glyph_name, glyph_data) in &app_state.workspace.font.glyphs {
                if glyph_data.unicode_values.contains(&ch) {
                    return Some(glyph_name.clone());
                }
            }
        }
    }
    None
}

/// Get all unicode codepoints available in the font
pub fn get_all_codepoints(app_state: &AppState) -> Vec<String> {
    let mut codepoints = Vec::new();

    for glyph_data in app_state.workspace.font.glyphs.values() {
        for &unicode_char in &glyph_data.unicode_values {
            let codepoint = format!("{:04X}", unicode_char as u32);
            if !codepoints.contains(&codepoint) {
                codepoints.push(codepoint);
            }
        }
    }

    // Sort and return
    codepoints.sort();
    codepoints
}

/// Find the next or previous codepoint in the font's available codepoints
pub fn cycle_codepoint_in_list(
    current_codepoint: Option<String>,
    app_state: &AppState,
    direction: CycleDirection,
) -> Option<String> {
    let codepoints = get_all_codepoints(app_state);

    if codepoints.is_empty() {
        return None;
    }

    // If no current codepoint, return the first one
    let current = match current_codepoint {
        Some(cp) => cp,
        None => return codepoints.first().cloned(),
    };

    // Find the position of the current codepoint
    if let Some(current_index) = codepoints.iter().position(|cp| cp == &current)
    {
        match direction {
            CycleDirection::Next => {
                let next_index = (current_index + 1) % codepoints.len();
                codepoints.get(next_index).cloned()
            }
            CycleDirection::Previous => {
                let prev_index = if current_index == 0 {
                    codepoints.len() - 1
                } else {
                    current_index - 1
                };
                codepoints.get(prev_index).cloned()
            }
        }
    } else {
        // Current codepoint not found, return first
        codepoints.first().cloned()
    }
}
