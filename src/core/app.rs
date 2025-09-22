//! Application initialization and configuration

use crate::core::cli::CliArgs;
use crate::core::io::gamepad::GamepadPlugin;
use crate::core::io::input::InputPlugin;
use crate::core::io::pointer::PointerPlugin;
use crate::core::settings::{BezySettings, DEFAULT_WINDOW_SIZE, WINDOW_TITLE};
use crate::core::state::GlyphNavigation;
use crate::editing::{FontEditorSystemSetsPlugin, SelectionPlugin, TextEditorPlugin};
use crate::rendering::{
    cameras::CameraPlugin, checkerboard::CheckerboardPlugin,
    zoom_aware_scaling::CameraResponsivePlugin, EntityPoolingPlugin, GlyphRenderingPlugin,
    MeshCachingPlugin, MetricsRenderingPlugin, SortHandleRenderingPlugin,
};
use crate::systems::{
    center_camera_on_startup_layout, create_startup_layout, exit_on_esc, load_fontir_font,
    BezySystems, CommandsPlugin, InputConsumerPlugin, TextShapingPlugin, UiInteractionPlugin,
    plugins::configure_default_plugins,
};
use crate::ui::edit_mode_toolbar::EditModeToolbarPlugin;
use crate::ui::file_menu::FileMenuPlugin;
use crate::ui::panes::coordinate_pane::CoordinatePanePlugin;
use crate::ui::panes::file_pane::FilePanePlugin;
use crate::ui::panes::glyph_pane::GlyphPanePlugin;
use crate::ui::theme::CurrentTheme;
#[cfg(debug_assertions)]
use crate::ui::theme_system::RuntimeThemePlugin;
use anyhow::Result;
use bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use tokio::sync::mpsc;
use crate::tui::communication::{AppMessage, TuiMessage};

/// Plugin group for core application functionality
#[derive(Default)]
pub struct CorePluginGroup;

impl PluginGroup for CorePluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(PointerPlugin)
            .add(InputPlugin)
            .add(GamepadPlugin)
            .add(InputConsumerPlugin)
            .add(FontEditorSystemSetsPlugin) // Must be added before other font editor plugins
            .add(TextEditorPlugin)
            // Unified text shaping for RTL support (includes Arabic and HarfBuzz)
            .add(TextShapingPlugin)
            .add(SelectionPlugin)
            .add(UiInteractionPlugin)
            .add(CommandsPlugin)
            .add(BezySystems)
    }
}

/// Plugin group for rendering functionality
#[derive(Default)]
pub struct RenderingPluginGroup;

impl PluginGroup for RenderingPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CameraPlugin)
            .add(CameraResponsivePlugin)
            .add(CheckerboardPlugin)
            .add(EntityPoolingPlugin)
            .add(MeshCachingPlugin)
            .add(MetricsRenderingPlugin)
            .add(SortHandleRenderingPlugin)
            .add(GlyphRenderingPlugin)
    }
}

/// Plugin group for editor UI
#[derive(Default)]
pub struct EditorPluginGroup;

impl PluginGroup for EditorPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(FilePanePlugin)
            .add(GlyphPanePlugin)
            .add(CoordinatePanePlugin)
            .add(EditModeToolbarPlugin) // ✅ Includes ConfigBasedToolbarPlugin - handles all tools automatically
            .add(FileMenuPlugin)
            // ✅ NEW SYSTEM: All tools are now automatically registered via EditModeToolbarPlugin
            // No need for manual tool plugin registration - everything is handled by toolbar_config.rs
            .add(crate::tools::PenToolPlugin) // Re-enabled - pen tool needs its business logic plugin
            .add(crate::tools::SelectToolPlugin) // Select tool business logic plugin
    }
}

/// Creates a fully configured Bevy GUI application ready to run
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

    // Note: FontIRAppState is initialized by load_fontir_font startup system
    // app.init_resource::<AppState>() // Old system - keeping for gradual migration
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
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        canvas: None,
                        prevent_default_event_handling: false,
                        ..window_config
                    }),
                    ..default()
                })
                .set(bevy::render::RenderPlugin {
                    render_creation: bevy::render::settings::RenderCreation::Automatic(
                        bevy::render::settings::WgpuSettings {
                            backends: Some(bevy::render::settings::Backends::GL),
                            power_preference: bevy::render::settings::PowerPreference::LowPower,
                            ..default()
                        },
                    ),
                    ..default()
                }),
        );
    }
}

