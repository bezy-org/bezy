//! Centralized keyboard shortcuts for toolbar tools
//!
//! This module provides a single system that handles all keyboard shortcuts
//! for tool switching, ensuring consistency and preventing conflicts.

use bevy::prelude::*;
use crate::tools::{SwitchToolEvent, ToolId, ToolState};
use super::toolbar_config::TOOLBAR_TOOLS;

/// System to handle keyboard shortcuts for all tools
///
/// This centralizes all tool keyboard shortcuts in one place, making it
/// easier to manage conflicts and ensure consistency.
pub fn handle_tool_keyboard_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    tool_state: Res<ToolState>,
    mut switch_events: EventWriter<SwitchToolEvent>,
    text_mode_active: Option<Res<super::text::TextModeActive>>,
) {
    // Skip if text mode is active (text tool needs raw keyboard input)
    if text_mode_active.map(|t| t.0).unwrap_or(false) {
        return;
    }

    // Check each tool's shortcut from the config
    for tool_config in TOOLBAR_TOOLS {
        if let Some(shortcut_char) = tool_config.shortcut {
            // Convert char to KeyCode
            let keycode = char_to_keycode(shortcut_char);

            if let Some(keycode) = keycode {
                if keyboard.just_pressed(keycode) {
                    // Map ToolBehavior to ToolId
                    if let Some(tool_id) = behavior_to_tool_id(&tool_config.behavior) {
                        // Don't re-activate the current tool
                        if !tool_state.is_active(tool_id) {
                            debug!("Keyboard shortcut '{}' pressed for tool: {}",
                                   shortcut_char, tool_config.name);

                            // Special handling for spacebar (Pan tool)
                            let temporary = shortcut_char == ' ';

                            switch_events.send(SwitchToolEvent {
                                tool: tool_id,
                                temporary,
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Convert a char to its corresponding KeyCode
fn char_to_keycode(c: char) -> Option<KeyCode> {
    match c.to_ascii_lowercase() {
        'a' => Some(KeyCode::KeyA),
        'b' => Some(KeyCode::KeyB),
        'c' => Some(KeyCode::KeyC),
        'd' => Some(KeyCode::KeyD),
        'e' => Some(KeyCode::KeyE),
        'f' => Some(KeyCode::KeyF),
        'g' => Some(KeyCode::KeyG),
        'h' => Some(KeyCode::KeyH),
        'i' => Some(KeyCode::KeyI),
        'j' => Some(KeyCode::KeyJ),
        'k' => Some(KeyCode::KeyK),
        'l' => Some(KeyCode::KeyL),
        'm' => Some(KeyCode::KeyM),
        'n' => Some(KeyCode::KeyN),
        'o' => Some(KeyCode::KeyO),
        'p' => Some(KeyCode::KeyP),
        'q' => Some(KeyCode::KeyQ),
        'r' => Some(KeyCode::KeyR),
        's' => Some(KeyCode::KeyS),
        't' => Some(KeyCode::KeyT),
        'u' => Some(KeyCode::KeyU),
        'v' => Some(KeyCode::KeyV),
        'w' => Some(KeyCode::KeyW),
        'x' => Some(KeyCode::KeyX),
        'y' => Some(KeyCode::KeyY),
        'z' => Some(KeyCode::KeyZ),
        ' ' => Some(KeyCode::Space),
        _ => None,
    }
}

/// Map ToolBehavior from config to ToolId
fn behavior_to_tool_id(behavior: &super::toolbar_config::ToolBehavior) -> Option<ToolId> {
    use super::toolbar_config::ToolBehavior;

    match behavior {
        ToolBehavior::Select => Some(ToolId::Select),
        ToolBehavior::Pan => Some(ToolId::Pan),
        ToolBehavior::Pen => Some(ToolId::Pen),
        ToolBehavior::Text => Some(ToolId::Text),
        ToolBehavior::Shapes => Some(ToolId::Shapes),
        ToolBehavior::Knife => Some(ToolId::Knife),
        ToolBehavior::Hyper => Some(ToolId::Hyper),
        ToolBehavior::Measure => Some(ToolId::Measure),
        ToolBehavior::Metaballs => Some(ToolId::Metaballs),
        ToolBehavior::Ai => Some(ToolId::Ai),
    }
}

/// Plugin to add keyboard shortcut handling
pub struct KeyboardShortcutPlugin;

impl Plugin for KeyboardShortcutPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_tool_keyboard_shortcuts
                .run_if(resource_exists::<ToolState>)
        );
    }
}