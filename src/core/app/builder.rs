//! Application builder and initialization
//!
//! This module provides the main app creation functions

use super::plugins::{CorePluginGroup, EditorPluginGroup, RenderingPluginGroup};
use crate::core::config::{BezySettings, CliArgs, DEFAULT_WINDOW_SIZE, WINDOW_TITLE};
use crate::core::state::{AppState, GlyphNavigation};
use crate::systems::{
    center_camera_on_startup_layout, create_startup_layout, exit_on_esc, load_fontir_font,
    plugins::{configure_default_plugins, configure_default_plugins_for_tui},
};
#[cfg(feature = "tui")]
use crate::tui::communication::{AppMessage, TuiMessage};
use crate::ui::theme::CurrentTheme;
use crate::utils::embedded_assets::EmbeddedAssetsPlugin;
#[cfg(debug_assertions)]
use crate::ui::theme_system::RuntimeThemePlugin;
use anyhow::Result;
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use tokio::sync::mpsc;

/// Creates a fully configured Bevy font editor application.
///
/// This is the main entry point for the Bezy font editor. It creates a complete
/// Bevy application with all necessary plugins, resources, and systems configured
/// for font editing functionality.
pub fn create_app(cli_args: CliArgs) -> Result<App> {
    #[cfg(not(target_arch = "wasm32"))]
    cli_args
        .validate()
        .map_err(|e| anyhow::anyhow!("CLI validation failed: {}", e))?;

    let mut app = App::new();
    configure_resources(&mut app, cli_args);
    configure_window_plugins(&mut app);
    add_plugin_groups(&mut app);
    add_startup_and_exit_systems(&mut app);
    Ok(app)
}

/// Sets up application resources and configuration
fn configure_resources(app: &mut App, cli_args: CliArgs) {
    let glyph_navigation = GlyphNavigation::default();
    let mut settings = BezySettings::default();

    // Set theme from CLI args (CLI overrides settings)
    let theme_variant = cli_args.get_theme_variant();
    settings.set_theme(theme_variant.clone());

    // Initialize current theme
    let current_theme = CurrentTheme::new(theme_variant);
    let background_color = current_theme.theme().background_color();

    app.insert_resource(cli_args)
        .insert_resource(glyph_navigation)
        .insert_resource(settings)
        .insert_resource(current_theme)
        .insert_resource(ClearColor(background_color));

    // Configure platform-specific window settings
    #[cfg(not(target_arch = "wasm32"))]
    {
        // In debug mode, use continuous updates for instant theme reloading
        // In release mode, use reactive mode for better performance
        #[cfg(debug_assertions)]
        app.insert_resource(WinitSettings {
            focused_mode: bevy::winit::UpdateMode::Continuous,
            unfocused_mode: bevy::winit::UpdateMode::Continuous,
        });

        #[cfg(not(debug_assertions))]
        app.insert_resource(WinitSettings::desktop_app());
    }

    #[cfg(target_arch = "wasm32")]
    app.insert_resource(WinitSettings::game());
}

/// Configure window and default plugins with platform-specific settings
fn configure_window_plugins(app: &mut App) {
    let window_config = Window {
        title: WINDOW_TITLE.to_string(),
        resolution: DEFAULT_WINDOW_SIZE.into(),
        ..default()
    };

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_plugins(configure_default_plugins().set(WindowPlugin {
            primary_window: Some(window_config),
            ..default()
        }));
    }

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugins(configure_default_plugins().set(WindowPlugin {
            primary_window: Some(window_config),
            ..default()
        }));
    }
}

#[cfg(feature = "tui")]
/// Configure window plugins for TUI mode (no window output)
fn configure_window_plugins_for_tui(app: &mut App) {
    let window_config = Window {
        title: WINDOW_TITLE.to_string(),
        resolution: DEFAULT_WINDOW_SIZE.into(),
        ..default()
    };

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_plugins(configure_default_plugins_for_tui().set(WindowPlugin {
            primary_window: Some(window_config),
            ..default()
        }));
    }

    #[cfg(target_arch = "wasm32")]
    {
        // TUI mode is not supported on WASM
        panic!("TUI mode is not supported on WASM platform");
    }
}

