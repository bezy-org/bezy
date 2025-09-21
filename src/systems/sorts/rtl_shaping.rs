//! RTL Text Shaping with HarfRust
//!
//! This module handles Right-to-Left text shaping for Arabic text input
//! using HarfRust to properly shape and position Arabic glyphs.

use bevy::prelude::*;
use harfrust::{Direction, GlyphBuffer, Language, Script, Shaper, ShaperBuilder, UnicodeBuffer};
use std::fs;

/// Resource to hold the shaped text results
#[derive(Resource, Default)]
pub struct ShapedTextCache {
    /// Cached shaped text results keyed by input text
    pub cache: std::collections::HashMap<String, Vec<ShapedGlyph>>,
}

/// Represents a shaped glyph with position information
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    /// The glyph name (e.g., "alef-ar", "beh-ar.init")
    pub glyph_name: String,
    /// The glyph index in the font
    pub glyph_id: u32,
    /// X offset from the previous glyph
    pub x_advance: f32,
    /// Y offset from the baseline
    pub y_advance: f32,
    /// X offset for positioning
    pub x_offset: f32,
    /// Y offset for positioning
    pub y_offset: f32,
}

/// Shape Arabic text using HarfRust (simplified for now)
/// Note: Full HarfRust integration requires proper font loading
/// which we'll implement later with actual font data
pub fn shape_arabic_text(text: &str) -> Result<Vec<ShapedGlyph>, String> {
    // For now, we'll do simple character-by-character mapping
    // Full HarfRust shaping will be implemented once we have proper font loading

    let mut shaped_glyphs = Vec::new();
    let chars: Vec<char> = text.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        let codepoint = *ch as u32;

        // Determine contextual form based on position
        let is_first = i == 0 || chars[i - 1].is_whitespace();
        let is_last = i == chars.len() - 1 || (i < chars.len() - 1 && chars[i + 1].is_whitespace());

        let form = if !is_first && !is_last {
            "medi"
        } else if !is_first && is_last {
            "fina"
        } else if is_first && !is_last {
            "init"
        } else {
            "isol"
        };

        let glyph_name = map_codepoint_to_glyph_name(codepoint, form);

        shaped_glyphs.push(ShapedGlyph {
            glyph_name,
            glyph_id: codepoint,
            x_advance: 600.0, // Default advance width
            y_advance: 0.0,
            x_offset: 0.0,
            y_offset: 0.0,
        });
    }

    Ok(shaped_glyphs)
}

