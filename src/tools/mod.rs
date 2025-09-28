//! # Core Tool Logic Module (`/src/tools/`)
//!
//! This module contains the **business logic and behavior** for all editing tools in Bezy.
//! This is where tools actually *do* their work - handling input, modifying glyphs, managing state, etc.
//!
//! ## Architecture Overview
//!
//! ```text
//! /src/tools/          ← YOU ARE HERE - Core tool behavior & logic
//! /src/ui/toolbars/    ← Visual toolbar UI, registration, configuration
//! ```
//!
//! ## Separation of Concerns
//!
//! - **`/src/tools/`** (this module): **WHAT tools do** - business logic, input handling, glyph modification
//! - **`/src/ui/toolbars/`**: **HOW tools appear** - UI rendering, icons, shortcuts, registration
//! - **`toolbar_config.rs`**: **Single source of truth** for toolbar configuration
//!
//! ## Tool Implementation Pattern
//!
//! Each tool file (e.g., `select.rs`, `pen.rs`) contains:
//! 1. **Core tool struct** implementing the `EditTool` trait
//! 2. **ECS systems** for input handling and behavior
//! 3. **State management** (resources, components)
//! 4. **Plugin** that registers the tool's systems with Bevy
//!
//! ## How Tools Work Together
//!
//! 1. **Tool behavior** defined here in `/src/tools/`
//! 2. **Tool configuration** defined in `/src/ui/toolbars/edit_mode_toolbar/toolbar_config.rs`
//! 3. **Automatic registration** via `ConfigBasedToolbarPlugin` bridges the two
//! 4. **User clicks toolbar** → tool activated → systems here handle the behavior
//!
//! ## Adding New Tools
//!
//! 1. **Create tool file** in this module (e.g., `my_tool.rs`)
//! 2. **Add to `toolbar_config.rs`** with icon, shortcut, ordering
//! 3. **Tool automatically appears** in toolbar and works
//!
//! This clean separation makes tools easy to understand, test, and maintain.

pub mod ai;
pub mod hyper;
pub mod knife;
pub mod measure;
pub mod metaballs;
pub mod pan;
pub mod pen;
pub mod select;
pub mod shapes;
pub mod text;

// Re-export all tools
pub use ai::*;
pub use hyper::*;
pub use knife::*;
pub use measure::*;
pub use metaballs::*;
pub use pan::*;
pub use pen::*;
pub use select::*;
pub use shapes::*;
pub use text::*;

use bevy::prelude::*;

/// Information about a tool
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: &'static str,
    pub display_name: &'static str,
    pub icon: &'static str,
    pub tooltip: &'static str,
    pub shortcut: Option<KeyCode>,
}

/// Trait that all editing tools must implement
pub trait EditTool: Send + Sync {
    /// Get information about this tool
    fn info(&self) -> ToolInfo;

    /// Called when the tool is activated
    fn on_activate(&mut self, commands: &mut Commands) {
        // Default implementation does nothing
        let _ = commands;
    }

    /// Called when the tool is deactivated
    fn on_deactivate(&mut self, commands: &mut Commands) {
        // Default implementation does nothing
        let _ = commands;
    }

    /// Called every frame while the tool is active
    fn update(&self, commands: &mut Commands) {
        // Default implementation does nothing
        let _ = commands;
    }
}

// ✅ NEW SYSTEM: Tools are now automatically registered from toolbar_config.rs
// No need for manual registration - just add tools to the config and they appear in the toolbar!
//
// ❌ OLD SYSTEM (DEPRECATED): Manual adapters in adapters.rs - no longer needed