/// Add all plugin groups to the application
fn add_plugin_groups(app: &mut App) {
    debug!("Adding plugin groups...");

    // Add embedded assets plugin to provide fonts when installed via cargo install
    app.add_plugins(EmbeddedAssetsPlugin);

    app.add_plugins((RenderingPluginGroup, EditorPluginGroup, CorePluginGroup));

    // Add runtime theme reload plugin for development
    #[cfg(debug_assertions)]
    app.add_plugins(RuntimeThemePlugin);

    debug!("All plugin groups added successfully");
}

/// Add startup and exit systems
fn add_startup_and_exit_systems(app: &mut App) {
    app.add_systems(Startup, (load_fontir_font, create_startup_layout).chain())
        .add_systems(Update, (exit_on_esc, center_camera_on_startup_layout));
}

#[cfg(feature = "tui")]
/// Creates a Bevy app configured for TUI mode with communication channels.
///
/// This variant sets up the app with channels for bi-directional communication
/// between the Bevy app and the TUI. It disables console logging to prevent
/// terminal corruption.
pub fn create_app_with_tui(
    cli_args: CliArgs,
    tui_rx: mpsc::UnboundedReceiver<TuiMessage>,
    app_tx: mpsc::UnboundedSender<AppMessage>,
) -> Result<App> {
    #[cfg(not(target_arch = "wasm32"))]
    cli_args
        .validate()
        .map_err(|e| anyhow::anyhow!("CLI validation failed: {}", e))?;

    let mut app = App::new();

    // Add TUI communication resource
    app.insert_resource(crate::core::tui_communication::TuiCommunication::new(
        tui_rx, app_tx,
    ));

    configure_resources(&mut app, cli_args);
    configure_window_plugins_for_tui(&mut app); // Use TUI-specific configuration
    add_plugin_groups(&mut app);
    add_startup_and_exit_systems(&mut app);

    // TUI mode needs immediate GUI updates - use continuous mode for instant responsiveness
    app.insert_resource(WinitSettings {
        focused_mode: bevy::winit::UpdateMode::Continuous,
        unfocused_mode: bevy::winit::UpdateMode::Continuous,
    });

    // Add TUI communication systems
    // TUI message handling needs to run in Input system set to ensure respawn queue is processed immediately
    app.add_systems(Update,
        handle_tui_messages.in_set(crate::editing::FontEditorSets::Input)
    );
    app.add_systems(Update, send_initial_font_data_to_tui);

    Ok(app)
}

