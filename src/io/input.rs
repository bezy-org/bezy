//! Centralized Input System for Bezy
//!
//! This module provides a unified input handling system that consolidates all
//! mouse, keyboard, and gamepad input processing. It ensures consistent
//! coordinate transformations and provides clear priority ordering for input
//! consumers.
//!
//! Key Features:
//! - Centralized input state management
//! - Consistent coordinate transformations using CursorInfo
//! - Clear input priority system
//! - Support for multiple input devices (mouse, keyboard, gamepad)
//! - Event-driven architecture for input consumers

use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::prelude::*;

use super::pointer::PointerInfo;
use crate::geometry::world_space::DPoint;
use bevy::input::mouse::MouseMotion;
use bevy::input::mouse::MouseWheel;

use crate::systems::ui_interaction::UiHoverState;
use bevy::input::gamepad::{
    GamepadAxis, GamepadAxisChangedEvent, GamepadButton, GamepadButtonChangedEvent,
    GamepadConnection,
};
use std::collections::HashMap;

/// Plugin for the centralized input system
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        debug!("[INPUT] Registering InputPlugin");

        app.init_resource::<InputState>()
            .init_resource::<InputPriority>()
            .add_event::<InputEvent>()
            .add_systems(PreUpdate, update_input_state)
            .add_systems(
                Update,
                (
                    process_input_events,
                    generate_mouse_drag_events,
                    clear_input_events,
                ),
            );

        debug!("[INPUT] InputPlugin registration complete");
    }
}

/// Centralized input state that tracks all input devices
#[derive(Resource, Default, Debug)]
pub struct InputState {
    /// Mouse state
    pub mouse: MouseState,
    /// Keyboard state
    pub keyboard: KeyboardState,
    /// Gamepad state (future expansion)
    pub gamepad: GamepadState,
    /// Current input mode (affects how input is processed)
    pub mode: InputMode,
    /// Whether input is currently being consumed by UI
    pub ui_consuming: bool,
}

/// Mouse input state
#[derive(Debug)]
pub struct MouseState {
    /// Current position in screen coordinates
    pub screen_position: Option<Vec2>,
    /// Current position in design space coordinates
    pub design_position: Option<DPoint>,
    /// Mouse wheel delta
    pub wheel: Vec2,
    /// Mouse movement delta
    pub motion: Vec2,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            screen_position: None,
            design_position: None,
            wheel: Vec2::ZERO,
            motion: Vec2::ZERO,
        }
    }
}

/// Keyboard input state
#[derive(Debug, Default)]
pub struct KeyboardState {
    /// Modifier key states
    pub modifiers: ModifierState,
    /// Text input buffer (for text editing)
    pub text_buffer: String,
}

/// Gamepad input state (placeholder for future expansion)
#[derive(Debug)]
pub struct GamepadState {
    /// Gamepad button states
    pub buttons: HashMap<(Gamepad, GamepadButton), f32>,
    /// Analog stick positions
    pub sticks: Vec2,
    /// Trigger values
    pub triggers: Vec2,
    /// Connected gamepads
    pub connected_gamepads: HashMap<Gamepad, GamepadConnection>,
    /// Axes
    pub axes: HashMap<(Gamepad, GamepadAxis), f32>,
}

impl Default for GamepadState {
    fn default() -> Self {
        Self {
            buttons: HashMap::new(),
            sticks: Vec2::ZERO,
            triggers: Vec2::ZERO,
            connected_gamepads: HashMap::new(),
            axes: HashMap::new(),
        }
    }
}

/// Generic button state tracking
#[derive(Debug)]
pub struct ButtonState<T: Copy + Eq + std::hash::Hash> {
    /// Currently pressed buttons
    pub pressed: std::collections::HashSet<T>,
    /// Buttons that were just pressed this frame
    pub just_pressed: std::collections::HashSet<T>,
    /// Buttons that were just released this frame
    pub just_released: std::collections::HashSet<T>,
}

impl<T: Copy + Eq + std::hash::Hash> Default for ButtonState<T> {
    fn default() -> Self {
        Self {
            pressed: std::collections::HashSet::new(),
            just_pressed: std::collections::HashSet::new(),
            just_released: std::collections::HashSet::new(),
        }
    }
}

/// Modifier key state
#[derive(Debug, Default, Clone)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