/// Map Unicode codepoint to UFO glyph name
pub fn map_codepoint_to_glyph_name(codepoint: u32, contextual_form: &str) -> String {
    // Map common Arabic characters to their UFO glyph names
    // This includes contextual forms (isolated, initial, medial, final)
    match codepoint {
        // Arabic Letter Alef (U+0627)
        0x0627 => match contextual_form {
            "fina" => "alef-ar.fina",
            _ => "alef-ar",
        },
        // Arabic Letter Beh (U+0628)
        0x0628 => match contextual_form {
            "init" => "beh-ar.init",
            "medi" => "beh-ar.medi",
            "fina" => "beh-ar.fina",
            _ => "beh-ar",
        },
        // Arabic Letter Teh (U+062A)
        0x062A => match contextual_form {
            "init" => "teh-ar.init",
            "medi" => "teh-ar.medi",
            "fina" => "teh-ar.fina",
            _ => "teh-ar",
        },
        // Arabic Letter Theh (U+062B)
        0x062B => match contextual_form {
            "init" => "theh-ar.init",
            "medi" => "theh-ar.medi",
            "fina" => "theh-ar.fina",
            _ => "theh-ar",
        },
        // Arabic Letter Jeem (U+062C)
        0x062C => match contextual_form {
            "init" => "jeem-ar.init",
            "medi" => "jeem-ar.medi",
            "fina" => "jeem-ar.fina",
            _ => "jeem-ar",
        },
        // Arabic Letter Hah (U+062D)
        0x062D => match contextual_form {
            "init" => "hah-ar.init",
            "medi" => "hah-ar.medi",
            "fina" => "hah-ar.fina",
            _ => "hah-ar",
        },
        // Arabic Letter Khah (U+062E)
        0x062E => match contextual_form {
            "init" => "khah-ar.init",
            "medi" => "khah-ar.medi",
            "fina" => "khah-ar.fina",
            _ => "khah-ar",
        },
        // Arabic Letter Dal (U+062F)
        0x062F => match contextual_form {
            "fina" => "dal-ar.fina",
            _ => "dal-ar",
        },
        // Arabic Letter Thal (U+0630)
        0x0630 => match contextual_form {
            "fina" => "thal-ar.fina",
            _ => "thal-ar",
        },
        // Arabic Letter Reh (U+0631)
        0x0631 => match contextual_form {
            "fina" => "reh-ar.fina",
            _ => "reh-ar",
        },
        // Arabic Letter Zain (U+0632)
        0x0632 => match contextual_form {
            "fina" => "zain-ar.fina",
            _ => "zain-ar",
        },
        // Arabic Letter Seen (U+0633)
        0x0633 => match contextual_form {
            "init" => "seen-ar.init",
            "medi" => "seen-ar.medi",
            "fina" => "seen-ar.fina",
            _ => "seen-ar",
        },
        // Arabic Letter Sheen (U+0634)
        0x0634 => match contextual_form {
            "init" => "sheen-ar.init",
            "medi" => "sheen-ar.medi",
            "fina" => "sheen-ar.fina",
            _ => "sheen-ar",
        },
        // Arabic Letter Sad (U+0635)
        0x0635 => match contextual_form {
            "init" => "sad-ar.init",
            "medi" => "sad-ar.medi",
            "fina" => "sad-ar.fina",
            _ => "sad-ar",
        },
        // Arabic Letter Dad (U+0636)
        0x0636 => match contextual_form {
            "init" => "dad-ar.init",
            "medi" => "dad-ar.medi",
            "fina" => "dad-ar.fina",
            _ => "dad-ar",
        },
        // Arabic Letter Tah (U+0637)
        0x0637 => match contextual_form {
            "init" => "tah-ar.init",
            "medi" => "tah-ar.medi",
            "fina" => "tah-ar.fina",
            _ => "tah-ar",
        },
        // Arabic Letter Zah (U+0638)
        0x0638 => match contextual_form {
            "init" => "zah-ar.init",
            "medi" => "zah-ar.medi",
            "fina" => "zah-ar.fina",
            _ => "zah-ar",
        },
        // Arabic Letter Ain (U+0639)
        0x0639 => match contextual_form {
            "init" => "ain-ar.init",
            "medi" => "ain-ar.medi",
            "fina" => "ain-ar.fina",
            _ => "ain-ar",
        },
        // Arabic Letter Ghain (U+063A)
        0x063A => match contextual_form {
            "init" => "ghain-ar.init",
            "medi" => "ghain-ar.medi",
            "fina" => "ghain-ar.fina",
            _ => "ghain-ar",
        },
        // Arabic Letter Feh (U+0641)
        0x0641 => match contextual_form {
            "init" => "feh-ar.init",
            "medi" => "feh-ar.medi",
            "fina" => "feh-ar.fina",
            _ => "feh-ar",
        },
        // Arabic Letter Qaf (U+0642)
        0x0642 => match contextual_form {
            "init" => "qaf-ar.init",
            "medi" => "qaf-ar.medi",
            "fina" => "qaf-ar.fina",
            _ => "qaf-ar",
        },
        // Arabic Letter Kaf (U+0643)
        0x0643 => match contextual_form {
            "init" => "kaf-ar.init",
            "medi" => "kaf-ar.medi",
            "fina" => "kaf-ar.fina",
            _ => "kaf-ar",
        },
        // Arabic Letter Lam (U+0644)
        0x0644 => match contextual_form {
            "init" => "lam-ar.init",
            "medi" => "lam-ar.medi",
            "fina" => "lam-ar.fina",
            _ => "lam-ar",
        },
        // Arabic Letter Meem (U+0645)
        0x0645 => match contextual_form {
            "init" => "meem-ar.init",
            "medi" => "meem-ar.medi",
            "fina" => "meem-ar.fina",
            _ => "meem-ar",
        },
        // Arabic Letter Noon (U+0646)
        0x0646 => match contextual_form {
            "init" => "noon-ar.init",
            "medi" => "noon-ar.medi",
            "fina" => "noon-ar.fina",
            _ => "noon-ar",
        },
        // Arabic Letter Heh (U+0647)
        0x0647 => match contextual_form {
            "init" => "heh-ar.init",
            "medi" => "heh-ar.medi",
            "fina" => "heh-ar.fina",
            _ => "heh-ar",
        },
        // Arabic Letter Waw (U+0648)
        0x0648 => match contextual_form {
            "fina" => "waw-ar.fina",
            _ => "waw-ar",
        },
        // Arabic Letter Yeh (U+064A)
        0x064A => match contextual_form {
            "init" => "yeh-ar.init",
            "medi" => "yeh-ar.medi",
            "fina" => "yeh-ar.fina",
            _ => "yeh-ar",
        },
        // Space
        0x0020 => "space",
        // Default: try to find by Unicode name
        _ => return format!("uni{codepoint:04X}"),
    }
    .to_string()
}

/// System to initialize RTL shaping resources
pub fn initialize_rtl_shaping(mut commands: Commands) {
    debug!("Initializing RTL text shaping resources");
    commands.init_resource::<ShapedTextCache>();
}
