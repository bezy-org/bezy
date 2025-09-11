//! Application initialization and configuration

use crate::core::cli::CliArgs;
use crate::core::io::gamepad::GamepadPlugin;
use crate::core::io::input::InputPlugin;
use crate::core::io::pointer::PointerPlugin;
use crate::core::settings::{BezySettings, DEFAULT_WINDOW_SIZE, WINDOW_TITLE};
use crate::core::state::GlyphNavigation;
use crate::editing::{FontEditorSystemSetsPlugin, SelectionPlugin, TextEditorPlugin, UndoPlugin};
use crate::rendering::{
    cameras::CameraPlugin, checkerboard::CheckerboardPlugin,
    zoom_aware_scaling::CameraResponsivePlugin, EntityPoolingPlugin, GlyphRenderingPlugin,
    MeshCachingPlugin, MetricsRenderingPlugin, SortHandleRenderingPlugin,
};
use crate::systems::{
    center_camera_on_startup_layout, create_startup_layout, exit_on_esc, load_fontir_font,
    BezySystems, CommandsPlugin, HarfBuzzShapingPlugin, InputConsumerPlugin, UiInteractionPlugin,
};
use crate::ui::edit_mode_toolbar::EditModeToolbarPlugin;
use crate::ui::file_menu::FileMenuPlugin;
use crate::ui::panes::coordinate_pane::CoordinatePanePlugin;
use crate::ui::panes::design_space::DesignSpacePlugin;
use crate::ui::panes::file_pane::FilePanePlugin;
use crate::ui::panes::glyph_pane::GlyphPanePlugin;
use crate::ui::theme::CurrentTheme;
#[cfg(debug_assertions)]
use crate::ui::themes::runtime_reload::RuntimeThemePlugin;
use anyhow::Result;
use bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy::prelude::*;
use bevy::winit::WinitSettings;

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
            // RE-ENABLED: HarfBuzz text shaping for RTL support
            .add(HarfBuzzShapingPlugin)
            .add(SelectionPlugin)
            .add(UndoPlugin)
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
            .add(DesignSpacePlugin)
            .add(FilePanePlugin)
            .add(GlyphPanePlugin)
            .add(CoordinatePanePlugin)
            .add(EditModeToolbarPlugin) // ✅ Includes ConfigBasedToolbarPlugin - handles all tools automatically
            .add(FileMenuPlugin)
            // ✅ NEW SYSTEM: All tools are now automatically registered via EditModeToolbarPlugin
            // No need for manual tool plugin registration - everything is handled by toolbar_config.rs
            .add(crate::tools::PenToolPlugin) // Re-enabled - pen tool needs its business logic plugin
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
    app.insert_resource(WinitSettings::desktop_app());

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
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
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
    info!("Adding plugin groups...");
    app.add_plugins((RenderingPluginGroup, EditorPluginGroup, CorePluginGroup));

    // Add runtime theme reload plugin for development
    #[cfg(debug_assertions)]
    app.add_plugins(RuntimeThemePlugin);

    info!("All plugin groups added successfully");
}

/// Add startup and exit systems
fn add_startup_and_exit_systems(app: &mut App) {
    app.add_systems(Startup, (load_fontir_font, create_startup_layout).chain())
        .add_systems(Update, (exit_on_esc, center_camera_on_startup_layout));
}
