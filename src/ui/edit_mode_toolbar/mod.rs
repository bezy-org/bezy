//! # Toolbar UI & Registration System (`/src/ui/toolbars/edit_mode_toolbar/`)
//!
//! This module manages the **visual toolbar interface and tool registration** for Bezy.
//! This is where tools are configured, displayed, and switching between tools is handled.
//!
//! ## Architecture Overview
//!
//! ```text
//! /src/tools/          ← Core tool behavior & logic
//! /src/ui/toolbars/    ← YOU ARE HERE - Visual toolbar UI, registration, configuration
//! ```
//!
//! ## Separation of Concerns
//!
//! - **`/src/tools/`**: **WHAT tools do** - business logic, input handling, glyph modification
//! - **`/src/ui/toolbars/`** (this module): **HOW tools appear** - UI rendering, icons, shortcuts, registration
//! - **`toolbar_config.rs`**: **Single source of truth** for toolbar configuration
//!
//! ## Current System (Config-Based - RECOMMENDED)
//!
//! The toolbar now uses a **centralized configuration system** that automatically handles everything:
//!
//! 1. **Edit `toolbar_config.rs`** - Define tools with icons, shortcuts, ordering
//! 2. **`ConfigBasedToolbarPlugin`** - Automatically registers tools from config  
//! 3. **Tools appear in toolbar** - No manual registration needed
//!
//! ## Key Files
//!
//! - **`toolbar_config.rs`** - ✅ **EDIT THIS** to modify toolbar (icons, shortcuts, ordering)
//! - **`config_loader.rs`** - ✅ Automatic tool registration from config
//! - **`ui.rs`** - ✅ Visual toolbar rendering and interaction
//! - **`adapters.rs`** - ❌ **LEGACY** - being phased out
//!
//! ## How to Modify the Toolbar
//!
//! **To add/remove/reorder tools**: Edit `toolbar_config.rs:75-166`
//! **To change tool behavior**: Edit the tool's file in `/src/tools/`
//! **To change toolbar appearance**: Edit `ui.rs`
//!
//! ### Core Components
//! - **Tool Registry**: Dynamic registration system for edit tools
//! - **EditTool Trait**: Common interface that all tools must implement
//! - **UI Module**: Handles the visual toolbar interface and mode switching
//! - **Plugin System**: Bevy plugin that coordinates everything
//!
//! ### Adding New Tools
//!
//! The system is designed for maximum ease of use. Adding a new tool requires:
//!
//! 1. **Create your tool file** (`my_tool.rs`):
//!    ```rust,ignore
//!    use bevy::prelude::*;
//!    
//!    // Define the types for the example
//!    type ToolId = &'static str;
//!    
//!    trait EditTool {
//!        fn id(&self) -> ToolId;
//!        fn name(&self) -> &'static str;
//!        fn icon(&self) -> &'static str;
//!        fn default_order(&self) -> i32 { 100 }
//!        fn update(&self, commands: &mut Commands);
//!    }
//!    
//!    #[derive(Resource)]
//!    struct ToolRegistry {
//!        tools: Vec<Box<dyn EditTool>>,
//!    }
//!    
//!    impl ToolRegistry {
//!        fn register_tool(&mut self, tool: Box<dyn EditTool>) {
//!            self.tools.push(tool);
//!        }
//!    }
//!
//!    pub struct MyTool;
//!
//!    impl EditTool for MyTool {
//!        fn id(&self) -> ToolId { "my_tool" }
//!        fn name(&self) -> &'static str { "My Tool" }
//!        fn icon(&self) -> &'static str { "\u{E019}" }
//!        fn default_order(&self) -> i32 { 50 }
//!        
//!        fn update(&self, commands: &mut Commands) {
//!            // Your tool's behavior while active
//!        }
//!    }
//!
//!    pub struct MyToolPlugin;
//!    impl Plugin for MyToolPlugin {
//!        fn build(&self, app: &mut App) {
//!            app.add_systems(Startup, register_my_tool);
//!        }
//!    }
//!
//!    fn register_my_tool(mut registry: ResMut<ToolRegistry>) {
//!        registry.register_tool(Box::new(MyTool));
//!    }
//!    ```
//!
//! 2. **Add module declaration**: `mod my_tool;` and
//!    `pub use my_tool::MyToolPlugin;`
//!
//! 3. **Register plugin**: `app.add_plugins(MyToolPlugin);`
//!
//! The tool automatically appears in the toolbar with proper ordering and
//! functionality.
//!
//! ## Key Features
//!
//! - **Dynamic Registration**: Tools register themselves at runtime
//! - **Configurable Ordering**: Control the order tools appear in the toolbar
//! - **Temporary Modes**: Support for temporary mode activation (e.g., holding
//!   spacebar for pan)
//! - **State Management**: Proper enter/exit lifecycle for each tool
//! - **UI Integration**: Visual feedback and interactive toolbar
//! - **Extensibility**: Easy to add new tools with minimal code changes

