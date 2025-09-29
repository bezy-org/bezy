//! Shapes tool for creating geometric primitives
//!
//! The shapes tool allows users to create common geometric shapes
//! like rectangles, circles, and polygons.

use super::{EditTool, ToolInfo};
use bevy::prelude::*;

/// The shapes tool implementation
pub struct ShapesTool;

impl EditTool for ShapesTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "shapes",
            display_name: "Shapes",
            icon: "\u{E016}", // Shapes icon
            tooltip: "Create geometric shapes",
            shortcut: Some(KeyCode::KeyS),
        }
    }

    fn on_activate(&mut self, commands: &mut Commands) {
        commands.insert_resource(crate::io::input::InputMode::Shape);
        debug!("Shapes tool activated");
    }

    fn on_deactivate(&mut self, commands: &mut Commands) {
        commands.insert_resource(crate::io::input::InputMode::Normal);
        debug!("Shapes tool deactivated");
    }
}

/// Plugin for the shapes tool
pub struct ShapesToolPlugin;

impl Plugin for ShapesToolPlugin {
    fn build(&self, _app: &mut App) {
        // Register any shapes-specific systems here
    }
}
