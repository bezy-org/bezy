//! Text Tool - Sort placement and text editing
//!
//! The text tool allows users to place sorts by clicking in world-space.
//! Sorts can be placed and edited with different modes:
//! - LTR Text Mode: left-to-right gap buffer layout in a text-editor-like buffer
//! - RTL Text Mode: right-to-left gap buffer layout in a text-editor-like buffer
//! - Insert mode: a cursor mode for basic text editing LTR and RTL text buffers
//! - Freeform mode: Sorts are positioned freely in the world-space
//! - Vim mode: LRT and RTL sorts are edited with vim-like keybindings

#![allow(clippy::manual_map)]

use crate::core::settings::BezySettings;
use crate::core::state::{
    AppState, FontIRAppState, GlyphNavigation, SortLayoutMode, TextEditorState,
    TextModeConfig,
};
use bevy::input::ButtonState;
use bevy::log::info;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::rendering::cameras::DesignCamera;
use crate::rendering::checkerboard::calculate_dynamic_grid_size;
use crate::ui::theme::*;
use crate::ui::theme::{PRESSED_BUTTON_COLOR, SORT_ACTIVE_METRICS_COLOR};
use crate::ui::themes::{CurrentTheme, ToolbarBorderRadius};
use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};

/// Resource to track if text mode is active
#[derive(Resource, Default)]
pub struct TextModeActive(pub bool);

/// Resource to track text mode state for sort placement
#[derive(Resource, Default)]
pub struct TextModeState {
    /// Current cursor position in world-space coordinates
    pub cursor_position: Option<Vec2>,
    /// Whether we're showing a sort placement preview
    pub showing_preview: bool,
}

/// Text placement modes for the submenu
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Resource)]
pub enum TextPlacementMode {
    LTRText,
    RTLText,
    #[default]
    Insert,
    Freeform,
}

impl TextPlacementMode {
    /// Get the icon for each text submenu mode
    pub fn get_icon(&self) -> &'static str {
        match self {
            TextPlacementMode::LTRText => "\u{E004}",
            TextPlacementMode::RTLText => "\u{F004}",
            TextPlacementMode::Insert => "\u{E017}",
            TextPlacementMode::Freeform => "\u{E006}",
        }
    }

    /// Get a human-readable name for this placement mode
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            TextPlacementMode::LTRText => "LTR Text",
            TextPlacementMode::RTLText => "RTL Text",
            TextPlacementMode::Insert => "Insert",
            TextPlacementMode::Freeform => "Freeform",
        }
    }

    /// Convert to SortLayoutMode
    pub fn to_sort_layout_mode(&self) -> SortLayoutMode {
        match self {
            TextPlacementMode::LTRText => SortLayoutMode::LTRText,
            TextPlacementMode::RTLText => SortLayoutMode::RTLText,
            TextPlacementMode::Insert => SortLayoutMode::LTRText,
            TextPlacementMode::Freeform => SortLayoutMode::Freeform,
        }
    }
}

/// Component to mark text submenu buttons
#[derive(Component)]
pub struct TextSubMenuButton;

/// Component to associate a button with its placement mode
#[derive(Component)]
pub struct TextModeButton {
    pub mode: TextPlacementMode,
}

/// Resource to track the current text placement mode
#[derive(Resource, Default)]
pub struct CurrentTextPlacementMode(pub TextPlacementMode);

pub struct TextTool;

impl EditTool for TextTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "text"
    }

    fn name(&self) -> &'static str {
        "Text"
    }

    fn icon(&self) -> &'static str {
        "\u{E017}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('t')
    }

    fn default_order(&self) -> i32 {
        40 // After drawing tools, around position 5
    }

    fn description(&self) -> &'static str {
        "Place text and create sorts in text mode or freeform mode"
    }

    fn update(&self, _commands: &mut Commands) {
        // Text tool behavior is handled by dedicated systems
    }

    fn on_enter(&self) {
        info!("Entered Text tool - Enhanced features:");
        info!("• Click to place sorts, type letters to add glyphs");
        info!("• Tab to switch Text mode/Freeform modes");
        info!("• 1-9 keys to switch glyphs, F1 for help");
        info!("• Arrow keys for navigation, Ctrl+S to show text mode");
    }

    fn on_exit(&self) {
        info!("Exited Text tool");
    }
}

