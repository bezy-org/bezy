//! Measure tool for measuring distances and angles
//!
//! The measure tool allows users to measure distances between points
//! and angles between segments.

use super::{EditTool, ToolInfo};
use bevy::prelude::*;

/// The measure tool implementation
pub struct MeasureTool;

impl EditTool for MeasureTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "measure",
            display_name: "Measure",
            icon: "\u{E015}", // Ruler icon
            tooltip: "Measure distances and angles",
            shortcut: Some(KeyCode::KeyM),
        }
    }

    fn on_activate(&mut self, commands: &mut Commands) {
        commands.insert_resource(crate::core::io::input::InputMode::Temporary);
        debug!("Measure tool activated");
    }

    fn on_deactivate(&mut self, commands: &mut Commands) {
        commands.insert_resource(crate::core::io::input::InputMode::Normal);
        debug!("Measure tool deactivated");
    }
}

/// Plugin for the measure tool
pub struct MeasureToolPlugin;

impl Plugin for MeasureToolPlugin {
    fn build(&self, _app: &mut App) {
        // Register any measure-specific systems here
    }
}
