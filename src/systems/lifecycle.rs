//! Application lifecycle systems
//!
//! This module contains systems that handle the application lifecycle,
//! from startup initialization to shutdown procedures.

use bevy::prelude::*;

/// System to exit the application when the Escape key is pressed
pub fn exit_on_esc(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit_events: EventWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        app_exit_events.write(AppExit::Success);
    }
}

/// System to load UFO font on startup
pub fn load_ufo_font(
    cli_args: Res<crate::core::cli::CliArgs>,
    mut app_state: ResMut<crate::core::state::AppState>,
) {
    // clap provides the default value, so ufo_path is guaranteed to be Some
    if let Some(path) = &cli_args.ufo_path {
        match app_state.load_font_from_path(path.clone()) {
            Ok(_) => {
                info!("Successfully loaded UFO font from: {}", path.display());
            }
            Err(e) => {
                error!("Failed to load UFO font: {}", e);
                error!("Font path: {}", path.display());
                error!("The application will continue but some features may not work correctly.");
            }
        }
    } else {
        warn!("No UFO font path specified, running without a font loaded.");
    }
}
