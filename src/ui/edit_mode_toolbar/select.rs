//! Select tool for the edit mode toolbar
//!
//! This tool provides selection and manipulation functionality for objects in the design space.
//! It's typically the default tool and allows users to select, move, and modify existing elements.

use crate::ui::edit_mode_toolbar::{EditModeSystem, EditTool, ToolRegistry};
use bevy::prelude::*;

/// Plugin to register selection mode systems
pub struct SelectModePlugin;

impl Plugin for SelectModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectModeActive>();
    }
}

/// Resource to track if select mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct SelectModeActive(pub bool);

pub struct SelectTool;

impl EditTool for SelectTool {
    fn id(&self) -> crate::ui::edit_mode_toolbar::ToolId {
        "select"
    }

    fn name(&self) -> &'static str {
        "Select"
    }

    fn icon(&self) -> &'static str {
        "\u{E010}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('v')
    }

    fn default_order(&self) -> i32 {
        10 // First tool in the toolbar
    }

    fn description(&self) -> &'static str {
        "Select and manipulate objects"
    }

    fn update(&self, commands: &mut Commands) {
        // Mark select mode as active while this tool is current
        commands.insert_resource(SelectModeActive(true));
        // Set the input mode to Select for the centralized input system
        commands.insert_resource(crate::core::io::input::InputMode::Select);
        info!("[SELECT TOOL] Update called - setting select mode active and input mode to Select");
    }

    fn on_enter(&self) {
        info!("Entered Select tool");
    }

    fn on_exit(&self) {
        info!("Exited Select tool");
    }
}

// Legacy compatibility struct
pub struct SelectMode;

impl EditModeSystem for SelectMode {
    fn update(&self, _commands: &mut Commands) {
        // Legacy implementation
    }
}

/// Plugin for the Select tool
pub struct SelectToolPlugin;

impl Plugin for SelectToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectModeActive>()
            .add_systems(Startup, register_select_tool)
            .add_systems(Update, reset_select_mode_when_inactive)
            .add_systems(Update, ensure_select_mode_active);
    }
}

fn register_select_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(SelectTool));
}

/// System to deactivate select mode when another tool is selected
pub fn reset_select_mode_when_inactive(
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
) {
    if current_tool.get_current() != Some("select") {
        // Mark select mode as inactive when not the current tool
        commands.insert_resource(SelectModeActive(false));
        // Reset input mode to Normal when not in select mode
        commands.insert_resource(crate::core::io::input::InputMode::Normal);
        debug!(
            "[SELECT MODE] Deactivated - current tool: {:?}",
            current_tool.get_current()
        );
    }
}

/// System to ensure select mode is active by default
pub fn ensure_select_mode_active(
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
) {
    // If no tool is currently selected, default to select
    if current_tool.get_current().is_none() {
        commands.insert_resource(SelectModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Select);
        info!("No tool selected, defaulting to select mode");
    } else {
        debug!(
            "[SELECT MODE] Current tool: {:?}",
            current_tool.get_current()
        );
    }
}
