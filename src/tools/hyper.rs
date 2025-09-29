//! Hyper tool for advanced curve drawing
//!
//! The hyper tool allows users to draw smooth hyperbezier curves
//! with automatic control point calculation.

use super::{EditTool, ToolInfo};
use bevy::prelude::*;

/// Resource to track if hyper mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct HyperModeActive(pub bool);

/// The hyper tool implementation
pub struct HyperTool;

impl EditTool for HyperTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "hyper",
            display_name: "Hyper",
            icon: "\u{E018}", // Hyper icon
            tooltip: "Draw smooth hyperbezier curves",
            shortcut: Some(KeyCode::KeyH),
        }
    }

    fn on_activate(&mut self, commands: &mut Commands) {
        commands.insert_resource(HyperModeActive(true));
        commands.insert_resource(crate::io::input::InputMode::Hyper);
        debug!("Hyper tool activated");
    }

    fn on_deactivate(&mut self, commands: &mut Commands) {
        commands.insert_resource(HyperModeActive(false));
        commands.insert_resource(crate::io::input::InputMode::Normal);
        debug!("Hyper tool deactivated");
    }
}

/// Plugin for the hyper tool
pub struct HyperToolPlugin;

impl Plugin for HyperToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HyperModeActive>();
    }
}
