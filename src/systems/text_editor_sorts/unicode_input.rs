//! Unicode input handling for text editor sorts
//!
//! This module provides Unicode character input support for the text editor,
//! enabling input of any Unicode character including Latin, Arabic, Hebrew,
//! Chinese, Japanese, Korean, and other global scripts.

use crate::core::state::fontir_app_state::FontIRAppState;
use crate::core::state::{AppState, TextEditorState};
use crate::systems::text_editor_sorts::input_utilities::{
    unicode_to_glyph_name, unicode_to_glyph_name_fontir,
};
use crate::systems::arabic_shaping::{get_arabic_position, ArabicPosition};
use crate::ui::toolbars::edit_mode_toolbar::text::{
    CurrentTextPlacementMode, TextPlacementMode,
};
use crate::ui::toolbars::edit_mode_toolbar::CurrentTool;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;

/// Handle Unicode character input using Bevy 0.16 keyboard events
/// This system provides comprehensive Unicode support for global scripts
pub fn handle_unicode_text_input(
    mut key_evr: EventReader<KeyboardInput>,
    mut text_editor_state: ResMut<TextEditorState>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<FontIRAppState>>,
    current_tool: Res<CurrentTool>,
    current_placement_mode: Res<CurrentTextPlacementMode>,
) {
    // EARLY RETURN: Skip all expensive work if no keyboard events
    if key_evr.is_empty() {
        debug!("Unicode input skipped - no keyboard events");
        return;
    }

    // DEBUG: Log system entry for any keyboard input
    let key_count = key_evr.len();
    debug!("Unicode input: {} keyboard events detected", key_count);
    debug!("Current tool: {:?}", current_tool.get_current());
    debug!("Current placement mode: {:?}", current_placement_mode.0);

    // Only handle input when text tool is active
    if current_tool.get_current() != Some("text") {
        debug!("Unicode input blocked: Text tool not active");
        return;
    }

    // Handle typing in Insert mode and text placement modes (RTL/LTR)
    if !matches!(current_placement_mode.0, TextPlacementMode::Insert | TextPlacementMode::RTLText | TextPlacementMode::LTRText) {
        debug!(
            "Unicode input blocked: Not in a text input mode (current: {:?})",
            current_placement_mode.0
        );
        return;
    }

    if key_count > 0 {
        debug!(
            "Unicode input: Processing {} keyboard events in text input mode ({:?})",
            key_count, current_placement_mode.0
        );
    }

    debug!("Unicode input: Processing in Insert mode");

    // Handle keyboard input events
    let event_count = key_evr.len();
    info!("Unicode input: Processing {} keyboard events", event_count);

    for ev in key_evr.read() {
        info!(
            "Unicode input: Keyboard event - key: {:?}, state: {:?}",
            ev.logical_key, ev.state
        );

        // Only process pressed keys
        let is_pressed = matches!(ev.state, ButtonState::Pressed);
        info!(
            "Unicode input: Key state - is_pressed: {}, raw state: {:?}",
            is_pressed, ev.state
        );

        if !is_pressed {
            debug!("Unicode input: Skipping non-pressed key event");
            continue;
        }

        match &ev.logical_key {
            // Handle Unicode character input
            Key::Character(character_string) => {
                info!(
                    "Unicode input: Character key pressed: '{}'",
                    character_string
                );
                // Process each character in the string (usually just one)
                for character in character_string.chars() {
                    info!(
                        "Unicode input: Processing character '{}' (U+{:04X})",
                        character, character as u32
                    );
                    // Skip control characters (except newline)
                    if character.is_control() && character != '\n' {
                        debug!("Unicode input: Skipping control character");
                        continue;
                    }

                    // Handle space character
                    if character == ' ' {
                        handle_space_character(
                            &mut text_editor_state,
                            &app_state,
                            &fontir_app_state,
                            &current_placement_mode,
                        );
                        continue;
                    }

                    // Skip newline character - handled by Key::Enter instead
                    // to avoid duplicate line break insertion
                    if character == '\n' {
                        debug!("Unicode input: Skipping '\\n' character - handled by Key::Enter");
                        continue;
                    }

                    // Handle regular Unicode character
                    debug!("Unicode input: Handling character '{}'", character);
                    handle_unicode_character(
                        character,
                        &mut text_editor_state,
                        &app_state,
                        &fontir_app_state,
                        &current_placement_mode,
                    );
                    debug!(
                        "Unicode input: Completed character '{}'",
                        character
                    );
                }
            }
            // Handle special keys
            Key::Backspace => {
                handle_backspace(
                    &mut text_editor_state,
                    &current_placement_mode,
                );
            }
            Key::Delete => {
                handle_delete(&mut text_editor_state, &current_placement_mode);
            }
            Key::Enter => {
                handle_newline_character(
                    &mut text_editor_state,
                    &current_placement_mode,
                );
            }
            Key::Space => {
                handle_space_character(
                    &mut text_editor_state,
                    &app_state,
                    &fontir_app_state,
                    &current_placement_mode,
                );
            }
            _ => {
                // Ignore other special keys
            }
        }
    }
}

