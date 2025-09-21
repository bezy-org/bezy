//! Select tool for font editing
//!
//! The select tool allows users to select and manipulate points, contours,
//! and other elements in the font editor.

use super::{EditTool, ToolInfo};
use bevy::prelude::*;

/// Resource to track if select mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct SelectModeActive(pub bool);

/// The select tool implementation
pub struct SelectTool;

impl EditTool for SelectTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "select",
            display_name: "Select",
            icon: "\u{E010}", // Select cursor icon
            tooltip: "Select and manipulate objects",
            shortcut: Some(KeyCode::KeyV),
        }
    }

    fn on_activate(&mut self, commands: &mut Commands) {
        commands.insert_resource(SelectModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Select);
        debug!("Select tool activated");
    }

    fn on_deactivate(&mut self, commands: &mut Commands) {
        commands.insert_resource(SelectModeActive(false));
        commands.insert_resource(crate::core::io::input::InputMode::Normal);
        debug!("Select tool deactivated");
    }
}

/// Plugin for the select tool
pub struct SelectToolPlugin;

impl Plugin for SelectToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectModeActive>();
    }
}