// --------- Plugins and Registration -----------

pub struct TextModePlugin;

impl Plugin for TextModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextModeActive>()
            .init_resource::<TextModeState>()
            .init_resource::<CurrentTextPlacementMode>()
            .init_resource::<TextModeConfig>()
            .add_systems(
                Update,
                (
                    update_text_mode_active,
                    handle_text_tool_shortcuts,
                    handle_text_mode_cursor,
                    // handle_text_mode_mouse_clicks, // DISABLED: Duplicate of handle_sort_placement_input in TextEditorPlugin
                    handle_text_mode_keyboard,
                    render_sort_preview,
                    reset_text_mode_when_inactive,
                    handle_text_mode_selection,
                    toggle_text_submenu_visibility,
                )
                    .chain(),
            );
    }
}

pub struct TextToolPlugin;

impl Plugin for TextToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_text_tool)
            .add_systems(PostStartup, spawn_text_submenu)
            .add_plugins(TextModePlugin);
    }
}

fn register_text_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(TextTool));
}

// --------- UI Systems -----------

/// Helper function to spawn a single text mode button using the unified system
fn spawn_text_mode_button(
    parent: &mut ChildSpawnerCommands,
    mode: TextPlacementMode,
    asset_server: &Res<AssetServer>,
    theme: &Res<CurrentTheme>,
) {
    // Use the unified toolbar button creation system for consistent styling with hover text
    crate::ui::toolbars::edit_mode_toolbar::ui::create_unified_toolbar_button_with_hover_text(
        parent,
        mode.get_icon(),
        Some(mode.display_name()), // Show the mode name on hover
        (TextSubMenuButton, TextModeButton { mode }),
        asset_server,
        theme,
    );
}

pub fn spawn_text_submenu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme: Res<CurrentTheme>,
) {
    let modes = [
        TextPlacementMode::Insert,
        TextPlacementMode::LTRText,
        TextPlacementMode::RTLText,
        TextPlacementMode::Freeform,
    ];

    // Create the parent submenu node (left-aligned to match main toolbar)
    let submenu_node = Node {
        position_type: PositionType::Absolute,
        top: Val::Px(TOOLBAR_CONTAINER_MARGIN + 74.0),
        left: Val::Px(TOOLBAR_CONTAINER_MARGIN),  // Now on the left to match toolbar
        flex_direction: FlexDirection::Row,
        padding: UiRect::all(Val::Px(TOOLBAR_PADDING)),
        margin: UiRect::all(Val::ZERO),
        row_gap: Val::Px(TOOLBAR_PADDING),
        display: Display::None,
        ..default()
    };

    // Spawn the submenu with all buttons
    commands
        .spawn((submenu_node, Name::new("TextSubMenu")))
        .with_children(|parent| {
            for mode in modes {
                spawn_text_mode_button(parent, mode, &asset_server, &theme);
            }
        });

    info!("Spawned text submenu with {} modes", modes.len());
}

pub fn handle_text_mode_selection(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &TextModeButton,
            Entity,
        ),
        With<TextSubMenuButton>,
    >,
    mut current_mode: ResMut<CurrentTextPlacementMode>,
    mut text_mode_config: ResMut<TextModeConfig>,
    children_query: Query<&Children>,
    mut text_query: Query<&mut TextColor>,
) {
    for (interaction, mut color, mut border_color, mode_button, entity) in
        &mut interaction_query
    {
        let is_current_mode = current_mode.0 == mode_button.mode;

        if *interaction == Interaction::Pressed && !is_current_mode {
            current_mode.0 = mode_button.mode;
            text_mode_config.default_placement_mode =
                mode_button.mode.to_sort_layout_mode();
            info!("Switched to text placement mode: {:?}", mode_button.mode);
        }

        // Use the unified button color system for consistent appearance with main toolbar
        crate::ui::toolbars::edit_mode_toolbar::ui::update_unified_button_colors(
            *interaction,
            is_current_mode,
            &mut color,
            &mut border_color,
        );
        
        // Use the unified text color system for consistent icon colors with main toolbar
        crate::ui::toolbars::edit_mode_toolbar::ui::update_unified_button_text_colors(
            entity,
            is_current_mode,
            &children_query,
            &mut text_query,
        );
    }
}