/// Current input processing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource, Default)]
pub enum InputMode {
    /// Normal editing mode
    #[default]
    Normal,
    /// Text editing mode
    Text,
    /// Selection mode
    Select,
    /// Pen tool mode
    Pen,
    /// Knife tool mode
    Knife,
    /// Shape tool mode
    Shape,
    /// Metaballs tool mode
    Metaballs,
    /// Metaball tool mode (alternative name)
    Metaball,
    /// Hyper tool mode
    Hyper,
    /// Pan tool mode
    Pan,
    /// Measure tool mode
    Measure,
    /// Temporary mode
    Temporary,
}

/// Input priority levels for determining which system handles input
#[derive(Resource, Debug)]
pub struct InputPriority {
    /// Systems that handle input at the highest priority (UI, modals, etc.)
    pub high_priority: Vec<String>,
    /// Systems that handle input at normal priority (tools, editing)
    pub normal_priority: Vec<String>,
    /// Systems that handle input at low priority (fallbacks, defaults)
    pub low_priority: Vec<String>,
}

impl Default for InputPriority {
    fn default() -> Self {
        Self {
            high_priority: vec![
                "ui_interaction".to_string(),
                "modal_dialog".to_string(),
                "text_editor".to_string(),
            ],
            normal_priority: vec![
                "selection".to_string(),
                "pen_tool".to_string(),
                "knife_tool".to_string(),
                "shape_tool".to_string(),
                "hyper_tool".to_string(),
                "sort_interaction".to_string(),
            ],
            low_priority: vec!["camera_control".to_string(), "default_actions".to_string()],
        }
    }
}

/// Input events that can be consumed by systems
#[derive(Event, Debug, Clone)]
pub enum InputEvent {
    /// Mouse click event
    MouseClick {
        button: MouseButton,
        position: DPoint,
        modifiers: ModifierState,
    },
    /// Mouse release event
    MouseRelease {
        button: MouseButton,
        position: DPoint,
        modifiers: ModifierState,
    },
    /// Mouse drag event
    MouseDrag {
        button: MouseButton,
        start_position: DPoint,
        current_position: DPoint,
        delta: Vec2,
        modifiers: ModifierState,
    },
    /// Mouse move event
    MouseMove { position: DPoint, delta: Vec2 },
    /// Mouse wheel event
    MouseWheel { delta: Vec2 },
    /// Key press event
    KeyPress {
        key: KeyCode,
        modifiers: ModifierState,
    },
    /// Key release event
    KeyRelease {
        key: KeyCode,
        modifiers: ModifierState,
    },
    /// Text input event
    TextInput { text: String },
    // Gamepad events removed for now due to Clone issues
}

/// System to update the centralized input state
#[allow(clippy::too_many_arguments)]
fn update_input_state(
    mut _input_state: ResMut<InputState>,
    pointer_info: Res<PointerInfo>,
    _mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    input_mode_resource: Option<Res<InputMode>>,

    _gamepad_axis_events: EventReader<GamepadAxisChangedEvent>,
    _gamepad_button_events: EventReader<GamepadButtonChangedEvent>,
    ui_hover_state: Res<UiHoverState>,
) {
    debug!("[INPUT] update_input_state called");

    // Update input mode from resource (CRITICAL: This was missing!)
    if let Some(input_mode) = input_mode_resource {
        if _input_state.mode != *input_mode {
            _input_state.mode = *input_mode;
        }
    }

    // Update mouse state
    update_mouse_state(
        &mut _input_state.mouse,
        &pointer_info,
        &_mouse_button_input,
        &mut mouse_motion,
        &mut mouse_wheel,
    );

    // Update keyboard state
    update_keyboard_state(&mut _input_state.keyboard, &keyboard_input);

    // Update gamepad state
    update_gamepad_state(&mut _input_state.gamepad);

    // Update UI consumption state
    _input_state.ui_consuming = ui_hover_state.is_hovering_ui;
}

/// Update mouse state from Bevy input resources
fn update_mouse_state(
    mouse_state: &mut MouseState,
    pointer_info: &PointerInfo,
    _mouse_button_input: &ButtonInput<MouseButton>,
    mouse_motion: &mut EventReader<MouseMotion>,
    mouse_wheel: &mut EventReader<MouseWheel>,
) {
    // Update position from cursor resource
    mouse_state.screen_position = Some(pointer_info.screen);
    mouse_state.design_position = Some(pointer_info.design);

    // Update button states - ButtonInput doesn't implement Clone, so we need to handle this differently
    // For now, we'll use the original ButtonInput resource directly

    // Update motion
    mouse_state.motion = Vec2::ZERO;
    for motion in mouse_motion.read() {
        mouse_state.motion += motion.delta;
    }

    // Update wheel
    mouse_state.wheel = Vec2::ZERO;
    for wheel in mouse_wheel.read() {
        mouse_state.wheel += wheel.y * Vec2::Y;
    }
}

