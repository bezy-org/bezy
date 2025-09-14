//! FontIR-based application lifecycle systems
//!
//! This module contains systems that handle font loading and management
//! using FontIR instead of the old custom data structures.

use crate::core::cli::CliArgs;
use crate::core::state::FontIRAppState;
use bevy::prelude::*;
use fontir::source::Source;

/// System to load UFO/designspace font on startup using FontIR
pub fn load_fontir_font(mut commands: Commands, cli_args: Res<CliArgs>) {
    // clap provides the default value, so font_source is guaranteed to be Some
    if let Some(path) = &cli_args.font_source {
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

                info!(
                    "Successfully loaded font with FontIR from: {}",
                    path.display()
                );
                commands.insert_resource(app_state);
            }
            Err(e) => {
                error!("Failed to load font with FontIR: {}", e);
                error!("Font path: {}", path.display());
                error!("The application will continue but some features may not work correctly.");

                // Don't insert any FontIR state if loading fails
                warn!("App will run without FontIR state - some features may not work");
            }
        }
    } else {
        info!("No font path specified, starting with empty default state.");
        // Don't insert any FontIRAppState resource - systems will handle this gracefully
        // The app will run in an empty state allowing users to start fresh
    }
}