#![allow(unused_imports)]

use bevy::prelude::*;
use std::collections::HashMap;

// NEW: Centralized configuration system
pub mod config_loader;
pub mod toolbar_config;

mod hyper;
pub mod keyboard_utils;
pub mod knife;
mod measure;
mod metaballs;
mod pan;
pub mod pen;
pub mod select;
mod shapes;
pub mod text;
pub mod ui;

// Add the spacebar toggle module
mod spacebar_toggle;

// Re-export the new tool system
pub use spacebar_toggle::{handle_spacebar_toggle, SpacebarToggleState};
pub use ui::{
    create_toolbar_button, create_toolbar_button_with_hover_text, handle_toolbar_mode_selection,
    spawn_edit_mode_toolbar, update_current_edit_mode, update_hover_text_visibility,
    update_toolbar_button_appearances, update_toolbar_button_colors,
    update_toolbar_button_text_colors,
};

// NEW: Re-export centralized configuration system
pub use config_loader::{ConfigBasedToolbarPlugin, ConfigurableTool};
pub use toolbar_config::{ToolBehavior, ToolConfig, TOOLBAR_TOOLS};

// Re-export legacy types for backward compatibility (commented out until UI is
// ported)
// pub use ui::{CurrentEditMode, EditMode};

/// Unique identifier for an edit tool
pub type ToolId = &'static str;

