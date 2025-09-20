//! JSON-based theme system with live reloading
//!
//! This system allows live editing of theme colors by watching JSON files
//! in the src/ui/themes directory and reloading them without recompilation.

use super::json_theme::JsonThemeManager;
use bevy::prelude::*;

use super::json_theme::{
    check_json_theme_changes, update_all_theme_properties_on_change,
    update_ui_pane_text_colors_on_theme_change,
};

/// Plugin for runtime theme reloading
pub struct RuntimeThemePlugin;

impl Plugin for RuntimeThemePlugin {
    fn build(&self, app: &mut App) {
        // Only enable hot reload in debug builds for performance
        #[cfg(debug_assertions)]
        {
            info!("🔥 Hot reload enabled for theme development");

            // Initialize JSON theme manager (don't preload themes to allow change detection)
            let theme_manager = JsonThemeManager::new();

            app.insert_resource(theme_manager).add_systems(
                Update,
                (
                    check_json_theme_changes,
                    update_all_theme_properties_on_change,
                    update_ui_pane_text_colors_on_theme_change,
                    // Keep the disabled system for reference but don't run it
                    // update_ui_colors_on_theme_change,
                ),
            );
        }

        #[cfg(not(debug_assertions))]
        {
            info!("🚀 Hot reload disabled for release performance");
        }
    }
}
