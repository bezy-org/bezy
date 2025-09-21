//! Knife tool for cutting contours
//!
//! The knife tool allows users to cut existing contours at specific points.

use super::{EditTool, ToolInfo};
use bevy::prelude::*;

/// Resource to track if knife mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct KnifeModeActive(pub bool);

/// The knife tool implementation
pub struct KnifeTool;

impl EditTool for KnifeTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "knife",
            display_name: "Knife",
            icon: "\u{E012}", // Knife icon
            tooltip: "Cut contours at specific points",
            shortcut: Some(KeyCode::KeyK),
        }
    }

    fn on_activate(&mut self, commands: &mut Commands) {
        commands.insert_resource(KnifeModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Knife);
        debug!("Knife tool activated");
    }

    fn on_deactivate(&mut self, commands: &mut Commands) {
        commands.insert_resource(KnifeModeActive(false));
        commands.insert_resource(crate::core::io::input::InputMode::Normal);
        debug!("Knife tool deactivated");
    }
}

/// Plugin for the knife tool
pub struct KnifeToolPlugin;

impl Plugin for KnifeToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KnifeModeActive>();
    }
}
