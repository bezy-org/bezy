//! commands.rs - Event-based command system for the font editor
//!
//! This file defines the application's command system using Bevy's event system:
//! 1. Event structs define different actions (file operations, glyph management)
//! 2. Handler functions process these events and update application state
//! 3. CommandsPlugin registers all events and connects handlers to the application
//!
//! To add new functionality, define a new event struct and corresponding handler,
//! then register both in the CommandsPlugin::build method.

#![allow(deprecated)]
#![allow(unused_mut)]

use crate::core::state::{AppState, FontIRAppState, GlyphNavigation};
use crate::editing::sort::{ActiveSort, Sort};
use crate::rendering::checkerboard::CheckerboardEnabled;
// BezyResult not used in current implementation
use bevy::prelude::*;
// Using String for glyph names in current norad version
use std::path::PathBuf;

#[derive(Event)]
pub struct OpenFileEvent {
    pub path: PathBuf,
}

#[derive(Event)]
pub struct SaveFileEvent;

#[derive(Event)]
pub struct SaveFileAsEvent {
    pub path: PathBuf,
}

#[derive(Event)]
pub struct NewGlyphEvent;

#[derive(Event)]
pub struct DeleteGlyphEvent {
    pub glyph_name: String,
}

#[derive(Event)]
pub struct RenameGlyphEvent {
    pub old_name: String,
    pub new_name: String,
}

#[derive(Event)]
pub struct OpenGlyphEditorEvent {
    pub glyph_name: String,
}

#[derive(Event, Debug)]
pub struct CycleCodepointEvent {
    pub direction: CodepointDirection,
}

#[derive(Debug)]
pub enum CodepointDirection {
    Next,
    Previous,
}

#[derive(Event)]
pub struct CreateContourEvent {
    #[allow(dead_code)]
    pub contour: norad::Contour,
}

pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        register_event_handlers(app);
    }
}

fn register_event_handlers(app: &mut App) {
    debug!("Registering command events, including CycleCodepointEvent");
    app.add_event::<OpenFileEvent>()
        .add_event::<SaveFileEvent>()
        .add_event::<SaveFileAsEvent>()
        .add_event::<NewGlyphEvent>()
        .add_event::<DeleteGlyphEvent>()
        .add_event::<RenameGlyphEvent>()
        .add_event::<OpenGlyphEditorEvent>()
        .add_event::<CycleCodepointEvent>()
        .add_event::<CreateContourEvent>()
        .add_systems(
            Update,
            (
                handle_open_file,
                handle_save_file,
                handle_save_file_as,
                handle_new_glyph,
                handle_delete_glyph,
                handle_rename_glyph,
                handle_open_glyph_editor,
                handle_cycle_codepoint,
                handle_create_contour,
                handle_codepoint_cycling,
                handle_save_shortcuts,
                handle_checkerboard_toggle,
            ),
        );
}

fn handle_open_file(
    mut events: EventReader<OpenFileEvent>,
    mut app_state: Option<ResMut<AppState>>,
) {
    for event in events.read() {
        if let Some(mut state) = app_state.as_mut() {
            match state.load_font_from_path(event.path.clone()) {
                Ok(_) => {
                    debug!("Successfully loaded font from {:?}", event.path);
                }
                Err(e) => {
                    error!("Failed to open file {:?}: {}", event.path, e);
                }
            }
        } else {
            warn!("Open file requested but AppState not available (using FontIR)");
        }
    }
}

fn handle_save_file(
    mut events: EventReader<SaveFileEvent>,
    mut app_state: Option<ResMut<AppState>>,
    mut fontir_app_state: Option<ResMut<FontIRAppState>>,
) {
    for _ in events.read() {
        if let Some(fontir_state) = fontir_app_state.as_mut() {
            match fontir_state.save_font() {
                Ok(_) => {
                    info!("Font saved successfully");
                }
                Err(e) => {
                    error!("FontIR save failed: {}", e);
                }
            }
        } else if let Some(mut state) = app_state.as_mut() {
            match state.save_font() {
                Ok(_) => {
                    debug!("Font saved successfully (legacy AppState)");
                }
                Err(e) => {
                    error!("Saving failed: {}", e);
                }
            }
        } else {
            warn!("Save file requested but no state available");
        }
    }
}

