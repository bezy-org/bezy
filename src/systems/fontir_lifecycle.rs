//! FontIR-based application lifecycle systems
//!
//! This module contains systems that handle font loading and management
//! using FontIR instead of the old custom data structures.

use crate::core::config::CliArgs;
use crate::core::state::FontIRAppState;
use bevy::prelude::*;
use fontir::source::Source;
use std::path::PathBuf;

/// Resource to track deferred font loading state
#[derive(Resource)]
pub struct DeferredFontLoading {
    pub font_path: Option<PathBuf>,
    pub loading: bool,
    pub loaded: bool,
}

impl Default for DeferredFontLoading {
    fn default() -> Self {
        Self {
            font_path: None,
            loading: false,
            loaded: false,
        }
    }
}

/// System to initialize deferred font loading on startup (fast)
pub fn load_fontir_font(mut commands: Commands, cli_args: Res<CliArgs>) {
    // Initialize deferred loading resource
    let deferred_loading = DeferredFontLoading {
        font_path: cli_args.font_source.clone(),
        loading: false,
        loaded: false,
    };

    commands.insert_resource(deferred_loading);

    if cli_args.font_source.is_some() {
        info!("Font loading deferred - window will appear immediately");
    } else {
        debug!("No font path specified, starting with empty default state.");
    }
}

/// System to actually load the font in the background after window is shown
pub fn deferred_fontir_font_loading(
    mut commands: Commands,
    mut deferred_loading: ResMut<DeferredFontLoading>,
) {
    if deferred_loading.loaded || deferred_loading.loading {
        return;
    }

    if let Some(path) = deferred_loading.font_path.clone() {
        deferred_loading.loading = true;

        info!("Starting background font loading from: {}", path.display());

        match FontIRAppState::from_path(path.clone()) {
            Ok(mut app_state) => {
                // Try to load glyphs if possible
                if let Err(e) = app_state.load_glyphs() {
                    warn!("Could not load glyphs: {}", e);
                }

                // Try to load kerning groups
                if let Err(e) = app_state.load_kerning_groups() {
                    warn!("Could not load kerning groups: {}", e);
                }

                debug!(
                    "Successfully loaded font with FontIR from: {}",
                    path.display()
                );
                commands.insert_resource(app_state);
                deferred_loading.loaded = true;
                deferred_loading.loading = false;

                info!("Font loading completed!");
            }
            Err(e) => {
                error!("Failed to load font with FontIR: {}", e);
                error!("Font path: {}", path.display());
                error!("The application will continue but some features may not work correctly.");

                // Mark as completed (even though it failed) so we don't keep trying
                deferred_loading.loaded = true;
                deferred_loading.loading = false;
                warn!("App will run without FontIR state - some features may not work");
            }
        }
    }
}
