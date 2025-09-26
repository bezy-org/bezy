//! Plugin management and configuration for the Bezy font editor
//!
//! This module organizes all the plugins and systems into logical groups,
//! making it easier to manage the application's architecture.

use bevy::gizmos::{config::DefaultGizmoConfigGroup, config::GizmoConfigStore};
use bevy::prelude::*;
use bevy::log::{LogPlugin, Level};

use crate::editing::sort::SortPlugin;
use crate::ui::themes::CurrentTheme;

/// Configure logging with performance optimization for release builds
pub fn configure_logging() -> LogPlugin {
    #[cfg(debug_assertions)]
    {
        // Debug builds: Show more detailed logging for development
        // Silence entity despawn warnings as they're expected in ECS
        LogPlugin {
            level: Level::INFO,
            filter: "bezy=info,bevy_render=warn,bevy_winit=warn,wgpu=warn,winit=warn,bevy_ecs::error::handler=error".to_string(),
            ..default()
        }
    }

    #[cfg(not(debug_assertions))]
    {
        // Release builds: Quieter logging, focus on warnings and errors
        // Silence entity despawn warnings as they're expected in ECS
        LogPlugin {
            level: Level::WARN,
            filter: "bezy=warn,bevy=warn,wgpu=error,winit=error,bevy_ecs::error::handler=error".to_string(),
            ..default()
        }
    }
}

/// Configure logging for TUI mode - disables console output to prevent terminal corruption
pub fn configure_logging_for_tui() -> LogPlugin {
    // When TUI is active, we need to disable all console output
    // This prevents logs from corrupting the TUI display
    LogPlugin {
        level: Level::ERROR,  // Only show critical errors (minimizes output)
        filter: "all=off".to_string(),  // Disable all logs via filter
        ..default()
    }
}

/// Configure default Bevy plugins for the application
#[allow(dead_code)]
pub fn configure_default_plugins() -> bevy::app::PluginGroupBuilder {
    DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bezy".into(), // Default title, will be updated by theme system
                resolution: (1024.0, 768.0).into(), // Default resolution, will be updated by theme system
                // Tell wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        })
        .set(configure_logging())
}

/// Configure default Bevy plugins for TUI mode
pub fn configure_default_plugins_for_tui() -> bevy::app::PluginGroupBuilder {
    DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bezy".into(),
                resolution: (1024.0, 768.0).into(),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        })
        .set(configure_logging_for_tui())  // Use TUI-specific logging that disables console output
}

/// System to configure gizmo appearance
fn configure_gizmos(mut gizmo_store: ResMut<GizmoConfigStore>, theme: Res<CurrentTheme>) {
    let (config, _) = gizmo_store.config_mut::<DefaultGizmoConfigGroup>();
    let line_width = theme.theme().gizmo_line_width();
    config.line.width = line_width;
    debug!("Configured gizmo line width to {}px", line_width);
}

/// Plugin to organize toolbar-related plugins
pub struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, _app: &mut App) {
        // Toolbar systems will be added when we port the UI toolbars
        debug!("ToolbarPlugin loaded - toolbar systems pending full port");
    }
}

/// Plugin to organize setup systems
pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, configure_gizmos);
    }
}

/// Main application plugin that bundles all internal plugins
pub struct BezySystems;

impl Plugin for BezySystems {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SetupPlugin,
            ToolbarPlugin,
            SortPlugin,
            // Additional plugins will be added as we port more components
            // Note: CameraPlugin is now handled by src/rendering/cameras.rs
        ));
    }
}
