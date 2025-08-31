//! Text Tool - Sort Placement and text editing
//!
//! The text tool allows users to place sorts by clicking in the design space.
//! Sorts can be placed and edited with different modes:
//! - Text mode: Sorts follow the gap buffer layout in a grid
//! - Insert mode: Sorts are positioned freely in the design space
//! - Freeform mode: Sorts are positioned freely in the design space
//! - Vim mode: Sorts are edited with vim-like keybindings

use crate::core::settings::BezySettings;
use crate::core::state::{
    AppState, GlyphNavigation, SortLayoutMode, TextEditorState, TextModeConfig,
};
use bevy::input::ButtonState;
use bevy::log::info;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::rendering::cameras::DesignCamera;
use crate::rendering::checkerboard::calculate_dynamic_grid_size;
use crate::ui::theme::SORT_ACTIVE_METRICS_COLOR;
use crate::ui::theme::*;
use crate::ui::edit_mode_toolbar::{EditTool, ToolRegistry};

// --------- Resources, Structs, Enums -----------

/// Resource to track if text mode is active
#[derive(Resource, Default)]
pub struct TextModeActive(pub bool);

/// Resource to track text mode state for sort placement
#[derive(Resource, Default)]
pub struct TextModeState {
    /// Current cursor position in design-space coordinates
    pub cursor_position: Option<Vec2>,
    /// Whether we're showing a sort placement preview
    pub showing_preview: bool,
}

/// Text placement modes for the submenu
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Resource)]
pub enum TextPlacementMode {
    /// Place sorts in the text mode (grid layout)
    #[default]
    Text,
    /// Insert and edit text within existing text mode sorts  
    Insert,
    /// Place sorts freely in the design space
    Freeform,
}

impl TextPlacementMode {
    /// Get the icon for each placement mode
    pub fn get_icon(&self) -> &'static str {
        match self {
            TextPlacementMode::Text => "\u{E004}",
            TextPlacementMode::Insert => "\u{F001}",
            TextPlacementMode::Freeform => "\u{E006}",
        }
    }

    /// Get a human-readable name for this placement mode
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            TextPlacementMode::Text => "Text",
            TextPlacementMode::Insert => "Insert",
            TextPlacementMode::Freeform => "Freeform",
        }
    }
    /// Convert to SortLayoutMode
    pub fn to_sort_layout_mode(&self) -> SortLayoutMode {
        match self {
            TextPlacementMode::Text => SortLayoutMode::LTRText, // Default to LTR
            TextPlacementMode::Insert => SortLayoutMode::LTRText, // Default to LTR
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
    fn id(&self) -> crate::ui::edit_mode_toolbar::ToolId {
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
                    // handle_text_mode_clicks, // DISABLED: Old input system
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

/// Helper function to spawn a single text mode button
fn spawn_text_mode_button(
    parent: &mut ChildSpawnerCommands,
    mode: TextPlacementMode,
    asset_server: &Res<AssetServer>,
) {
    parent
        .spawn(Node {
            margin: UiRect::all(Val::Px(TOOLBAR_ITEM_SPACING)),
            ..default()
        })
        .with_children(|button_container| {
            button_container
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(64.0),
                        height: Val::Px(64.0),
                        padding: UiRect::all(Val::ZERO),
                        border: UiRect::all(Val::Px(TOOLBAR_BORDER_WIDTH)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderRadius::all(Val::Px(TOOLBAR_BORDER_RADIUS)),
                    BorderColor(NORMAL_BUTTON_OUTLINE_COLOR),
                    BackgroundColor(NORMAL_BUTTON_COLOR),
                    TextSubMenuButton,
                    TextModeButton { mode },
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(mode.get_icon().to_string()),
                        TextFont {
                            font: asset_server.load(GROTESK_FONT_PATH),
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(TOOLBAR_ICON_COLOR),
                    ));
                });
        });
}

pub fn spawn_text_submenu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let modes = [
        TextPlacementMode::Text,
        TextPlacementMode::Insert,
        TextPlacementMode::Freeform,
    ];

    // Create the parent submenu node (right-aligned to match main toolbar)
    let submenu_node = Node {
        position_type: PositionType::Absolute,
        top: Val::Px(TOOLBAR_CONTAINER_MARGIN + 74.0),
        right: Val::Px(TOOLBAR_CONTAINER_MARGIN),  // Changed from left to right
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
                spawn_text_mode_button(parent, mode, &asset_server);
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
        ),
        With<TextSubMenuButton>,
    >,
    mut current_mode: ResMut<CurrentTextPlacementMode>,
    mut text_mode_config: ResMut<TextModeConfig>,
) {
    for (interaction, mut color, mut border_color, mode_button) in
        &mut interaction_query
    {
        let is_current_mode = current_mode.0 == mode_button.mode;

        if *interaction == Interaction::Pressed && !is_current_mode {
            current_mode.0 = mode_button.mode;
            text_mode_config.default_placement_mode =
                mode_button.mode.to_sort_layout_mode();
            info!("Switched to text placement mode: {:?}", mode_button.mode);
        }

        match (*interaction, is_current_mode) {
            (Interaction::Pressed, _) | (_, true) => {
                *color = PRESSED_BUTTON_COLOR.into();
                border_color.0 = PRESSED_BUTTON_OUTLINE_COLOR;
            }
            (Interaction::Hovered, false) => {
                *color = HOVERED_BUTTON_COLOR.into();
                border_color.0 = HOVERED_BUTTON_OUTLINE_COLOR;
            }
            (Interaction::None, false) => {
                *color = NORMAL_BUTTON_COLOR.into();
                border_color.0 = NORMAL_BUTTON_OUTLINE_COLOR;
            }
        }
    }
}

