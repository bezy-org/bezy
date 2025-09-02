//! Arabic text shaping implementation using HarfBuzz
//!
//! This module provides the core implementation for Arabic text shaping,
//! handling contextual forms (isolated, initial, medial, final) and ligatures.

use crate::core::state::fontir_app_state::FontIRAppState;
use crate::core::state::text_editor::buffer::{SortEntry, SortKind};
use crate::core::state::{SortLayoutMode, TextEditorState};
use crate::systems::text_shaping::{ShapedGlyph, ShapedText, TextDirection};
use bevy::prelude::*;
use std::collections::HashMap;

/// Resource to cache shaped text results
#[derive(Resource, Default)]
pub struct ArabicShapingCache {
    /// Cache of shaped text by input string
    pub shaped_texts: HashMap<String, ShapedText>,
}

/// Shape Arabic text using HarfBuzz
pub fn shape_arabic_text(
    text: &str,
    direction: TextDirection,
    fontir_state: &FontIRAppState,
) -> Result<ShapedText, String> {
    // For MVP, we'll use a simpler approach that doesn't require font bytes
    // We'll shape the text and map to contextual forms based on position

    let input_codepoints: Vec<char> = text.chars().collect();
    let mut shaped_glyphs = Vec::new();

    // Analyze each character's position for contextual forms
    for (i, &ch) in input_codepoints.iter().enumerate() {
        if is_arabic_letter(ch) {
            let position = get_arabic_position(&input_codepoints, i);
            let glyph_name = get_contextual_glyph_name(ch, position, fontir_state)?;

            // Get advance width from FontIR
            let advance_width = fontir_state.get_glyph_advance_width(&glyph_name);

            shaped_glyphs.push(ShapedGlyph {
                glyph_id: 0, // We don't need actual glyph ID for MVP
                codepoint: ch,
                glyph_name,
                advance_width,
                x_offset: 0.0,
                y_offset: 0.0,
                cluster: i as u32,
            });
        } else {
            // Non-Arabic characters pass through unchanged
            let glyph_name = if let Some(name) = unicode_to_glyph_name(ch, fontir_state) {
                name
            } else {
                format!("uni{:04X}", ch as u32)
            };

            let advance_width = fontir_state.get_glyph_advance_width(&glyph_name);

            shaped_glyphs.push(ShapedGlyph {
                glyph_id: 0,
                codepoint: ch,
                glyph_name,
                advance_width,
                x_offset: 0.0,
                y_offset: 0.0,
                cluster: i as u32,
            });
        }
    }

    Ok(ShapedText {
        input_codepoints,
        shaped_glyphs,
        direction,
        is_complex_shaped: true,
    })
}

/// Position of an Arabic letter in a word
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArabicPosition {
    Isolated,
    Initial,
    Medial,
    Final,
}

/// Check if a character is an Arabic letter that needs shaping
fn is_arabic_letter(ch: char) -> bool {
    let code = ch as u32;
    // Basic Arabic letters that connect
    (0x0621..=0x064A).contains(&code)
}

/// Check if an Arabic letter can connect to the next letter
fn can_connect_to_next(ch: char) -> bool {
    // Non-connecting Arabic letters
    match ch {
        '\u{0621}' | // Hamza
        '\u{0622}' | '\u{0623}' | '\u{0624}' | '\u{0625}' | // Alef variants
        '\u{0627}' | // Alef
        '\u{0629}' | // Teh Marbuta
        '\u{062F}' | // Dal
        '\u{0630}' | // Thal
        '\u{0631}' | // Reh
        '\u{0632}' | // Zain
        '\u{0648}' | // Waw
        '\u{0649}' => false, // Alef Maksura
        _ => is_arabic_letter(ch),
    }
}

/// Check if an Arabic letter can connect to the previous letter
fn can_connect_to_prev(ch: char) -> bool {
    is_arabic_letter(ch)
}