/// Add all plugin groups to the application
fn add_plugin_groups(app: &mut App) {
    debug!("Adding plugin groups...");

    // Add embedded assets plugin to provide fonts when installed via cargo install
    app.add_plugins(crate::utils::embedded_assets::EmbeddedAssetsPlugin);

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

/// Creates a fully configured Bevy GUI application with TUI support
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
    app.insert_resource(crate::core::tui_communication::TuiCommunication::new(tui_rx, app_tx));

    configure_resources(&mut app, cli_args);
    configure_window_plugins(&mut app);
    add_plugin_groups(&mut app);
    add_startup_and_exit_systems(&mut app);

    // Add TUI communication systems
    app.add_systems(Update, (handle_tui_messages, send_initial_font_data_to_tui, test_keyboard_sort_switching));

    Ok(app)
}

/// System to handle messages from TUI
fn handle_tui_messages(
    mut tui_comm: ResMut<crate::core::tui_communication::TuiCommunication>,
    mut glyph_nav: ResMut<GlyphNavigation>,
    mut fontir_state: Option<ResMut<crate::core::state::FontIRAppState>>,
    mut text_editor_state: Option<ResMut<crate::core::state::TextEditorState>>,
) {
    while let Some(message) = tui_comm.try_recv() {
        match message {
            TuiMessage::SelectGlyph(unicode_codepoint) => {
                println!("🎯 TUI GLYPH SELECTION: Received request for Unicode U+{:04X}", unicode_codepoint);
                info!("TUI requested glyph selection: U+{:04X}", unicode_codepoint);

                // Convert Unicode to char for processing
                let target_char = char::from_u32(unicode_codepoint);
                let glyph_name = format!("U+{:04X}", unicode_codepoint);

                // Update both glyph tracking systems
                glyph_nav.set_current_glyph(glyph_name.clone());

                // Also update FontIR state if available
                if let Some(ref mut fontir_state) = fontir_state {
                    fontir_state.set_current_glyph(Some(glyph_name.clone()));
                    info!("Updated FontIR current glyph to: {}", glyph_name);
                }

                // Find and activate the sort with the matching glyph name in the text editor state
                if let Some(ref mut text_state) = text_editor_state {
                    println!("📋 BUFFER DEBUG: Text editor buffer has {} sorts", text_state.buffer.len());

                    // Log all available sorts for debugging
                    for i in 0..text_state.buffer.len() {
                        if let Some(sort) = text_state.buffer.get(i) {
                            let codepoint_display = if let Some(cp) = sort.kind.codepoint() {
                                format!("U+{:04X} ('{}')", cp as u32, cp)
                            } else {
                                "no codepoint".to_string()
                            };
                            println!("  Sort[{}]: {} | glyph='{}'", i, codepoint_display, sort.kind.glyph_name());
                        }
                    }

                    // Find the index of the sort with the matching Unicode codepoint
                    let mut target_index = None;
                    if let Some(target_char) = target_char {
                        for i in 0..text_state.buffer.len() {
                            if let Some(sort) = text_state.buffer.get(i) {
                                if let Some(sort_codepoint) = sort.kind.codepoint() {
                                    if sort_codepoint == target_char {
                                        target_index = Some(i);
                                        println!("🎯 MATCH FOUND: Sort[{}] matches requested Unicode U+{:04X} ('{}')", i, unicode_codepoint, target_char);
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if let Some(index) = target_index {
                        // Use the TextEditorState's activate_sort method which handles deactivation properly
                        if text_state.activate_sort(index) {
                            println!("✅ SUCCESS: Activated sort at index {} for Unicode U+{:04X}", index, unicode_codepoint);
                            info!("Successfully activated sort at index {} for Unicode U+{:04X}", index, unicode_codepoint);
                        } else {
                            println!("❌ FAILED: Could not activate sort at index {} for Unicode U+{:04X}", index, unicode_codepoint);
                            warn!("Failed to activate sort at index {} for Unicode U+{:04X}", index, unicode_codepoint);
                        }
                    } else {
                        println!("⚠️ NOT FOUND: No existing sort found for Unicode U+{:04X} in text buffer", unicode_codepoint);
                        info!("No existing sort found for Unicode U+{:04X} in text buffer", unicode_codepoint);
                    }
                } else {
                    info!("No TextEditorState available for glyph activation");
                }

                // Send confirmation back to TUI
                tui_comm.send_current_glyph(glyph_name);
            }
            TuiMessage::RequestGlyphList => {
                info!("TUI requested glyph list");
                if let Some(ref fontir_state) = fontir_state {
                    send_glyph_list_to_tui(&mut tui_comm, fontir_state);
                } else {
                    tui_comm.send_log("No font loaded - please use --edit to load a font".to_string());
                }
            }
            TuiMessage::RequestFontInfo => {
                info!("TUI requested font info");
                if let Some(ref fontir_state) = fontir_state {
                    send_font_info_to_tui(&mut tui_comm, fontir_state);
                } else {
                    tui_comm.send_log("No font loaded - please use --edit to load a font".to_string());
                }
            }
            TuiMessage::ChangeZoom(zoom) => {
                info!("TUI requested zoom change: {}", zoom);
            }
            TuiMessage::Quit => {
                info!("TUI requested quit");
                // The TUI handles its own quit, this is just informational
            }
        }
    }
}

/// Send glyph list from FontIR to TUI
fn send_glyph_list_to_tui(
    tui_comm: &mut ResMut<crate::core::tui_communication::TuiCommunication>,
    fontir_state: &crate::core::state::FontIRAppState,
) {
    let mut glyphs = Vec::new();

    // Extract glyph data from FontIR
    for (glyph_name, glyph) in &fontir_state.glyph_cache {
        // Get the first available instance for this glyph
        if let Some((_location, glyph_instance)) = glyph.sources().iter().next() {
            // Try to find Unicode codepoints for this glyph from context
            let unicode_value = if let Some(_context) = &fontir_state.context {
                // Look up codepoints in the context - need to find the right field
                // For now, try to parse from glyph name if it looks like a Unicode name
                if glyph_name.starts_with("uni") && glyph_name.len() == 7 {
                    u32::from_str_radix(&glyph_name[3..], 16).ok()
                } else if glyph_name.len() == 1 {
                    // Single character glyph name
                    glyph_name.chars().next().map(|c| c as u32)
                } else {
                    None
                }
            } else {
                None
            };

            let glyph_info = crate::tui::communication::GlyphInfo {
                codepoint: glyph_name.clone(),
                name: Some(glyph_name.clone()),
                unicode: unicode_value,
                width: Some(glyph_instance.width as f32),
            };

            glyphs.push(glyph_info);
        }
    }

    // Sort glyphs by Unicode value, then by name
    glyphs.sort_by(|a, b| {
        match (a.unicode, b.unicode) {
            (Some(a_unicode), Some(b_unicode)) => a_unicode.cmp(&b_unicode),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.codepoint.cmp(&b.codepoint),
        }
    });

    info!("Sending {} glyphs to TUI", glyphs.len());
    tui_comm.send_glyph_list(glyphs);
    tui_comm.send_log(format!("Loaded {} glyphs from font", fontir_state.glyph_cache.len()));
}

/// Send font info from FontIR to TUI
fn send_font_info_to_tui(
    tui_comm: &mut ResMut<crate::core::tui_communication::TuiCommunication>,
    fontir_state: &crate::core::state::FontIRAppState,
) {
    let mut font_info = crate::tui::communication::FontInfo {
        family_name: None,
        style_name: None,
        version: None,
        ascender: None,
        descender: None,
        cap_height: None,
        x_height: None,
        units_per_em: None,
    };

    // Extract basic font info from FontIR context
    if let Some(context) = &fontir_state.context {
        // Get static metadata
        let static_metadata = context.static_metadata.get();
        font_info.units_per_em = Some(static_metadata.units_per_em as f32);

        // Set basic font information - we'll improve this later
        font_info.family_name = Some("Font Family".to_string());
        font_info.style_name = Some("Regular".to_string());
        font_info.version = Some("1.0".to_string());

        // Use reasonable default values for metrics
        font_info.ascender = Some(800.0);
        font_info.descender = Some(-200.0);
        font_info.cap_height = Some(700.0);
        font_info.x_height = Some(500.0);
    }

    info!("Sending font info to TUI");
    tui_comm.send_font_info(font_info);
}

/// System to send initial font data to TUI when font loads
fn send_initial_font_data_to_tui(
    mut tui_comm: Option<ResMut<crate::core::tui_communication::TuiCommunication>>,
    fontir_state: Option<ResMut<crate::core::state::FontIRAppState>>,
    mut sent_initial_data: Local<bool>,
) {
    // Only send data once when both TUI communication and font are available
    if !*sent_initial_data {
        if let (Some(mut tui_comm), Some(fontir_state)) = (tui_comm.as_mut(), fontir_state.as_ref()) {
            send_glyph_list_to_tui(&mut tui_comm, fontir_state);
            send_font_info_to_tui(&mut tui_comm, fontir_state);
            *sent_initial_data = true;
        }
    }
}

/// Test system to simulate TUI glyph selection via keyboard
fn test_keyboard_sort_switching(
    mut text_editor_state: Option<ResMut<crate::core::state::TextEditorState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut glyph_nav: ResMut<GlyphNavigation>,
    mut fontir_state: Option<ResMut<crate::core::state::FontIRAppState>>,
) {
    // Press '1' key to switch to Unicode U+0041 ('A') (matches test.ufo)
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        let unicode_codepoint = 0x0041u32; // 'A'
        println!("🎯 KEYBOARD TEST: Simulating TUI selection of Unicode U+{:04X}", unicode_codepoint);

        // Convert Unicode to char for processing (same as TUI code)
        let target_char = char::from_u32(unicode_codepoint);
        let glyph_name = format!("U+{:04X}", unicode_codepoint);

        // Update both glyph tracking systems (same as TUI code)
        glyph_nav.set_current_glyph(glyph_name.clone());

        if let Some(ref mut fontir_state) = fontir_state {
            fontir_state.set_current_glyph(Some(glyph_name.clone()));
            println!("✅ Updated FontIR current glyph to: {}", glyph_name);
        }

        // Find and activate the sort with the matching Unicode codepoint
        if let Some(ref mut text_state) = text_editor_state {
            // First, list all available sorts
            println!("📋 Available sorts:");
            for i in 0..text_state.buffer.len() {
                if let Some(sort) = text_state.buffer.get(i) {
                    let codepoint_display = if let Some(cp) = sort.kind.codepoint() {
                        format!("U+{:04X} ('{}')", cp as u32, cp)
                    } else {
                        "no codepoint".to_string()
                    };
                    println!("  Sort[{}]: {} | glyph='{}', active={}",
                             i, codepoint_display, sort.kind.glyph_name(), sort.is_active);
                }
            }

            // Find the index of the sort with the matching Unicode codepoint
            let mut target_index = None;
            if let Some(target_char) = target_char {
                for i in 0..text_state.buffer.len() {
                    if let Some(sort) = text_state.buffer.get(i) {
                        if let Some(sort_codepoint) = sort.kind.codepoint() {
                            if sort_codepoint == target_char {
                                target_index = Some(i);
                                break;
                            }
                        }
                    }
                }
            }

            if let Some(index) = target_index {
                // Use the TextEditorState's activate_sort method
                if text_state.activate_sort(index) {
                    println!("✅ SUCCESS: Activated sort at index {} for Unicode U+{:04X}", index, unicode_codepoint);
                } else {
                    println!("❌ FAILED: Could not activate sort at index {} for Unicode U+{:04X}", index, unicode_codepoint);
                }
            } else {
                println!("⚠️ NOT FOUND: No existing sort found for Unicode U+{:04X} in text buffer", unicode_codepoint);
            }
        } else {
            println!("❌ No TextEditorState available for glyph activation");
        }
    }

    // Press '2' key to switch to Unicode U+0041 ('A') again (only glyph in test.ufo)
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        let unicode_codepoint = 0x0041u32; // 'A'
        println!("🎯 KEYBOARD TEST: Simulating TUI selection of Unicode U+{:04X}", unicode_codepoint);

        if let Some(ref mut text_state) = text_editor_state {
            let target_char = char::from_u32(unicode_codepoint);
            let mut target_index = None;

            if let Some(target_char) = target_char {
                for i in 0..text_state.buffer.len() {
                    if let Some(sort) = text_state.buffer.get(i) {
                        if let Some(sort_codepoint) = sort.kind.codepoint() {
                            if sort_codepoint == target_char {
                                target_index = Some(i);
                                break;
                            }
                        }
                    }
                }
            }

            if let Some(index) = target_index {
                if text_state.activate_sort(index) {
                    println!("✅ SUCCESS: Activated sort at index {} for Unicode U+{:04X}", index, unicode_codepoint);
                } else {
                    println!("❌ FAILED: Could not activate sort at index {} for Unicode U+{:04X}", index, unicode_codepoint);
                }
            } else {
                println!("⚠️ NOT FOUND: No existing sort found for Unicode U+{:04X}", unicode_codepoint);
            }
        }
    }
}