/// Handle a single Unicode character input
fn handle_unicode_character(
    character: char,
    text_editor_state: &mut TextEditorState,
    app_state: &Option<Res<AppState>>,
    fontir_app_state: &Option<Res<FontIRAppState>>,
    current_placement_mode: &CurrentTextPlacementMode,
) {
    // Find glyph name for this Unicode character
    let glyph_name = if let Some(app_state) = app_state.as_ref() {
        unicode_to_glyph_name(character, app_state)
    } else if let Some(fontir_state) = fontir_app_state.as_ref() {
        // For FontIR, use enhanced Arabic-aware glyph lookup with contextual shaping
        get_contextual_arabic_glyph_name(character, text_editor_state, fontir_state)
    } else {
        None
    };

    if let Some(glyph_name) = glyph_name {
        info!("✅ Unicode input: Found glyph '{}' for character '{}' (U+{:04X})", 
              glyph_name, character, character as u32);
        
        // Get advance width
        let advance_width =
            get_glyph_advance_width(&glyph_name, app_state, fontir_app_state);

        // REMOVED: Automatic text root creation
        // Text roots should only be created by clicking with the text tool
        // This was causing duplicate sorts - one from clicking, one from typing

        // Insert the character
        match current_placement_mode.0 {
            TextPlacementMode::Insert => {
                info!("🔍 DEBUG: About to insert character '{}' as glyph '{}' at cursor position {}", 
                      character, glyph_name, text_editor_state.cursor_position);
                info!("🔍 DEBUG: Buffer state BEFORE insert: {} sorts", text_editor_state.buffer.len());
                
                text_editor_state.insert_sort_at_cursor(
                    glyph_name.clone(),
                    advance_width,
                    Some(character),
                );
                
                info!("🔍 DEBUG: Buffer state AFTER insert: {} sorts, cursor at {}", 
                      text_editor_state.buffer.len(), text_editor_state.cursor_position);
                info!("Unicode input: Inserted '{}' (U+{:04X}) as glyph '{}' in Insert mode", 
                      character, character as u32, glyph_name);
            }
            TextPlacementMode::LTRText | TextPlacementMode::RTLText => {
                let mode_name = if matches!(
                    current_placement_mode.0,
                    TextPlacementMode::LTRText
                ) {
                    "LTR Text"
                } else {
                    "RTL Text"
                };
                
                info!("🔍 DEBUG: About to insert character '{}' as glyph '{}' at cursor position {} in {} mode", 
                      character, glyph_name, text_editor_state.cursor_position, mode_name);
                info!("🔍 DEBUG: Buffer state BEFORE insert: {} sorts", text_editor_state.buffer.len());
                
                text_editor_state.insert_sort_at_cursor(
                    glyph_name.clone(),
                    advance_width,
                    Some(character),
                );
                
                info!("🔍 DEBUG: Buffer state AFTER insert: {} sorts, cursor at {}", 
                      text_editor_state.buffer.len(), text_editor_state.cursor_position);
                info!("Unicode input: Inserted '{}' (U+{:04X}) as glyph '{}' in {} mode", 
                      character, character as u32, glyph_name, mode_name);
                      
                // DEBUG: Check what was actually inserted
                for (i, entry) in text_editor_state.buffer.iter().enumerate() {
                    if let crate::core::state::text_editor::buffer::SortKind::Glyph { glyph_name: g, .. } = &entry.kind {
                        info!("🔍 BUFFER[{}]: glyph='{}', is_active={}, layout_mode={:?}", 
                              i, g, entry.is_active, entry.layout_mode);
                    }
                }
            }
            TextPlacementMode::Freeform => {
                // In freeform mode, characters are placed freely - for now use same logic
                text_editor_state.insert_sort_at_cursor(
                    glyph_name.clone(),
                    advance_width,
                    Some(character),
                );
                info!("Unicode input: Inserted '{}' (U+{:04X}) as glyph '{}' in Freeform mode", 
                      character, character as u32, glyph_name);
            }
        }
    } else {
        warn!(
            "❌ Unicode input: No glyph found for character '{}' (U+{:04X})",
            character, character as u32
        );
        
        // Try to check if this is an Arabic character
        if (character as u32) >= 0x0600 && (character as u32) <= 0x06FF {
            warn!("❌ Unicode input: This is an Arabic character but no glyph mapping found");
        }
    }
}

