//! # Legacy Adapter System (DEPRECATED)
//!
//! ⚠️  **THIS MODULE IS DEPRECATED AND WILL BE REMOVED**
//!
//! These adapters were used to bridge tools with the old manual registration system.
//! The new config-based system in `toolbar_config.rs` + `config_loader.rs` handles
//! everything automatically now.
//!
//! ## Migration Status
//! - ✅ **NEW SYSTEM**: `ConfigBasedToolbarPlugin` automatically registers tools from `toolbar_config.rs`
//! - ❌ **OLD SYSTEM**: Manual adapter registration (this file) - no longer used
//!
//! ## What to do instead
//! - **To modify toolbar**: Edit `/src/ui/toolbars/edit_mode_toolbar/toolbar_config.rs`
//! - **To add new tools**: Add to config, no manual adapters needed

use super::*;
use crate::ui::edit_mode_toolbar::{EditTool as LegacyEditTool, ToolId};
use bevy::prelude::*;

/// Adapter for the new select tool to work with legacy system
pub struct SelectToolAdapter;

impl LegacyEditTool for SelectToolAdapter {
    fn id(&self) -> ToolId {
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
        10 // First tool in toolbar
    }

    fn description(&self) -> &'static str {
        "Select and manipulate objects"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(select::SelectModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Select);
    }

    fn on_enter(&self) {
        info!("Select tool activated");
    }

    fn on_exit(&self) {
        info!("Select tool deactivated");
    }
}

/// Adapter for the new pen tool to work with legacy system
pub struct PenToolAdapter;

impl LegacyEditTool for PenToolAdapter {
    fn id(&self) -> ToolId {
        "pen"
    }

    fn name(&self) -> &'static str {
        "Pen"
    }

    fn icon(&self) -> &'static str {
        "\u{E011}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('p')
    }

    fn default_order(&self) -> i32 {
        20 // After select
    }

    fn description(&self) -> &'static str {
        "Draw paths and contours"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(pen::PenModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Pen);
    }

    fn on_enter(&self) {
        info!("Pen tool activated");
    }

    fn on_exit(&self) {
        info!("Pen tool deactivated");
    }
}

/// Adapter for the new text tool to work with legacy system
pub struct TextToolAdapter;

impl LegacyEditTool for TextToolAdapter {
    fn id(&self) -> ToolId {
        "text"
    }

    fn name(&self) -> &'static str {
        "Text"
    }

    fn icon(&self) -> &'static str {
        "\u{E017}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('t')
    }

    fn default_order(&self) -> i32 {
        40 // After drawing tools
    }

    fn description(&self) -> &'static str {
        "Place text and create sorts (Tab for modes)"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(text::TextModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Text);
    }

    fn on_enter(&self) {
        info!("Text tool activated with submenu");
    }

    fn on_exit(&self) {
        info!("Text tool deactivated");
    }
}

/// ⚠️ DEPRECATED: Plugin to register all the clean tool adapters
///
/// This plugin is now a no-op since the config-based system handles registration.
/// It remains only for backward compatibility and will be removed.
pub struct CleanToolsPlugin;

impl Plugin for CleanToolsPlugin {
    fn build(&self, _app: &mut App) {
        // 🎉 NEW SYSTEM: Tools are now registered automatically by ConfigBasedToolbarPlugin
        // This plugin is now redundant - all registration happens via toolbar_config.rs

        warn!("⚠️  CleanToolsPlugin is DEPRECATED - remove from your app.add_plugins()");
        info!(
            "✅ Use ConfigBasedToolbarPlugin instead (already included in EditModeToolbarPlugin)"
        );
        info!("📝 To modify toolbar: Edit src/ui/toolbars/edit_mode_toolbar/toolbar_config.rs");
    }
}

/// Adapter for the shapes tool to work with legacy system
pub struct ShapesToolAdapter;

impl LegacyEditTool for ShapesToolAdapter {
    fn id(&self) -> ToolId {
        "shapes"
    }

    fn name(&self) -> &'static str {
        "Shapes"
    }

    fn icon(&self) -> &'static str {
        "\u{E016}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('s')
    }

    fn default_order(&self) -> i32 {
        30 // After pen, before text
    }

    fn description(&self) -> &'static str {
        "Create geometric shapes"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(crate::core::io::input::InputMode::Shape);
    }

    fn on_enter(&self) {
        info!("Shapes tool activated");
    }

    fn on_exit(&self) {
        info!("Shapes tool deactivated");
    }
}

/// Adapter for the knife tool to work with legacy system
pub struct KnifeToolAdapter;

impl LegacyEditTool for KnifeToolAdapter {
    fn id(&self) -> ToolId {
        "knife"
    }

    fn name(&self) -> &'static str {
        "Knife"
    }

    fn icon(&self) -> &'static str {
        "\u{E012}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('k')
    }

    fn default_order(&self) -> i32 {
        110 // Advanced tool
    }

    fn description(&self) -> &'static str {
        "Cut contours at specific points"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(knife::KnifeModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Knife);
    }

    fn on_enter(&self) {
        info!("Knife tool activated");
    }

    fn on_exit(&self) {
        info!("Knife tool deactivated");
    }
}

/// Adapter for the hyper tool to work with legacy system
pub struct HyperToolAdapter;

impl LegacyEditTool for HyperToolAdapter {
    fn id(&self) -> ToolId {
        "hyper"
    }

    fn name(&self) -> &'static str {
        "Hyper"
    }

    fn icon(&self) -> &'static str {
        "\u{E018}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('h')
    }

    fn default_order(&self) -> i32 {
        100 // Advanced tool
    }

    fn description(&self) -> &'static str {
        "Draw smooth hyperbezier curves"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(hyper::HyperModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Hyper);
    }

    fn on_enter(&self) {
        info!("Hyper tool activated");
    }

    fn on_exit(&self) {
        info!("Hyper tool deactivated");
    }
}

/// Adapter for the metaballs tool to work with legacy system
pub struct MetaballsToolAdapter;

impl LegacyEditTool for MetaballsToolAdapter {
    fn id(&self) -> ToolId {
        "metaballs"
    }

    fn name(&self) -> &'static str {
        "Metaballs"
    }

    fn icon(&self) -> &'static str {
        "\u{E019}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('m')
    }

    fn default_order(&self) -> i32 {
        120 // Advanced tool
    }

    fn description(&self) -> &'static str {
        "Create organic shapes with metaball effects"
    }

    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(metaballs::MetaballsModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Metaballs);
    }

    fn on_enter(&self) {
        info!("Metaballs tool activated");
    }

    fn on_exit(&self) {
        info!("Metaballs tool deactivated");
    }
}