/// Update keyboard state from Bevy input resources
fn update_keyboard_state(
    keyboard_state: &mut KeyboardState,
    keyboard_input: &ButtonInput<KeyCode>,
) {
    // Update key states - ButtonInput doesn't implement Clone, so we need to handle this differently
    // For now, we'll use the original ButtonInput resource directly

    // Clear text buffer - we now use Bevy's native TextInputEvent system
    keyboard_state.text_buffer.clear();

    // Update modifier states
    keyboard_state.modifiers.shift =
        keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);
    keyboard_state.modifiers.ctrl = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight);
    keyboard_state.modifiers.alt =
        keyboard_input.pressed(KeyCode::AltLeft) || keyboard_input.pressed(KeyCode::AltRight);
    keyboard_state.modifiers.super_key =
        keyboard_input.pressed(KeyCode::SuperLeft) || keyboard_input.pressed(KeyCode::SuperRight);
}

/// Update gamepad state (placeholder for future implementation)
fn update_gamepad_state(_gamepad_state: &mut GamepadState) {
    // TODO: Implement gamepad support
}

/// System to process input events and send them to consumers
pub fn process_input_events(
    input_state: Res<InputState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut input_events: EventWriter<InputEvent>,
) {
    debug!("[INPUT] Processing input events");
    // Process mouse events
    process_mouse_events(&input_state, &mut input_events);
    // Process keyboard events
    process_keyboard_events(&keyboard_input, &input_state, &mut input_events);
    // Process gamepad events
    process_gamepad_events(&input_state, &mut input_events);
    // Log all events generated this frame
    // (We can't read EventWriter, so add logs in process_keyboard_events)
}

/// Process mouse events and create InputEvent instances
fn process_mouse_events(input_state: &InputState, input_events: &mut EventWriter<InputEvent>) {
    let mouse = &input_state.mouse;
    let _modifiers = &input_state.keyboard.modifiers;

    // Note: Mouse click/release events are now handled by the original ButtonInput resources
    // This function is simplified since we removed the stored ButtonInput from InputState

    // Mouse move events
    if mouse.motion != Vec2::ZERO {
        if let Some(position) = mouse.design_position {
            input_events.write(InputEvent::MouseMove {
                position,
                delta: mouse.motion,
            });
        }
    }

    // Mouse wheel events
    if mouse.wheel != Vec2::ZERO {
        input_events.write(InputEvent::MouseWheel { delta: mouse.wheel });
    }
}

/// System to generate MouseClick, MouseDrag, and MouseRelease events from mouse button state and motion
fn generate_mouse_drag_events(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    input_state: Res<InputState>,
    mut input_events: EventWriter<InputEvent>,
    mut drag_state: Local<Option<(MouseButton, DPoint)>>,
) {
    let modifiers = &input_state.keyboard.modifiers;

    if let Some(position) = input_state.mouse.design_position {
        // Check for just pressed buttons (MouseClick events)
        for button in mouse_button_input.get_just_pressed() {
            debug!(
                "Mouse button just pressed: {:?} at position {:?}",
                button, position
            );
            debug!(
                "Mouse position details: screen={:?}, design={:?}",
                input_state.mouse.screen_position, input_state.mouse.design_position
            );
            *drag_state = Some((*button, position));
            input_events.write(InputEvent::MouseClick {
                button: *button,
                position,
                modifiers: modifiers.clone(),
            });
        }

        // Check for just released buttons (MouseRelease events)
        for button in mouse_button_input.get_just_released() {
            debug!(
                "Mouse button just released: {:?} at position {:?}",
                button, position
            );
            input_events.write(InputEvent::MouseRelease {
                button: *button,
                position,
                modifiers: modifiers.clone(),
            });
            // Clear drag state if this was the button we were dragging
            if let Some((drag_button, _)) = *drag_state {
                if drag_button == *button {
                    *drag_state = None;
                }
            }
        }

        // Check for ongoing drag (MouseDrag events)
        if let Some((drag_button, start_pos)) = *drag_state {
            if mouse_button_input.pressed(drag_button) && input_state.mouse.motion != Vec2::ZERO {
                input_events.write(InputEvent::MouseDrag {
                    button: drag_button,
                    start_position: start_pos,
                    current_position: position,
                    delta: input_state.mouse.motion,
                    modifiers: modifiers.clone(),
                });
            }
        }
    }
}