/// Handle space character input
fn handle_space_character(
    text_editor_state: &mut TextEditorState,
    app_state: &Option<Res<AppState>>,
    fontir_app_state: &Option<Res<FontIRAppState>>,
    _current_placement_mode: &CurrentTextPlacementMode,
) {
    let glyph_name = "space".to_string();

    // Check if space glyph exists
    let glyph_exists = if let Some(app_state) = app_state.as_ref() {
        app_state.workspace.font.glyphs.contains_key(&glyph_name)
    } else if let Some(fontir_state) = fontir_app_state.as_ref() {
        fontir_state.get_glyph(&glyph_name).is_some()
    } else {
        false
    };

    if glyph_exists {
        let advance_width =
            get_glyph_advance_width(&glyph_name, app_state, fontir_app_state);

        // REMOVED: Automatic text root creation
        // Text roots should only be created by clicking with the text tool
        // This was causing duplicate sorts - one from clicking, one from typing

        text_editor_state.insert_sort_at_cursor(
            glyph_name,
            advance_width,
            Some(' '), // We know this is Unicode U+0020 (space character)
        );
        info!("Unicode input: Inserted space character");
    } else {
        // Fallback: insert a space-width advance without glyph
        let space_width = 250.0; // Default space width
        text_editor_state.insert_sort_at_cursor(
            "space".to_string(),
            space_width,
            Some(' '), // Even in fallback, we know it's U+0020
        );
        info!("Unicode input: Inserted space character (fallback)");
    }
}

/// Handle newline character input
fn handle_newline_character(
    text_editor_state: &mut TextEditorState,
    current_placement_mode: &CurrentTextPlacementMode,
) {
    match current_placement_mode.0 {
        TextPlacementMode::Insert => {
            text_editor_state.insert_line_break_at_cursor();
            info!("Unicode input: Inserted line break in Insert mode");
        }
        TextPlacementMode::LTRText | TextPlacementMode::RTLText => {
            // In Text mode, newlines might move to next line in grid
            text_editor_state.insert_line_break_at_cursor();
            let mode_name = if matches!(
                current_placement_mode.0,
                TextPlacementMode::LTRText
            ) {
                "LTR Text"
            } else {
                "RTL Text"
            };
            info!("Unicode input: Inserted line break in {} mode", mode_name);
        }
        TextPlacementMode::Freeform => {
            // In Freeform mode, newlines might not be meaningful
            info!("Unicode input: Newline ignored in Freeform mode");
        }
    }
}

