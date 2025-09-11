//! Plugin management and configuration for the Bezy font editor
//!
//! This module organizes all the plugins and systems into logical groups,
//! making it easier to manage the application's architecture.

use bevy::gizmos::{config::DefaultGizmoConfigGroup, config::GizmoConfigStore};
use bevy::prelude::*;

use crate::editing::sort::SortPlugin;
use crate::ui::theme::{GIZMO_LINE_WIDTH, WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};

/// Configure default Bevy plugins for the application
#[allow(dead_code)]
pub fn configure_default_plugins() -> bevy::app::PluginGroupBuilder {
    DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: WINDOW_TITLE.into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                // Tell wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        })
        // Disable Bevy's default LogPlugin since we're using our own custom logger
        .build()
        .disable::<bevy::log::LogPlugin>()
}

/// System to configure gizmo appearance
fn configure_gizmos(mut gizmo_store: ResMut<GizmoConfigStore>) {
    let (config, _) = gizmo_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line.width = GIZMO_LINE_WIDTH;
    info!("Configured gizmo line width to {}px", GIZMO_LINE_WIDTH);
}

/// Plugin to organize drawing-related systems
pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, crate::rendering::draw::draw_origin_cross);
        debug!("DrawPlugin loaded - added draw_origin_cross system");
    }
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
            DrawPlugin,
            ToolbarPlugin,
            SortPlugin,
            // Additional plugins will be added as we port more components
            // Note: CameraPlugin is now handled by src/rendering/cameras.rs
        ));
    }
}
