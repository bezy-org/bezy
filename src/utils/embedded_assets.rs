//! Embedded assets for the Bezy font editor
//!
//! This module embeds UI fonts and provides them when the assets directory
//! is not available (e.g., after cargo install).

use bevy::prelude::*;
use std::path::Path;

// Embed the font files at compile time
pub const BEZY_GROTESK_BYTES: &[u8] = include_bytes!("../../assets/fonts/BezyGrotesk-Regular.ttf");
pub const HASUBI_MONO_BYTES: &[u8] = include_bytes!("../../assets/fonts/HasubiMono-Regular.ttf");

// Store font handles as resources so they can be accessed globally
#[derive(Resource, Default)]
pub struct EmbeddedFonts {
    pub bezy_grotesk: Option<Handle<Font>>,
    pub hasubi_mono: Option<Handle<Font>>,
}

/// Plugin that provides embedded fonts when assets directory is not available
pub struct EmbeddedAssetsPlugin;

impl Plugin for EmbeddedAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EmbeddedFonts>()
            .add_systems(PreStartup, setup_embedded_fonts);
    }
}

fn setup_embedded_fonts(
    mut fonts: ResMut<Assets<Font>>,
    mut embedded_fonts: ResMut<EmbeddedFonts>,
) {
    // Always load embedded fonts - they'll be used as fallback
    debug!("Loading embedded fonts as fallback");

    // Load BezyGrotesk font
    match Font::try_from_bytes(BEZY_GROTESK_BYTES.to_vec()) {
        Ok(font) => {
            let handle = fonts.add(font);
            embedded_fonts.bezy_grotesk = Some(handle.clone());
            debug!("✅ Embedded BezyGrotesk-Regular.ttf ready");
        }
        Err(e) => {
            error!("Failed to load embedded BezyGrotesk-Regular.ttf: {:?}", e);
        }
    }

    // Load HasubiMono font
    match Font::try_from_bytes(HASUBI_MONO_BYTES.to_vec()) {
        Ok(font) => {
            let handle = fonts.add(font);
            embedded_fonts.hasubi_mono = Some(handle.clone());
            debug!("✅ Embedded HasubiMono-Regular.ttf ready");
        }
        Err(e) => {
            error!("Failed to load embedded HasubiMono-Regular.ttf: {:?}", e);
        }
    }
}

/// Extension trait for AssetServer to load fonts with embedded fallback
pub trait AssetServerFontExt {
    /// Load a font with automatic fallback to embedded version
    fn load_font_with_fallback(&self, path: &str, embedded_fonts: &EmbeddedFonts) -> Handle<Font>;
}

impl AssetServerFontExt for AssetServer {
    fn load_font_with_fallback(&self, path: &str, embedded_fonts: &EmbeddedFonts) -> Handle<Font> {
        // Check if assets directory exists
        let assets_path = Path::new("assets");

        if assets_path.exists() {
            // Use normal asset loading if assets exist
            self.load(path)
        } else {
            // Use embedded fonts when assets don't exist
            match path {
                "fonts/BezyGrotesk-Regular.ttf" | "fonts/bezy-grotesk-regular.ttf" => {
                    embedded_fonts
                        .bezy_grotesk
                        .clone()
                        .unwrap_or_else(|| self.load(path))
                }
                "fonts/HasubiMono-Regular.ttf" => embedded_fonts
                    .hasubi_mono
                    .clone()
                    .unwrap_or_else(|| self.load(path)),
                _ => {
                    warn!("No embedded fallback for font: {}", path);
                    self.load(path)
                }
            }
        }
    }
}

/// Macro to simplify font loading with embedded fallback
#[macro_export]
macro_rules! load_font {
    ($asset_server:expr, $embedded_fonts:expr, $path:expr) => {{
        use $crate::utils::embedded_assets::AssetServerFontExt;
        $asset_server.load_font_with_fallback($path, $embedded_fonts)
    }};
}