/// Determine the position of an Arabic letter in a word
pub fn get_arabic_position(text: &[char], index: usize) -> ArabicPosition {
    let ch = text[index];

    // Check previous character
    let has_prev = index > 0 && {
        let prev = text[index - 1];
        is_arabic_letter(prev) && can_connect_to_next(prev)
    };

    // Check next character
    let has_next = index + 1 < text.len() && {
        let next = text[index + 1];
        is_arabic_letter(next) && can_connect_to_prev(next)
    };

    // Determine position based on connections
    match (has_prev, can_connect_to_next(ch) && has_next) {
        (false, false) => ArabicPosition::Isolated,
        (false, true) => ArabicPosition::Initial,
        (true, true) => ArabicPosition::Medial,
        (true, false) => ArabicPosition::Final,
    }
}

/// Get the contextual glyph name for an Arabic letter
fn get_contextual_glyph_name(
    ch: char,
    position: ArabicPosition,
    fontir_state: &FontIRAppState,
) -> Result<String, String> {
    // First, get the base glyph name
    let base_name = get_arabic_base_name(ch);

    // Try different naming conventions for contextual forms
    // Bezy Grotesk uses: {letter}-ar.{form}
    let suffix = match position {
        ArabicPosition::Isolated => "", // Isolated form has no suffix in this font
        ArabicPosition::Initial => ".init",
        ArabicPosition::Medial => ".medi",
        ArabicPosition::Final => ".fina",
    };

    // For isolated position, just use the base name
    if position == ArabicPosition::Isolated {
        if fontir_state.get_glyph_names().contains(&base_name) {
            return Ok(base_name);
        }
    } else {
        // Try with suffix for other positions
        let contextual_name = format!("{base_name}{suffix}");
        if fontir_state.get_glyph_names().contains(&contextual_name) {
            return Ok(contextual_name);
        }
    }

    // Fallback to base name without suffix
    if fontir_state.get_glyph_names().contains(&base_name) {
        return Ok(base_name);
    }

    // Last resort: try the base name or uni code
    if fontir_state.get_glyph_names().contains(&base_name) {
        Ok(base_name)
    } else {
        // Use Unicode naming as ultimate fallback
        Ok(format!("uni{:04X}", ch as u32))
    }
}

/// Get the base glyph name for an Arabic character
fn get_arabic_base_name(ch: char) -> String {
    // Using the naming convention from bezy-grotesk font: {letter}-ar
    match ch as u32 {
        0x0621 => "hamza-ar".to_string(),
        0x0622 => "alefMadda-ar".to_string(),
        0x0623 => "alefHamzaabove-ar".to_string(),
        0x0624 => "wawHamza-ar".to_string(),
        0x0625 => "alefHamzabelow-ar".to_string(),
        0x0626 => "yehHamza-ar".to_string(),
        0x0627 => "alef-ar".to_string(),
        0x0628 => "beh-ar".to_string(),
        0x0629 => "tehMarbuta-ar".to_string(),
        0x062A => "teh-ar".to_string(),
        0x062B => "theh-ar".to_string(),
        0x062C => "jeem-ar".to_string(),
        0x062D => "hah-ar".to_string(),
        0x062E => "khah-ar".to_string(),
        0x062F => "dal-ar".to_string(),
        0x0630 => "thal-ar".to_string(),
        0x0631 => "reh-ar".to_string(),
        0x0632 => "zain-ar".to_string(),
        0x0633 => "seen-ar".to_string(),
        0x0634 => "sheen-ar".to_string(),
        0x0635 => "sad-ar".to_string(),
        0x0636 => "dad-ar".to_string(),
        0x0637 => "tah-ar".to_string(),
        0x0638 => "zah-ar".to_string(),
        0x0639 => "ain-ar".to_string(),
        0x063A => "ghain-ar".to_string(),
        0x0641 => "feh-ar".to_string(),
        0x0642 => "qaf-ar".to_string(),
        0x0643 => "kaf-ar".to_string(),
        0x0644 => "lam-ar".to_string(),
        0x0645 => "meem-ar".to_string(),
        0x0646 => "noon-ar".to_string(),
        0x0647 => "heh-ar".to_string(),
        0x0648 => "waw-ar".to_string(),
        0x0649 => "alefMaksura-ar".to_string(),
        0x064A => "yeh-ar".to_string(),
        _ => format!("uni{:04X}", ch as u32),
    }
}

