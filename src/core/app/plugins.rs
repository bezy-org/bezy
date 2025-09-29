//! Plugin group definitions for the Bezy application
//!
//! Organized into logical groups for clarity and maintainability

use bevy::app::{PluginGroup, PluginGroupBuilder};

/// Plugin group for core application functionality
#[derive(Default)]
pub struct CorePluginGroup;

impl PluginGroup for CorePluginGroup {
    fn build(self) -> PluginGroupBuilder {
        use crate::editing::{FontEditorSystemSetsPlugin, SelectionPlugin, TextEditorPlugin};
        use crate::io::{gamepad::GamepadPlugin, input::InputPlugin, pointer::PointerPlugin};
        use crate::systems::{
            BezySystems, CommandsPlugin, InputConsumerPlugin, TextShapingPlugin,
            UiInteractionPlugin,
        };

        PluginGroupBuilder::start::<Self>()
            .add(PointerPlugin)
            .add(InputPlugin)
            .add(GamepadPlugin)
            .add(InputConsumerPlugin)
            .add(FontEditorSystemSetsPlugin) // Must be added before other font editor plugins
            .add(TextEditorPlugin)
            .add(TextShapingPlugin) // Unified text shaping for RTL support
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
        use crate::rendering::{
            cameras::CameraPlugin, checkerboard::CheckerboardPlugin,
            sort_renderer::SortLabelRenderingPlugin, zoom_aware_scaling::CameraResponsivePlugin,
            EntityPoolingPlugin, GlyphRenderingPlugin, MeshCachingPlugin, MetricsRenderingPlugin,
            PostEditingRenderingPlugin, SortHandleRenderingPlugin,
        };

        PluginGroupBuilder::start::<Self>()
            .add(PostEditingRenderingPlugin) // Must be first to configure SystemSets
            .add(CameraPlugin)
            .add(CameraResponsivePlugin)
            .add(CheckerboardPlugin)
            .add(EntityPoolingPlugin)
            .add(MeshCachingPlugin)
            .add(MetricsRenderingPlugin)
            .add(SortHandleRenderingPlugin)
            .add(SortLabelRenderingPlugin) // Sort label rendering (text labels)
            .add(GlyphRenderingPlugin)
    }
}

/// Plugin group for editor UI
#[derive(Default)]
pub struct EditorPluginGroup;

impl PluginGroup for EditorPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        use crate::ui::edit_mode_toolbar::EditModeToolbarPlugin;
        use crate::ui::file_menu::FileMenuPlugin;
        use crate::ui::panes::coordinate_pane::CoordinatePanePlugin;
        use crate::ui::panes::file_pane::FilePanePlugin;
        use crate::ui::panes::glyph_pane::GlyphPanePlugin;

        PluginGroupBuilder::start::<Self>()
            .add(FilePanePlugin)
            .add(GlyphPanePlugin)
            .add(CoordinatePanePlugin)
            .add(EditModeToolbarPlugin) // Handles all tools automatically
            .add(FileMenuPlugin)
            // Tool business logic plugins
            .add(crate::tools::PenToolPlugin)
            .add(crate::tools::SelectToolPlugin)
    }
}
