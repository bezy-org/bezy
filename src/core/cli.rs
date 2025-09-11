//! Command line interface for the Bezy font editor
//!
//! Handles parsing command line arguments and provides
//! validation for user inputs. Many CLI options are documented with
//! examples to help users understand the expected format.

use crate::core::settings::DEFAULT_UFO_PATH;
use crate::ui::themes::ThemeVariant;
use bevy::prelude::*;
use clap::Parser;
use std::path::PathBuf;

/// Bezy CLI arguments
///
/// Examples:
///   bezy                                # Load default font
///   bezy --edit my-font.ufo             # Edit specific font
///   bezy --edit ~/Fonts/MyFont.ufo      # Edit font with full path
///   bezy --edit my-variable.designspace # Edit variable font
///   bezy --theme lightmode              # Use light mode theme
///   bezy --theme strawberry             # Use strawberry theme
///   bezy --no-default-buffer            # Start without default LTR buffer (for testing)
#[derive(Parser, Debug, Resource)]
#[clap(
    name = "bezy",
    version,
    about = "A font editor built with Rust and Bevy",
    long_about = "Bezy is a cross-platform font editor that supports UFO (Unified Font Object) files. It provides glyph editing capabilities with a modern, game-engine-powered interface."
)]
pub struct CliArgs {
    /// Path to a font source to edit (UFO or designspace)
    ///
    /// The source should be either a valid UFO version 3 directory structure
    /// or a .designspace file for variable fonts.
    /// If not specified, loads the default sample font.
    #[clap(
        long = "edit",
        short = 'e',
        default_value = DEFAULT_UFO_PATH,
        help = "Font source to edit (UFO or designspace)",
        long_help = "Path to a font source to edit. Accepts UFO directories (.ufo) for single master fonts or designspace files (.designspace) for variable fonts with multiple masters."
    )]
    pub font_source: Option<PathBuf>,

    /// Theme to use for the interface
    ///
    /// Available themes: darkmode (default), lightmode, strawberry, campfire.
    /// Custom themes can be added by creating new theme files.
    #[clap(
        long = "theme",
        short = 't',
        help = "Theme to use",
        long_help = "Theme to use for the interface. Available themes: darkmode (default), lightmode, strawberry, campfire"
    )]
    pub theme: Option<String>,

    /// Disable creation of default buffer on startup (for testing/debugging)
    ///
    /// By default, Bezy creates an LTR text buffer at startup to provide
    /// an immediate editing environment. This flag disables that behavior
    /// for testing isolated text flows or debugging positioning issues.
    #[clap(
        long = "no-default-buffer",
        help = "Disable default LTR buffer creation on startup",
        long_help = "Disable creation of the default LTR text buffer on startup. Useful for testing isolated text flows or debugging positioning issues."
    )]
    pub no_default_buffer: bool,
}

impl CliArgs {
    /// Validate the CLI arguments after parsing
    ///
    /// This ensures that all paths exist and are valid before the application starts,
    /// providing clear error messages for common mistakes.
    pub fn validate(&self) -> Result<(), String> {
        // Skip validation for WASM builds since filesystem works differently
        #[cfg(target_arch = "wasm32")]
        {
            return Ok(());
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = &self.font_source {
                if !path.exists() {
                    return Err(format!(
                        "Font source does not exist: {}\nMake sure the path is correct and the file exists.",
                        path.display()
                    ));
                }

                // Check if it's a .designspace file or a UFO directory
                if path.is_dir() {
                    // It's a directory - check if it's a valid UFO
                    let meta_info = path.join("metainfo.plist");
                    if !meta_info.exists() {
                        return Err(format!(
                            "Not a valid UFO directory: missing metainfo.plist in {}\nMake sure this is a valid UFO directory.",
                            path.display()
                        ));
                    }
                } else if path.is_file() {
                    // It's a file - check if it's a designspace
                    if let Some(extension) = path.extension() {
                        if extension != "designspace" {
                            return Err(format!(
                                "Unsupported file type: {}\nOnly .designspace files are supported for non-directory sources.",
                                path.display()
                            ));
                        }
                    } else {
                        return Err(format!(
                            "File has no extension: {}\nExpected a .designspace file.",
                            path.display()
                        ));
                    }
                } else {
                    return Err(format!(
                        "Path is neither a file nor a directory: {}\nPath must be either a UFO directory or a .designspace file.",
                        path.display()
                    ));
                }
            } else {
                // If no font source provided, use default
                return Err("Please provide a font source path as an argument.\nExample: bezy --edit assets/fonts/bezy-grotesk-regular.ufo".to_string());
            }

            // Validate theme if provided
            if let Some(theme_name) = &self.theme {
                if ThemeVariant::parse(theme_name).is_none() {
                    let available_themes = ThemeVariant::all_names().join(", ");
                    return Err(format!(
                        "Unknown theme: '{theme_name}'\nAvailable themes: {available_themes}"
                    ));
                }
            }

            Ok(())
        }
    }

    /// Create default CLI args for web builds
    ///
    /// For WASM builds, we use a default font path since command line arguments
    /// are not available in the browser environment.
    #[cfg(target_arch = "wasm32")]
    pub fn default_for_web() -> Self {
        Self {
            font_source: Some(PathBuf::from(DEFAULT_UFO_PATH)),
            theme: None,              // Use default theme for web builds
            no_default_buffer: false, // Enable default buffer for web builds
        }
    }

    /// Get the font source path, guaranteed to be Some after validation
    #[allow(dead_code)]
    pub fn get_font_source(&self) -> &PathBuf {
        self.font_source
            .as_ref()
            .expect("Font source should be validated")
    }

    /// Get the theme variant from CLI args or default
    pub fn get_theme_variant(&self) -> ThemeVariant {
        self.theme
            .as_ref()
            .and_then(|theme_name| ThemeVariant::parse(theme_name))
            .unwrap_or_default()
    }
}
