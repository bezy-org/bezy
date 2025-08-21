//! Input utility functions for keyboard and character handling
//!
//! This module provides utilities for:
//! - Tool shortcut key detection
//! - Key code to character conversion with shift state handling
//! - Unicode character to glyph name mapping

use crate::core::state::{AppState, FontIRAppState};
use crate::systems::text_shaping::{
    needs_complex_shaping,
};
use bevy::prelude::*;

/// Check if a key is used as a tool shortcut
#[allow(dead_code)]
pub fn is_tool_shortcut_key(key: KeyCode) -> bool {
    matches!(
        key,
        KeyCode::KeyT |  // Text tool
        KeyCode::KeyP |  // Pen tool  
        KeyCode::KeyV |  // Select tool
        KeyCode::KeyK |  // Knife tool
        KeyCode::KeyH |  // Hyper tool
        KeyCode::KeyR |  // Shapes tool
        KeyCode::KeyM // Measure/Metaballs tool
    )
}

/// Convert key code to character, considering shift state
pub fn key_code_to_char(
    key: KeyCode,
    keyboard_input: &ButtonInput<KeyCode>,
) -> Option<char> {
    let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft)
        || keyboard_input.pressed(KeyCode::ShiftRight);

    match key {
        KeyCode::KeyA => Some(if shift_pressed { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift_pressed { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift_pressed { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift_pressed { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift_pressed { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift_pressed { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift_pressed { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift_pressed { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift_pressed { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift_pressed { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift_pressed { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift_pressed { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift_pressed { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift_pressed { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift_pressed { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift_pressed { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift_pressed { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift_pressed { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift_pressed { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift_pressed { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift_pressed { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift_pressed { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift_pressed { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift_pressed { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift_pressed { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift_pressed { 'Z' } else { 'z' }),
        KeyCode::Digit0 => Some(if shift_pressed { ')' } else { '0' }),
        KeyCode::Digit1 => Some(if shift_pressed { '!' } else { '1' }),
        KeyCode::Digit2 => Some(if shift_pressed { '@' } else { '2' }),
        KeyCode::Digit3 => Some(if shift_pressed { '#' } else { '3' }),
        KeyCode::Digit4 => Some(if shift_pressed { '$' } else { '4' }),
        KeyCode::Digit5 => Some(if shift_pressed { '%' } else { '5' }),
        KeyCode::Digit6 => Some(if shift_pressed { '^' } else { '6' }),
        KeyCode::Digit7 => Some(if shift_pressed { '&' } else { '7' }),
        KeyCode::Digit8 => Some(if shift_pressed { '*' } else { '8' }),
        KeyCode::Digit9 => Some(if shift_pressed { '(' } else { '9' }),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some(if shift_pressed { '_' } else { '-' }),
        KeyCode::Equal => Some(if shift_pressed { '+' } else { '=' }),
        KeyCode::BracketLeft => Some(if shift_pressed { '{' } else { '[' }),
        KeyCode::BracketRight => Some(if shift_pressed { '}' } else { ']' }),
        KeyCode::Backslash => Some(if shift_pressed { '|' } else { '\\' }),
        KeyCode::Semicolon => Some(if shift_pressed { ':' } else { ';' }),
        KeyCode::Quote => Some(if shift_pressed { '"' } else { '\'' }),
        KeyCode::Comma => Some(if shift_pressed { '<' } else { ',' }),
        KeyCode::Period => Some(if shift_pressed { '>' } else { '.' }),
        KeyCode::Slash => Some(if shift_pressed { '?' } else { '/' }),
        KeyCode::Backquote => Some(if shift_pressed { '~' } else { '`' }),
        _ => None,
    }
}

/// Convert Unicode character to glyph name using font data (AppState version)
pub fn unicode_to_glyph_name(
    unicode_char: char,
    app_state: &AppState,
) -> Option<String> {
    // First try to find the glyph by Unicode codepoint
    let _codepoint_hex = format!("{:04X}", unicode_char as u32);

    // Look for glyph with this Unicode codepoint
    for (glyph_name, glyph_data) in &app_state.workspace.font.glyphs {
        if glyph_data.unicode_values.contains(&unicode_char) {
            return Some(glyph_name.clone());
        }
    }

    // If no exact match, try to find a glyph with the same name as the character
    let char_name = unicode_char.to_string();
    if app_state.workspace.font.glyphs.contains_key(&char_name) {
        return Some(char_name);
    }

    // For common characters, try some fallback mappings
    let fallback_mapping = match unicode_char {
        ' ' => "space",
        '0' => "zero",
        '1' => "one",
        '2' => "two",
        '3' => "three",
        '4' => "four",
        '5' => "five",
        '6' => "six",
        '7' => "seven",
        '8' => "eight",
        '9' => "nine",
        _ => return unicode_char_to_standard_glyph_name(unicode_char),
    };

    if app_state
        .workspace
        .font
        .glyphs
        .contains_key(fallback_mapping)
    {
        Some(fallback_mapping.to_string())
    } else {
        // Try Unicode-based naming for Arabic and other scripts
        unicode_char_to_standard_glyph_name(unicode_char)
    }
}

/// Convert Unicode character to glyph name using FontIR font data
pub fn unicode_to_glyph_name_fontir(
    unicode_char: char,
    fontir_state: &FontIRAppState,
) -> Option<String> {
    // Get all available glyph names
    let glyph_names = fontir_state.get_glyph_names();

    // First, try standard Unicode-based naming
    if let Some(standard_name) =
        unicode_char_to_standard_glyph_name(unicode_char)
    {
        if glyph_names.contains(&standard_name) {
            return Some(standard_name);
        }
    }

    // For Arabic characters, try multiple naming conventions
    if is_arabic_character(unicode_char) {
        if let Some(arabic_name) =
            try_arabic_glyph_naming(unicode_char, &glyph_names)
        {
            return Some(arabic_name);
        }
    }

    // Try the character itself as glyph name
    let char_name = unicode_char.to_string();
    if glyph_names.contains(&char_name) {
        return Some(char_name);
    }

    // Log when we can't find an Arabic character
    if is_arabic_character(unicode_char) {
        info!(
            "Arabic character '{}' (U+{:04X}) not found in font",
            unicode_char, unicode_char as u32
        );
        debug!(
            "Available glyph names: {:?}",
            glyph_names
                .iter()
                .filter(|name| name.contains("uni") || name.contains("arab"))
                .collect::<Vec<_>>()
        );
    }

    None
}

/// Check if a character is Arabic
fn is_arabic_character(ch: char) -> bool {
    let code = ch as u32;
    // Arabic block: U+0600-U+06FF
    (0x0600..=0x06FF).contains(&code) ||
    // Arabic Supplement: U+0750-U+077F  
    (0x0750..=0x077F).contains(&code) ||
    // Arabic Extended-A: U+08A0-U+08FF
    (0x08A0..=0x08FF).contains(&code)
}

/// Try different Arabic glyph naming conventions
fn try_arabic_glyph_naming(
    unicode_char: char,
    available_glyphs: &[String],
) -> Option<String> {
    let codepoint = unicode_char as u32;

    // Try various naming conventions for Arabic
    let string_attempts = vec![
        format!("uni{:04X}", codepoint), // uni0627 for Arabic Alef
        format!("u{:04X}", codepoint),   // u0627
        format!("arab{:04X}", codepoint), // arab0627
        format!("arabic{:04X}", codepoint), // arabic0627
        format!("U+{:04X}", codepoint),  // U+0627
    ];

    // Check string-based naming attempts
    for name in string_attempts {
        if available_glyphs.contains(&name) {
            info!(
                "Found Arabic glyph '{}' for character '{}' (U+{:04X})",
                name, unicode_char, codepoint
            );
            return Some(name);
        }
    }

    // Try Arabic character name
    if let Some(arabic_name) = arabic_character_name(unicode_char) {
        if available_glyphs.contains(&arabic_name) {
            info!(
                "Found Arabic glyph '{}' for character '{}' (U+{:04X})",
                arabic_name, unicode_char, codepoint
            );
            return Some(arabic_name);
        }
    }

    None
}

/// Get standard glyph name for common Unicode characters
fn unicode_char_to_standard_glyph_name(unicode_char: char) -> Option<String> {
    match unicode_char {
        ' ' => Some("space".to_string()),
        '0' => Some("zero".to_string()),
        '1' => Some("one".to_string()),
        '2' => Some("two".to_string()),
        '3' => Some("three".to_string()),
        '4' => Some("four".to_string()),
        '5' => Some("five".to_string()),
        '6' => Some("six".to_string()),
        '7' => Some("seven".to_string()),
        '8' => Some("eight".to_string()),
        '9' => Some("nine".to_string()),
        // Standard Unicode naming for non-ASCII characters
        ch if ch as u32 > 127 => Some(format!("uni{:04X}", ch as u32)),
        _ => None,
    }
}

/// Get Arabic character name for common Arabic letters
fn arabic_character_name(unicode_char: char) -> Option<String> {
    // Using the Bezy Grotesk naming convention: {letter}-ar
    match unicode_char as u32 {
        0x0627 => Some("alef-ar".to_string()),
        0x0628 => Some("beh-ar".to_string()),
        0x062A => Some("teh-ar".to_string()),
        0x062B => Some("theh-ar".to_string()),
        0x062C => Some("jeem-ar".to_string()),
        0x062D => Some("hah-ar".to_string()),
        0x062E => Some("khah-ar".to_string()),
        0x062F => Some("dal-ar".to_string()),
        0x0630 => Some("thal-ar".to_string()),
        0x0631 => Some("reh-ar".to_string()),
        0x0632 => Some("zain-ar".to_string()),
        0x0633 => Some("seen-ar".to_string()),
        0x0634 => Some("sheen-ar".to_string()),
        0x0635 => Some("sad-ar".to_string()),
        0x0636 => Some("dad-ar".to_string()),
        0x0637 => Some("tah-ar".to_string()),
        0x0638 => Some("zah-ar".to_string()),
        0x0639 => Some("ain-ar".to_string()),
        0x063A => Some("ghain-ar".to_string()),
        0x0641 => Some("feh-ar".to_string()),
        0x0642 => Some("qaf-ar".to_string()),
        0x0643 => Some("kaf-ar".to_string()),
        0x0644 => Some("lam-ar".to_string()),
        0x0645 => Some("meem-ar".to_string()),
        0x0646 => Some("noon-ar".to_string()),
        0x0647 => Some("heh-ar".to_string()),
        0x0648 => Some("waw-ar".to_string()),
        0x064A => Some("yeh-ar".to_string()),
        _ => None,
    }
}
