//! # Select Tool
//!
//! The select tool allows users to select and manipulate points, contours,
//! and other elements in the font editor. Click to select individual points,
//! drag to create marquee selections, and use various keyboard modifiers
//! to modify selections.

use super::{EditTool, ToolInfo};
use bevy::input::keyboard::KeyCode;
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
        debug!("Entered Select tool");
    }

    fn on_deactivate(&mut self, commands: &mut Commands) {
        commands.insert_resource(SelectModeActive(false));
        commands.insert_resource(crate::core::io::input::InputMode::Normal);
        debug!("Exited Select tool");
    }
}

/// Plugin for the Select tool - handles all selection behavior and systems
pub struct SelectToolPlugin;

impl Plugin for SelectToolPlugin {
    fn build(&self, app: &mut App) {
        debug!("🔍 Registering SelectToolPlugin systems");
        app.init_resource::<SelectModeActive>()
            .add_systems(Startup, select_tool_startup_log)
            .add_systems(
                Update,
                (
                    handle_select_tool_activation,
                    reset_select_mode_when_inactive,
                    debug_select_tool_state,
                ),
            );
    }
}

/// Startup system to confirm select tool plugin loaded
fn select_tool_startup_log() {
    debug!("🔍 SelectToolPlugin successfully initialized");
}

/// System to handle select tool activation when CurrentTool changes to "select"
fn handle_select_tool_activation(
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
    select_mode: Res<SelectModeActive>,
) {
    if current_tool.is_changed() && current_tool.get_current() == Some("select") {
        // Activate select tool
        if !select_mode.0 {
            debug!("🔍 SELECT_DEBUG: Activating select tool via CurrentTool change");
            commands.insert_resource(SelectModeActive(true));
            commands.insert_resource(crate::core::io::input::InputMode::Select);
        }
    }
}

/// System to deactivate select mode when another tool is selected
fn reset_select_mode_when_inactive(
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
) {
    if current_tool.get_current() != Some("select") {
        // Mark select mode as inactive when not the current tool
        commands.insert_resource(SelectModeActive(false));
        debug!("[SELECT TOOL] Deactivating select mode - current tool: {:?}", current_tool.get_current());
    }
}

/// Debug system to track select tool state
fn debug_select_tool_state(
    select_mode: Res<SelectModeActive>,
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
) {
    if select_mode.is_changed() || current_tool.is_changed() {
        debug!(
            "🔍 SELECT_DEBUG: SelectModeActive = {}, CurrentTool = {:?}",
            select_mode.0,
            current_tool.get_current()
        );
    }
}