/// Trait that defines a complete edit tool with all its metadata and behavior.
///
/// This trait provides everything needed to create a fully functional toolbar tool.
/// Tools are self-contained and define their own appearance, behavior, and lifecycle.
///
/// # Required Methods
///
/// - `id()`: Unique string identifier (e.g., "select", "pen", "eraser")
/// - `name()`: Human-readable name displayed in UI
/// - `icon()`: Unicode character for the toolbar button
/// - `update()`: Core tool behavior, called every frame while active
///
/// # Optional Methods
///
/// All other methods have sensible defaults but can be overridden:
/// - `shortcut_key()`: Keyboard shortcut (e.g., Some('v') for Select)
/// - `default_order()`: Toolbar position (lower numbers = earlier position)
/// - `description()`: Tooltip text and help documentation
/// - `on_enter()`: Setup when tool becomes active
/// - `on_exit()`: Cleanup when switching away from tool
/// - `supports_temporary_mode()`: Whether tool can be temporarily activated via spacebar
///
/// # Example Implementation
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use crate::ui::edit_mode_toolbar::{EditTool, ToolId};
///
/// trait EditTool {
///     fn id(&self) -> ToolId;
///     fn name(&self) -> &'static str;
///     fn icon(&self) -> &'static str;
///     fn shortcut_key(&self) -> Option<char> { None }
///     fn default_order(&self) -> i32 { 100 }
///     fn update(&self, commands: &mut Commands);
///     fn on_enter(&self) {}
///     fn on_exit(&self) {}
/// }
///
/// pub struct MyCustomTool {
///     // Tool-specific state can go here
///     pub some_setting: bool,
/// }
///
/// impl EditTool for MyCustomTool {
///     fn id(&self) -> ToolId { "my_custom_tool" }
///     fn name(&self) -> &'static str { "Custom Tool" }
///     fn icon(&self) -> &'static str { "\u{E020}" }
///     fn shortcut_key(&self) -> Option<char> { Some('c') }
///     fn default_order(&self) -> i32 { 75 }
///     
///     fn update(&self, commands: &mut Commands) {
///         // Implement tool behavior (handle input, modify entities, etc.)
///         // This runs every frame while the tool is active
///     }
///     
///     fn on_enter(&self) {
///         // Setup tool state, change cursor, etc.
///     }
///     
///     fn on_exit(&self) {
///         // Cleanup, restore cursor, etc.
///     }
/// }
/// ```
pub trait EditTool: Send + Sync + 'static {
    /// Unique identifier for this tool (used internally)
    fn id(&self) -> ToolId;

    /// Display name shown in UI
    fn name(&self) -> &'static str;

    /// Unicode icon character for the toolbar button
    fn icon(&self) -> &'static str;

    /// Optional keyboard shortcut (e.g., Some('v') for Select tool)
    fn shortcut_key(&self) -> Option<char> {
        None
    }

    /// Default ordering priority (lower numbers appear first)
    /// Common values: 10=Select, 20=Pen, 30=Eraser, 50=Shapes, 100=Utilities
    fn default_order(&self) -> i32 {
        100
    }

    /// Description for tooltips/help
    fn description(&self) -> &'static str {
        ""
    }

    /// Called every frame while this tool is active
    fn update(&self, commands: &mut Commands);

    /// Called when switching to this tool
    fn on_enter(&self) {}

    /// Called when switching away from this tool
    fn on_exit(&self) {}

    /// Whether this tool supports temporary activation via spacebar (e.g., pan tool)
    fn supports_temporary_mode(&self) -> bool {
        false
    }
}

/// Registry for all available edit tools.
///
/// This resource manages all registered tools and their display order in the toolbar.
/// Tools register themselves during app startup, and the registry handles ordering
/// based on `default_order()` values and custom ordering preferences.
///
/// # Usage
///
/// Tools typically register themselves in their plugin's `build()` method:
///
/// ```rust,ignore
/// fn register_my_tool(mut tool_registry: ResMut<ToolRegistry>) {
///     tool_registry.register_tool(Box::new(MyTool));
/// }
/// ```
///
/// The registry automatically handles ordering and makes tools available to the UI system.
#[derive(Resource, Default)]
pub struct ToolRegistry {
    tools: HashMap<ToolId, Box<dyn EditTool>>,
    ordered_tool_ids: Vec<ToolId>,
    ordering_dirty: bool,
}

impl ToolRegistry {
    /// Register a new tool with the registry
    pub fn register_tool(&mut self, tool: Box<dyn EditTool>) {
        let id = tool.id();
        debug!("Registering tool: {} ({})", tool.name(), id);
        self.tools.insert(id, tool);
        self.ordering_dirty = true;
    }

    /// Get a tool by its ID
    pub fn get_tool(&self, id: ToolId) -> Option<&dyn EditTool> {
        self.tools.get(id).map(|t| t.as_ref())
    }

    /// Get all tools in their proper display order
    pub fn get_ordered_tools(&mut self) -> &[ToolId] {
        if self.ordering_dirty {
            self.rebuild_ordering();
        }
        &self.ordered_tool_ids
    }

    /// Get all registered tool IDs (unordered)
    pub fn get_all_tool_ids(&self) -> Vec<ToolId> {
        self.tools.keys().copied().collect()
    }

