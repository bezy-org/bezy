//! Keyboard input handling for text editor sorts

use super::rtl_shaping::ShapedTextCache;
use crate::core::state::fontir_app_state::FontIRAppState;
use crate::core::state::text_editor::TextEditorState;
use crate::core::state::AppState;
use crate::ui::edit_mode_toolbar::text::TextPlacementMode;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

/// Handle text editor keyboard input
pub fn handle_text_editor_keyboard_input() {
    // TODO: Implement keyboard input handling
}

/// Handle Arabic text input with HarfRust shaping
pub fn handle_arabic_text_input(
    _commands: Commands,
    mut key_evr: EventReader<KeyboardInput>,
    mut text_editor_state: ResMut<TextEditorState>,
    current_placement_mode: Res<TextPlacementMode>,
    _fontir_app_state: Option<Res<FontIRAppState>>,
    _app_state: Option<Res<AppState>>,
    _shaped_cache: ResMut<ShapedTextCache>,
) {
    // Only process in RTL text mode
    if *current_placement_mode != TextPlacementMode::RTLText {
        return;
    }

    // Check for Arabic text input
    for ev in key_evr.read() {
        if ev.state != bevy::input::ButtonState::Pressed {
            continue;
        }

        // Get the character from the keyboard event
        let key_code = ev.key_code;

        // Map key codes to Arabic characters (simplified example)
        let arabic_char = match key_code {
            // This is a simplified mapping - in reality, you'd need proper keyboard layout handling
            KeyCode::KeyA => Some('ا'), // Alef
            KeyCode::KeyB => Some('ب'), // Beh
            KeyCode::KeyT => Some('ت'), // Teh
            KeyCode::KeyJ => Some('ج'), // Jeem
            KeyCode::KeyH => Some('ح'), // Hah
            KeyCode::KeyK => Some('ك'), // Kaf
            KeyCode::KeyL => Some('ل'), // Lam
            KeyCode::KeyM => Some('م'), // Meem
            KeyCode::KeyN => Some('ن'), // Noon
            KeyCode::KeyR => Some('ر'), // Reh
            KeyCode::KeyS => Some('س'), // Seen
            KeyCode::KeyW => Some('و'), // Waw
            KeyCode::KeyY => Some('ي'), // Yeh
            _ => None,
        };

        if let Some(ch) = arabic_char {
            info!("Arabic input detected: {}", ch);

            // Build the current text context for shaping
            let mut context = String::new();

            // Get previous characters for context (simplified)
            if text_editor_state.cursor_position > 0 {
                // Get the last few sorts for context
                let start = text_editor_state.cursor_position.saturating_sub(3);
                for i in start..text_editor_state.cursor_position {
                    if let Some(sort) = text_editor_state.buffer.get(i) {
                        // Try to get the character from the sort kind
                        if let crate::core::state::text_editor::buffer::SortKind::Glyph {
                            glyph_name,
                            ..
                        } = &sort.kind
                        {
                            if let Some(ch) = glyph_name_to_char(glyph_name) {
                                context.push(ch);
                            }
                        }
                    }
                }
            }

            // Add the new character
            context.push(ch);

            // Shape the text with HarfRust (we'll implement actual shaping later)
            // For now, use simplified glyph names without contextual forms
            // These are more likely to exist in the font
            let glyph_name = get_simple_arabic_glyph_name(ch);

            // Insert the shaped glyph into the text editor
            insert_arabic_glyph(
                &mut text_editor_state,
                &glyph_name,
                &_app_state,
                &_fontir_app_state,
            );

            info!("Inserted Arabic glyph: {}", glyph_name);
        }
    }
}

/// Get simple Arabic glyph name without contextual forms
fn get_simple_arabic_glyph_name(ch: char) -> String {
    // Use basic glyph names that are more likely to exist in the font
    match ch {
        'ا' => "alef-ar".to_string(),
        'ب' => "beh-ar".to_string(),
        'ت' => "teh-ar".to_string(),
        'ج' => "jeem-ar".to_string(),
        'ح' => "hah-ar".to_string(),
        'ك' => "kaf-ar".to_string(),
        'ل' => "lam-ar".to_string(),
        'م' => "meem-ar".to_string(),
        'ن' => "noon-ar".to_string(),
        'ر' => "reh-ar".to_string(),
        'س' => "seen-ar".to_string(),
        'و' => "waw-ar".to_string(),
        'ي' => "yeh-ar".to_string(),
        _ => format!("uni{:04X}", ch as u32), // Fallback to Unicode name
    }
}

