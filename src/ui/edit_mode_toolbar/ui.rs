//! Edit Mode Toolbar UI
//!
//! This sub-module implements the user interface for the edit mode toolbar,
//! which dynamically generates toolbar buttons based on registered tools.
//! The system automatically discovers and displays all registered tools with
//! proper ordering and visual feedback. To add a new tool, implement the
//! `EditTool` trait and register it with `ToolRegistry::register_tool()`.
//!
//! ## Consistent Button Rendering System
//!
//! This module provides a comprehensive button rendering system that ensures
//! consistent visual appearance across all toolbar buttons (main toolbar and submenus).
//!
//! ### Key Features
//!
//! - **Consistent Button Creation**: `create_toolbar_button()` creates buttons with
//!   identical styling, sizing, borders, and color handling
//! - **Consistent Color System**: `update_toolbar_button_colors()` ensures all buttons use
//!   the same color states (normal, hovered, pressed/active)
//! - **Icon Alignment**: `create_button_icon_text()` provides consistent icon centering
//!   and font sizing across all buttons
//!
//! ### For Submenu Developers
//!
//! When creating submenu buttons, always use the consistent system:
//! ```rust,ignore
//! // 1. Create the button with consistent styling
//! create_toolbar_button(
//!     parent,
//!     icon_string,
//!     (YourSubMenuButton, YourModeButton { mode }),
//!     &asset_server,
//!     &theme,
//! );
//!
//! // 2. Handle background/border color updates
//! update_toolbar_button_colors(
//!     interaction,
//!     is_active,
//!     &mut background_color,
//!     &mut border_color,
//! );
//!
//! // 3. Handle icon text color updates (for bright white active icons)
//! update_toolbar_button_text_colors(
//!     entity,
//!     is_active,
//!     &children_query,
//!     &mut text_query,
//! );
//! ```
//!
//! This approach ensures perfect visual consistency between main toolbar and all submenus,
//! making it easy to maintain a professional, unified interface.

use crate::ui::theme::{
    BUTTON_ICON_SIZE, GROTESK_FONT_PATH, HOVERED_BUTTON_COLOR, HOVERED_BUTTON_OUTLINE_COLOR,
    MONO_FONT_PATH, NORMAL_BUTTON_COLOR, NORMAL_BUTTON_OUTLINE_COLOR, PRESSED_BUTTON_COLOR,
    PRESSED_BUTTON_ICON_COLOR, PRESSED_BUTTON_OUTLINE_COLOR, TOOLBAR_BORDER_WIDTH,
    TOOLBAR_BUTTON_SIZE, TOOLBAR_CONTAINER_MARGIN, TOOLBAR_ICON_COLOR, TOOLBAR_GRID_SPACING,
    TOOLBAR_ITEM_SPACING, TOOLBAR_PADDING, WIDGET_TEXT_FONT_SIZE,
};
use crate::ui::themes::{CurrentTheme, ToolbarBorderRadius};
use crate::ui::edit_mode_toolbar::*;
use bevy::prelude::*;

// COMPONENTS ------------------------------------------------------------------

/// Component marker for toolbar buttons - used for querying toolbar entities
#[derive(Component)]
pub struct EditModeToolbarButton;

/// Component that stores the tool ID for each toolbar button
#[derive(Component)]
pub struct ToolButtonData {
    pub tool_id: ToolId,
}

/// Component marker for hover text entities
#[derive(Component)]
pub struct ButtonHoverText;

// TOOLBAR CREATION ------------------------------------------------------------

/// Creates the main edit mode toolbar with all registered tools
pub fn spawn_edit_mode_toolbar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme: Res<CurrentTheme>,
    mut tool_registry: ResMut<ToolRegistry>,
) {
    let ordered_tool_ids = tool_registry.get_ordered_tools().to_vec();
    info!(
        "Spawning edit-mode toolbar with {} tools",
        ordered_tool_ids.len()
    );
    commands
        .spawn(create_toolbar_container())
        .with_children(|parent| {
            for tool_id in ordered_tool_ids {
                if let Some(tool) = tool_registry.get_tool(tool_id) {
                    create_tool_button(parent, tool, &asset_server, &theme);
                } else {
                    warn!("Tool '{}' not found in registry", tool_id);
                }
            }
        });
}

