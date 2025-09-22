//! Command line interface for the Bezy font editor
//!
//! Handles parsing command line arguments and provides
//! validation for user inputs. Many CLI options are documented with
//! examples to help users understand the expected format.

use crate::core::config_file::ConfigFile;
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
///   bezy --theme light                  # Use light theme
///   bezy --theme strawberry             # Use strawberry theme
///   bezy --no-default-buffer            # Start without default LTR buffer (for testing)
#[derive(Parser, Debug, Resource, Clone)]
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
    /// If not specified, opens an empty default state.
    #[clap(
        long = "edit",
        short = 'e',
        help = "Font source to edit (UFO or designspace)",
        long_help = "Path to a font source to edit. Accepts UFO directories (.ufo) for single master fonts or designspace files (.designspace) for variable fonts with multiple masters. If not specified, opens an empty default state."
    )]
    pub font_source: Option<PathBuf>,

    /// Theme to use for the interface
    ///
    /// Available themes: dark (default), light, strawberry, campfire.
    /// Custom themes can be added by creating new theme files.
    #[clap(
        long = "theme",
        short = 't',
        help = "Theme to use",
        long_help = "Theme to use for the interface. Available themes: dark (default), light, strawberry, campfire"
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

    /// Initialize user configuration directory with settings and themes
    ///
    /// This creates the ~/.config/bezy directory with:
    /// - settings.json: User preferences like default theme
    /// - themes/: Copies of all default themes that you can customize
    /// This allows full customization without modifying the app installation.
    #[clap(
        long = "new-config",
        help = "Initialize user config directory with settings and themes",
        long_help = "Initialize the ~/.config/bezy directory with a settings.json file and copies of all default themes. This allows you to customize themes and set preferences like default theme without needing command line arguments."
    )]
    pub new_config: bool,

    /// Disable Terminal User Interface (TUI) mode
    ///
    /// By default, Bezy launches with a TUI (Terminal User Interface) alongside
    /// the main editor window. The TUI provides tabs for codepoint browsing,
    /// font information, and logs. Use this flag to run without the TUI.
    #[clap(
        long = "no-tui",
        help = "Disable Terminal User Interface mode",
        long_help = "Disable the Terminal User Interface (TUI) that normally runs alongside the main editor. By default, Bezy shows a TUI in the terminal with tabs for codepoint browsing, font information, and real-time log viewing. Use this flag to run the GUI only."
    )]
    pub no_tui: bool,
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
    /// For WASM builds, we start with an empty state since command line arguments
    /// are not available in the browser environment.
    #[cfg(target_arch = "wasm32")]
    pub fn default_for_web() -> Self {
        Self {
            font_source: None,        // Start with empty state for web builds
            theme: None,              // Use default theme for web builds
            no_default_buffer: false, // Enable default buffer for web builds
        }
    }

    /// Get the font source path if provided
    #[allow(dead_code)]
    pub fn get_font_source(&self) -> Option<&PathBuf> {
        self.font_source.as_ref()
    }

    /// Get the theme variant from CLI args, config file, or default
    ///
    /// Priority order:
    /// 1. CLI argument (--theme)
    /// 2. Config file setting (~/.config/bezy/settings.json)
    /// 3. Built-in default (dark theme)
    pub fn get_theme_variant(&self) -> ThemeVariant {
        // First check CLI args
        if let Some(theme_name) = &self.theme {
            if let Some(variant) = ThemeVariant::parse(theme_name) {
                debug!("Using theme from CLI: {}", theme_name);
                return variant;
            }
        }

        // Then check config file
        if let Some(config) = ConfigFile::load() {
            if let Some(theme_name) = config.default_theme {
                if let Some(variant) = ThemeVariant::parse(&theme_name) {
                    debug!("Using theme from config file: {}", theme_name);
                    return variant;
                }
            }
        }

        // Finally use built-in default
        debug!("Using default theme: dark");
        ThemeVariant::default()
    }
}
