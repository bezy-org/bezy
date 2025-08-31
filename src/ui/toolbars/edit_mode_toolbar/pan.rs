//! Pan tool for the edit mode toolbar
//!
//! This tool provides camera panning functionality, allowing users to navigate around the design space.
//! It integrates with the bevy_pancam system and supports temporary activation via spacebar.
//! When active, it enables "presentation mode" - hiding grid, metrics, and editing helpers for clean viewing.

use crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive;
use crate::ui::toolbars::edit_mode_toolbar::{EditModeSystem, EditTool, ToolRegistry};
use bevy::prelude::*;
use bevy::ui::Display;
use bevy_pancam::PanCam;

/// Resource to track when the pan tool is active for presentation mode
/// When pan tool is active, hide grid, metrics, panes, and editing helpers
#[derive(Resource, Default)]
pub struct PresentationMode {
    pub active: bool,
}

pub struct PanTool;

impl EditTool for PanTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "pan"
    }

    fn name(&self) -> &'static str {
        "Pan"
    }

    fn icon(&self) -> &'static str {
        "\u{E014}" // Pan/hand icon from the UI font
    }

    fn shortcut_key(&self) -> Option<char> {
        Some(' ') // Spacebar for temporary pan mode
    }

    fn default_order(&self) -> i32 {
        90 // Near the end, utility tool
    }

    fn description(&self) -> &'static str {
        "Pan and navigate the canvas"
    }

    fn supports_temporary_mode(&self) -> bool {
        true // Pan tool supports temporary activation with spacebar
    }

    fn update(&self, commands: &mut Commands) {
        // Ensure select mode is disabled while in pan mode
        commands.insert_resource(SelectModeActive(false));
        // Enable presentation mode
        commands.insert_resource(PresentationMode { active: true });
    }

    fn on_enter(&self) {
        // Note: PanCam enabling is handled by the toggle_pancam_on_mode_change system
        info!("Entered Pan tool - camera panning should be enabled, presentation mode active");
    }

    fn on_exit(&self) {
        // Note: PanCam disabling is handled by the toggle_pancam_on_mode_change system
        info!("Exited Pan tool - camera panning should be disabled, presentation mode disabled");
    }
}

// Legacy compatibility struct
pub struct PanMode;

impl EditModeSystem for PanMode {
    fn update(&self, commands: &mut Commands) {
        // Ensure select mode is disabled while in pan mode
        commands.insert_resource(SelectModeActive(false));
    }

    fn on_enter(&self) {
        // Enable panning on all PanCam components
        info!("Entering pan mode - enabling camera panning");
    }

    fn on_exit(&self) {
        // Disable panning on all PanCam components
        info!("Exiting pan mode - disabling camera panning");
    }
}

// System to enable/disable the PanCam component when entering/exiting pan mode
pub fn toggle_pancam_on_mode_change(
    mut query: Query<&mut PanCam>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
) {
    // Only run this system when the current tool changes
    if current_tool.is_changed() {
        let should_enable = current_tool.get_current() == Some("pan");

        for mut pancam in query.iter_mut() {
            // Only log if we're actually changing the state
            if pancam.enabled != should_enable {
                pancam.enabled = should_enable;
                if should_enable {
                    info!("PanCam enabled");
                } else {
                    info!("PanCam disabled");
                }
            }
        }
    }
}

// System to manage presentation mode based on current tool
pub fn manage_presentation_mode(
    mut commands: Commands,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    presentation_mode: Option<Res<PresentationMode>>,
) {
    // Only run this system when the current tool changes
    if current_tool.is_changed() {
        let is_pan_active = current_tool.get_current() == Some("pan");
        let current_mode = presentation_mode.as_ref().is_some_and(|pm| pm.active);

        info!(
            "🎭 TOOL CHANGED: current_tool={:?}, is_pan_active={}, current_presentation_mode={}",
            current_tool.get_current(),
            is_pan_active,
            current_mode
        );

        if is_pan_active {
            info!("🎭 ACTIVATING PRESENTATION MODE - hiding grid, metrics, and editing helpers");
            commands.insert_resource(PresentationMode { active: true });
        } else {
            info!("🎭 DEACTIVATING PRESENTATION MODE - showing normal editing interface");
            commands.insert_resource(PresentationMode { active: false });
        }
    }
}

// Type alias for complex pane query to reduce complexity
type PaneQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut Node, Option<&'static Name>),
    Or<(
        With<crate::ui::panes::coordinate_pane::CoordinatePane>,
        With<crate::ui::panes::file_pane::FilePane>,
        With<crate::ui::panes::glyph_pane::GlyphPane>,
    )>,
>;

// System to hide/show panes based on presentation mode - DISPLAY APPROACH
pub fn manage_pane_visibility(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    mut pane_query: PaneQuery,
) {
    // Always log current state for debugging
    let spacebar_pressed = keyboard_input.pressed(KeyCode::Space);
    let pan_tool_selected = current_tool.get_current() == Some("pan");
    let should_hide_panes = spacebar_pressed || pan_tool_selected;

    // Log every few frames for debugging
    static mut DEBUG_COUNTER: u32 = 0;
    unsafe {
        DEBUG_COUNTER += 1;
        if DEBUG_COUNTER % 60 == 0 {
            // Log every second at 60fps
            info!(
                "🎭 PANE DEBUG: spacebar={}, pan_tool={}, current_tool={:?}, should_hide={}",
                spacebar_pressed,
                pan_tool_selected,
                current_tool.get_current(),
                should_hide_panes
            );

            let pane_count = pane_query.iter().count();
            info!("🎭 PANE DEBUG: Found {} total panes", pane_count);
            for (_entity, node, name) in pane_query.iter() {
                let pane_name = name.map(|n| n.as_str()).unwrap_or("Unknown");
                info!("🎭 PANE DEBUG: '{}' display: {:?}", pane_name, node.display);
            }
        }
    }

    let target_display = if should_hide_panes {
        Display::None
    } else {
        Display::Flex
    };

    // Update the main pane entities using Display property
    for (_entity, mut node, name) in pane_query.iter_mut() {
        let pane_name = name.map(|n| n.as_str()).unwrap_or("Unknown");

        if node.display != target_display {
            info!(
                "🎭 MAIN PANE UPDATE: '{}' from {:?} to {:?}",
                pane_name, node.display, target_display
            );
            node.display = target_display;
        }
    }
}

/// Plugin for the Pan tool
pub struct PanToolPlugin;

impl Plugin for PanToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PresentationMode>()
            .add_systems(Startup, register_pan_tool)
            .add_systems(
                Update,
                (
                    toggle_pancam_on_mode_change,
                    manage_presentation_mode,
                    manage_pane_visibility,
                ),
            );
    }
}

fn register_pan_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(PanTool));
}
