//! Configuration-Based Tool Loading System
//!
//! This module automatically creates toolbar tools from the centralized configuration
//! in toolbar_config.rs. No need to manually register tools anywhere else!

use super::toolbar_config::{ToolBehavior, ToolConfig};
use super::{EditTool, ToolId, ToolRegistry};
use bevy::prelude::*;

/// Universal tool that adapts its behavior based on configuration
pub struct ConfigurableTool {
    config: &'static ToolConfig,
}

impl ConfigurableTool {
    pub fn new(config: &'static ToolConfig) -> Self {
        Self { config }
    }
}

impl EditTool for ConfigurableTool {
    fn id(&self) -> ToolId {
        self.config.id
    }

    fn name(&self) -> &'static str {
        self.config.name
    }

    fn icon(&self) -> &'static str {
        self.config.icon
    }

    fn shortcut_key(&self) -> Option<char> {
        self.config.shortcut
    }

    fn default_order(&self) -> i32 {
        self.config.order
    }

    fn description(&self) -> &'static str {
        self.config.description
    }

    fn update(&self, commands: &mut Commands) {
        use crate::core::io::input::InputMode;

        // Delegate to the appropriate behavior based on config
        match self.config.behavior {
            ToolBehavior::Select => {
                // Set input mode for select tool
                commands.insert_resource(InputMode::Select);
                // Deactivate pen mode when switching to other tools
                commands.insert_resource(crate::tools::pen::PenModeActive(false));
            }
            ToolBehavior::Pan => {
                // Set input mode for pan tool
                commands.insert_resource(InputMode::Pan);
                // Deactivate pen mode when switching to other tools
                commands.insert_resource(crate::tools::pen::PenModeActive(false));
            }
            ToolBehavior::Pen => {
                // Set input mode for pen tool
                commands.insert_resource(InputMode::Pen);
                // Also set PenModeActive for compatibility with pen tool systems
                commands.insert_resource(crate::tools::pen::PenModeActive(true));
                println!(
                    "🖊️ PEN_DEBUG: Pen tool activated - InputMode::Pen and PenModeActive(true) set"
                );
            }
            ToolBehavior::Text => {
                // Set input mode for text tool
                commands.insert_resource(InputMode::Text);
                // Deactivate pen mode when switching to other tools
                commands.insert_resource(crate::tools::pen::PenModeActive(false));
            }
            ToolBehavior::Shapes => {
                // Set input mode for shapes tool
                commands.insert_resource(InputMode::Shape);
                // Deactivate pen mode when switching to other tools
                commands.insert_resource(crate::tools::pen::PenModeActive(false));
            }
            ToolBehavior::Knife => {
                // Set input mode for knife tool
                commands.insert_resource(InputMode::Knife);
                // Deactivate pen mode when switching to other tools
                commands.insert_resource(crate::tools::pen::PenModeActive(false));
            }
            ToolBehavior::Hyper => {
                // Set input mode for hyper tool
                commands.insert_resource(InputMode::Hyper);
                // Deactivate pen mode when switching to other tools
                commands.insert_resource(crate::tools::pen::PenModeActive(false));
            }
            ToolBehavior::Measure => {
                // Set input mode for measure tool
                commands.insert_resource(InputMode::Measure);
                // Deactivate pen mode when switching to other tools
                commands.insert_resource(crate::tools::pen::PenModeActive(false));
            }
            ToolBehavior::Metaballs => {
                // Set input mode for metaballs tool
                commands.insert_resource(InputMode::Metaball);
                // Deactivate pen mode when switching to other tools
                commands.insert_resource(crate::tools::pen::PenModeActive(false));
            }
            ToolBehavior::Ai => {
                // Set input mode for AI tool
                commands.insert_resource(InputMode::Normal);
                // Deactivate pen mode when switching to other tools
                commands.insert_resource(crate::tools::pen::PenModeActive(false));
                // Activate AI mode
                commands.insert_resource(crate::tools::ai::AiModeActive(true));
            }
        }

        debug!("{} tool activated", self.config.name);
    }

    fn on_enter(&self) {
        debug!(
            "✅ {} TOOL: Entered {} mode",
            self.config.name.to_uppercase(),
            self.config.name
        );
    }

    fn on_exit(&self) {
        debug!(
            "❌ {} TOOL: Exited {} mode",
            self.config.name.to_uppercase(),
            self.config.name
        );
    }
}

/// Automatically register all enabled tools from the configuration
pub fn register_tools_from_config(mut tool_registry: ResMut<ToolRegistry>) {
    debug!("🔧 Loading toolbar tools from configuration...");

    // Print the current configuration for debugging
    super::toolbar_config::print_toolbar_config();

    let enabled_tools = ToolConfig::get_enabled_tools();
    debug!(
        "📋 Found {} enabled tools in configuration",
        enabled_tools.len()
    );

    for config in enabled_tools {
        let tool = ConfigurableTool::new(config);
        tool_registry.register_tool(Box::new(tool));
        debug!("✅ Registered tool: {} ({})", config.name, config.id);
    }

    debug!("🎉 Toolbar configuration loaded successfully!");
}

/// Plugin that loads tools from configuration
pub struct ConfigBasedToolbarPlugin;

impl Plugin for ConfigBasedToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_tools_from_config);
    }
}
