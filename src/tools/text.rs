//! Text tool for placing and editing text sorts
//!
//! The text tool allows users to place sorts by clicking in the design space.
//! Sorts can be placed and edited with different modes:
//! - Text mode: Sorts follow the gap buffer layout in a grid
//! - Insert mode: Keyboard-based editing within existing text mode sorts
//! - Freeform mode: Sorts are positioned freely in the design space

use super::{EditTool, ToolInfo};
use crate::core::state::{SortLayoutMode, TextModeConfig};
use bevy::prelude::*;
/// Resource to track if text mode is active
#[derive(Resource, Default)]
pub struct TextModeActive(pub bool);

/// Resource to track text mode state for sort placement
#[derive(Resource, Default)]
pub struct TextModeState {
    /// Current cursor position in design-space coordinates
    pub cursor_position: Option<Vec2>,
    /// Whether we're showing a sort placement preview
    pub showing_preview: bool,
}

/// Text placement modes for the submenu
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Resource)]
pub enum TextPlacementMode {
    /// Place sorts in left-to-right text mode
    #[default]
    LTRText,
    /// Place sorts in right-to-left text mode (Arabic/Hebrew)
    RTLText,
    /// Insert and edit text within existing text mode sorts  
    Insert,
    /// Place sorts freely in the design space
    Freeform,
}

impl TextPlacementMode {
    /// Get the icon for each placement mode
    pub fn get_icon(&self) -> &'static str {
        match self {
            TextPlacementMode::LTRText => "\u{E004}",
            TextPlacementMode::RTLText => "\u{E005}", // Use a different icon for RTL
            TextPlacementMode::Insert => "\u{F001}",
            TextPlacementMode::Freeform => "\u{E006}",
        }
    }

    /// Get a human-readable name for this placement mode
    pub fn display_name(&self) -> &'static str {
        match self {
            TextPlacementMode::LTRText => "LTR Text",
            TextPlacementMode::RTLText => "RTL Text",
            TextPlacementMode::Insert => "Insert",
            TextPlacementMode::Freeform => "Freeform",
        }
    }

    /// Convert to SortLayoutMode
    pub fn to_sort_layout_mode(&self) -> SortLayoutMode {
        match self {
            TextPlacementMode::LTRText => SortLayoutMode::LTRText,
            TextPlacementMode::RTLText => SortLayoutMode::RTLText,
            TextPlacementMode::Insert => SortLayoutMode::LTRText, // Default to LTR for insert mode
            TextPlacementMode::Freeform => SortLayoutMode::Freeform,
        }
    }

}

/// Component to mark text submenu buttons
#[derive(Component)]
pub struct TextSubMenuButton;

/// Component to associate a button with its placement mode
#[derive(Component)]
pub struct TextModeButton {
    pub mode: TextPlacementMode,
}

/// Resource to track the current text placement mode
#[derive(Resource, Default)]
pub struct CurrentTextPlacementMode(pub TextPlacementMode);

/// The text tool implementation
pub struct TextTool;

impl EditTool for TextTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "text",
            display_name: "Text",
            icon: "\u{E017}", // Text icon
            tooltip: "Place text and create sorts",
            shortcut: Some(KeyCode::KeyT),
        }
    }

    fn on_activate(&mut self, commands: &mut Commands) {
        commands.insert_resource(TextModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Text);

        info!("Text tool activated - Enhanced features:");
        info!("• Click to place sorts, type letters to add glyphs");
        info!("• 1-9 keys to switch glyphs, F1 for help");
        info!("• Arrow keys for navigation, Ctrl+S to show text mode");
    }

    fn on_deactivate(&mut self, commands: &mut Commands) {
        commands.insert_resource(TextModeActive(false));
        commands.insert_resource(crate::core::io::input::InputMode::Normal);

        info!("Text tool deactivated");
    }

    fn update(&self, _commands: &mut Commands) {
        // Text tool behavior is handled by dedicated systems
    }
}

/// Plugin for the text tool
pub struct TextToolPlugin;

impl Plugin for TextToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextModeActive>()
            .init_resource::<TextModeState>()
            .init_resource::<CurrentTextPlacementMode>()
            .init_resource::<TextModeConfig>();
    }
}
