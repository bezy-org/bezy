//! Text editor component for the Bezy font editor
//!
//! This module provides a simple text input component that can be used
//! for various text editing needs within the application.

use bevy::prelude::*;

/// Component for the text editor
#[allow(dead_code)]
#[derive(Component, Default)]
pub struct TextEditor {
    text: String,
    cursor_position: usize,
}

impl TextEditor {
    /// Get the current text content
    #[allow(dead_code)]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the text content
    #[allow(dead_code)]
    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.cursor_position = self.text.len();
    }

    /// Get the cursor position
    #[allow(dead_code)]
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }
}

/// System to spawn the text editor
#[allow(dead_code)]
pub fn spawn_text_editor(mut commands: Commands) {
    commands.spawn((
        TextEditor::default(),
        Node::default(),
        BackgroundColor(Color::srgb(0.9, 0.9, 0.9)),
    ));
}

/// System to handle text input
pub fn handle_text_input(
    mut text_editor: Query<&mut TextEditor>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut editor) = text_editor.single_mut() else {
        return;
    };
    let cursor_pos = editor.cursor_position;

    // Handle backspace
    if keyboard.just_pressed(KeyCode::Backspace) {
        if cursor_pos > 0 {
            editor.text.remove(cursor_pos - 1);
            editor.cursor_position -= 1;
        }
        return;
    }

    // Handle space
    if keyboard.just_pressed(KeyCode::Space) {
        editor.text.insert(cursor_pos, ' ');
        editor.cursor_position += 1;
        return;
    }

    // Handle letters
    let mut input_char = None;
    for key in keyboard.get_just_pressed() {
        input_char = match key {
            KeyCode::KeyA => Some('a'),
            KeyCode::KeyB => Some('b'),
            KeyCode::KeyC => Some('c'),
            KeyCode::KeyD => Some('d'),
            KeyCode::KeyE => Some('e'),
            KeyCode::KeyF => Some('f'),
            KeyCode::KeyG => Some('g'),
            KeyCode::KeyH => Some('h'),
            KeyCode::KeyI => Some('i'),
            KeyCode::KeyJ => Some('j'),
            KeyCode::KeyK => Some('k'),
            KeyCode::KeyL => Some('l'),
            KeyCode::KeyM => Some('m'),
            KeyCode::KeyN => Some('n'),
            KeyCode::KeyO => Some('o'),
            KeyCode::KeyP => Some('p'),
            KeyCode::KeyQ => Some('q'),
            KeyCode::KeyR => Some('r'),
            KeyCode::KeyS => Some('s'),
            KeyCode::KeyT => Some('t'),
            KeyCode::KeyU => Some('u'),
            KeyCode::KeyV => Some('v'),
            KeyCode::KeyW => Some('w'),
            KeyCode::KeyX => Some('x'),
            KeyCode::KeyY => Some('y'),
            KeyCode::KeyZ => Some('z'),
            _ => None,
        };
        if input_char.is_some() {
            break;
        }
    }

    if let Some(c) = input_char {
        editor.text.insert(cursor_pos, c);
        editor.cursor_position += 1;
    }
}

/// System to update text display
pub fn update_text_display(
    text_editor: Query<(Entity, &TextEditor), Changed<TextEditor>>,
) {
    for (_entity, editor) in text_editor.iter() {
        info!("Text: {}", editor.text);
    }
}

/// Plugin to set up the text editor
pub struct TextEditorPlugin;

impl Plugin for TextEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_text_editor)
            .add_systems(Update, (handle_text_input, update_text_display));
    }
}
