pub mod gamepad;
pub mod input;
pub mod pointer;

// Explicit re-exports for public API
// Gamepad functionality
pub use gamepad::{GamepadInfo, GamepadManager, GamepadPlugin, get_gamepad_camera, get_gamepad_movement, is_gamepad_connected};
// Input system
pub use input::{
    ButtonState, InputConsumer, InputEvent, InputMode, InputPlugin, InputPriority, InputState,
    KeyboardState, ModifierState, MouseState, GamepadState, process_input_events,
    helpers, // Helper functions submodule
};
// Pointer/mouse tracking
pub use pointer::{PointerInfo, PointerPlugin};