pub fn toggle_text_submenu_visibility(
    mut submenu_query: Query<(&mut Node, &Name)>,
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
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
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
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
pub fn handle_text_mode_sort_placement(
    _commands: Commands,
    mut text_editor_state: ResMut<TextEditorState>,
    app_state: Res<AppState>,
    glyph_navigation: Res<GlyphNavigation>,
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    current_placement_mode: Res<CurrentTextPlacementMode>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
    pointer_info: Res<crate::core::io::pointer::PointerInfo>,
    mut camera_query: Query<&mut Projection, With<DesignCamera>>,
) {
    if current_tool.get_current() != Some("text") {
        return;
    }
    if ui_hover_state.is_hovering_ui {
        return;
    }

    // Get camera zoom for grid snapping
    let zoom_scale = camera_query
        .single_mut()
        .map(|mut p| {
            if let Projection::Orthographic(ortho) = p.as_mut() {
                ortho.scale
            } else {
                1.0
            }
        })
        .unwrap_or(1.0);
    let grid_size = calculate_dynamic_grid_size(zoom_scale);
    let raw_cursor_world_pos = pointer_info.design.to_raw();
    let snapped_position =
        (raw_cursor_world_pos / grid_size).round() * grid_size;

    // Determine which glyph to place
    let glyph_name = match &glyph_navigation.current_glyph {
        Some(name) => name.clone(),
        None => {
            if app_state.workspace.font.glyphs.contains_key("a") {
                "a".to_string()
            } else if let Some(first_glyph) =
                app_state.workspace.font.glyphs.keys().next()
            {
                first_glyph.clone()
            } else {
                warn!("No glyphs available in font");
                return;
            }
        }
    };

    // Get glyph advance width
    let advance_width = if let Some(glyph_data) =
        app_state.workspace.font.glyphs.get(&glyph_name)
    {
        glyph_data.advance_width as f32
    } else {
        600.0
    };

    let sort_position = snapped_position; // SIMPLIFIED: no offset

    info!(
        "DEBUG: pointer_info.design = ({:.2}, {:.2}), snapped_position = ({:.2}, {:.2}), sort_position = ({:.2}, {:.2})",
        pointer_info.design.x, pointer_info.design.y,
        snapped_position.x, snapped_position.y,
        sort_position.x, sort_position.y
    );

    match current_placement_mode.0 {
        TextPlacementMode::Text => {
            text_editor_state.create_text_sort_at_position(
                glyph_name.clone(),
                sort_position,
                advance_width,
                current_placement_mode.0.to_sort_layout_mode(),
                None, // No codepoint for clicked glyphs
            );
            info!(
                "DEBUG: [PLACE] Placed sort at ({:.1}, {:.1})",
                sort_position.x, sort_position.y
            );
            info!(
                "Placed sort '{}' in text mode at position ({:.1}, {:.1})",
                glyph_name, sort_position.x, sort_position.y
            );
        }
        TextPlacementMode::Insert => {
            info!("Insert mode: Use keyboard to edit text mode sorts, not mouse clicks");
        }
        TextPlacementMode::Freeform => {
            text_editor_state.add_freeform_sort(
                glyph_name.clone(),
                sort_position,
                advance_width,
                None, // No codepoint for clicked glyphs
            );
            info!(
                "Placed sort '{}' in freeform mode at position ({:.1}, {:.1})",
                glyph_name, sort_position.x, sort_position.y
            );
        }
    }
}

// -- Preview Rendering and Keyboard Handling --

#[allow(clippy::too_many_arguments)]
pub fn render_sort_preview(
    mut gizmos: Gizmos,
    text_mode_active: Res<TextModeActive>,
    _text_mode_state: Res<TextModeState>,
    _text_editor_state: Option<Res<TextEditorState>>,
    current_placement_mode: Res<CurrentTextPlacementMode>,
    glyph_navigation: Res<GlyphNavigation>,
    app_state: Res<AppState>,
    pointer_info: Res<crate::core::io::pointer::PointerInfo>,
    camera_query: Query<&Projection, With<DesignCamera>>,
) {
    debug!("[PREVIEW] Entered render_sort_preview");
    if !text_mode_active.0 {
        debug!("[PREVIEW] Early return: text_mode_active is false");
        return;
    }
    if current_placement_mode.0 == TextPlacementMode::Insert {
        debug!("[PREVIEW] Early return: placement mode is Insert");
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

    let preview_color = Color::srgb(1.0, 0.5, 0.0).with_alpha(0.8);

    if let Some(glyph_name) = &glyph_navigation.current_glyph {
        debug!("[PREVIEW] current_glyph: {}", glyph_name);
        if let Some(glyph_data) =
            app_state.workspace.font.glyphs.get(glyph_name)
        {
            debug!(
                "[PREVIEW] Drawing preview for glyph '{}' at ({:.1}, {:.1})",
                glyph_name, snapped_position.x, snapped_position.y
            );
            // Draw glyph outline
            crate::rendering::glyph_outline::draw_glyph_outline_at_position(
                &mut gizmos,
                &glyph_data.outline,
                snapped_position,
            );
            // Draw metrics if available
            crate::rendering::metrics::draw_metrics_at_position(
                &mut gizmos,
                glyph_data.advance_width as f32,
                &app_state.workspace.info.metrics,
                snapped_position,
                preview_color,
            );
        } else {
            debug!(
                "[PREVIEW] No glyph_data found for '{}', cannot draw preview",
                glyph_name
            );
        }
    } else {
        debug!("[PREVIEW] No current_glyph set, cannot draw preview");
    }
}

// -- Keyboard Input and Helpers --

pub fn handle_text_tool_shortcuts(
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut current_tool: ResMut<
        crate::ui::edit_mode_toolbar::CurrentTool,
    >,
    mut current_placement_mode: ResMut<CurrentTextPlacementMode>,
    mut text_mode_config: ResMut<TextModeConfig>,
    text_editor_state: Option<Res<TextEditorState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT)
        && current_tool.get_current() != Some("text")
    {
        current_tool.switch_to("text");
        info!("Activated text tool via keyboard shortcut");
        keyboard_input.clear_just_pressed(KeyCode::KeyT);
    }
    if current_tool.get_current() == Some("text")
        && keyboard_input.just_pressed(KeyCode::Tab)
    {
        let new_mode = match current_placement_mode.0 {
            TextPlacementMode::Text => TextPlacementMode::Insert,
            TextPlacementMode::Insert => TextPlacementMode::Freeform,
            TextPlacementMode::Freeform => TextPlacementMode::Text,
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
    _text_editor_state: Option<ResMut<TextEditorState>>,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    app_state: Res<AppState>,
    mut glyph_navigation: ResMut<GlyphNavigation>,
    text_mode_state: Res<TextModeState>,
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    // Add query to check for selected points
    selected_points: Query<
        Entity,
        With<crate::editing::selection::components::Selected>,
    >,
) {
    if !text_mode_active.0 || current_tool.get_current() != Some("text") {
        return;
    }
    if current_placement_mode.0 == TextPlacementMode::Insert {
        return;
    }

    // Check if there are any selected points - if so, don't handle arrow keys
    // This gives priority to the nudge system
    let has_selected_points = !selected_points.is_empty();
    if has_selected_points {
        debug!("[TEXT_TOOLBAR] Skipping arrow key handling - {} selected points found", selected_points.iter().count());
        return;
    }

    let mut text_editor_state = match _text_editor_state {
        Some(state) => state,
        None => return,
    };

    if current_placement_mode.0 == TextPlacementMode::Text {
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
            text_editor_state.move_cursor_up();
            debug!(
                "Text mode: moved cursor up to position {}",
                text_editor_state.cursor_position
            );
            keyboard_input.clear_just_pressed(KeyCode::ArrowUp);
        }
        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            text_editor_state.move_cursor_down();
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
        if keyboard_input.just_pressed(KeyCode::Backspace) {
            if text_editor_state.cursor_position > 0 {
                text_editor_state.move_cursor_left();
                text_editor_state.delete_sort_at_cursor();
                debug!("Text mode: backspace deleted sort");
            }
            keyboard_input.clear_just_pressed(KeyCode::Backspace);
        }
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
            let glyph_names: Vec<String> =
                app_state.workspace.font.glyphs.keys().cloned().collect();
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
            if app_state.workspace.font.glyphs.contains_key("a") {
                "a".to_string()
            } else if let Some(first_glyph) =
                app_state.workspace.font.glyphs.keys().next()
            {
                first_glyph.clone()
            } else {
                return;
            }
        }
    };

    let default_advance_width = if let Some(glyph_data) =
        app_state.workspace.font.glyphs.get(&default_glyph_name)
    {
        glyph_data.advance_width as f32
    } else {
        600.0
    };

    let pressed_keys: Vec<KeyCode> =
        keyboard_input.get_just_pressed().cloned().collect();

    for key in pressed_keys {
        let character_glyph = match key {
            KeyCode::KeyA => Some("a"),
            KeyCode::KeyB => Some("b"),
            KeyCode::KeyC => Some("c"),
            KeyCode::KeyD => Some("d"),
            KeyCode::KeyE => Some("e"),
            KeyCode::KeyF => Some("f"),
            KeyCode::KeyG => Some("g"),
            KeyCode::KeyH => Some("h"),
            KeyCode::KeyI => Some("i"),
            KeyCode::KeyJ => Some("j"),
            KeyCode::KeyK => Some("k"),
            KeyCode::KeyL => Some("l"),
            KeyCode::KeyM => Some("m"),
            KeyCode::KeyN => Some("n"),
            KeyCode::KeyO => Some("o"),
            KeyCode::KeyP => Some("p"),
            KeyCode::KeyQ => Some("q"),
            KeyCode::KeyR => Some("r"),
            KeyCode::KeyS => Some("s"),
            KeyCode::KeyT => Some("t"),
            KeyCode::KeyU => Some("u"),
            KeyCode::KeyV => Some("v"),
            KeyCode::KeyW => Some("w"),
            KeyCode::KeyX => Some("x"),
            KeyCode::KeyY => Some("y"),
            KeyCode::KeyZ => Some("z"),
            KeyCode::Digit0 => Some("zero"),
            KeyCode::Digit1 => Some("one"),
            KeyCode::Digit2 => Some("two"),
            KeyCode::Digit3 => Some("three"),
            KeyCode::Digit4 => Some("four"),
            KeyCode::Digit5 => Some("five"),
            KeyCode::Digit6 => Some("six"),
            KeyCode::Digit7 => Some("seven"),
            KeyCode::Digit8 => Some("eight"),
            KeyCode::Digit9 => Some("nine"),
            KeyCode::Space => Some("space"),
            _ => None,
        };
        if let Some(char_glyph) = character_glyph {
            if app_state.workspace.font.glyphs.contains_key(char_glyph) {
                let char_advance_width = if let Some(glyph_data) =
                    app_state.workspace.font.glyphs.get(char_glyph)
                {
                    glyph_data.advance_width as f32
                } else {
                    default_advance_width
                };
                match current_placement_mode.0 {
                    TextPlacementMode::Text => {
                        let position = text_mode_state
                            .cursor_position
                            .unwrap_or(Vec2::ZERO);
                        text_editor_state.create_text_sort_at_position(
                            char_glyph.to_string(),
                            position,
                            char_advance_width,
                            current_placement_mode.0.to_sort_layout_mode(),
                            None, // TODO: Map keycode to character
                        );
                        info!(
                            "Placed sort '{}' in text mode via keyboard at position ({:.1}, {:.1})",
                            char_glyph, position.x, position.y
                        );
                    }
                    TextPlacementMode::Insert => {
                        info!(
                            "Insert mode: Keyboard typing should be handled by text editor system"
                        );
                    }
                    TextPlacementMode::Freeform => {
                        let position = text_mode_state
                            .cursor_position
                            .unwrap_or(Vec2::ZERO);
                        text_editor_state.add_freeform_sort(
                            char_glyph.to_string(),
                            position,
                            char_advance_width,
                            None, // TODO: Map keycode to character
                        );
                        info!(
                            "Placed sort '{}' in freeform mode via keyboard at position ({:.1}, {:.1})",
                            char_glyph, position.x, position.y
                        );
                    }
                }
                keyboard_input.clear_just_pressed(key);
            } else {
                debug!("Glyph '{}' not found in font, skipping", char_glyph);
            }
        }
    }
}
