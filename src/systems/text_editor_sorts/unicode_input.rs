//! Unicode input handling for text editor sorts
//!
//! This module provides Unicode character input support for the text editor,
//! enabling input of any Unicode character including Latin, Arabic, Hebrew,
//! Chinese, Japanese, Korean, and other global scripts.

use crate::core::state::fontir_app_state::FontIRAppState;
use crate::core::state::{AppState, TextEditorState};
use crate::systems::arabic_shaping::{get_arabic_position, ArabicPosition};
use crate::systems::text_editor_sorts::input_utilities::{
    unicode_to_glyph_name, unicode_to_glyph_name_fontir,
};
use crate::ui::edit_mode_toolbar::text::{CurrentTextPlacementMode, TextPlacementMode};
use crate::ui::edit_mode_toolbar::CurrentTool;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::prelude::DetectChangesMut;

/// Handle Unicode character input using Bevy 0.16 keyboard events
/// This system provides comprehensive Unicode support for global scripts
pub fn handle_unicode_text_input(
    mut commands: Commands,
    mut key_evr: EventReader<KeyboardInput>,
    mut text_editor_state: ResMut<TextEditorState>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<FontIRAppState>>,
    current_tool: Res<CurrentTool>,
    current_placement_mode: Res<CurrentTextPlacementMode>,
    active_buffer: Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
    mut buffer_query: Query<(&crate::core::state::text_editor::text_buffer::TextBuffer, &mut crate::core::state::text_editor::text_buffer::BufferCursor)>,
    mut respawn_queue: ResMut<crate::systems::text_editor_sorts::sort_entities::BufferSortRespawnQueue>,
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
    if !matches!(
        current_placement_mode.0,
        TextPlacementMode::Insert | TextPlacementMode::RTLText | TextPlacementMode::LTRText
    ) {
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
                            &mut commands,
                            &mut text_editor_state,
                            &app_state,
                            &fontir_app_state,
                            &current_placement_mode,
                            &active_buffer,
                            &mut buffer_query,
                            &mut respawn_queue,
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
                        &mut commands,
                        &mut text_editor_state,
                        &app_state,
                        &fontir_app_state,
                        &current_placement_mode,
                        &active_buffer,
                        &mut buffer_query,
                        &mut respawn_queue,
                    );
                    debug!("Unicode input: Completed character '{}'", character);
                }
            }
            // Handle special keys
            Key::Backspace => {
                handle_backspace(&mut text_editor_state, &current_placement_mode, &active_buffer, &mut buffer_query, &mut respawn_queue);
            }
            Key::Delete => {
                handle_delete(&mut text_editor_state, &current_placement_mode, &active_buffer, &mut buffer_query, &mut respawn_queue);
            }
            Key::Enter => {
                handle_newline_character(&mut text_editor_state, &current_placement_mode, &active_buffer, &mut buffer_query, &mut respawn_queue);
            }
            Key::Space => {
                handle_space_character(
                    &mut commands,
                    &mut text_editor_state,
                    &app_state,
                    &fontir_app_state,
                    &current_placement_mode,
                    &active_buffer,
                    &mut buffer_query,
                    &mut respawn_queue,
                );
            }
            Key::ArrowLeft => {
                handle_arrow_left(&mut text_editor_state, &active_buffer, &mut buffer_query);
            }
            Key::ArrowRight => {
                handle_arrow_right(&mut text_editor_state, &active_buffer, &mut buffer_query);
            }
            Key::ArrowUp => {
                handle_arrow_up(&mut text_editor_state, &active_buffer, &mut buffer_query, &fontir_app_state);
            }
            Key::ArrowDown => {
                handle_arrow_down(&mut text_editor_state, &active_buffer, &mut buffer_query, &fontir_app_state);
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
    commands: &mut Commands,
    text_editor_state: &mut ResMut<TextEditorState>,
    app_state: &Option<Res<AppState>>,
    fontir_app_state: &Option<Res<FontIRAppState>>,
    current_placement_mode: &CurrentTextPlacementMode,
    active_buffer: &Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
    buffer_query: &mut Query<(&crate::core::state::text_editor::text_buffer::TextBuffer, &mut crate::core::state::text_editor::text_buffer::BufferCursor)>,
    respawn_queue: &mut ResMut<crate::systems::text_editor_sorts::sort_entities::BufferSortRespawnQueue>,
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
        info!(
            "✅ Unicode input: Found glyph '{}' for character '{}' (U+{:04X})",
            glyph_name, character, character as u32
        );
        
        // CRITICAL DEBUG: Show exactly what we're inserting
        info!("🔍 DEBUG: About to insert glyph '{}' for character '{}'", glyph_name, character);

        // Get advance width
        let advance_width = get_glyph_advance_width(&glyph_name, app_state, fontir_app_state);

        // REMOVED: Automatic text root creation
        // Text roots should only be created by clicking with the text tool
        // This was causing duplicate sorts - one from clicking, one from typing

        // Insert the character using new buffer entity system
        match current_placement_mode.0 {
            TextPlacementMode::Insert => {
                info!(
                    "🔍 DEBUG: About to insert character '{}' as glyph '{}'",
                    character, glyph_name
                );
                info!(
                    "🔍 DEBUG: Buffer state BEFORE insert: {} sorts",
                    text_editor_state.buffer.len()
                );

                // NEW BUFFER ENTITY SYSTEM: Use buffer entities instead of legacy cursor
                insert_character_at_buffer_cursor(
                    character,
                    glyph_name.clone(),
                    advance_width,
                    commands,
                    text_editor_state,
                    active_buffer,
                    buffer_query,
                    respawn_queue,
                );

                info!(
                    "🔍 DEBUG: Buffer state AFTER insert: {} sorts",
                    text_editor_state.buffer.len()
                );
                info!(
                    "Unicode input: Inserted '{}' (U+{:04X}) as glyph '{}' in Insert mode",
                    character, character as u32, glyph_name
                );
            }
            TextPlacementMode::LTRText | TextPlacementMode::RTLText => {
                let mode_name = if matches!(current_placement_mode.0, TextPlacementMode::LTRText) {
                    "LTR Text"
                } else {
                    "RTL Text"
                };

                info!("🔍 DEBUG: About to insert character '{}' as glyph '{}' in {} mode", 
                      character, glyph_name, mode_name);
                info!(
                    "🔍 DEBUG: Buffer state BEFORE insert: {} sorts",
                    text_editor_state.buffer.len()
                );

                // NEW BUFFER ENTITY SYSTEM: Use buffer entities instead of legacy cursor
                insert_character_at_buffer_cursor(
                    character,
                    glyph_name.clone(),
                    advance_width,
                    commands,
                    text_editor_state,
                    active_buffer,
                    buffer_query,
                    respawn_queue,
                );

                info!(
                    "🔍 DEBUG: Buffer state AFTER insert: {} sorts",
                    text_editor_state.buffer.len()
                );
                info!(
                    "Unicode input: Inserted '{}' (U+{:04X}) as glyph '{}' in {} mode",
                    character, character as u32, glyph_name, mode_name
                );

                // DEBUG: Check what was actually inserted
                for (i, entry) in text_editor_state.buffer.iter().enumerate() {
                    if let crate::core::state::text_editor::buffer::SortKind::Glyph {
                        glyph_name: g,
                        ..
                    } = &entry.kind
                    {
                        info!(
                            "🔍 BUFFER[{}]: glyph='{}', is_active={}, layout_mode={:?}",
                            i, g, entry.is_active, entry.layout_mode
                        );
                    }
                }
            }
            TextPlacementMode::Freeform => {
                // In freeform mode, characters are placed freely - use same buffer entity logic
                insert_character_at_buffer_cursor(
                    character,
                    glyph_name.clone(),
                    advance_width,
                    commands,
                    text_editor_state,
                    active_buffer,
                    buffer_query,
                    respawn_queue,
                );
                info!(
                    "Unicode input: Inserted '{}' (U+{:04X}) as glyph '{}' in Freeform mode",
                    character, character as u32, glyph_name
                );
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
    commands: &mut Commands,
    text_editor_state: &mut ResMut<TextEditorState>,
    app_state: &Option<Res<AppState>>,
    fontir_app_state: &Option<Res<FontIRAppState>>,
    _current_placement_mode: &CurrentTextPlacementMode,
    active_buffer: &Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
    buffer_query: &mut Query<(&crate::core::state::text_editor::text_buffer::TextBuffer, &mut crate::core::state::text_buffer::BufferCursor)>,
    respawn_queue: &mut ResMut<crate::systems::text_editor_sorts::sort_entities::BufferSortRespawnQueue>,
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
        let advance_width = get_glyph_advance_width(&glyph_name, app_state, fontir_app_state);

        // NEW BUFFER ENTITY SYSTEM: Use buffer entities instead of legacy cursor
        insert_character_at_buffer_cursor(
            ' ',
            glyph_name,
            advance_width,
            commands,
            text_editor_state,
            active_buffer,
            buffer_query,
            respawn_queue,
        );
        info!("Unicode input: Inserted space character");
    } else {
        // Fallback: insert a space-width advance without glyph
        let space_width = 250.0; // Default space width
        insert_character_at_buffer_cursor(
            ' ',
            "space".to_string(),
            space_width,
            commands,
            text_editor_state,
            active_buffer,
            buffer_query,
            respawn_queue,
        );
        info!("Unicode input: Inserted space character (fallback)");
    }
}

/// Handle newline character input
fn handle_newline_character(
    text_editor_state: &mut ResMut<TextEditorState>,
    current_placement_mode: &CurrentTextPlacementMode,
    active_buffer: &Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
    buffer_query: &mut Query<(&crate::core::state::text_editor::text_buffer::TextBuffer, &mut crate::core::state::text_editor::text_buffer::BufferCursor)>,
    respawn_queue: &mut ResMut<crate::systems::text_editor_sorts::sort_entities::BufferSortRespawnQueue>,
) {
    match current_placement_mode.0 {
        TextPlacementMode::Insert | TextPlacementMode::LTRText | TextPlacementMode::RTLText => {
            // Use buffer cursor system to insert line break at correct position
            insert_line_break_at_buffer_cursor(text_editor_state, active_buffer, buffer_query, respawn_queue);
            let mode_name = match current_placement_mode.0 {
                TextPlacementMode::Insert => "Insert",
                TextPlacementMode::LTRText => "LTR Text", 
                TextPlacementMode::RTLText => "RTL Text",
                _ => "Unknown"
            };
            info!("Unicode input: Inserted line break in {} mode", mode_name);
        }
        TextPlacementMode::Freeform => {
            // In Freeform mode, newlines might not be meaningful
            info!("Unicode input: Newline ignored in Freeform mode");
        }
    }
}

/// Insert line break at buffer cursor position with proper respawn queue management
fn insert_line_break_at_buffer_cursor(
    text_editor_state: &mut ResMut<TextEditorState>,
    active_buffer: &Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
    buffer_query: &mut Query<(&crate::core::state::text_editor::text_buffer::TextBuffer, &mut crate::core::state::text_editor::text_buffer::BufferCursor)>,
    respawn_queue: &mut ResMut<crate::systems::text_editor_sorts::sort_entities::BufferSortRespawnQueue>,
) -> bool {
    // Get the active buffer entity
    let Some(active_buffer_res) = active_buffer else {
        warn!("⚠️ LINEBREAK: No active buffer found");
        return false;
    };

    let Some(buffer_entity) = active_buffer_res.buffer_entity else {
        warn!("⚠️ LINEBREAK: Active buffer has no entity");
        return false;
    };
    let Ok((_text_buffer, mut buffer_cursor)) = buffer_query.get_mut(buffer_entity) else {
        warn!("⚠️ LINEBREAK: Could not access buffer cursor for entity {:?}", buffer_entity);
        return false;
    };

    let cursor_position = buffer_cursor.position;
    let buffer_id = _text_buffer.id;
    let layout_mode = _text_buffer.layout_mode.clone();
    let insert_buffer_index = cursor_position;

    info!("📝 LINEBREAK: Inserting line break at buffer index {} (cursor at {}) in buffer {:?} (layout: {:?})", 
          insert_buffer_index, cursor_position, buffer_id.0, layout_mode);

    // Create line break entry
    let new_line_break = crate::core::state::text_editor::buffer::SortEntry {
        kind: crate::core::state::text_editor::buffer::SortKind::LineBreak,
        is_active: false,
        layout_mode: layout_mode,
        root_position: bevy::prelude::Vec2::ZERO,
        buffer_cursor_position: None,
        buffer_id: Some(buffer_id),
    };

    // Insert the line break into the text editor buffer
    text_editor_state.buffer.insert(insert_buffer_index, new_line_break);
    
    // CRITICAL: Queue respawn for all buffer indices that shifted due to insertion
    // When we insert at index N, all existing entities at indices N and above need respawning
    // because their buffer indices shifted by +1
    for i in insert_buffer_index..text_editor_state.buffer.len() {
        respawn_queue.indices.push(i);
        info!("🔄 RESPAWN QUEUE: Added buffer index {} to respawn queue due to line break insertion", i);
    }

    // Update the cursor position in the buffer entity (advance by 1 to position after line break)
    buffer_cursor.position = cursor_position + 1;
    
    // Mark text editor state as changed for rendering updates
    text_editor_state.set_changed();

    info!("✅ LINEBREAK: Successfully inserted line break at buffer index {}, cursor moved to position {}", 
          insert_buffer_index, buffer_cursor.position);
    
    true
}

/// Handle backspace key
fn handle_backspace(
    text_editor_state: &mut ResMut<TextEditorState>,
    current_placement_mode: &CurrentTextPlacementMode,
    active_buffer: &Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
    buffer_query: &mut Query<(&crate::core::state::text_editor::text_buffer::TextBuffer, &mut crate::core::state::text_editor::text_buffer::BufferCursor)>,
    respawn_queue: &mut ResMut<crate::systems::text_editor_sorts::sort_entities::BufferSortRespawnQueue>,
) {
    match current_placement_mode.0 {
        TextPlacementMode::Insert | TextPlacementMode::LTRText | TextPlacementMode::Freeform => {
            // For LTR text and Insert mode: backspace deletes character to the LEFT of cursor
            delete_character_at_buffer_cursor(text_editor_state, active_buffer, buffer_query, respawn_queue, true);
            let mode_name = match current_placement_mode.0 {
                TextPlacementMode::Insert => "Insert",
                TextPlacementMode::LTRText => "LTR Text", 
                TextPlacementMode::Freeform => "Freeform",
                _ => "Unknown"
            };
            info!("Unicode input: Backspace in {} mode", mode_name);
        }
        TextPlacementMode::RTLText => {
            // For RTL text: backspace deletes character to the RIGHT of cursor
            delete_character_at_buffer_cursor(text_editor_state, active_buffer, buffer_query, respawn_queue, false);
            info!("Unicode input: Backspace in RTL Text mode");
        }
    }
}

/// Handle delete key
fn handle_delete(
    text_editor_state: &mut ResMut<TextEditorState>,
    current_placement_mode: &CurrentTextPlacementMode,
    active_buffer: &Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
    buffer_query: &mut Query<(&crate::core::state::text_editor::text_buffer::TextBuffer, &mut crate::core::state::text_editor::text_buffer::BufferCursor)>,
    respawn_queue: &mut ResMut<crate::systems::text_editor_sorts::sort_entities::BufferSortRespawnQueue>,
) {
    match current_placement_mode.0 {
        TextPlacementMode::Insert | TextPlacementMode::LTRText | TextPlacementMode::Freeform => {
            // For LTR text and Insert mode: delete key deletes character to the RIGHT of cursor
            delete_character_at_buffer_cursor(text_editor_state, active_buffer, buffer_query, respawn_queue, false);
            let mode_name = match current_placement_mode.0 {
                TextPlacementMode::Insert => "Insert",
                TextPlacementMode::LTRText => "LTR Text", 
                TextPlacementMode::Freeform => "Freeform",
                _ => "Unknown"
            };
            info!("Unicode input: Delete in {} mode", mode_name);
        }
        TextPlacementMode::RTLText => {
            // For RTL text: delete key deletes character to the LEFT of cursor  
            delete_character_at_buffer_cursor(text_editor_state, active_buffer, buffer_query, respawn_queue, true);
            info!("Unicode input: Delete in RTL Text mode");
        }
    }
}

/// Delete character at buffer cursor position with proper respawn queue management
fn delete_character_at_buffer_cursor(
    text_editor_state: &mut ResMut<TextEditorState>,
    active_buffer: &Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
    buffer_query: &mut Query<(&crate::core::state::text_editor::text_buffer::TextBuffer, &mut crate::core::state::text_editor::text_buffer::BufferCursor)>,
    respawn_queue: &mut ResMut<crate::systems::text_editor_sorts::sort_entities::BufferSortRespawnQueue>,
    delete_to_left: bool, // true = backspace (delete left), false = delete key (delete right)
) -> bool {
    // Get the active buffer entity
    let Some(active_buffer_res) = active_buffer else {
        warn!("⚠️ DELETE: No active buffer found");
        return false;
    };

    let Some(buffer_entity) = active_buffer_res.buffer_entity else {
        warn!("⚠️ DELETE: Active buffer has no entity");
        return false;
    };
    let Ok((_text_buffer, mut buffer_cursor)) = buffer_query.get_mut(buffer_entity) else {
        warn!("⚠️ DELETE: Could not access buffer cursor for entity {:?}", buffer_entity);
        return false;
    };

    let cursor_position = buffer_cursor.position;
    
    // Calculate which buffer index to delete based on direction
    let delete_buffer_index = if delete_to_left {
        // Backspace: delete character to the left of cursor (cursor_position - 1)
        if cursor_position == 0 {
            warn!("⚠️ DELETE: Cannot delete to left of cursor at position 0");
            return false; // Can't delete before beginning
        }
        cursor_position - 1
    } else {
        // Delete key: delete character at cursor position (cursor_position)
        if cursor_position >= text_editor_state.buffer.len() {
            warn!("⚠️ DELETE: Cannot delete at/past end of buffer (cursor: {}, buffer len: {})", 
                  cursor_position, text_editor_state.buffer.len());
            return false; // Can't delete past end
        }
        cursor_position
    };

    // Verify the buffer index is valid
    if delete_buffer_index >= text_editor_state.buffer.len() {
        warn!("⚠️ DELETE: Invalid delete index {} (buffer len: {})", 
              delete_buffer_index, text_editor_state.buffer.len());
        return false;
    }

    // Get info about what we're deleting for logging
    let deleted_glyph_name = if let Some(sort) = text_editor_state.buffer.get(delete_buffer_index) {
        sort.kind.glyph_name().to_string()
    } else {
        "unknown".to_string()
    };

    info!("🗑️ DELETE: Deleting character '{}' at buffer index {} (cursor at {}, delete_to_left: {})", 
          deleted_glyph_name, delete_buffer_index, cursor_position, delete_to_left);

    // Delete from the buffer
    let deleted_sort = text_editor_state.buffer.delete(delete_buffer_index);
    if deleted_sort.is_none() {
        warn!("⚠️ DELETE: Failed to delete from buffer at index {}", delete_buffer_index);
        return false;
    }

    // CRITICAL: Queue respawn for all buffer indices that shifted due to deletion
    // When we delete at index N, all existing entities at indices N+1 and above need respawning
    // because their buffer indices shifted by -1
    for i in delete_buffer_index..text_editor_state.buffer.len() {
        respawn_queue.indices.push(i);
        info!("🔄 RESPAWN QUEUE: Added buffer index {} to respawn queue due to deletion", i);
    }

    // Update cursor position based on deletion direction
    if delete_to_left {
        // Backspace: cursor moves left by 1
        buffer_cursor.position = cursor_position - 1;
        info!("⬅️ DELETE: Cursor moved left to position {}", buffer_cursor.position);
    } else {
        // Delete key: cursor stays in same position (but content shifted left)
        // No cursor position change needed
        info!("➡️ DELETE: Cursor remains at position {}", cursor_position);
    }

    // Mark text editor state as changed for rendering updates
    text_editor_state.set_changed();

    info!("✅ DELETE: Successfully deleted character '{}' from buffer index {}", 
          deleted_glyph_name, delete_buffer_index);
    
    true
}

/// Get advance width for a glyph from either AppState or FontIR
fn get_glyph_advance_width(
    glyph_name: &str,
    app_state: &Option<Res<AppState>>,
    fontir_app_state: &Option<Res<FontIRAppState>>,
) -> f32 {
    if let Some(app_state) = app_state.as_ref() {
        if let Some(glyph_data) = app_state.workspace.font.glyphs.get(glyph_name) {
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

    info!(
        "🔤 Direct shaping: Analyzing Arabic character '{}' ({})",
        character, base_name
    );

    // Build text context from current buffer for position analysis
    let mut text_chars = Vec::new();
    for entry in text_editor_state.buffer.iter() {
        if let crate::core::state::text_editor::buffer::SortKind::Glyph {
            codepoint: Some(ch),
            ..
        } = &entry.kind
        {
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
            let contextual = format!("{base_name}.init");
            // Check if this form exists in the font
            if fontir_state.get_glyph_names().contains(&contextual) {
                contextual
            } else {
                base_name.clone()
            }
        }
        ArabicPosition::Medial => {
            let contextual = format!("{base_name}.medi");
            if fontir_state.get_glyph_names().contains(&contextual) {
                contextual
            } else {
                base_name.clone()
            }
        }
        ArabicPosition::Final => {
            let contextual = format!("{base_name}.fina");
            if fontir_state.get_glyph_names().contains(&contextual) {
                contextual
            } else {
                base_name.clone()
            }
        }
    };

    info!(
        "🔤 Direct shaping: '{}' at position {:?} → '{}'",
        base_name, position, contextual_name
    );
    Some(contextual_name)
}

/// Insert a character at the current buffer cursor position using the new buffer entity system
fn insert_character_at_buffer_cursor(
    character: char,
    glyph_name: String,
    advance_width: f32,
    _commands: &mut Commands,
    text_editor_state: &mut ResMut<TextEditorState>,
    active_buffer: &Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
    buffer_query: &mut Query<(&crate::core::state::text_editor::text_buffer::TextBuffer, &mut crate::core::state::text_editor::text_buffer::BufferCursor)>,
    respawn_queue: &mut ResMut<crate::systems::text_editor_sorts::sort_entities::BufferSortRespawnQueue>,
) -> bool {
    info!("🔍 INSERT DEBUG: character='{}', glyph_name='{}', advance_width={:.1}", 
          character, glyph_name, advance_width);
    // Get the active buffer entity
    let Some(active_buffer_res) = active_buffer else {
        warn!("❌ INSERT: No ActiveTextBuffer resource found");
        return false;
    };
    
    let Some(buffer_entity) = active_buffer_res.buffer_entity else {
        warn!("❌ INSERT: No active buffer entity set");
        return false;
    };
    
    // Get buffer information and cursor position
    let Ok((text_buffer, mut buffer_cursor)) = buffer_query.get_mut(buffer_entity) else {
        warn!("❌ INSERT: Could not query active buffer entity {:?}", buffer_entity);
        return false;
    };
    
    let cursor_position = buffer_cursor.position;
    let buffer_id = text_buffer.id;
    let layout_mode = text_buffer.layout_mode.clone();
    
    info!(
        "🔍 INSERT: Character '{}' at buffer cursor position {} in buffer {:?} (layout: {:?})",
        character, cursor_position, buffer_id.0, layout_mode
    );
    
    // DEBUG: Show buffer state before insertion
    info!("🔍 INSERT DEBUG: Buffer state before insertion:");
    for (i, sort) in text_editor_state.buffer.iter().enumerate() {
        if sort.buffer_id == Some(buffer_id) {
            info!("  [{}] glyph='{}', buffer_id={:?}", i, sort.kind.glyph_name(), sort.buffer_id);
        }
    }
    
    // Create the new sort entry
    use crate::core::state::text_editor::buffer::{SortEntry, SortKind, SortLayoutMode};
    
    // Calculate the world position for the new character based on text flow
    let new_position = calculate_character_position(
        &text_editor_state,
        buffer_id,
        cursor_position,
        text_buffer.root_position,
        &layout_mode,
    );
    
    let new_sort = SortEntry {
        kind: SortKind::Glyph {
            codepoint: Some(character),
            glyph_name: glyph_name.clone(),
            advance_width,
        },
        is_active: false, // Don't make new sorts active by default
        layout_mode: layout_mode.clone(),
        root_position: new_position, // Calculated position based on text flow
        buffer_cursor_position: None,
        buffer_id: Some(buffer_id), // Inherit buffer ID from buffer entity
    };
    
    info!("🔍 INSERT DEBUG: Created sort with glyph_name='{}' for character='{}'", 
          new_sort.kind.glyph_name(), character);
    
    // SIMPLE CURSOR-BASED INSERTION: Find where to insert based on cursor position
    
    // Find all sorts that belong to this buffer (in order they appear in the buffer)
    let mut buffer_sort_indices = Vec::new();
    for (i, sort) in text_editor_state.buffer.iter().enumerate() {
        if sort.buffer_id == Some(buffer_id) {
            buffer_sort_indices.push(i);
        }
    }
    
    // Insert at cursor position within this buffer's sequence
    let insert_buffer_index = if buffer_sort_indices.is_empty() {
        // First sort for this buffer - insert at the end of all sorts
        text_editor_state.buffer.len()
    } else if cursor_position >= buffer_sort_indices.len() {
        // Cursor is at or beyond the end - insert after the last sort of this buffer
        buffer_sort_indices.last().unwrap() + 1
    } else {
        // Insert at the cursor position within this buffer's sequence
        buffer_sort_indices[cursor_position]
    };
    
    info!(
        "🔍 INSERT: Inserting character '{}' at buffer index {} (buffer has {} existing sorts, cursor at {})",
        character, insert_buffer_index, buffer_sort_indices.len(), cursor_position
    );
    
    // Insert the new sort into the text editor buffer
    text_editor_state.buffer.insert(insert_buffer_index, new_sort);
    
    // CRITICAL FIX: Queue respawn for all buffer indices that shifted due to insertion
    // When we insert at index N, all existing entities at indices N and above need respawning
    // because their buffer indices shifted by +1
    for i in insert_buffer_index..text_editor_state.buffer.len() {
        respawn_queue.indices.push(i);
        info!("🔄 RESPAWN QUEUE: Added buffer index {} to respawn queue due to insertion", i);
    }
    
    // DEBUG: Verify what actually got inserted
    if let Some(inserted_sort) = text_editor_state.buffer.get(insert_buffer_index) {
        info!("🔍 INSERT DEBUG: Verified inserted sort at index {}: glyph_name='{}', character codepoint={:?}", 
              insert_buffer_index, inserted_sort.kind.glyph_name(), 
              if let crate::core::state::text_editor::buffer::SortKind::Glyph { codepoint, .. } = &inserted_sort.kind {
                  codepoint.map(|c| format!("'{}'", c))
              } else { None });
    }
    
    // Update the cursor position in the buffer entity (advance by 1)
    buffer_cursor.position = cursor_position + 1;
    
    info!(
        "✅ INSERT: Successfully inserted '{}' as glyph '{}', cursor advanced to position {}",
        character, glyph_name, buffer_cursor.position
    );
    
    // Mark the text editor state as changed to trigger entity spawning
    use bevy::prelude::DetectChangesMut;
    text_editor_state.set_changed();
    
    // Return true to indicate successful insertion
    true
}

/// Calculate the world position for a new character based on text flow
fn calculate_character_position(
    text_editor_state: &crate::core::state::TextEditorState,
    buffer_id: crate::core::state::text_editor::buffer::BufferId,
    cursor_position: usize,
    buffer_root_position: bevy::math::Vec2,
    layout_mode: &crate::core::state::text_editor::buffer::SortLayoutMode,
) -> bevy::math::Vec2 {
    use crate::core::state::text_editor::buffer::{SortKind, SortLayoutMode};
    
    // Find all sorts that belong to this buffer in order
    let mut buffer_sorts = Vec::new();
    for sort in text_editor_state.buffer.iter() {
        if sort.buffer_id == Some(buffer_id) {
            buffer_sorts.push(sort);
        }
    }
    
    // UNIFIED APPROACH: Calculate cumulative advance width up to cursor position
    // This works for ALL cursor positions including 0 (no special cases needed)
    let mut x_offset = 0.0;
    let mut y_offset = 0.0; // For future line break support
    
    for (i, sort) in buffer_sorts.iter().enumerate() {
        if i >= cursor_position {
            break; // Don't include sorts after cursor position
        }
        
        match &sort.kind {
            SortKind::Glyph { advance_width, .. } => {
                match layout_mode {
                    SortLayoutMode::LTRText => {
                        // LTR: accumulate advances to the right
                        x_offset += advance_width;
                    }
                    SortLayoutMode::RTLText => {
                        // RTL: accumulate advances to the left
                        x_offset -= advance_width;
                    }
                    SortLayoutMode::Freeform => {
                        // Freeform sorts shouldn't use this function, but handle gracefully
                        x_offset += advance_width;
                    }
                }
            }
            SortKind::LineBreak => {
                // Future: handle line breaks
                x_offset = 0.0;
                y_offset -= 1000.0; // Simple line height for now
            }
        }
    }
    
    bevy::math::Vec2::new(
        buffer_root_position.x + x_offset,
        buffer_root_position.y + y_offset,
    )
}

/// Handle left arrow key press - move cursor left in the active buffer
fn handle_arrow_left(
    text_editor_state: &mut ResMut<TextEditorState>,
    active_buffer: &Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
    buffer_query: &mut Query<(&crate::core::state::text_editor::text_buffer::TextBuffer, &mut crate::core::state::text_editor::text_buffer::BufferCursor)>,
) {
    let Some(active_buffer) = active_buffer.as_ref() else {
        debug!("No active buffer for arrow left");
        return;
    };
    
    let Some(buffer_entity) = active_buffer.buffer_entity else {
        debug!("No buffer entity for arrow left");
        return;
    };
    
    let Ok((_text_buffer, mut buffer_cursor)) = buffer_query.get_mut(buffer_entity) else {
        debug!("Buffer entity not found for arrow left");
        return;
    };
    
    // Move cursor left if not at the beginning
    if buffer_cursor.position > 0 {
        buffer_cursor.position -= 1;
        info!("⬅️ Moved cursor left to position {}", buffer_cursor.position);
    } else {
        debug!("Cursor already at beginning, cannot move left");
    }
    
    // Mark text editor state as changed to trigger cursor rendering update
    use bevy::prelude::DetectChangesMut;
    text_editor_state.set_changed();
}

/// Handle right arrow key press - move cursor right in the active buffer
fn handle_arrow_right(
    text_editor_state: &mut ResMut<TextEditorState>,
    active_buffer: &Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
    buffer_query: &mut Query<(&crate::core::state::text_editor::text_buffer::TextBuffer, &mut crate::core::state::text_editor::text_buffer::BufferCursor)>,
) {
    let Some(active_buffer) = active_buffer.as_ref() else {
        debug!("No active buffer for arrow right");
        return;
    };
    
    let Some(buffer_entity) = active_buffer.buffer_entity else {
        debug!("No buffer entity for arrow right");
        return;
    };
    
    let Ok((text_buffer, mut buffer_cursor)) = buffer_query.get_mut(buffer_entity) else {
        debug!("Buffer entity not found for arrow right");
        return;
    };
    
    // Count how many sorts belong to this buffer to determine max cursor position
    let buffer_sort_count = text_editor_state
        .buffer
        .iter()
        .filter(|sort| sort.buffer_id == Some(text_buffer.id))
        .count();
    
    // Move cursor right if not at the end
    if buffer_cursor.position < buffer_sort_count {
        buffer_cursor.position += 1;
        info!("➡️ Moved cursor right to position {}", buffer_cursor.position);
    } else {
        debug!("Cursor already at end, cannot move right");
    }
    
    // Mark text editor state as changed to trigger cursor rendering update
    use bevy::prelude::DetectChangesMut;
    text_editor_state.set_changed();
}

/// Handle up arrow key press - move cursor to previous line in the active buffer
fn handle_arrow_up(
    text_editor_state: &mut ResMut<TextEditorState>,
    active_buffer: &Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
    buffer_query: &mut Query<(&crate::core::state::text_editor::text_buffer::TextBuffer, &mut crate::core::state::text_editor::text_buffer::BufferCursor)>,
    fontir_app_state: &Option<Res<crate::core::state::fontir_app_state::FontIRAppState>>,
) {
    let Some(active_buffer) = active_buffer.as_ref() else {
        debug!("No active buffer for arrow up");
        return;
    };
    
    let Some(buffer_entity) = active_buffer.buffer_entity else {
        debug!("No buffer entity for arrow up");
        return;
    };
    
    let Ok((text_buffer, mut buffer_cursor)) = buffer_query.get_mut(buffer_entity) else {
        debug!("Buffer entity not found for arrow up");
        return;
    };
    
    let current_position = buffer_cursor.position;
    
    // Find the new cursor position using line-aware navigation
    if let Some(new_position) = calculate_line_navigation_position(
        text_editor_state,
        text_buffer.id,
        current_position,
        LineNavigation::Up,
        fontir_app_state,
    ) {
        buffer_cursor.position = new_position;
        info!("⬆️ Moved cursor up from position {} to position {}", current_position, new_position);
    } else {
        debug!("Cursor already at top line, cannot move up");
    }
    
    // Mark text editor state as changed to trigger cursor rendering update
    use bevy::prelude::DetectChangesMut;
    text_editor_state.set_changed();
}

/// Handle down arrow key press - move cursor to next line in the active buffer
fn handle_arrow_down(
    text_editor_state: &mut ResMut<TextEditorState>,
    active_buffer: &Option<Res<crate::core::state::text_editor::text_buffer::ActiveTextBuffer>>,
    buffer_query: &mut Query<(&crate::core::state::text_editor::text_buffer::TextBuffer, &mut crate::core::state::text_editor::text_buffer::BufferCursor)>,
    fontir_app_state: &Option<Res<crate::core::state::fontir_app_state::FontIRAppState>>,
) {
    let Some(active_buffer) = active_buffer.as_ref() else {
        debug!("No active buffer for arrow down");
        return;
    };
    
    let Some(buffer_entity) = active_buffer.buffer_entity else {
        debug!("No buffer entity for arrow down");
        return;
    };
    
    let Ok((text_buffer, mut buffer_cursor)) = buffer_query.get_mut(buffer_entity) else {
        debug!("Buffer entity not found for arrow down");
        return;
    };
    
    let current_position = buffer_cursor.position;
    
    // Find the new cursor position using line-aware navigation
    if let Some(new_position) = calculate_line_navigation_position(
        text_editor_state,
        text_buffer.id,
        current_position,
        LineNavigation::Down,
        fontir_app_state,
    ) {
        buffer_cursor.position = new_position;
        info!("⬇️ Moved cursor down from position {} to position {}", current_position, new_position);
    } else {
        debug!("Cursor already at bottom line, cannot move down");
    }
    
    // Mark text editor state as changed to trigger cursor rendering update
    use bevy::prelude::DetectChangesMut;
    text_editor_state.set_changed();
}

/// Direction for line navigation
#[derive(Debug, Clone, Copy)]
enum LineNavigation {
    Up,
    Down,
}

/// Calculate cursor position for up/down line navigation
/// This function handles line breaks and tries to maintain horizontal position
fn calculate_line_navigation_position(
    text_editor_state: &TextEditorState,
    buffer_id: crate::core::state::text_editor::buffer::BufferId,
    current_position: usize,
    direction: LineNavigation,
    fontir_app_state: &Option<Res<crate::core::state::fontir_app_state::FontIRAppState>>,
) -> Option<usize> {
    use crate::core::state::text_editor::buffer::{SortKind, SortLayoutMode};
    
    // Get all sorts that belong to this buffer
    let mut buffer_sorts = Vec::new();
    for sort in text_editor_state.buffer.iter() {
        if sort.buffer_id == Some(buffer_id) {
            buffer_sorts.push(sort);
        }
    }
    
    if buffer_sorts.is_empty() {
        return None;
    }
    
    // Build line structure: find line breaks and calculate x positions
    let mut line_starts = vec![0]; // Line starts at positions in buffer_sorts
    let mut x_positions = vec![0.0]; // X position for each cursor position
    
    let mut current_x = 0.0;
    for (i, sort) in buffer_sorts.iter().enumerate() {
        match &sort.kind {
            SortKind::LineBreak => {
                // Line break: start new line
                line_starts.push(i + 1);
                current_x = 0.0;
                x_positions.push(current_x);
            }
            SortKind::Glyph { advance_width, glyph_name, .. } => {
                // Get advance width for this glyph
                let width = if let Some(fontir_state) = fontir_app_state.as_ref() {
                    fontir_state.get_glyph_advance_width(glyph_name)
                } else {
                    *advance_width
                };
                
                // Handle text direction
                match sort.layout_mode {
                    SortLayoutMode::LTRText => {
                        current_x += width;
                    }
                    SortLayoutMode::RTLText => {
                        current_x -= width;
                    }
                    SortLayoutMode::Freeform => {
                        current_x += width; // Treat as LTR for line navigation
                    }
                }
                x_positions.push(current_x);
            }
        }
    }
    
    // Find which line the current position is on
    let mut current_line = 0;
    for (line_index, &line_start) in line_starts.iter().enumerate() {
        if current_position >= line_start {
            current_line = line_index;
        } else {
            break;
        }
    }
    
    // Get the x position of the cursor at current position
    let current_x = x_positions.get(current_position).copied().unwrap_or(0.0);
    
    // Calculate target line
    let target_line = match direction {
        LineNavigation::Up => {
            if current_line == 0 {
                return None; // Already at top line
            }
            current_line - 1
        }
        LineNavigation::Down => {
            if current_line + 1 >= line_starts.len() {
                return None; // Already at bottom line
            }
            current_line + 1
        }
    };
    
    // Find the position in the target line that's closest to current_x
    let target_line_start = line_starts[target_line];
    let target_line_end = if target_line + 1 < line_starts.len() {
        line_starts[target_line + 1]
    } else {
        buffer_sorts.len()
    };
    
    // Find the closest position by x coordinate
    let mut best_position = target_line_start;
    let mut best_distance = f32::INFINITY;
    
    for position in target_line_start..=target_line_end {
        if let Some(x) = x_positions.get(position) {
            let distance = (x - current_x).abs();
            if distance < best_distance {
                best_distance = distance;
                best_position = position;
            }
        }
    }
    
    Some(best_position)
}

/// Legacy function - replaced by handle_unicode_text_input
pub fn handle_unicode_input() {
    // This function is kept for compatibility but does nothing
    // Use handle_unicode_text_input instead
}