/// Convert glyph name back to character for context building
fn glyph_name_to_char(glyph_name: &str) -> Option<char> {
    // Map common Arabic glyph names back to characters
    match glyph_name {
        s if s.starts_with("alef-ar") => Some('ا'),
        s if s.starts_with("beh-ar") => Some('ب'),
        s if s.starts_with("teh-ar") => Some('ت'),
        s if s.starts_with("jeem-ar") => Some('ج'),
        s if s.starts_with("hah-ar") => Some('ح'),
        s if s.starts_with("kaf-ar") => Some('ك'),
        s if s.starts_with("lam-ar") => Some('ل'),
        s if s.starts_with("meem-ar") => Some('م'),
        s if s.starts_with("noon-ar") => Some('ن'),
        s if s.starts_with("reh-ar") => Some('ر'),
        s if s.starts_with("seen-ar") => Some('س'),
        s if s.starts_with("waw-ar") => Some('و'),
        s if s.starts_with("yeh-ar") => Some('ي'),
        "space" => Some(' '),
        _ => None,
    }
}

/// Insert an Arabic glyph into the text editor
fn insert_arabic_glyph(
    text_editor_state: &mut TextEditorState,
    glyph_name: &str,
    app_state: &Option<Res<AppState>>,
    fontir_app_state: &Option<Res<FontIRAppState>>,
) {
    use crate::core::state::text_editor::buffer::{SortData, SortKind};
    use crate::core::state::SortLayoutMode;

    // Get proper advance width from font metrics (same as LTR mode)
    let advance_width = get_glyph_advance_width(glyph_name, app_state, fontir_app_state);

    info!(
        "🔍 ARABIC INSERT: glyph '{}' with advance_width: {:.1}",
        glyph_name, advance_width
    );

    // Create a new sort entry with the Arabic glyph
    let sort = SortData {
        kind: SortKind::Glyph {
            codepoint: None, // We could map this back from glyph name if needed
            glyph_name: glyph_name.to_string(),
            advance_width, // Use proper font metrics
        },
        is_active: false,
        layout_mode: SortLayoutMode::RTLText,
        root_position: Vec2::ZERO, // Will be calculated by layout system
        buffer_cursor_position: Some(text_editor_state.cursor_position),
        buffer_id: None, // Will be set when added to buffer
    };

    // Insert at cursor position
    text_editor_state
        .buffer
        .insert(text_editor_state.cursor_position, sort);
    text_editor_state.cursor_position += 1;
    // Mark the state as changed (the buffer tracks changes internally)
}

/// Get glyph advance width from font metrics (shared with LTR mode)
fn get_glyph_advance_width(
    glyph_name: &str,
    app_state: &Option<Res<AppState>>,
    fontir_app_state: &Option<Res<FontIRAppState>>,
) -> f32 {
    if let Some(app_state) = app_state.as_ref() {
        if let Some(glyph_data) = app_state.workspace.font.glyphs.get(glyph_name) {
            let width = glyph_data.advance_width as f32;
            info!(
                "📏 ADVANCE WIDTH: Glyph '{}' from AppState = {}",
                glyph_name, width
            );
            return width;
        }
        warn!(
            "⚠️ ADVANCE WIDTH: Glyph '{}' not found in AppState",
            glyph_name
        );
    } else if let Some(fontir_state) = fontir_app_state.as_ref() {
        let width = fontir_state.get_glyph_advance_width(glyph_name);
        info!(
            "📏 ADVANCE WIDTH: Glyph '{}' from FontIR = {}",
            glyph_name, width
        );
        return width;
    }

    // Fallback default width
    warn!(
        "⚠️ ADVANCE WIDTH: Using fallback width 500.0 for glyph '{}'",
        glyph_name
    );
    500.0
}
