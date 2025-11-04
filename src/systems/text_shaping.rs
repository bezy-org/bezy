//! Unified Text Shaping System
//!
//! This module provides comprehensive text shaping for Arabic and other complex scripts
//! using multiple approaches:
//! - Simple Arabic contextual form mapping for basic support
//! - HarfBuzz integration for advanced shaping (experimental)
//! - Fallback character mapping for Latin and other scripts
//!
//! The shaping system integrates with the existing text editor to provide
//! proper Arabic text rendering support while maintaining compatibility
//! with Latin text editing.


use crate::core::state::text_editor::buffer::SortKind;
use crate::core::state::{SortLayoutMode, TextEditorState};
use bevy::prelude::*;
use std::collections::HashMap;

// HarfBuzz imports (conditional compilation could be added later)
use harfrust::{
    Direction, FontRef, Language, Script, ShaperData, ShaperInstance, UnicodeBuffer,
};
use std::path::PathBuf;
use std::str::FromStr;
use tempfile::TempDir;

/// Resource to cache text shaping information
#[derive(Resource, Default)]
pub struct TextShapingCache {
    /// Cache of shaped text by input string
    pub shaped_texts: HashMap<String, ShapedText>,
    /// HarfBuzz-specific cache data
    pub harfbuzz_cache: HarfBuzzShapingCache,
    /// Arabic shaping cache
    pub arabic_cache: HashMap<String, ShapedText>,
}

/// HarfBuzz-specific cache resource
#[derive(Default)]
pub struct HarfBuzzShapingCache {
    /// Temporary directory for compiled fonts
    _temp_dir: Option<TempDir>,
    /// Path to last compiled font binary
    _compiled_font_path: Option<PathBuf>,
    /// Last compilation timestamp for cache invalidation
    _last_compiled: Option<std::time::Instant>,
    /// Whether HarfBuzz is available for use (future feature)
    #[allow(dead_code)]
    harfbuzz_available: bool,
    /// Cache of shaped text results
    shaped_cache: HashMap<String, ShapedText>,
}

