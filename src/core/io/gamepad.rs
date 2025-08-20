//! Gamepad Input Support Module
//!
//! This module provides basic gamepad support for Bezy.
//! It manages gamepad connections and input state.

use bevy::input::gamepad::GamepadConnectionEvent;
use bevy::prelude::*;

/// Information about a connected gamepad
#[derive(Debug, Clone)]
pub struct GamepadInfo {
    pub entity: Entity,
    pub name: String,
}

/// Resource tracking gamepad state
#[derive(Resource, Debug, Default)]
pub struct GamepadManager {
    pub active_gamepad: Option<GamepadInfo>,
    pub left_stick: Vec2,
    pub right_stick: Vec2,
    pub left_trigger: f32,
    pub right_trigger: f32,
}

/// Plugin for gamepad functionality
pub struct GamepadPlugin;

impl Plugin for GamepadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GamepadManager>().add_systems(
            Update,
            (gamepad_connection_system, gamepad_input_system),
        );
    }
}

/// System to handle gamepad connections/disconnections
fn gamepad_connection_system(
    mut gamepad_manager: ResMut<GamepadManager>,
    mut connection_events: EventReader<GamepadConnectionEvent>,
) {
    for event in connection_events.read() {
        info!("Gamepad connection event: {:?}", event);

        // For now, just track the first connected gamepad
        if gamepad_manager.active_gamepad.is_none() {
            gamepad_manager.active_gamepad = Some(GamepadInfo {
                entity: event.gamepad,
                name: "Gamepad".to_string(),
            });
        }
    }
}

/// System to update gamepad input state
fn gamepad_input_system(
    gamepad_manager: Res<GamepadManager>,
    gamepads: Query<&Gamepad>,
) {
    if let Some(info) = &gamepad_manager.active_gamepad {
        if let Ok(_gamepad) = gamepads.get(info.entity) {
            // Placeholder for reading gamepad input
            // In a full implementation, we would read axis values and button states here
            debug!("Gamepad active: {:?}", info.entity);
        }
    }
}

/// Check if a gamepad is connected
pub fn is_gamepad_connected(gamepad_manager: &GamepadManager) -> bool {
    gamepad_manager.active_gamepad.is_some()
}

/// Get movement input from the left stick
pub fn get_gamepad_movement(gamepad_manager: &GamepadManager) -> Vec2 {
    if gamepad_manager.left_stick.length() > 0.2 {
        gamepad_manager.left_stick
    } else {
        Vec2::ZERO
    }
}

/// Get camera input from the right stick  
pub fn get_gamepad_camera(gamepad_manager: &GamepadManager) -> Vec2 {
    if gamepad_manager.right_stick.length() > 0.2 {
        gamepad_manager.right_stick
    } else {
        Vec2::ZERO
    }
}