    /// Rebuild the tool ordering based on default_order() values
    fn rebuild_ordering(&mut self) {
        let mut tools_with_order: Vec<(ToolId, i32)> = self
            .tools
            .iter()
            .map(|(id, tool)| (*id, tool.default_order()))
            .collect();

        // Sort by order value (lower numbers first)
        tools_with_order.sort_by_key(|(_, order)| *order);

        self.ordered_tool_ids = tools_with_order.into_iter().map(|(id, _)| id).collect();

        self.ordering_dirty = false;
    }
}

/// Current active tool state
#[derive(Resource, Default)]
pub struct CurrentTool {
    pub current: Option<ToolId>,
    pub previous: Option<ToolId>,
}

impl CurrentTool {
    /// Switch to a new tool
    pub fn switch_to(&mut self, new_tool: ToolId) {
        self.previous = self.current;
        self.current = Some(new_tool);
        debug!("Switched to tool: {}", new_tool);
    }

    /// Get the currently active tool
    pub fn get_current(&self) -> Option<ToolId> {
        self.current
    }

    /// Get the previously active tool
    pub fn get_previous(&self) -> Option<ToolId> {
        self.previous
    }
}

/// Custom tool ordering configuration (optional).
///
/// This resource allows you to override the default tool ordering in the toolbar.
/// By default, tools are ordered by their `default_order()` values, but you can
/// specify a custom order here.
///
/// # Usage Examples
///
/// ## Setting Custom Order
///
/// ```rust,ignore
/// fn setup_toolbar_order(mut tool_ordering: ResMut<ToolOrdering>) {
///     // Put select first, then pen, then custom tools
///     tool_ordering.set_order(vec![
///         "select",
///         "pen",
///         "my_custom_tool",
///         "eraser",
///         // Any unspecified tools will appear after these in default order
///     ]);
/// }
/// ```
///
/// ## Using Preset Orders
///
/// ```rust,ignore
/// fn setup_design_workflow(mut tool_ordering: ResMut<ToolOrdering>) {
///     // Optimized for design-focused work
///     tool_ordering.set_design_focused_order();
/// }
///
/// fn setup_annotation_workflow(mut tool_ordering: ResMut<ToolOrdering>) {
///     // Optimized for annotation and markup work
///     tool_ordering.set_annotation_focused_order();
/// }
/// ```
///
/// ## Dynamic Ordering
///
/// ```rust,ignore
/// fn setup_user_preference_order(
///     mut tool_ordering: ResMut<ToolOrdering>,
///     user_prefs: Res<UserPreferences>
/// ) {
///     match user_prefs.workflow_type {
///         WorkflowType::Design => tool_ordering.set_design_focused_order(),
///         WorkflowType::Annotation => tool_ordering.set_annotation_focused_order(),
///         WorkflowType::Custom => tool_ordering.set_order(user_prefs.custom_tool_order.clone()),
///     }
/// }
/// ```
// New tool exports (using current available exports)
pub use hyper::HyperToolPlugin;
pub use knife::{KnifeModeActive, KnifeToolPlugin};
pub use measure::MeasureToolPlugin;
pub use metaballs::MetaballsToolPlugin;
pub use text::TextToolPlugin;

// Legacy compatibility exports (will be removed after migration)
pub use hyper::HyperTool;
pub use knife::KnifeTool;
pub use pan::{PanMode, PanToolPlugin, PresentationMode};
pub use pen::PenMode;
pub use shapes::ShapesToolPlugin;

pub use select::{SelectMode, SelectModeActive};

// Legacy trait (will be removed after migration)
pub trait EditModeSystem: Send + Sync + 'static {
    #[allow(dead_code)]
    fn update(&self, commands: &mut Commands);
    #[allow(dead_code)]
    fn on_enter(&self) {}
    #[allow(dead_code)]
    fn on_exit(&self) {}
}