/// Component to mark text that has been shaped
#[derive(Component, Debug, Clone)]
pub struct ShapedText {
    /// Original input text as Unicode codepoints
    pub input_codepoints: Vec<char>,
    /// Shaped glyph information from shaping engine
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

/// Position of an Arabic letter in a word (for contextual forms)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArabicPosition {
    Isolated,
    Initial,
    Medial,
    Final,
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

impl From<TextDirection> for Direction {
    fn from(direction: TextDirection) -> Self {
        match direction {
            TextDirection::LeftToRight => Direction::LeftToRight,
            TextDirection::RightToLeft => Direction::RightToLeft,
            TextDirection::TopToBottom => Direction::TopToBottom,
            TextDirection::BottomToTop => Direction::BottomToTop,
        }
    }
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

// ===== ARABIC CONTEXTUAL SHAPING =====

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
) -> Result<String, String> {
    // TEMPORARILY DISABLED: FontIR removed
    // Use Unicode naming as fallback
    let _ = position;
    Ok(format!("uni{:04X}", ch as u32))
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

/// Shape Arabic text using contextual form mapping
/// TEMPORARILY DISABLED: FontIR removed
pub fn shape_arabic_text(
    text: &str,
    direction: TextDirection,
) -> Result<ShapedText, String> {
    let input_codepoints: Vec<char> = text.chars().collect();
    let shaped_glyphs: Vec<ShapedGlyph> = input_codepoints.iter().enumerate().map(|(i, &ch)| {
        let glyph_name = format!("uni{:04X}", ch as u32);
        ShapedGlyph {
            glyph_id: 0,
            codepoint: ch,
            glyph_name,
            advance_width: 500.0, // Default width
            x_offset: 0.0,
            y_offset: 0.0,
            cluster: i as u32,
        }
    }).collect();

    Ok(ShapedText {
        input_codepoints,
        shaped_glyphs,
        direction,
        is_complex_shaped: false,
    })
}

/// Helper function to map Unicode to glyph name
fn unicode_to_glyph_name(ch: char) -> Option<String> {
    use crate::systems::sorts::input_utilities::unicode_to_glyph_name_fontir;
    unicode_to_glyph_name_fontir(ch)
}

// ===== HARFBUZZ INTEGRATION =====

/// Get font bytes for HarfBuzz shaping (using existing TTF file for now)
pub fn compile_font_for_shaping(
    _cache: &mut HarfBuzzShapingCache,
) -> Result<Vec<u8>, String> {
    // HACK: For proof of concept, use the existing TTF file directly
    // TODO: This should compile from FontIR using fontc, but fontc has issues with Arabic composite glyphs

    debug!("🔤 HarfBuzz: Loading existing BezyGrotesk-Regular.ttf for shaping");

    let font_bytes = if std::path::Path::new("assets").exists() {
        // Use file system if assets exist
        std::fs::read("assets/fonts/BezyGrotesk-Regular.ttf")
            .map_err(|e| format!("Failed to load BezyGrotesk-Regular.ttf: {e}"))?
    } else {
        // Use embedded font data
        debug!("🔤 Using embedded font for text shaping");
        crate::utils::embedded_assets::BEZY_GROTESK_BYTES.to_vec()
    };

    debug!(
        "🔤 HarfBuzz: Loaded {} bytes from TTF file",
        font_bytes.len()
    );
    Ok(font_bytes)
}

/// Shape text using HarfBuzz with compiled font
/// TEMPORARILY DISABLED: FontIR removed
#[allow(dead_code)]
pub fn shape_text_with_harfbuzz(
    text: &str,
    direction: TextDirection,
    _cache: &mut HarfBuzzShapingCache,
) -> Result<ShapedText, String> {
    // Fallback to simple mapping
    let input_codepoints: Vec<char> = text.chars().collect();
    let shaped_glyphs: Vec<ShapedGlyph> = input_codepoints.iter().enumerate().map(|(i, &ch)| {
        ShapedGlyph {
            glyph_id: 0,
            codepoint: ch,
            glyph_name: format!("uni{:04X}", ch as u32),
            advance_width: 500.0,
            x_offset: 0.0,
            y_offset: 0.0,
            cluster: i as u32,
        }
    }).collect();

    Ok(ShapedText {
        input_codepoints,
        shaped_glyphs,
        direction,
        is_complex_shaped: false,
    })
}

/// Perform actual HarfBuzz text shaping using harfrust
fn perform_harfbuzz_shaping(
    text: &str,
    direction: TextDirection,
    font_bytes: &[u8],
) -> Result<ShapedText, String> {
    // Create harfrust font from compiled font bytes
    let font_ref = FontRef::from_index(font_bytes, 0)
        .map_err(|e| format!("Failed to create harfrust FontRef: {e:?}"))?;

    // Create shaper data and instance
    let shaper_data = ShaperData::new(&font_ref);
    let shaper_instance = ShaperInstance::from_variations(&font_ref, &[] as &[harfrust::Variation]);
    let shaper = shaper_data
        .shaper(&font_ref)
        .instance(Some(&shaper_instance))
        .build();

    // Create buffer and add text
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);

    // Set buffer properties
    buffer.set_direction(direction.into());
    buffer.set_script(get_script_for_text(text));
    buffer.set_language(Language::from_str("ar").unwrap_or(Language::from_str("en").unwrap()));

    // Guess remaining properties automatically
    buffer.guess_segment_properties();

    // Perform HarfBuzz shaping
    let glyph_buffer = shaper.shape(buffer, &[]);

    // Extract shaped glyph information
    let input_codepoints: Vec<char> = text.chars().collect();
    let mut shaped_glyphs = Vec::new();

    let glyph_infos = glyph_buffer.glyph_infos();
    let glyph_positions = glyph_buffer.glyph_positions();

    // Debug output to understand what HarfBuzz returns
    debug!(
        "🔤 HarfBuzz: Shaped {} characters into {} glyphs",
        input_codepoints.len(),
        glyph_infos.len()
    );

    for (i, glyph_info) in glyph_infos.iter().enumerate() {
        debug!(
            "🔤 HarfBuzz: Glyph[{}] - ID: {}, cluster: {}",
            i, glyph_info.glyph_id, glyph_info.cluster
        );
        // Get glyph name from glyph ID
        let glyph_name = get_glyph_name_from_id(glyph_info.glyph_id);

        // Get original codepoint from cluster index
        let codepoint = if (glyph_info.cluster as usize) < input_codepoints.len() {
            input_codepoints[glyph_info.cluster as usize]
        } else {
            '\u{FFFD}' // Replacement character
        };

        // Get glyph position info
        let pos = glyph_positions.get(i).cloned().unwrap_or_default();

        // harfrust uses units per em directly (no scaling needed)
        let advance_width = pos.x_advance as f32;

        shaped_glyphs.push(ShapedGlyph {
            glyph_id: glyph_info.glyph_id,
            codepoint,
            glyph_name,
            advance_width,
            x_offset: pos.x_offset as f32,
            y_offset: pos.y_offset as f32,
            cluster: glyph_info.cluster,
        });
    }

    debug!(
        "HarfBuzz shaped '{}' into {} glyphs",
        text,
        shaped_glyphs.len()
    );

    Ok(ShapedText {
        input_codepoints,
        shaped_glyphs,
        direction,
        is_complex_shaped: true,
    })
}

/// Get glyph name from glyph ID using FontIR
fn get_glyph_name_from_id(glyph_id: u32) -> String {
    // HACK: For proof of concept with "اشهد", let's create a manual mapping
    // based on what we see in the debug output
    // TODO: This needs proper font table parsing to get actual glyph names

    debug!("🔤 HarfBuzz: Mapping glyph ID {} to name", glyph_id);

    // HACK: Manual mapping based on actual HarfBuzz test output for "اشهد"
    // From test_harfbuzz_arabic.rs output:
    // Glyph[0]: ID 93 = dal-ar.fina (rightmost in RTL)
    // Glyph[1]: ID 170 = heh-ar.medi
    // Glyph[2]: ID 107 = sheen-ar.init
    // Glyph[3]: ID 54 = alef-ar (leftmost in RTL)

    match glyph_id {
        // Confirmed glyph IDs from HarfBuzz shaping test for "اشهد"
        54 => "alef-ar".to_string(),        // Alef isolated
        93 => "dal-ar.fina".to_string(),    // Dal final form
        107 => "sheen-ar.init".to_string(), // Sheen initial form
        170 => "heh-ar.medi".to_string(),   // Heh medial form

        _ => {
            warn!(
                "🔤 HarfBuzz: Unknown glyph ID {}, returning gid{}",
                glyph_id, glyph_id
            );
            format!("gid{glyph_id}")
        }
    }
}

/// Get appropriate script for HarfBuzz based on text content
fn get_script_for_text(text: &str) -> Script {
    use harfrust::script;

    if text.chars().any(|ch| {
        let code = ch as u32;
        (0x0600..=0x06FF).contains(&code) // Arabic block
    }) {
        script::ARABIC
    } else {
        script::LATIN
    }
}

// ===== SYSTEM IMPLEMENTATIONS =====

/// System to perform basic text shaping for Arabic and complex scripts
/// TEMPORARILY DISABLED: FontIR removed
#[allow(dead_code)]
pub fn shape_arabic_text_system(
    _shaping_cache: ResMut<TextShapingCache>,
    _text_editor_state: Res<TextEditorState>,
) {
    // FontIR removal: Shaping system temporarily disabled
}

/// System to shape Arabic text in the text buffer using contextual forms
/// Only processes Arabic text - exits early for non-Arabic text
/// TEMPORARILY DISABLED: FontIR removed
#[allow(dead_code)]
pub fn shape_arabic_buffer_system(
    mut _text_editor_state: ResMut<TextEditorState>,
) {
    // FontIR removal: Shaping system temporarily disabled
}


/// System for HarfBuzz text shaping with font compilation
/// System for HarfBuzz text shaping with font compilation
/// TEMPORARILY DISABLED: FontIR removed
#[allow(dead_code)]
pub fn harfbuzz_shaping_system(
    mut _text_editor_state: ResMut<TextEditorState>,
    mut _shaping_cache: ResMut<TextShapingCache>,
) {
    // FontIR removal: Shaping system temporarily disabled
}
/// Unified plugin to register all text shaping systems
pub struct TextShapingPlugin;

impl Plugin for TextShapingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextShapingCache>().add_systems(
            Update,
            (
                shape_arabic_text_system,
                shape_arabic_buffer_system,
                harfbuzz_shaping_system,
            )
                .in_set(crate::editing::FontEditorSets::TextBuffer),
        );
    }
}
