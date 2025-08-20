//! JSON-based theme system with live reloading
//!
//! This system allows live editing of theme colors by watching JSON files
//! in the src/ui/themes directory and reloading them without recompilation.

use super::json_theme::JsonThemeManager;
use bevy::prelude::*;

use super::json_theme::{
    check_json_theme_changes, update_border_radius_on_theme_change,
};

/// Plugin for runtime theme reloading
pub struct RuntimeThemePlugin;

impl Plugin for RuntimeThemePlugin {
    fn build(&self, app: &mut App) {
        // Always enable for testing - remove cfg(debug_assertions) temporarily
        {
            // Initialize JSON theme manager (don't preload themes to allow change detection)
            let theme_manager = JsonThemeManager::new();

            app.insert_resource(theme_manager).add_systems(
                Update,
                (
                    check_json_theme_changes,
                    update_border_radius_on_theme_change,
                ),
            );
        }
    }
}
