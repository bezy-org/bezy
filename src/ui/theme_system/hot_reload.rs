//! Hot reloading system for theme files
//!
//! This module allows live updating of theme colors while the application is running.
//! Simply edit a theme file and save it to see changes immediately.

use super::{embedded_themes, CurrentTheme};
use bevy::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// Resource to track theme file modification times
#[derive(Resource)]
pub struct ThemeHotReload {
    /// Path to the theme files directory
    theme_dir: PathBuf,
    /// Last modification time for each theme file
    last_modified: std::collections::HashMap<String, SystemTime>,
    /// Timer for checking file changes
    check_timer: Timer,
}

impl Default for ThemeHotReload {
    fn default() -> Self {
        // Use user themes directory if it exists, otherwise don't watch anything
        let theme_dir = if embedded_themes::user_themes_dir_exists() {
            embedded_themes::get_user_themes_dir()
        } else {
            // Return an empty path that won't match any files
            PathBuf::new()
        };

        Self {
            theme_dir,
            last_modified: std::collections::HashMap::new(),
            check_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        }
    }
}

/// System that checks for theme file changes and reloads them
pub fn hot_reload_themes(
    mut hot_reload: ResMut<ThemeHotReload>,
    mut current_theme: ResMut<CurrentTheme>,
    time: Res<Time>,
) {
    hot_reload.check_timer.tick(time.delta());

    if !hot_reload.check_timer.just_finished() {
        return;
    }

    // Only check if we have a valid theme directory
    if hot_reload.theme_dir.as_os_str().is_empty() {
        return;
    }

    let current_theme_name = current_theme.variant.name();
    let theme_file = format!("{current_theme_name}.json");
    let theme_path = hot_reload.theme_dir.join(&theme_file);

    // Check if the current theme file has been modified
    if let Ok(metadata) = fs::metadata(&theme_path) {
        if let Ok(modified) = metadata.modified() {
            let last_check = hot_reload.last_modified.get(&theme_file).copied();

            // If file was modified since last check, reload the theme
            if last_check.is_none_or(|last| modified > last) {
                info!("Theme file {} was modified, reloading...", theme_file);
                hot_reload
                    .last_modified
                    .insert(theme_file.clone(), modified);

                // Force a theme switch to the same theme to reload colors
                let variant = current_theme.variant.clone();
                current_theme.switch_to(variant);

                info!("Theme reloaded successfully!");
            }
        }
    }
}

/// Plugin for hot reloading themes
pub struct ThemeHotReloadPlugin;

impl Plugin for ThemeHotReloadPlugin {
    #[allow(unused_variables)]
    fn build(&self, app: &mut App) {
        // Only enable hot reload in development builds
        #[cfg(debug_assertions)]
        {
            app.init_resource::<ThemeHotReload>()
                .add_systems(Update, hot_reload_themes);

            if embedded_themes::user_themes_dir_exists() {
                info!(
                    "Theme hot reloading enabled! Edit themes in {:?} to see changes live.",
                    embedded_themes::get_user_themes_dir()
                );
            } else {
                info!("Theme hot reloading enabled (no user themes directory found).");
            }
        }
    }
}

/// Alternative approach: Keyboard shortcut to reload current theme
pub fn reload_theme_on_keypress(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut current_theme: ResMut<CurrentTheme>,
) {
    // Press Ctrl/Cmd + R to reload the current theme
    let ctrl_held =
        keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if ctrl_held && keyboard.just_pressed(KeyCode::KeyR) {
        let variant = current_theme.variant.clone();
        current_theme.switch_to(variant);
        info!("Theme reloaded!");
    }
}