/// Creates the main toolbar container with proper positioning and styling
fn create_toolbar_container() -> impl Bundle {
    Node {
        position_type: PositionType::Absolute,
        top: Val::Px(TOOLBAR_CONTAINER_MARGIN),
        left: Val::Px(TOOLBAR_CONTAINER_MARGIN),
        flex_direction: FlexDirection::Row,
        padding: UiRect::all(Val::Px(TOOLBAR_PADDING)),
        margin: UiRect::all(Val::ZERO),
        row_gap: Val::ZERO,
        ..default()
    }
}

// BUTTON CREATION -------------------------------------------------------------

/// Creates a single tool button with proper styling and components
fn create_tool_button(
    parent: &mut ChildSpawnerCommands,
    tool: &dyn EditTool,
    asset_server: &AssetServer,
    theme: &Res<CurrentTheme>,
) {
    parent
        .spawn(Node {
            margin: UiRect::all(Val::Px(TOOLBAR_GRID_SPACING)),
            ..default()
        })
        .with_children(|button_container| {
            create_button_entity(button_container, tool, asset_server, theme);
        });
}

/// Creates the button entity with all required components
fn create_button_entity(
    parent: &mut ChildSpawnerCommands,
    tool: &dyn EditTool,
    asset_server: &AssetServer,
    theme: &Res<CurrentTheme>,
) -> Entity {
    parent
        .spawn((
            Button,
            EditModeToolbarButton,
            ToolButtonData { tool_id: tool.id() },
            create_button_styling(),
            BackgroundColor(NORMAL_BUTTON_COLOR),
            BorderColor(NORMAL_BUTTON_OUTLINE_COLOR),
            BorderRadius::all(Val::Px(theme.theme().toolbar_border_radius())),
            ToolbarBorderRadius,
        ))
        .with_children(|button| {
            create_button_text(button, tool, asset_server);
        })
        .id()
}