/// Helper function to map Unicode to glyph name
fn unicode_to_glyph_name(ch: char, fontir_state: &FontIRAppState) -> Option<String> {
    use crate::systems::text_editor_sorts::input_utilities::unicode_to_glyph_name_fontir;
    unicode_to_glyph_name_fontir(ch, fontir_state)
}

/// System to shape Arabic text in the text buffer
pub fn shape_arabic_buffer_system(
    mut text_editor_state: ResMut<TextEditorState>,
    fontir_state: Option<Res<FontIRAppState>>,
) {
    let Some(fontir_state) = fontir_state else {
        return;
    };

    // TEMPORARY: Force shaping to always run for debugging
    // if !text_editor_state.is_changed() {
    //     return;
    // }

    // Check if we have any Arabic text that needs shaping
    let mut needs_shaping = false;
    let mut arabic_chars = Vec::new();
    for entry in text_editor_state.buffer.iter() {
        if let SortKind::Glyph {
            codepoint: Some(ch),
            ..
        } = &entry.kind
        {
            if is_arabic_letter(*ch) {
                needs_shaping = true;
                arabic_chars.push(*ch);
            }
        }
    }

    if !needs_shaping {
        return;
    }

    info!(
        "🔤 Arabic shaping: Found {} Arabic characters, reshaping buffer",
        arabic_chars.len()
    );

    // Collect text runs that need shaping
    let mut text_runs = Vec::new();
    let mut current_run = String::new();
    let mut run_start = 0;
    let mut run_indices = Vec::new();

    for (i, entry) in text_editor_state.buffer.iter().enumerate() {
        match &entry.kind {
            SortKind::Glyph {
                codepoint: Some(ch),
                ..
            } => {
                if current_run.is_empty() {
                    run_start = i;
                }
                current_run.push(*ch);
                run_indices.push(i);
            }
            SortKind::LineBreak => {
                if !current_run.is_empty() {
                    text_runs.push((run_start, current_run.clone(), run_indices.clone()));
                    current_run.clear();
                    run_indices.clear();
                }
            }
            _ => {}
        }
    }

    // Don't forget the last run
    if !current_run.is_empty() {
        text_runs.push((run_start, current_run, run_indices));
    }

    // Shape each text run and update the buffer
    for (_start_idx, text, indices) in text_runs {
        // Check if this run contains Arabic
        if !text.chars().any(is_arabic_letter) {
            continue;
        }

        // Determine direction (simplified for MVP)
        let direction = if text.chars().any(is_arabic_letter) {
            TextDirection::RightToLeft
        } else {
            TextDirection::LeftToRight
        };

        // Shape the text
        if let Ok(shaped) = shape_arabic_text(&text, direction, &fontir_state) {
            info!(
                "🔤 Arabic shaping: Shaped text '{}' into {} glyphs",
                text,
                shaped.shaped_glyphs.len()
            );
            // Update buffer entries with shaped glyph names
            for (buffer_idx, shaped_glyph) in indices.iter().zip(shaped.shaped_glyphs.iter()) {
                if let Some(entry) = text_editor_state.buffer.get_mut(*buffer_idx) {
                    if let SortKind::Glyph {
                        glyph_name,
                        advance_width,
                        ..
                    } = &mut entry.kind
                    {
                        let old_name = glyph_name.clone();
                        *glyph_name = shaped_glyph.glyph_name.clone();
                        *advance_width = shaped_glyph.advance_width;
                        info!(
                            "🔤 Arabic shaping: Updated '{}' (U+{:04X}) from '{}' to '{}'",
                            shaped_glyph.codepoint,
                            shaped_glyph.codepoint as u32,
                            old_name,
                            shaped_glyph.glyph_name
                        );
                    }
                }
            }
        } else {
            warn!("🔤 Arabic shaping: Failed to shape text '{}'", text);
        }
    }
}

/// Plugin to register Arabic shaping systems
pub struct ArabicShapingPlugin;

impl Plugin for ArabicShapingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArabicShapingCache>().add_systems(
            Update,
            shape_arabic_buffer_system.in_set(crate::editing::FontEditorSets::TextBuffer),
        );
    }
}