pub fn toggle_text_submenu_visibility(
    mut submenu_query: Query<(&mut Node, &Name)>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
) {
    let is_text_tool_active = current_tool.get_current() == Some("text");
    for (mut style, name) in submenu_query.iter_mut() {
        if name.as_str() == "TextSubMenu" {
            style.display = if is_text_tool_active {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

pub fn update_text_mode_active(
    mut text_mode_active: ResMut<TextModeActive>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
) {
    let is_text_mode = current_tool.get_current() == Some("text");
    if text_mode_active.0 != is_text_mode {
        text_mode_active.0 = is_text_mode;
        debug!("Text mode active state changed: {}", is_text_mode);
    }
}

// -- Input, Preview, and Placement Logic --

pub fn handle_text_mode_cursor(
    text_mode_active: Res<TextModeActive>,
    mut text_mode_state: ResMut<TextModeState>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
    pointer_info: Res<crate::core::io::pointer::PointerInfo>,
) {
    if !text_mode_active.0 {
        return;
    }
    if ui_hover_state.is_hovering_ui {
        return;
    }
    let cursor_moved = !cursor_moved_events.is_empty();
    cursor_moved_events.clear();
    let raw_cursor_world_pos = pointer_info.design.to_raw();
    let position_changed =
        text_mode_state.cursor_position != Some(raw_cursor_world_pos);
    text_mode_state.cursor_position = Some(raw_cursor_world_pos);
    text_mode_state.showing_preview = true;
    if cursor_moved || position_changed {
        debug!(
            "Text mode cursor updated: raw=({:.1}, {:.1})",
            raw_cursor_world_pos.x, raw_cursor_world_pos.y
        );
    }
}

#[allow(clippy::too_many_arguments)]
/// Handle mouse clicks for sort placement in text mode
#[allow(clippy::too_many_arguments)]
pub fn handle_text_mode_mouse_clicks(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    text_mode_active: Res<TextModeActive>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
    pointer_info: Res<crate::core::io::pointer::PointerInfo>,
    mut current_placement_mode: ResMut<CurrentTextPlacementMode>,
    mut text_editor_state: ResMut<TextEditorState>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    glyph_navigation: Res<GlyphNavigation>,
    mut camera_query: Query<&mut Projection, With<DesignCamera>>,
) {
    // Check which state is available
    let (using_fontir, glyph_names, advance_width) = if let Some(fontir_state) =
        fontir_app_state.as_ref()
    {
        let names = fontir_state.get_glyph_names();
        let advance = fontir_state.get_glyph_advance_width("a"); // Default glyph for advance
        (true, names, advance)
    } else if let Some(app_state) = app_state.as_ref() {
        let names: Vec<String> =
            app_state.workspace.font.glyphs.keys().cloned().collect();
        let advance = app_state
            .workspace
            .font
            .glyphs
            .get("a")
            .map(|g| g.advance_width as f32)
            .unwrap_or(600.0);
        (false, names, advance)
    } else {
        warn!("Text mode mouse clicks disabled - neither AppState nor FontIR available");
        return;
    };

    // Debug text mode state
    debug!("Text mode click handler: text_mode_active={}, current_tool={:?}, ui_hover={}", 
           text_mode_active.0, current_tool.get_current(), ui_hover_state.is_hovering_ui);

    // Only handle clicks when text mode is active and we're the active tool
    if !text_mode_active.0 || current_tool.get_current() != Some("text") {
        debug!(
            "Text mode click handler: early return - not active or wrong tool"
        );
        return;
    }

    // Don't handle clicks when hovering over UI
    if ui_hover_state.is_hovering_ui {
        debug!("Text mode click handler: early return - hovering over UI");
        return;
    }

    // Handle left mouse click
    if mouse_button_input.just_pressed(MouseButton::Left) {
        debug!("Text mode: Left mouse clicked - checking for sort handle clicks first");

        // First check if we're clicking on a sort handle (regardless of placement mode)
        let world_position = pointer_info.design.to_raw();
        let handle_tolerance = 50.0;

        // Get font metrics from appropriate source
        let font_metrics = if using_fontir {
            if let Some(fontir_state) = fontir_app_state.as_ref() {
                let metrics = fontir_state.get_font_metrics();
                Some(crate::core::state::FontMetrics {
                    units_per_em: metrics.units_per_em as f64,
                    ascender: metrics.ascender.map(|a| a as f64),
                    descender: metrics.descender.map(|d| d as f64),
                    line_height: metrics.line_gap.unwrap_or(0.0) as f64,
                    x_height: None,
                    cap_height: None,
                    italic_angle: None,
                })
            } else {
                None
            }
        } else if let Some(app_state) = app_state.as_ref() {
            Some(app_state.workspace.info.metrics.clone())
        } else {
            None
        };

        let font_metrics_ref = font_metrics.as_ref();

        if let Some(clicked_sort_index) = text_editor_state
            .find_sort_handle_at_position(
                world_position,
                handle_tolerance,
                font_metrics_ref,
            )
        {
            info!(
                "Clicked on sort handle at index {}, letting selection system handle activation",
                clicked_sort_index
            );
            // Don't place a new sort when clicking on a handle
            // Let the selection system handle activation through auto_activate_selected_sorts
            return;
        }

        // If not clicking on a handle, proceed with sort placement
        debug!("Not clicking on handle - attempting sort placement");
        let did_place_text_sort = handle_text_mode_sort_placement(
            &mut text_editor_state,
            &glyph_navigation,
            &current_placement_mode,
            &pointer_info,
            &mut camera_query,
            &glyph_names,
            advance_width,
        );

        // If we placed a text sort, automatically switch to Insert mode
        if did_place_text_sort
            && (current_placement_mode.0 == TextPlacementMode::LTRText
                || current_placement_mode.0 == TextPlacementMode::RTLText)
        {
            current_placement_mode.0 = TextPlacementMode::Insert;
            info!("Auto-switched to Insert mode after placing text sort");
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_text_mode_sort_placement(
    _text_editor_state: &mut ResMut<TextEditorState>,
    _glyph_navigation: &Res<GlyphNavigation>,
    _current_placement_mode: &CurrentTextPlacementMode,
    _pointer_info: &Res<crate::core::io::pointer::PointerInfo>,
    _camera_query: &mut Query<&mut Projection, With<DesignCamera>>,
    _glyph_names: &[String],
    _default_advance_width: f32,
) -> bool {
    // DISABLED: This function was creating duplicate sorts
    // Sort placement is now handled centrally by handle_sort_placement_input in TextEditorPlugin
    false
}

// -- Preview Rendering and Keyboard Handling --

#[allow(clippy::too_many_arguments)]
pub fn render_sort_preview(
    _gizmos: Gizmos,
    text_mode_active: Res<TextModeActive>,
    _text_mode_state: Res<TextModeState>,
    _text_editor_state: Option<Res<TextEditorState>>,
    current_placement_mode: Res<CurrentTextPlacementMode>,
    glyph_navigation: Res<GlyphNavigation>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<FontIRAppState>>,
    pointer_info: Res<crate::core::io::pointer::PointerInfo>,
    camera_query: Query<&Projection, With<DesignCamera>>,
    mut preview_metrics_state: ResMut<
        crate::rendering::metrics::PreviewMetricsState,
    >,
) {
    info!("[PREVIEW] Entered render_sort_preview - text_mode_active: {}, placement_mode: {:?}", text_mode_active.0, current_placement_mode.0);
    if !text_mode_active.0 {
        preview_metrics_state.active = false;
        debug!("[PREVIEW] Early return: text_mode_active is false - disabled metrics preview");
        return;
    }
    if current_placement_mode.0 == TextPlacementMode::Insert {
        preview_metrics_state.active = false;
        info!("[PREVIEW] DISABLING preview: placement mode is Insert");
        return;
    }

    let zoom_scale = camera_query
        .single()
        .map(|p| {
            if let Projection::Orthographic(ortho) = p {
                ortho.scale
            } else {
                1.0
            }
        })
        .unwrap_or(1.0);
    let grid_size = calculate_dynamic_grid_size(zoom_scale);
    let snapped_position =
        (pointer_info.design.to_raw() / grid_size).round() * grid_size;
    debug!(
        "[PREVIEW] Placement mode: {:?}, snapped_position: ({:.1}, {:.1})",
        current_placement_mode.0, snapped_position.x, snapped_position.y
    );

    let _preview_color = Color::srgb(1.0, 0.5, 0.0).with_alpha(0.8);

    // Determine the appropriate preview glyph based on placement mode
    let preview_glyph_name = match current_placement_mode.0 {
        TextPlacementMode::RTLText => "alef-ar".to_string(), // Arabic Alef for RTL
        _ => glyph_navigation.current_glyph.clone().unwrap_or_else(|| "a".to_string()), // Current glyph or 'a' for LTR
    };
    
    debug!("[PREVIEW] Using preview glyph: {} (placement mode: {:?})", preview_glyph_name, current_placement_mode.0);

    // Try FontIR first, then fall back to AppState
    if let Some(fontir_state) = &fontir_app_state {
        debug!("[PREVIEW] Using FontIR for preview");
        if let Some(_glyph_paths) =
            fontir_state.get_glyph_paths_with_edits(&preview_glyph_name)
        {
            debug!(
                "[PREVIEW] Drawing FontIR preview for glyph '{}' at ({:.1}, {:.1})",
                preview_glyph_name, snapped_position.x, snapped_position.y
            );

            // TODO: Implement mesh-based glyph preview
            // For now, just show metrics without glyph outline

            // Update mesh-based preview metrics state for FontIR
            let advance_width =
                fontir_state.get_glyph_advance_width(&preview_glyph_name);
            preview_metrics_state.active = true;
            preview_metrics_state.position = snapped_position;
            preview_metrics_state.glyph_name = preview_glyph_name.clone();
            preview_metrics_state.advance_width = advance_width;
            preview_metrics_state.color =
                PRESSED_BUTTON_COLOR.with_alpha(0.8);
            debug!(
                "[PREVIEW] Updated mesh-based preview metrics for '{}' at ({:.1}, {:.1})",
                preview_glyph_name, snapped_position.x, snapped_position.y
            );
        } else {
            // No glyph paths found - disable preview metrics
            preview_metrics_state.active = false;
            debug!(
                "[PREVIEW] No FontIR glyph_paths found for '{}', cannot draw preview - disabled metrics",
                preview_glyph_name
            );
        }
    } else if let Some(app_state) = &app_state {
        debug!("[PREVIEW] Using AppState for preview");
        if let Some(glyph_data) =
            app_state.workspace.font.glyphs.get(&preview_glyph_name)
        {
            debug!(
                "[PREVIEW] Drawing AppState preview for glyph '{}' at ({:.1}, {:.1})",
                preview_glyph_name, snapped_position.x, snapped_position.y
            );

            // TODO: Implement mesh-based glyph preview
            // For now, just show metrics without glyph outline

            // Update mesh-based preview metrics state for AppState
            let advance_width = glyph_data.advance_width as f32;
            preview_metrics_state.active = true;
            preview_metrics_state.position = snapped_position;
            preview_metrics_state.glyph_name = preview_glyph_name.clone();
            preview_metrics_state.advance_width = advance_width;
            preview_metrics_state.color =
                PRESSED_BUTTON_COLOR.with_alpha(0.8);
            debug!(
                "[PREVIEW] Updated mesh-based preview metrics for '{}' at ({:.1}, {:.1})",
                preview_glyph_name, snapped_position.x, snapped_position.y
            );
        } else {
            // No glyph data found - disable preview metrics
            preview_metrics_state.active = false;
            debug!(
                "[PREVIEW] No AppState glyph_data found for '{}', cannot draw preview - disabled metrics",
                preview_glyph_name
            );
        }
    } else {
        // No font state available - disable preview metrics
        preview_metrics_state.active = false;
        debug!("[PREVIEW] Neither FontIR nor AppState available, cannot draw preview - disabled metrics preview");
    }
}

// -- Keyboard Input and Helpers --

pub fn handle_text_tool_shortcuts(
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut current_tool: ResMut<
        crate::ui::toolbars::edit_mode_toolbar::CurrentTool,
    >,
    mut current_placement_mode: ResMut<CurrentTextPlacementMode>,
    mut text_mode_config: ResMut<TextModeConfig>,
    text_editor_state: Option<Res<TextEditorState>>,
    text_mode_active: Res<TextModeActive>,
) {
    // Check if single-char hotkeys should be disabled for text input
    let should_disable = super::keyboard_utils::should_disable_single_char_hotkeys(
        Some(&text_mode_active),
        Some(&current_placement_mode),
    );
    
    // Only activate text tool with 'T' key when not in insert mode
    if keyboard_input.just_pressed(KeyCode::KeyT)
        && current_tool.get_current() != Some("text")
        && !should_disable
    {
        current_tool.switch_to("text");
        info!("Activated text tool via keyboard shortcut");
        keyboard_input.clear_just_pressed(KeyCode::KeyT);
    }
    if current_tool.get_current() == Some("text")
        && keyboard_input.just_pressed(KeyCode::Tab)
    {
        let new_mode = match current_placement_mode.0 {
            TextPlacementMode::LTRText => TextPlacementMode::RTLText,
            TextPlacementMode::RTLText => TextPlacementMode::Insert,
            TextPlacementMode::Insert => TextPlacementMode::Freeform,
            TextPlacementMode::Freeform => TextPlacementMode::LTRText,
        };
        current_placement_mode.0 = new_mode;
        text_mode_config.default_placement_mode =
            new_mode.to_sort_layout_mode();
        info!("Switched text placement mode to: {:?}", new_mode);
        keyboard_input.clear_just_pressed(KeyCode::Tab);
    }
    if current_tool.get_current() == Some("text")
        && keyboard_input.just_pressed(KeyCode::KeyS)
        && (keyboard_input.pressed(KeyCode::ControlLeft)
            || keyboard_input.pressed(KeyCode::ControlRight))
    {
        if let Some(text_editor_state) = text_editor_state {
            let buffer_text: String = text_editor_state
                .buffer
                .iter()
                .map(|entry| entry.kind.glyph_name().to_string())
                .collect::<Vec<String>>()
                .join(" ");
            info!("Current text buffer: {}", buffer_text);
            info!("Buffer length: {} sorts", text_editor_state.buffer.len());
            info!("Cursor position: {}", text_editor_state.cursor_position);
            info!("Current mode: {:?}", current_placement_mode.0);
        }
    }
    if current_tool.get_current() == Some("text")
        && keyboard_input.just_pressed(KeyCode::F1)
    {
        info!("=== TEXT TOOL HELP ===");
        info!("T - Activate text tool");
        info!("Tab - Switch between Text mode/Insert/Freeform modes");
        info!("TEXT MODE:");
        info!("  • Click to place glyphs");
        info!("  • Type letters to create sorts");
        info!("  • Arrow keys for navigation");
        info!("INSERT MODE:");
        info!("  • Arrow keys to move cursor");
        info!("  • Type to insert text at cursor");
        info!("  • Backspace/Delete to edit text");
        info!("  • No sort placement preview");
        info!("FREEFORM MODE:");
        info!("  • Click to place glyphs freely");
        info!("  • Type letters to create sorts");
        info!("1-9 - Switch to glyph by number");
        info!("Home/End - Go to start/end (Insert mode)");
        info!("Ctrl+S - Show current text buffer");
        info!("Escape - Exit text tool");
        info!("F1 - Show this help");
        info!("====================");
    }
    if current_tool.get_current() == Some("text")
        && keyboard_input.just_pressed(KeyCode::Escape)
    {
        if let Some(previous_tool) = current_tool.get_previous() {
            current_tool.switch_to(previous_tool);
            info!(
                "Exited text tool via Escape key, returned to: {}",
                previous_tool
            );
        } else {
            current_tool.switch_to("select");
            info!("Exited text tool via Escape key, returned to select tool");
        }
    }
}

pub fn reset_text_mode_when_inactive(
    text_mode_active: Res<TextModeActive>,
    mut text_mode_state: ResMut<TextModeState>,
) {
    if !text_mode_active.0 {
        text_mode_state.cursor_position = None;
        text_mode_state.showing_preview = false;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_text_mode_keyboard(
    text_mode_active: Res<TextModeActive>,
    current_placement_mode: Res<CurrentTextPlacementMode>,
    mut text_editor_state: ResMut<TextEditorState>,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<FontIRAppState>>,
    mut glyph_navigation: ResMut<GlyphNavigation>,
    _text_mode_state: Res<TextModeState>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    // Add query to check for selected points
    selected_points: Query<
        Entity,
        With<crate::editing::selection::components::Selected>,
    >,
) {
    if !text_mode_active.0 || current_tool.get_current() != Some("text") {
        return;
    }

    // Get font data from either AppState or FontIR
    let font_has_glyph: Box<dyn Fn(&str) -> bool> = if let Some(app_state) =
        app_state.as_ref()
    {
        Box::new(move |glyph_name: &str| -> bool {
            app_state.workspace.font.glyphs.contains_key(glyph_name)
        })
    } else if let Some(fontir_state) = fontir_app_state.as_ref() {
        Box::new(move |glyph_name: &str| -> bool {
            fontir_state.get_glyph(glyph_name).is_some()
        })
    } else {
        warn!("Text mode keyboard disabled - neither AppState nor FontIR available");
        return;
    };

    // Check if there are any selected points - if so, don't handle arrow keys
    // This gives priority to the nudge system
    let has_selected_points = !selected_points.is_empty();
    if has_selected_points {
        debug!("[TEXT_TOOLBAR] Skipping arrow key handling - {} selected points found", selected_points.iter().count());
        return;
    }

    // text_editor_state is now available directly as ResMut

    if current_placement_mode.0 == TextPlacementMode::LTRText
        || current_placement_mode.0 == TextPlacementMode::RTLText
    {
        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            text_editor_state.move_cursor_left();
            debug!(
                "Text mode: moved cursor left to position {}",
                text_editor_state.cursor_position
            );
            keyboard_input.clear_just_pressed(KeyCode::ArrowLeft);
        }
        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            text_editor_state.move_cursor_right();
            debug!(
                "Text mode: moved cursor right to position {}",
                text_editor_state.cursor_position
            );
            keyboard_input.clear_just_pressed(KeyCode::ArrowRight);
        }
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            text_editor_state.move_cursor_up_multiline();
            debug!(
                "Text mode: moved cursor up to position {}",
                text_editor_state.cursor_position
            );
            keyboard_input.clear_just_pressed(KeyCode::ArrowUp);
        }
        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            text_editor_state.move_cursor_down_multiline();
            debug!(
                "Text mode: moved cursor down to position {}",
                text_editor_state.cursor_position
            );
            keyboard_input.clear_just_pressed(KeyCode::ArrowDown);
        }
        if keyboard_input.just_pressed(KeyCode::Home) {
            text_editor_state.move_cursor_to(0);
            debug!("Text mode: moved cursor to beginning");
            keyboard_input.clear_just_pressed(KeyCode::Home);
        }
        if keyboard_input.just_pressed(KeyCode::End) {
            let end_position = text_editor_state.buffer.len();
            text_editor_state.move_cursor_to(end_position);
            debug!("Text mode: moved cursor to end");
            keyboard_input.clear_just_pressed(KeyCode::End);
        }
        if keyboard_input.just_pressed(KeyCode::Delete) {
            text_editor_state.delete_sort_at_cursor();
            debug!("Text mode: deleted sort at cursor position");
            keyboard_input.clear_just_pressed(KeyCode::Delete);
        }
        // NOTE: Backspace handling moved to Unicode input system to avoid duplicate deletion
        // The Unicode input system (handle_unicode_text_input) handles Key::Backspace
        // and should be the single source of truth for text input events
    }

    // Handle Insert mode cursor navigation
    if current_placement_mode.0 == TextPlacementMode::Insert {
        debug!("Checking Insert mode keyboard input...");
        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            info!("Arrow left pressed in Insert mode");
            text_editor_state.move_cursor_left();
            info!("Insert mode: moved cursor left");
            debug!(
                "Insert mode: moved cursor left to position {}",
                text_editor_state.cursor_position
            );
            keyboard_input.clear_just_pressed(KeyCode::ArrowLeft);
        }
        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            text_editor_state.move_cursor_right();
            debug!(
                "Insert mode: moved cursor right to position {}",
                text_editor_state.cursor_position
            );
            keyboard_input.clear_just_pressed(KeyCode::ArrowRight);
        }
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            text_editor_state.move_cursor_up_multiline();
            debug!(
                "Insert mode: moved cursor up to position {}",
                text_editor_state.cursor_position
            );
            keyboard_input.clear_just_pressed(KeyCode::ArrowUp);
        }
        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            text_editor_state.move_cursor_down_multiline();
            debug!(
                "Insert mode: moved cursor down to position {}",
                text_editor_state.cursor_position
            );
            keyboard_input.clear_just_pressed(KeyCode::ArrowDown);
        }
        if keyboard_input.just_pressed(KeyCode::Home) {
            text_editor_state.move_cursor_to(0);
            debug!("Insert mode: moved cursor to beginning");
            keyboard_input.clear_just_pressed(KeyCode::Home);
        }
        if keyboard_input.just_pressed(KeyCode::End) {
            let end_position = text_editor_state.buffer.len();
            text_editor_state.move_cursor_to(end_position);
            debug!("Insert mode: moved cursor to end");
            keyboard_input.clear_just_pressed(KeyCode::End);
        }
        if keyboard_input.just_pressed(KeyCode::Delete) {
            text_editor_state.delete_sort_at_cursor();
            debug!("Insert mode: deleted sort at cursor position");
            keyboard_input.clear_just_pressed(KeyCode::Delete);
        }
        // NOTE: Backspace handling moved to Unicode input system to avoid duplicate deletion
        // The Unicode input system (handle_unicode_text_input) handles Key::Backspace
        // and should be the single source of truth for text input events
        // NOTE: Enter key handling has been moved to the Unicode input system
        // to avoid duplicate line break insertion. The Unicode system handles
        // both Key::Enter and '\n' character input in one place.
    }

    let mut glyph_switched = false;
    for (i, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ]
    .iter()
    .enumerate()
    {
        if keyboard_input.just_pressed(*key) {
            // Get available glyphs from either source
            let glyph_names: Vec<String> =
                if let Some(app_state) = app_state.as_ref() {
                    app_state.workspace.font.glyphs.keys().cloned().collect()
                } else if let Some(fontir_state) = fontir_app_state.as_ref() {
                    fontir_state.get_glyph_names()
                } else {
                    vec![]
                };
            if let Some(glyph_name) = glyph_names.get(i) {
                glyph_navigation.current_glyph = Some(glyph_name.clone());
                info!(
                    "Switched to glyph '{}' via number key {}",
                    glyph_name,
                    i + 1
                );
                glyph_switched = true;
                keyboard_input.clear_just_pressed(*key);
                break;
            }
        }
    }

    if glyph_switched {
        return;
    }

    let default_glyph_name = match &glyph_navigation.current_glyph {
        Some(name) => name.clone(),
        None => {
            if font_has_glyph("a") {
                "a".to_string()
            } else {
                // Get first available glyph
                let glyph_names: Vec<String> = if let Some(app_state) =
                    app_state.as_ref()
                {
                    app_state.workspace.font.glyphs.keys().cloned().collect()
                } else if let Some(fontir_state) = fontir_app_state.as_ref() {
                    fontir_state.get_glyph_names()
                } else {
                    vec![]
                };

                if let Some(first_glyph) = glyph_names.first() {
                    first_glyph.clone()
                } else {
                    return;
                }
            }
        }
    };

    let _default_advance_width = if let Some(app_state) = app_state.as_ref() {
        app_state
            .workspace
            .font
            .glyphs
            .get(&default_glyph_name)
            .map(|g| g.advance_width as f32)
            .unwrap_or(600.0)
    } else if let Some(fontir_state) = fontir_app_state.as_ref() {
        fontir_state.get_glyph_advance_width(&default_glyph_name)
    } else {
        600.0
    };
}