fn handle_save_file_as(
    mut events: EventReader<SaveFileAsEvent>,
    mut app_state: Option<ResMut<AppState>>,
) {
    for event in events.read() {
        if let Some(mut state) = app_state.as_mut() {
            match state.save_font_as(event.path.clone()) {
                Ok(_) => {
                    debug!("Font saved to {:?}", event.path);
                }
                Err(e) => {
                    error!("Failed to save file to {:?}: {}", event.path, e);
                }
            }
        } else {
            warn!("Save file as requested but AppState not available (using FontIR)");
        }
    }
}

fn handle_new_glyph(
    mut event_reader: EventReader<NewGlyphEvent>,
    _app_state: Option<ResMut<AppState>>,
) {
    for _event in event_reader.read() {
        debug!("New glyph creation requested");
        // Implementation here when needed
    }
}

fn handle_delete_glyph(
    mut event_reader: EventReader<DeleteGlyphEvent>,
    _app_state: Option<ResMut<AppState>>,
) {
    for event in event_reader.read() {
        debug!("Delete glyph requested for {:?}", event.glyph_name);
        // Implementation here when needed
    }
}

fn handle_rename_glyph(
    mut event_reader: EventReader<RenameGlyphEvent>,
    _app_state: Option<ResMut<AppState>>,
) {
    for event in event_reader.read() {
        debug!(
            "Rename glyph requested: {:?} -> {:?}",
            event.old_name, event.new_name
        );
        // Implementation here when needed
    }
}

fn handle_open_glyph_editor(
    mut event_reader: EventReader<OpenGlyphEditorEvent>,
    _app_state: Option<ResMut<AppState>>,
) {
    for event in event_reader.read() {
        debug!("Open glyph editor requested for {:?}", event.glyph_name);
        // Implementation here when needed
    }
}

fn handle_cycle_codepoint(
    mut event_reader: EventReader<CycleCodepointEvent>,
    mut glyph_navigation: ResMut<GlyphNavigation>,
    app_state: Option<Res<AppState>>,
    fontir_state: Option<Res<FontIRAppState>>,
    mut active_sorts: Query<&mut Sort, With<ActiveSort>>,
) {
    for event in event_reader.read() {
        debug!("Received codepoint cycling event: {:?}", event.direction);

        // Get available codepoints from either UFO or FontIR
        let available_codepoints = if let Some(state) = app_state.as_ref() {
            crate::core::state::get_all_codepoints(state)
        } else if let Some(fontir) = fontir_state.as_ref() {
            // For FontIR, get all glyph names (they are the codepoints in this system)
            fontir.get_glyph_names()
        } else {
            warn!("Codepoint cycling requested but neither AppState nor FontIRAppState available");
            return;
        };

        if available_codepoints.is_empty() {
            debug!("No codepoints found in font");
            return;
        }

        let current_codepoint = glyph_navigation.get_codepoint_string();
        debug!(
            "Current codepoint: '{}', available: {:?}",
            current_codepoint, available_codepoints
        );

        // Calculate next codepoint
        let current_index = available_codepoints
            .iter()
            .position(|cp| cp == &current_codepoint);

        let next_codepoint = if let Some(index) = current_index {
            match event.direction {
                CodepointDirection::Next => {
                    let next_index = (index + 1) % available_codepoints.len();
                    Some(available_codepoints[next_index].clone())
                }
                CodepointDirection::Previous => {
                    let prev_index = if index == 0 {
                        available_codepoints.len() - 1
                    } else {
                        index - 1
                    };
                    Some(available_codepoints[prev_index].clone())
                }
            }
        } else {
            // If current codepoint not found, start from first
            warn!(
                "Current codepoint '{}' not found in available list, using first available",
                current_codepoint
            );
            available_codepoints.first().cloned()
        };

        // Set the new codepoint if found
        if let Some(new_codepoint) = next_codepoint {
            glyph_navigation.set_codepoint(new_codepoint.clone());

            // Update all active sorts to use the new glyph
            for mut sort in active_sorts.iter_mut() {
                let old_glyph = sort.glyph_name.clone();
                sort.glyph_name = new_codepoint.clone();
                debug!(
                    "Updated active sort from glyph '{}' to '{}'",
                    old_glyph, new_codepoint
                );
            }

            debug!(
                "Switched to codepoint: {}",
                glyph_navigation.get_codepoint_string()
            );
        } else {
            debug!("No next/previous codepoint found");
        }
    }
}