/// Handle backspace key
fn handle_backspace(
    text_editor_state: &mut TextEditorState,
    current_placement_mode: &CurrentTextPlacementMode,
) {
    match current_placement_mode.0 {
        TextPlacementMode::Insert => {
            text_editor_state.delete_sort_at_cursor();
            info!("Unicode input: Backspace in Insert mode");
        }
        TextPlacementMode::LTRText | TextPlacementMode::RTLText => {
            // delete_sort_at_cursor already handles deleting to the left of cursor and updating cursor position
            text_editor_state.delete_sort_at_cursor();
            let mode_name = if matches!(
                current_placement_mode.0,
                TextPlacementMode::LTRText
            ) {
                "LTR Text"
            } else {
                "RTL Text"
            };
            info!("Unicode input: Backspace in {} mode", mode_name);
        }
        TextPlacementMode::Freeform => {
            // delete_sort_at_cursor already handles deleting to the left of cursor and updating cursor position
            text_editor_state.delete_sort_at_cursor();
            info!("Unicode input: Backspace in Freeform mode");
        }
    }
}

/// Handle delete key
fn handle_delete(
    text_editor_state: &mut TextEditorState,
    _current_placement_mode: &CurrentTextPlacementMode,
) {
    text_editor_state.delete_sort_at_cursor();
    info!("Unicode input: Delete key pressed");
}

/// Get advance width for a glyph from either AppState or FontIR
fn get_glyph_advance_width(
    glyph_name: &str,
    app_state: &Option<Res<AppState>>,
    fontir_app_state: &Option<Res<FontIRAppState>>,
) -> f32 {
    if let Some(app_state) = app_state.as_ref() {
        if let Some(glyph_data) =
            app_state.workspace.font.glyphs.get(glyph_name)
        {
            return glyph_data.advance_width as f32;
        }
    } else if let Some(fontir_state) = fontir_app_state.as_ref() {
        return fontir_state.get_glyph_advance_width(glyph_name);
    }

    // Fallback default width
    500.0
}

/// Get contextual Arabic glyph name by analyzing position in text buffer
fn get_contextual_arabic_glyph_name(
    character: char,
    text_editor_state: &TextEditorState,
    fontir_state: &FontIRAppState,
) -> Option<String> {
    // First get the base glyph name
    let base_name = unicode_to_glyph_name_fontir(character, fontir_state)?;
    
    // Check if this is an Arabic character that needs contextual shaping
    if (character as u32) < 0x0600 || (character as u32) > 0x06FF {
        return Some(base_name);
    }
    
    info!("🔤 Direct shaping: Analyzing Arabic character '{}' ({})", character, base_name);
    
    // Build text context from current buffer for position analysis
    let mut text_chars = Vec::new();
    for entry in text_editor_state.buffer.iter() {
        if let crate::core::state::text_editor::buffer::SortKind::Glyph { codepoint: Some(ch), .. } = &entry.kind {
            text_chars.push(*ch);
        }
    }
    
    // Add the current character at cursor position
    let cursor_pos = text_editor_state.cursor_position;
    if cursor_pos <= text_chars.len() {
        text_chars.insert(cursor_pos, character);
    } else {
        text_chars.push(character);
    }
    
    // Determine Arabic position
    let position = get_arabic_position(&text_chars, cursor_pos);
    
    // Apply contextual form
    let contextual_name = match position {
        ArabicPosition::Isolated => base_name.clone(),
        ArabicPosition::Initial => {
            let contextual = format!("{}.init", base_name);
            // Check if this form exists in the font
            if fontir_state.get_glyph_names().contains(&contextual) {
                contextual
            } else {
                base_name.clone()
            }
        },
        ArabicPosition::Medial => {
            let contextual = format!("{}.medi", base_name);
            if fontir_state.get_glyph_names().contains(&contextual) {
                contextual
            } else {
                base_name.clone()
            }
        },
        ArabicPosition::Final => {
            let contextual = format!("{}.fina", base_name);
            if fontir_state.get_glyph_names().contains(&contextual) {
                contextual
            } else {
                base_name.clone()
            }
        },
    };
    
    info!("🔤 Direct shaping: '{}' at position {:?} → '{}'", base_name, position, contextual_name);
    Some(contextual_name)
}

/// Legacy function - replaced by handle_unicode_text_input
pub fn handle_unicode_input() {
    // This function is kept for compatibility but does nothing
    // Use handle_unicode_text_input instead
}
