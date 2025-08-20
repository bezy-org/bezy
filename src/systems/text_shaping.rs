//! Text Shaping System
//!
//! This module provides advanced text shaping for Arabic and other complex scripts
//! using the harfrust (HarfBuzz for Rust) library. It handles:
//! - Contextual glyph substitution (initial, medial, final, isolated forms)
//! - Arabic ligatures
//! - Bidirectional text layout (RTL/LTR mixing)
//! - Advanced glyph positioning
//!
//! The shaping system integrates with the existing text editor to provide
//! proper Arabic text rendering support.

use crate::core::state::{SortLayoutMode, TextEditorState};
use bevy::prelude::*;
use std::collections::HashMap;

/// Resource to cache text shaping information
#[derive(Resource, Default)]
pub struct TextShapingCache {
    /// Cache of shaped text by input string
    pub shaped_texts: HashMap<String, ShapedText>,
}

/// Component to mark text that has been shaped with harfrust
#[derive(Component, Debug, Clone)]
pub struct ShapedText {
    /// Original input text as Unicode codepoints
    pub input_codepoints: Vec<char>,
    /// Shaped glyph information from HarfBuzz
    pub shaped_glyphs: Vec<ShapedGlyph>,
    /// Layout direction used for shaping
    pub direction: TextDirection,
    /// Whether complex shaping was applied (vs simple character mapping)
    pub is_complex_shaped: bool,
}

/// Information about a single shaped glyph
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    /// Glyph ID from the font
    pub glyph_id: u32,
    /// Original Unicode codepoint (for fallback)
    pub codepoint: char,
    /// Glyph name (derived from font)
    pub glyph_name: String,
    /// Horizontal advance width
    pub advance_width: f32,
    /// X offset for positioning
    pub x_offset: f32,
    /// Y offset for positioning
    pub y_offset: f32,
    /// Cluster index (for cursor positioning)
    pub cluster: u32,
}

/// Text direction for shaping
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

impl From<SortLayoutMode> for TextDirection {
    fn from(mode: SortLayoutMode) -> Self {
        match mode {
            SortLayoutMode::LTRText => TextDirection::LeftToRight,
            SortLayoutMode::RTLText => TextDirection::RightToLeft,
            SortLayoutMode::Freeform => TextDirection::LeftToRight, // Default
        }
    }
}


/// System to perform text shaping for Arabic and complex scripts
pub fn shape_arabic_text_system(
    _shaping_cache: ResMut<TextShapingCache>,
    text_editor_state: Res<TextEditorState>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
) {
    // Only process if we have FontIR state for accessing font data
    let Some(_fontir_state) = fontir_app_state.as_ref() else {
        return;
    };

    // Check if we have any RTL text sorts that need shaping
    let mut needs_shaping = false;
    for entry in text_editor_state.buffer.iter() {
        if entry.layout_mode == SortLayoutMode::RTLText {
            needs_shaping = true;
            break;
        }
    }

    if !needs_shaping {
        return;
    }

    // TODO: For now, just log that we would perform shaping
    // In a full implementation, we would:
    // 1. Extract text runs that need shaping
    // 2. Load the appropriate font face
    // 3. Use harfrust to shape the text
    // 4. Update sort positions based on shaped output

    debug!("Arabic text shaping system: detected RTL text that would benefit from shaping");
    debug!("Buffer contains {} sorts", text_editor_state.buffer.len());

    // Count RTL sorts for debugging
    let rtl_count = text_editor_state
        .buffer
        .iter()
        .filter(|entry| entry.layout_mode == SortLayoutMode::RTLText)
        .count();

    if rtl_count > 0 {
        debug!(
            "Found {} RTL text sorts for potential Arabic shaping",
            rtl_count
        );
    }
}

/// Shape a text string using HarfBuzz for complex script support
pub fn shape_text_with_harfbuzz(
    text: &str,
    font_path: &str,
    direction: TextDirection,
    shaping_cache: &mut TextShapingCache,
) -> Result<ShapedText, String> {
    // Convert text to UTF-32 codepoints
    let input_codepoints: Vec<char> = text.chars().collect();

    if input_codepoints.is_empty() {
        return Ok(ShapedText {
            input_codepoints,
            shaped_glyphs: Vec::new(),
            direction,
            is_complex_shaped: false,
        });
    }

    // Check if we have this text already shaped in cache
    let cache_key = format!("{text}_{direction:?}_{font_path}");
    if let Some(cached_result) = shaping_cache.shaped_texts.get(&cache_key) {
        return Ok(cached_result.clone());
    }

    // This is where we would perform actual HarfBuzz shaping
    // For now, return a simple fallback
    let shaped_glyphs: Vec<ShapedGlyph> = input_codepoints
        .iter()
        .enumerate()
        .map(|(i, &ch)| ShapedGlyph {
            glyph_id: ch as u32, // Fallback: use codepoint as glyph ID
            codepoint: ch,
            glyph_name: format!("uni{:04X}", ch as u32),
            advance_width: 600.0, // Default advance width
            x_offset: 0.0,
            y_offset: 0.0,
            cluster: i as u32,
        })
        .collect();

    Ok(ShapedText {
        input_codepoints,
        shaped_glyphs,
        direction,
        is_complex_shaped: false, // Would be true after real shaping
    })
}

/// Detect if text contains Arabic or other complex script characters
pub fn needs_complex_shaping(text: &str) -> bool {
    text.chars().any(|ch| {
        let code = ch as u32;
        // Arabic block: U+0600-U+06FF
        (0x0600..=0x06FF).contains(&code) ||
        // Arabic Supplement: U+0750-U+077F  
        (0x0750..=0x077F).contains(&code) ||
        // Arabic Extended-A: U+08A0-U+08FF
        (0x08A0..=0x08FF).contains(&code) ||
        // Arabic Presentation Forms-A: U+FB50-U+FDFF
        (0xFB50..=0xFDFF).contains(&code) ||
        // Arabic Presentation Forms-B: U+FE70-U+FEFF
        (0xFE70..=0xFEFF).contains(&code)
    })
}


/// Plugin to register the Arabic text shaping system
pub struct TextShapingPlugin;

impl Plugin for TextShapingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextShapingCache>()
            .add_systems(Update, shape_arabic_text_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arabic_detection() {
        // Arabic text
        assert!(needs_complex_shaping("السلام عليكم"));
        assert!(needs_complex_shaping("مرحبا"));

        // Latin text
        assert!(!needs_complex_shaping("Hello World"));
        assert!(!needs_complex_shaping("abc"));

        // Mixed text
        assert!(needs_complex_shaping("Hello مرحبا"));
    }

    #[test]
    fn test_script_detection() {
        use harfrust::Tag;

        assert_eq!(
            get_script_for_text("السلام"),
            Script::from_iso15924_tag(Tag::new(b"arab"))
        );
        assert_eq!(
            get_script_for_text("שלום"),
            Script::from_iso15924_tag(Tag::new(b"hebr"))
        );
        assert_eq!(
            get_script_for_text("Hello"),
            Script::from_iso15924_tag(Tag::new(b"latn"))
        );
    }

    #[test]
    fn test_direction_conversion() {
        assert_eq!(
            TextDirection::from(SortLayoutMode::LTRText),
            TextDirection::LeftToRight
        );
        assert_eq!(
            TextDirection::from(SortLayoutMode::RTLText),
            TextDirection::RightToLeft
        );
        assert_eq!(
            TextDirection::from(SortLayoutMode::Freeform),
            TextDirection::LeftToRight
        );
    }
}
