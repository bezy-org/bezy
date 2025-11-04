use crate::core::state::{GlyphNavigation, TextEditorState};
use crate::systems::sorts::sort_entities::BufferSortRespawnQueue;
use crate::ui::edit_mode_toolbar::text::TextPlacementMode;
use crate::ui::edit_mode_toolbar::CurrentTool;
use bevy::prelude::*;

/// Handle TUI message to select/insert a glyph
pub fn handle_glyph_selection(
    unicode_codepoint: u32,
    glyph_nav: &mut ResMut<GlyphNavigation>,
    text_editor_state: &mut Option<ResMut<TextEditorState>>,
    respawn_queue: &mut ResMut<BufferSortRespawnQueue>,
    current_tool: &Option<Res<CurrentTool>>,
    text_placement_mode: &Option<Res<TextPlacementMode>>,
) -> Result<String, String> {
    info!("TUI requested glyph selection: U+{:04X}", unicode_codepoint);

    // Convert Unicode to char for processing
    let target_char = char::from_u32(unicode_codepoint);

    // Use Unicode format for glyph name
    let glyph_name = if let Some(_target_char) = target_char {
        format!("U+{:04X}", unicode_codepoint)
    } else {
        return Err("Invalid Unicode codepoint".to_string());
    };

    // Update glyph tracking
    glyph_nav.set_current_glyph(glyph_name.clone());

    // Check if we're in Text tool's Insert mode
    let is_text_tool = current_tool
        .as_ref()
        .map(|tool| tool.get_current() == Some("text"))
        .unwrap_or(false);

    let is_insert_mode = text_placement_mode
        .as_ref()
        .map(|mode| **mode == TextPlacementMode::Insert)
        .unwrap_or(false);

    if let Some(ref mut text_state) = text_editor_state {
        if is_text_tool && is_insert_mode {
            // In Insert mode: insert a new sort at cursor position (like typing)
            if let Some(target_char) = target_char {
                // Default advance width
                let advance_width = 500.0;

                // Insert the sort at the cursor position (like typing would)
                text_state.insert_sort_at_cursor_with_respawn(
                    glyph_name.clone(),
                    advance_width,
                    Some(target_char),
                    Some(respawn_queue),
                );

                // Mark the text editor state as changed to trigger entity spawning
                use bevy::prelude::DetectChangesMut;
                text_state.set_changed();

                // Force more aggressive updates by modifying viewport to trigger rerender
                let current_viewport = text_state.viewport_offset;
                text_state.viewport_offset = current_viewport + bevy::math::Vec2::new(0.001, 0.0);
                text_state.viewport_offset = current_viewport; // Reset it but triggers change detection

                info!(
                    "Inserted new sort for glyph '{}' (U+{:04X}) at cursor position",
                    glyph_name, unicode_codepoint
                );
                Ok(glyph_name)
            } else {
                Err("Invalid Unicode codepoint".to_string())
            }
        } else {
            // Not in Insert mode: change what the active sort displays
            match change_active_sort_glyph(
                unicode_codepoint,
                &target_char,
                &glyph_name,
                text_state,
                respawn_queue,
            ) {
                Ok(_) => Ok(glyph_name),
                Err(msg) => Err(msg),
            }
        }
    } else {
        Err("No TextEditorState available for glyph change".to_string())
    }
}

/// Change what glyph the active sort displays
fn change_active_sort_glyph(
    unicode_codepoint: u32,
    target_char: &Option<char>,
    glyph_name: &str,
    text_state: &mut ResMut<TextEditorState>,
    respawn_queue: &mut ResMut<BufferSortRespawnQueue>,
) -> Result<(), String> {
    // Find the active sort
    let mut active_sort_index = None;
    for i in 0..text_state.buffer.len() {
        if let Some(sort) = text_state.buffer.get(i) {
            if sort.is_active {
                active_sort_index = Some(i);
                break;
            }
        }
    }

    if let Some(index) = active_sort_index {
        // Change what glyph the active sort displays
        if let Some(target_char) = target_char {
            // Default advance width
            let advance_width = 500.0;

            // Update the sort's displayed glyph
            if let Some(sort) = text_state.buffer.get_mut(index) {
                sort.kind = crate::core::state::text_editor::buffer::SortKind::Glyph {
                    codepoint: Some(*target_char),
                    glyph_name: glyph_name.to_string(),
                    advance_width,
                };
                info!(
                    "Changed active sort to display glyph U+{:04X}",
                    unicode_codepoint
                );

                // Queue the sort for respawn so the visual updates
                respawn_queue.indices.push(index);
                debug!("Queued sort at index {} for respawn", index);

                // Mark the text editor state as changed to trigger visual updates
                use bevy::prelude::DetectChangesMut;
                text_state.set_changed();

                // Force more aggressive updates by modifying viewport to trigger rerender
                let current_viewport = text_state.viewport_offset;
                text_state.viewport_offset = current_viewport + bevy::math::Vec2::new(0.001, 0.0);
                text_state.viewport_offset = current_viewport; // Reset it but triggers change detection

                Ok(())
            } else {
                Err("Failed to get mutable reference to active sort".to_string())
            }
        } else {
            Err("Invalid Unicode codepoint".to_string())
        }
    } else {
        Err("No active sort - click on a glyph in the editor first".to_string())
    }
}