/// System to handle keyboard shortcuts to cycle through codepoints
pub fn handle_codepoint_cycling(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cycle_event: EventWriter<CycleCodepointEvent>,
) {
    // Check for Home key to go to next codepoint (like in Glyphs.app)
    if keyboard.just_pressed(KeyCode::Home) {
        debug!("Detected Home key, cycling to next codepoint");
        cycle_event.write(CycleCodepointEvent {
            direction: CodepointDirection::Next,
        });
    }

    // Check for End key to go to previous codepoint (like in Glyphs.app)
    if keyboard.just_pressed(KeyCode::End) {
        debug!("Detected End key, cycling to previous codepoint");
        cycle_event.write(CycleCodepointEvent {
            direction: CodepointDirection::Previous,
        });
    }

    // Keep the existing Shift+Plus/Minus shortcuts as alternatives
    let shift_pressed =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if shift_pressed {
        // Check for Shift+= (Plus) to move to next codepoint
        if keyboard.just_pressed(KeyCode::Equal) {
            debug!("Detected Shift+= key combination, cycling to next codepoint");
            cycle_event.write(CycleCodepointEvent {
                direction: CodepointDirection::Next,
            });
        }

        // Check for Shift+- (Minus) to move to previous codepoint
        if keyboard.just_pressed(KeyCode::Minus) {
            debug!("Detected Shift+- key combination, cycling to previous codepoint");
            cycle_event.write(CycleCodepointEvent {
                direction: CodepointDirection::Previous,
            });
        }
    }
}

/// System to handle keyboard shortcuts for saving the font
///
/// This system watches for Command+S (macOS) or Ctrl+S (Windows/Linux)
/// and triggers a save operation when detected
pub fn handle_save_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut save_event: EventWriter<SaveFileEvent>,
) {
    // Check for Command (macOS) or Control (Windows/Linux)
    let modifier_pressed = keyboard.pressed(KeyCode::SuperLeft)
        || keyboard.pressed(KeyCode::SuperRight)
        || keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight);

    // If modifier is pressed and S is just pressed, trigger save
    if modifier_pressed && keyboard.just_pressed(KeyCode::KeyS) {
        debug!("Detected Command+S / Ctrl+S key combination, saving font");
        save_event.write(SaveFileEvent);
    }
}

/// Handler for adding a new contour to the current glyph
fn handle_create_contour(
    mut event_reader: EventReader<CreateContourEvent>,
    mut app_state: Option<ResMut<AppState>>,
    glyph_navigation: Option<Res<GlyphNavigation>>,
) {
    for _event in event_reader.read() {
        debug!("Handling CreateContourEvent");

        // Get the glyph name first
        if let (Some(state), Some(nav)) = (app_state.as_ref(), glyph_navigation.as_ref()) {
            if let Some(glyph_name) = nav.find_glyph(state) {
                // Try to add the contour to the glyph
                // Note: This will need to be implemented when we have the full glyph editing system
                debug!("Would add contour to glyph: {}", glyph_name);
                // TODO: Implement contour creation when glyph editing is ready
            } else {
                warn!("No current glyph selected for contour creation");
            }
        } else {
            warn!("Create contour requested but AppState/GlyphNavigation not available (using FontIR)");
        }
    }
}

/// System to handle keyboard shortcuts for toggling the checkerboard grid
///
/// This system watches for Command+G (macOS) or Ctrl+G (Windows/Linux)
/// and toggles the checkerboard visibility when detected
pub fn handle_checkerboard_toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut checkerboard_enabled: ResMut<CheckerboardEnabled>,
) {
    // Check for Command (macOS) or Control (Windows/Linux)
    let modifier_pressed = keyboard.pressed(KeyCode::SuperLeft)
        || keyboard.pressed(KeyCode::SuperRight)
        || keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight);

    // If modifier is pressed and G is just pressed, toggle checkerboard
    if modifier_pressed && keyboard.just_pressed(KeyCode::KeyG) {
        checkerboard_enabled.enabled = !checkerboard_enabled.enabled;
        let status = if checkerboard_enabled.enabled {
            "enabled"
        } else {
            "disabled"
        };
        debug!("Checkerboard grid {}", status);
        debug!(
            "Detected Command+G / Ctrl+G key combination, toggling checkerboard to: {}",
            status
        );
    }
}
