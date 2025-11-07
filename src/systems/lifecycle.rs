//! Application lifecycle systems
//!
//! This module contains systems that handle the application lifecycle,
//! from startup initialization to shutdown procedures.

use bevy::prelude::*;
use bevy::window::WindowCloseRequested;

/// System to exit the application when the Escape key is pressed
pub fn exit_on_esc(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit_events: EventWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        app_exit_events.write(AppExit::Success);
    }
}

/// System to handle window close button clicks
pub fn handle_window_close(
    mut close_events: EventReader<WindowCloseRequested>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for _ in close_events.read() {
        info!("Window close requested, exiting application");
        app_exit_events.write(AppExit::Success);
    }
}

#[cfg(feature = "tui")]
/// System to notify TUI when app is exiting
///
/// Critical for preventing deadlock on window close: The TUI thread runs in a separate
/// thread with tokio::select! waiting for messages. When Bevy exits (via ESC or window
/// close button), this system sends AppMessage::Shutdown to break the TUI event loop.
/// Without this, the main thread would hang at tui_handle.join() waiting for a thread
/// that never exits, causing the macOS beachball.
pub fn notify_tui_on_exit(
    mut exit_events: EventReader<AppExit>,
    tui_comm: Option<Res<crate::core::tui_communication::TuiCommunication>>,
) {
    for _ in exit_events.read() {
        if let Some(tui) = &tui_comm {
            use crate::tui::communication::AppMessage;
            let _ = tui.send(AppMessage::Shutdown);
            info!("Sent shutdown message to TUI");
        }
    }
}

/// System to load UFO font on startup
pub fn load_ufo_font(
    cli_args: Res<crate::core::config::CliArgs>,
    mut app_state: ResMut<crate::core::state::AppState>,
    #[cfg(feature = "tui")] tui_comm: Option<Res<crate::core::tui_communication::TuiCommunication>>,
) {
    // clap provides the default value, so font_source is guaranteed to be Some
    if let Some(path) = &cli_args.font_source {
        match app_state.load_font_from_path(path.clone()) {
            Ok(_) => {
                debug!("Successfully loaded UFO font from: {}", path.display());

                #[cfg(feature = "tui")]
                if let Some(tui) = &tui_comm {
                    use crate::tui::communication::AppMessage;
                    let _ = tui.send(AppMessage::FontLoaded(path.display().to_string()));
                }
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
