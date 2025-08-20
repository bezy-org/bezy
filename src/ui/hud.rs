//! Head-up display (HUD) management for the Bezy font editor
//!
//! This module manages the top-level UI components that are not part of the
//! zoomable design space, including toolbars and information panels.

use crate::ui::panes::glyph_pane::spawn_glyph_pane;
use crate::ui::themes::CurrentTheme;
use bevy::prelude::*;

/// Spawns all non-debug HUD elements (toolbars, etc.)
///
/// Note: Many components are now spawned automatically by their respective plugins.
/// This function serves as a coordination point for UI elements that need manual spawning.
pub fn spawn_hud(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    theme: &Res<CurrentTheme>,
) {
    debug!("HUD spawning initiated");

    // Spawn glyph information pane
    spawn_glyph_pane(commands, asset_server, theme);

    // Placeholder for future HUD components:
    // - Coordinate display pane
    // - Status bar
    // - Menu system

    info!("HUD setup complete with glyph pane");
}

/// Plugin to manage HUD-related systems
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud_system);
    }
}

/// System to set up the HUD during startup
fn setup_hud_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme: Res<CurrentTheme>,
) {
    spawn_hud(&mut commands, &asset_server, &theme);
}