#[cfg(feature = "tui")]
/// System to handle messages from TUI
#[allow(clippy::too_many_arguments)]
fn handle_tui_messages(
    mut tui_comm: ResMut<crate::core::tui_communication::TuiCommunication>,
    mut glyph_nav: ResMut<GlyphNavigation>,
    mut fontir_state: Option<ResMut<crate::core::state::FontIRAppState>>,
    mut text_editor_state: Option<ResMut<crate::core::state::TextEditorState>>,
    mut respawn_queue: ResMut<crate::systems::sorts::sort_entities::BufferSortRespawnQueue>,
    current_tool: Option<Res<crate::ui::edit_mode_toolbar::CurrentTool>>,
    text_placement_mode: Option<Res<crate::ui::edit_mode_toolbar::text::TextPlacementMode>>,
    app_state: Option<Res<AppState>>,
) {
    while let Some(message) = tui_comm.try_recv() {
        match message {
            TuiMessage::SelectGlyph(unicode_codepoint) => {
                // Delegate to TUI module handler
                match crate::tui::message_handler::handle_glyph_selection(
                    unicode_codepoint,
                    &mut glyph_nav,
                    &mut fontir_state,
                    &mut text_editor_state,
                    &mut respawn_queue,
                    &current_tool,
                    &text_placement_mode,
                ) {
                    Ok(glyph_name) => {
                        // Send confirmation back to TUI
                        tui_comm.send_current_glyph(glyph_name);

                        // Force immediate GUI redraw by triggering all change detection
                        use bevy::prelude::DetectChangesMut;
                        if let Some(ref mut text_state) = text_editor_state {
                            text_state.set_changed();
                            // Force viewport micro-change to trigger rendering pipeline
                            let current_viewport = text_state.viewport_offset;
                            text_state.viewport_offset = current_viewport + bevy::math::Vec2::new(0.0001, 0.0001);
                            text_state.viewport_offset = current_viewport;
                        }
                        glyph_nav.set_changed();
                        respawn_queue.set_changed();
                    }
                    Err(error_message) => {
                        // Send error message to TUI log
                        tui_comm.send_log(error_message);
                    }
                }
            }
            TuiMessage::RequestGlyphList => {
                info!("TUI requested glyph list");
                if let Some(ref fontir_state) = fontir_state {
                    send_glyph_list_to_tui(&mut tui_comm, fontir_state, app_state.as_deref());
                } else {
                    tui_comm
                        .send_log("No font loaded - please use --edit to load a font".to_string());
                }
            }
            TuiMessage::RequestFontInfo => {
                info!("TUI requested font info");
                if let Some(ref fontir_state) = fontir_state {
                    send_font_info_to_tui(&mut tui_comm, fontir_state);
                } else {
                    tui_comm
                        .send_log("No font loaded - please use --edit to load a font".to_string());
                }
            }
            TuiMessage::ChangeZoom(zoom) => {
                info!("TUI requested zoom change: {}", zoom);
            }
            TuiMessage::ForceRedraw => {
                // Force all systems to mark themselves as changed to trigger updates
                if let Some(ref mut text_state) = text_editor_state {
                    use bevy::prelude::DetectChangesMut;
                    text_state.set_changed();

                    // Force viewport change to trigger rendering
                    let current_viewport = text_state.viewport_offset;
                    text_state.viewport_offset = current_viewport + bevy::math::Vec2::new(0.001, 0.0);
                    text_state.viewport_offset = current_viewport;
                }

                {
                    use bevy::prelude::DetectChangesMut;
                    glyph_nav.set_changed();
                }
                respawn_queue.set_changed();

                info!("Force redraw requested by TUI");
            }
            TuiMessage::QAReportReady(report) => {
                info!("QA report ready: {:?}", report);
                // TODO: Handle QA report
            }
            TuiMessage::QAAnalysisFailed(error) => {
                warn!("QA analysis failed: {}", error);
                tui_comm.send_log(format!("QA analysis failed: {}", error));
            }
            TuiMessage::Quit => {
                info!("TUI requested quit");
                // TODO: Handle quit request
            }
        }
    }
}

#[cfg(feature = "tui")]
/// System to send initial font data to TUI on startup
fn send_initial_font_data_to_tui(
    mut tui_comm: ResMut<crate::core::tui_communication::TuiCommunication>,
    fontir_state: Option<Res<crate::core::state::FontIRAppState>>,
    app_state: Option<Res<AppState>>,
) {
    if let Some(fontir_state) = fontir_state {
        if fontir_state.is_changed() {
            send_font_info_to_tui(&mut tui_comm, &fontir_state);
            send_glyph_list_to_tui(&mut tui_comm, &fontir_state, app_state.as_deref());
        }
    }
}

#[cfg(feature = "tui")]
fn send_glyph_list_to_tui(
    tui_comm: &mut crate::core::tui_communication::TuiCommunication,
    fontir_state: &crate::core::state::FontIRAppState,
    app_state: Option<&AppState>,
) {
    // Use the proper function that extracts Unicode values from FontIR
    let glyphs = crate::tui::communication::generate_glyph_list(fontir_state, app_state);
    tui_comm.send_glyph_list(glyphs);
}

#[cfg(feature = "tui")]
fn send_font_info_to_tui(
    tui_comm: &mut crate::core::tui_communication::TuiCommunication,
    _fontir_state: &crate::core::state::FontIRAppState,
) {
    let font_info = crate::tui::communication::FontInfo {
        family_name: Some("Unknown".to_string()), // TODO: Get from fontir_state
        style_name: Some("Regular".to_string()),  // TODO: Get from fontir_state
        version: None,
        ascender: None,
        descender: None,
        cap_height: None,
        x_height: None,
        units_per_em: Some(1000.0),              // TODO: Get from fontir_state
    };

    tui_comm.send_font_info(font_info);
}