/// Process keyboard events and create InputEvent instances
fn process_keyboard_events(
    keyboard_input: &ButtonInput<KeyCode>,
    input_state: &InputState,
    input_events: &mut EventWriter<InputEvent>,
) {
    let modifiers = &input_state.keyboard.modifiers;
    for key in keyboard_input.get_just_pressed() {
        debug!("[INPUT] Generating KeyPress event for key: {:?}", key);
        input_events.write(InputEvent::KeyPress {
            key: *key,
            modifiers: modifiers.clone(),
        });
    }
    for key in keyboard_input.get_just_released() {
        debug!("[INPUT] Generating KeyRelease event for key: {:?}", key);
        input_events.write(InputEvent::KeyRelease {
            key: *key,
            modifiers: modifiers.clone(),
        });
    }
}

/// Process gamepad events and create InputEvent instances
fn process_gamepad_events(_input_state: &InputState, _input_events: &mut EventWriter<InputEvent>) {
    // TODO: Implement gamepad event processing
}

/// System to clear input events at the end of each frame
fn clear_input_events(_input_events: EventWriter<InputEvent>) {
    // Events are automatically cleared by Bevy at the end of each frame
}

/// Helper trait for input consumers to check if they should handle input
pub trait InputConsumer {
    /// Check if this consumer should handle the given input event
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool;

    /// Handle the input event
    fn handle_input(&mut self, event: &InputEvent, input_state: &InputState);
}

/// Helper functions for common input checks
pub mod helpers {
    use super::*;

    // Note: These functions now need to be called with the original ButtonInput resources
    // since we removed the stored ButtonInput from InputState to avoid Clone issues

    /// Check if a mouse button is currently pressed
    pub fn is_mouse_pressed(_input_state: &InputState, _button: MouseButton) -> bool {
        // This would need to be called with the actual ButtonInput<MouseButton> resource
        false // Placeholder
    }

    /// Check if a mouse button was just pressed
    pub fn is_mouse_just_pressed(_input_state: &InputState, _button: MouseButton) -> bool {
        // This would need to be called with the actual ButtonInput<MouseButton> resource
        false // Placeholder
    }

    /// Check if a mouse button was just released
    pub fn is_mouse_just_released(_input_state: &InputState, _button: MouseButton) -> bool {
        // This would need to be called with the actual ButtonInput<MouseButton> resource
        false // Placeholder
    }

    /// Check if a key is currently pressed
    pub fn is_key_pressed(_input_state: &InputState, _key: KeyCode) -> bool {
        // This would need to be called with the actual ButtonInput<KeyCode> resource
        false // Placeholder
    }

    /// Check if a key was just pressed
    pub fn is_key_just_pressed(_input_state: &InputState, _key: KeyCode) -> bool {
        // This would need to be called with the actual ButtonInput<KeyCode> resource
        false // Placeholder
    }

    /// Check if a key was just released
    pub fn is_key_just_released(_input_state: &InputState, _key: KeyCode) -> bool {
        // This would need to be called with the actual ButtonInput<KeyCode> resource
        false // Placeholder
    }

    /// Check if any modifier key is pressed
    pub fn has_modifier(input_state: &InputState) -> bool {
        let mods = &input_state.keyboard.modifiers;
        mods.shift || mods.ctrl || mods.alt || mods.super_key
    }

    /// Check if shift is pressed
    pub fn is_shift_pressed(input_state: &InputState) -> bool {
        input_state.keyboard.modifiers.shift
    }

    /// Check if ctrl is pressed
    pub fn is_ctrl_pressed(input_state: &InputState) -> bool {
        input_state.keyboard.modifiers.ctrl
    }

    /// Check if alt is pressed
    pub fn is_alt_pressed(input_state: &InputState) -> bool {
        input_state.keyboard.modifiers.alt
    }

    /// Get the current mouse position in design space
    pub fn get_mouse_design_position(input_state: &InputState) -> Option<DPoint> {
        input_state.mouse.design_position
    }

    /// Get the current mouse position in screen space
    pub fn get_mouse_screen_position(input_state: &InputState) -> Option<Vec2> {
        input_state.mouse.screen_position
    }

    /// Check if input is being consumed by UI
    pub fn is_ui_consuming(input_state: &InputState) -> bool {
        input_state.ui_consuming
    }

    /// Check if the current input mode matches the given mode
    pub fn is_input_mode(input_state: &InputState, mode: InputMode) -> bool {
        input_state.mode == mode
    }
}
