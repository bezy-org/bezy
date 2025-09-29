//! Pan tool for navigating the design space
//!
//! The pan tool allows users to move around the design space by dragging.

use super::{EditTool, ToolInfo};
use bevy::prelude::*;

/// The pan tool implementation
pub struct PanTool;

impl EditTool for PanTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "pan",
            display_name: "Pan",
            icon: "\u{E014}", // Hand icon
            tooltip: "Navigate the design space",
            shortcut: Some(KeyCode::Space), // Spacebar for temporary pan
        }
    }

    fn on_activate(&mut self, commands: &mut Commands) {
        // Use Temporary mode for pan tool
        commands.insert_resource(crate::io::input::InputMode::Temporary);
        debug!("Pan tool activated");
    }

    fn on_deactivate(&mut self, commands: &mut Commands) {
        commands.insert_resource(crate::io::input::InputMode::Normal);
        debug!("Pan tool deactivated");
    }
}

/// Plugin for the pan tool
pub struct PanToolPlugin;

impl Plugin for PanToolPlugin {
    fn build(&self, _app: &mut App) {
        // Register any pan-specific systems here
    }
}