/// System to initialize the default tool (Select) on startup
fn initialize_default_tool(
    mut current_tool: ResMut<CurrentTool>,
    tool_registry: Res<ToolRegistry>,
) {
    debug!("Attempting to initialize default tool...");
    debug!("Available tools: {:?}", tool_registry.get_all_tool_ids());
    debug!(
        "Current tool before initialization: {:?}",
        current_tool.get_current()
    );

    // Set Select as the default tool
    if tool_registry.get_tool("select").is_some() {
        current_tool.switch_to("select");
        debug!("Initialized with default tool: select");
    } else {
        warn!("Select tool not found in registry! Will use first available tool.");
        // Fallback to first available tool if select is not available
        let all_tools = tool_registry.get_all_tool_ids();
        if let Some(&first_tool) = all_tools.first() {
            current_tool.switch_to(first_tool);
            debug!("Initialized with fallback tool: {}", first_tool);
        }
    }

    debug!(
        "Current tool after initialization: {:?}",
        current_tool.get_current()
    );
}

/// ✅ UNIFIED TOOLBAR SYSTEM - Main Plugin
///
/// This plugin provides the complete toolbar system using the new config-based architecture.
///
/// ## What This Plugin Includes:
/// - **ConfigBasedToolbarPlugin**: Automatically registers all tools from `toolbar_config.rs`
/// - **Tool behavior plugins**: Pan, Measure, Text (provide ECS systems for tool behavior)
/// - **UI systems**: Toolbar rendering, button interactions, tool switching
/// - **Spacebar toggle**: Temporary pan mode when holding spacebar
///
/// ## No Manual Registration Needed:
/// Just add `EditModeToolbarPlugin` to your app - it handles everything automatically!
///
/// ## To Modify Toolbar:
/// Edit `toolbar_config.rs` - all tools, icons, shortcuts, and ordering are configured there.
pub struct EditModeToolbarPlugin;

impl Plugin for EditModeToolbarPlugin {
    fn build(&self, app: &mut App) {
        debug!("🚀 Building EditModeToolbarPlugin with unified config-based system!");
        app
            // Initialize the new tool system
            .init_resource::<ToolRegistry>()
            .init_resource::<CurrentTool>()
            // Legacy resources (will be removed after migration)
            // .init_resource::<CurrentPrimitiveType>()  // Will be added when shapes is ported
            // .init_resource::<ActivePrimitiveDrawing>()  // Will be added when shapes is ported
            // .init_resource::<CurrentCornerRadius>()  // Will be added when shapes is ported
            // .init_resource::<UiInteractionState>()  // Will be added when shapes is ported
            .init_resource::<SpacebarToggleState>()
            // ✅ NEW SYSTEM: Centralized configuration system handles all tool registration
            .add_plugins(config_loader::ConfigBasedToolbarPlugin)
            // ✅ BEHAVIOR PLUGINS: These provide the actual tool behavior (systems, input handling)
            // These are still needed because they contain the ECS systems that make tools work
            .add_plugins(PanToolPlugin) // Pan tool input handling and behavior
            .add_plugins(MeasureToolPlugin) // Measure tool rendering and interaction
            .add_plugins(TextToolPlugin) // Text tool with submenu functionality
            .add_plugins(ShapesToolPlugin) // Shapes tool with submenu functionality
            .add_plugins(KnifeToolPlugin) // Knife tool for cutting paths
            .add_plugins(crate::tools::ai::AiToolPlugin) // AI tool with submenu functionality
            // ✅ NOTE: Tool registration (toolbar buttons) is automatic via ConfigBasedToolbarPlugin
            // ✅ NOTE: Tool behavior (what tools do) still needs these individual behavior plugins
            .add_systems(
                PostStartup,
                (
                    spawn_edit_mode_toolbar,
                    initialize_default_tool.after(spawn_edit_mode_toolbar),
                    // handle_primitive_selection  // Will be added when shapes is ported
                ),
            )
            .add_systems(Update, handle_spacebar_toggle)
            .add_systems(
                Update,
                (
                    handle_toolbar_mode_selection,
                    update_toolbar_button_appearances,
                    update_hover_text_visibility,
                ),
            )
            .add_systems(Update, update_current_edit_mode);
    }
}