/// Creates the button styling configuration
fn create_button_styling() -> Node {
    Node {
        width: Val::Px(TOOLBAR_BUTTON_SIZE),
        height: Val::Px(TOOLBAR_BUTTON_SIZE),
        padding: UiRect::all(Val::ZERO),
        margin: UiRect::all(Val::ZERO),
        border: UiRect::all(Val::Px(TOOLBAR_BORDER_WIDTH)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

/// Creates the button text with the tool's icon
fn create_button_text(
    parent: &mut ChildSpawnerCommands,
    tool: &dyn EditTool,
    asset_server: &AssetServer,
) {
    create_button_icon_text(parent, tool.icon(), asset_server);
}

/// Creates properly centered button icon text - shared helper for consistent alignment
/// This should be used by all toolbar buttons (main toolbar and submenus) for consistent icon centering
pub fn create_button_icon_text(
    parent: &mut ChildSpawnerCommands,
    icon: &str,
    asset_server: &AssetServer,
) {
    parent.spawn((
        Node {
            // Vertical centering adjustment - ensures icons are properly centered in buttons
            margin: UiRect::top(Val::Px(4.0)),
            ..default()
        },
        Text::new(icon.to_string()),
        TextFont {
            font: asset_server.load(GROTESK_FONT_PATH),
            font_size: BUTTON_ICON_SIZE,
            ..default()
        },
        TextColor(TOOLBAR_ICON_COLOR),
    ));
}

/// Creates a standard button with consistent styling and returns a builder for adding components
/// This should be used by most toolbar buttons (main toolbar and submenus) for visual consistency
pub fn create_toolbar_button<T: Bundle>(
    parent: &mut ChildSpawnerCommands,
    icon: &str,
    additional_components: T,
    asset_server: &AssetServer,
    theme: &Res<CurrentTheme>,
) {
    create_toolbar_button_with_hover_text(
        parent,
        icon,
        None,
        additional_components,
        asset_server,
        theme,
    );
}

/// Creates a standard button with hover text support
/// This version allows specifying the hover text to display when the button is hovered
pub fn create_toolbar_button_with_hover_text<T: Bundle>(
    parent: &mut ChildSpawnerCommands,
    icon: &str,
    _hover_text: Option<&str>,
    additional_components: T,
    asset_server: &AssetServer,
    theme: &Res<CurrentTheme>,
) {
    // Note: _hover_text parameter is now ignored since hover text is handled dynamically
    parent
        .spawn(Node {
            margin: UiRect::all(Val::Px(TOOLBAR_GRID_SPACING)),
            ..default()
        })
        .with_children(|button_container| {
            button_container
                .spawn((
                    Button,
                    additional_components,
                    create_button_styling(),
                    BackgroundColor(NORMAL_BUTTON_COLOR),
                    BorderColor(NORMAL_BUTTON_OUTLINE_COLOR),
                    BorderRadius::all(Val::Px(theme.theme().toolbar_border_radius())),
                    ToolbarBorderRadius,
                ))
                .with_children(|button| {
                    create_button_icon_text(button, icon, asset_server);
                });
        });
}

/// Updates standard button colors with consistent styling
/// This should be used by all standard button color update systems for consistency
pub fn update_toolbar_button_colors(
    interaction: Interaction,
    is_active: bool,
    background_color: &mut BackgroundColor,
    border_color: &mut BorderColor,
) {
    let (bg_color, border_color_value) = match (interaction, is_active) {
        (Interaction::Pressed, _) | (_, true) => {
            (PRESSED_BUTTON_COLOR, PRESSED_BUTTON_OUTLINE_COLOR)
        }
        (Interaction::Hovered, false) => (HOVERED_BUTTON_COLOR, HOVERED_BUTTON_OUTLINE_COLOR),
        (Interaction::None, false) => (NORMAL_BUTTON_COLOR, NORMAL_BUTTON_OUTLINE_COLOR),
    };

    *background_color = BackgroundColor(bg_color);
    *border_color = BorderColor(border_color_value);
}

/// Updates button text (icon) colors using the unified color system
/// This should be used by all button text color update systems for consistency
pub fn update_toolbar_button_text_colors(
    entity: Entity,
    is_active: bool,
    children_query: &Query<&Children>,
    text_query: &mut Query<&mut TextColor>,
) {
    let children = match children_query.get(entity) {
        Ok(children) => children,
        Err(_) => return,
    };

    let new_color = if is_active {
        PRESSED_BUTTON_ICON_COLOR // Bright white for active buttons
    } else {
        TOOLBAR_ICON_COLOR // Light gray for normal buttons
    };

    // Update text colors for all children of this button
    for &child_entity in children {
        if let Ok(mut text_color) = text_query.get_mut(child_entity) {
            text_color.0 = new_color;
        }
    }
}

// INTERACTION HANDLING --------------------------------------------------------

/// Handles toolbar button interactions and tool switching
#[allow(clippy::type_complexity)]
pub fn handle_toolbar_mode_selection(
    interaction_query: Query<
        (&Interaction, &ToolButtonData),
        (Changed<Interaction>, With<EditModeToolbarButton>),
    >,
    mut current_tool: ResMut<CurrentTool>,
    tool_registry: Res<ToolRegistry>,
) {
    for (interaction, tool_button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            println!(
                "🖊️ PEN_DEBUG: Button pressed for tool: {}",
                tool_button.tool_id
            );
            switch_to_tool(tool_button.tool_id, &mut current_tool, &tool_registry);
        }
    }
}

/// Updates button visual states based on interaction and current tool
pub fn update_toolbar_button_appearances(
    interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &ToolButtonData,
            Entity,
        ),
        With<EditModeToolbarButton>,
    >,
    mut text_query: Query<&mut TextColor>,
    children_query: Query<&Children>,
    current_tool: Res<CurrentTool>,
) {
    let current_tool_id = current_tool.get_current();
    for (interaction, mut background_color, mut border_color, tool_button, entity) in
        interaction_query
    {
        let is_current_tool = current_tool_id == Some(tool_button.tool_id);
        update_button_colors(
            *interaction,
            is_current_tool,
            &mut background_color,
            &mut border_color,
        );
        update_button_text_color(entity, is_current_tool, &children_query, &mut text_query);
    }
}

/// Switches to a new tool, handling lifecycle methods
fn switch_to_tool(
    new_tool_id: ToolId,
    current_tool: &mut ResMut<CurrentTool>,
    tool_registry: &Res<ToolRegistry>,
) {
    if current_tool.get_current() == Some(new_tool_id) {
        return;
    }
    exit_current_tool(current_tool, tool_registry);
    enter_new_tool(new_tool_id, current_tool, tool_registry);
}

/// Exits the currently active tool
fn exit_current_tool(current_tool: &mut ResMut<CurrentTool>, tool_registry: &Res<ToolRegistry>) {
    if let Some(current_id) = current_tool.get_current() {
        if let Some(current_tool_impl) = tool_registry.get_tool(current_id) {
            current_tool_impl.on_exit();
        }
    }
}

/// Enters a new tool and updates the current tool state
fn enter_new_tool(
    new_tool_id: ToolId,
    current_tool: &mut ResMut<CurrentTool>,
    tool_registry: &Res<ToolRegistry>,
) {
    if let Some(new_tool_impl) = tool_registry.get_tool(new_tool_id) {
        new_tool_impl.on_enter();
    }
    current_tool.switch_to(new_tool_id);
    info!("Switched to tool: {}", new_tool_id);
}

// VISUAL UPDATES --------------------------------------------------------------

/// Updates button colors based on interaction state and current tool
fn update_button_colors(
    interaction: Interaction,
    is_current_tool: bool,
    background_color: &mut BackgroundColor,
    border_color: &mut BorderColor,
) {
    // Use consistent color system
    update_toolbar_button_colors(interaction, is_current_tool, background_color, border_color);
}

/// Updates text color for button children based on current tool state
fn update_button_text_color(
    entity: Entity,
    is_current_tool: bool,
    children_query: &Query<&Children>,
    text_query: &mut Query<&mut TextColor>,
) {
    // Use consistent text color system
    update_toolbar_button_text_colors(entity, is_current_tool, children_query, text_query);
}

/// Updates hover text visibility based on button interaction states
/// This works for any button with the Button component, not just main toolbar buttons
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn update_hover_text_visibility(
    mut commands: Commands,
    // Main toolbar buttons
    toolbar_button_query: Query<(&Interaction, Entity, &ToolButtonData), With<Button>>,
    // Pen submenu buttons
    pen_button_query: Query<
        (
            &Interaction,
            &crate::ui::edit_mode_toolbar::pen::PenModeButton,
        ),
        (With<Button>, Without<ToolButtonData>),
    >,
    // Text submenu buttons
    text_button_query: Query<
        (
            &Interaction,
            &crate::ui::edit_mode_toolbar::text::TextModeButton,
        ),
        (
            With<Button>,
            Without<ToolButtonData>,
            Without<crate::ui::edit_mode_toolbar::pen::PenModeButton>,
        ),
    >,
    // Shapes submenu buttons
    shapes_button_query: Query<
        (
            &Interaction,
            &crate::ui::edit_mode_toolbar::shapes::ShapeModeButton,
        ),
        (
            With<Button>,
            Without<ToolButtonData>,
            Without<crate::ui::edit_mode_toolbar::pen::PenModeButton>,
            Without<crate::ui::edit_mode_toolbar::text::TextModeButton>,
        ),
    >,
    // AI submenu buttons
    ai_button_query: Query<
        (
            &Interaction,
            &crate::tools::ai::AiOperationButton,
        ),
        (
            With<Button>,
            Without<ToolButtonData>,
            Without<crate::ui::edit_mode_toolbar::pen::PenModeButton>,
            Without<crate::ui::edit_mode_toolbar::text::TextModeButton>,
            Without<crate::ui::edit_mode_toolbar::shapes::ShapeModeButton>,
        ),
    >,
    // Check submenu visibility by name (exclude hover text entities)
    submenu_query: Query<(&Node, &Name), Without<ButtonHoverText>>,
    mut hover_text_query: Query<(Entity, &mut Text, &mut Node), With<ButtonHoverText>>,
    tool_registry: Res<ToolRegistry>,
    asset_server: Res<AssetServer>,
    // Get camera for zoom level
    camera_query: Query<&Projection, With<crate::rendering::cameras::DesignCamera>>,
) {
    let mut hovered_text: Option<String> = None;

    // Check main toolbar buttons
    for (interaction, _button_entity, tool_data) in toolbar_button_query.iter() {
        if *interaction == Interaction::Hovered {
            if let Some(tool) = tool_registry.get_tool(tool_data.tool_id) {
                hovered_text = Some(tool.name().to_string());
                break;
            }
        }
    }

    // Check pen submenu buttons
    if hovered_text.is_none() {
        for (interaction, pen_mode_button) in pen_button_query.iter() {
            if *interaction == Interaction::Hovered {
                hovered_text = Some(pen_mode_button.mode.get_name().to_string());
                break;
            }
        }
    }

    // Check text submenu buttons
    if hovered_text.is_none() {
        for (interaction, text_mode_button) in text_button_query.iter() {
            if *interaction == Interaction::Hovered {
                hovered_text = Some(text_mode_button.mode.display_name().to_string());
                break;
            }
        }
    }

    // Check shapes submenu buttons
    if hovered_text.is_none() {
        for (interaction, shape_mode_button) in shapes_button_query.iter() {
            if *interaction == Interaction::Hovered {
                hovered_text = Some(shape_mode_button.shape_type.get_name().to_string());
                break;
            }
        }
    }

    // Check AI submenu buttons
    if hovered_text.is_none() {
        for (interaction, ai_operation_button) in ai_button_query.iter() {
            if *interaction == Interaction::Hovered {
                hovered_text = Some(ai_operation_button.operation.display_name().to_string());
                break;
            }
        }
    }

    // Calculate vertical position based on submenu visibility
    // Use grid spacing for consistent layout - smaller gap for better visual connection
    let base_offset = TOOLBAR_CONTAINER_MARGIN + TOOLBAR_BUTTON_SIZE + TOOLBAR_GRID_SPACING * 2.0;

    // Check if any submenu is visible
    let mut submenu_visible = false;
    for (node, name) in submenu_query.iter() {
        if (name.as_str() == "PenSubMenu"
            || name.as_str() == "TextSubMenu"
            || name.as_str() == "ShapesSubMenu"
            || name.as_str() == "AiSubMenu")
            && node.display != Display::None
        {
            submenu_visible = true;
            break;
        }
    }

    // Calculate position: if submenu visible, position below submenu; otherwise below main toolbar
    let vertical_offset = if submenu_visible {
        // Position below submenu: container margin + main toolbar + spacing + submenu + smaller spacing
        TOOLBAR_CONTAINER_MARGIN + TOOLBAR_BUTTON_SIZE + TOOLBAR_GRID_SPACING * 2.0
            + TOOLBAR_BUTTON_SIZE + TOOLBAR_GRID_SPACING * 2.0
    } else {
        // Position below main toolbar with consistent spacing
        base_offset
    };

    // Determine what text to show
    let display_text = if let Some(text_content) = hovered_text {
        // Show tool name when hovering
        text_content
    } else {
        // Show zoom level when not hovering
        if let Ok(projection) = camera_query.single() {
            // Get the actual zoom scale from the orthographic projection
            let zoom_scale = match projection {
                Projection::Orthographic(ortho) => ortho.scale,
                _ => 1.0,
            };
            // Invert the scale since smaller scale = zoomed in
            let zoom_percentage = ((1.0 / zoom_scale) * 100.0) as i32;
            format!("Zoom: {zoom_percentage}%")
        } else {
            String::new()
        }
    };

    // Create or update hover text
    if !display_text.is_empty() {
        // Try to get a single hover text entity
        let query_result = hover_text_query.single_mut();
        if let Ok((_, mut text, mut style)) = query_result {
            // Update existing hover text
            text.0 = display_text;
            style.top = Val::Px(vertical_offset);
            style.display = Display::Flex;
        } else {
            // Create new hover text if none exists
            commands.spawn((
                Text::new(display_text),
                TextFont {
                    font: asset_server.load(MONO_FONT_PATH),
                    font_size: WIDGET_TEXT_FONT_SIZE,
                    ..default()
                },
                TextColor(TOOLBAR_ICON_COLOR), // Light gray color to match unselected icons
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(vertical_offset),
                    left: Val::Px(TOOLBAR_CONTAINER_MARGIN + TOOLBAR_GRID_SPACING), // Align with button left edge
                    display: Display::Flex,                        // Show immediately
                    ..default()
                },
                ButtonHoverText,
            ));
        }
    } else {
        // Hide hover text when there's nothing to display
        for (_hover_entity, _text, mut style) in hover_text_query.iter_mut() {
            style.display = Display::None;
        }
    }
}

// ============================================================================
// TOOL UPDATES
// ============================================================================

/// Updates the current edit mode by calling the active tool's update method
///
/// This system only runs when the tool changes, not every frame, to avoid
/// infinite activation loops.
pub fn update_current_edit_mode(
    mut commands: Commands,
    current_tool: Res<CurrentTool>,
    tool_registry: Res<ToolRegistry>,
) {
    // Only update when the tool actually changes
    if current_tool.is_changed() {
        if let Some(current_tool_id) = current_tool.get_current() {
            if let Some(tool) = tool_registry.get_tool(current_tool_id) {
                tool.update(&mut commands);
                debug!("Tool changed to: {}", current_tool_id);
            }
        }
    }
}
