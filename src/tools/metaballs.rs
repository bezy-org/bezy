//! Metaballs tool for organic shape creation
//!
//! The metaballs tool allows users to create smooth organic shapes
//! using metaball field effects.

use super::{EditTool, ToolInfo};
use bevy::prelude::*;

/// Resource to track if metaballs mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct MetaballsModeActive(pub bool);

/// The metaballs tool implementation
pub struct MetaballsTool;

impl EditTool for MetaballsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "metaballs",
            display_name: "Metaballs",
            icon: "\u{E019}", // Metaballs icon
            tooltip: "Create organic shapes with metaball effects",
            shortcut: Some(KeyCode::KeyM),
        }
    }

    fn on_activate(&mut self, commands: &mut Commands) {
        commands.insert_resource(MetaballsModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Metaballs);
        info!("Metaballs tool activated");
    }

    fn on_deactivate(&mut self, commands: &mut Commands) {
        commands.insert_resource(MetaballsModeActive(false));
        commands.insert_resource(crate::core::io::input::InputMode::Normal);
        info!("Metaballs tool deactivated");
    }
}

/// Plugin for the metaballs tool
pub struct MetaballsToolPlugin;

impl Plugin for MetaballsToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MetaballsModeActive>();
    }
}
