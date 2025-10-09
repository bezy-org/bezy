//! # Select Tool
//!
//! The select tool allows users to select and manipulate points, contours,
//! and other elements in the font editor. Click to select individual points,
//! drag to create marquee selections, and use various keyboard modifiers
//! to modify selections.

mod select_state;

use super::{EditTool, ToolInfo};
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use select_state::SelectToolDragState;

/// Resource to track if select mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct SelectModeActive(pub bool);

/// The select tool implementation
pub struct SelectTool;

impl EditTool for SelectTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "select",
            display_name: "Select",
            icon: "\u{E010}", // Select cursor icon
            tooltip: "Select and manipulate objects",
            shortcut: Some(KeyCode::KeyV),
        }
    }

    fn on_activate(&mut self, commands: &mut Commands) {
        commands.insert_resource(SelectModeActive(true));
        commands.insert_resource(crate::io::input::InputMode::Select);
        debug!("Entered Select tool");
    }

    fn on_deactivate(&mut self, commands: &mut Commands) {
        commands.insert_resource(SelectModeActive(false));
        commands.insert_resource(crate::io::input::InputMode::Normal);
        debug!("Exited Select tool");
    }
}

/// Plugin for the Select tool - handles all selection behavior and systems
pub struct SelectToolPlugin;

impl Plugin for SelectToolPlugin {
    fn build(&self, app: &mut App) {
        debug!("🔍 Registering SelectToolPlugin systems");
        app.init_resource::<SelectModeActive>()
            .add_systems(Startup, select_tool_startup_log)
            .add_systems(
                Update,
                (
                    handle_select_tool_activation,
                    sync_select_mode_with_tool_state,
                    // Let the existing selection system handle actual selection
                    // We just manage the tool state
                )
                .chain()
                .run_if(resource_exists::<crate::tools::ToolState>),
            );
    }
}

// DISABLED: Conflicts with existing selection system in /src/editing/selection/
// The existing system uses InputEvent events and handles selection properly
#[allow(dead_code)]
fn handle_select_tool_input(
    tool_state: Res<crate::tools::ToolState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::rendering::cameras::DesignCamera>>,
    mut commands: Commands,
    selectable_query: Query<
        (Entity, &GlobalTransform, Option<&crate::editing::selection::components::Selected>),
        With<crate::editing::selection::components::Selectable>,
    >,
    keyboard: Res<ButtonInput<KeyCode>>,
    drag_state: Res<SelectToolDragState>,
) {
    // Only process input if select tool is active
    if !tool_state.is_active(crate::tools::ToolId::Select) {
        return;
    }

    // Debug: Log mouse events
    if mouse_input.just_pressed(MouseButton::Left) {
        debug!("🔍 SELECT DEBUG: Mouse just pressed");
    }
    if mouse_input.just_released(MouseButton::Left) {
        debug!("🔍 SELECT DEBUG: Mouse just released, is_dragging={}", drag_state.is_dragging);
    }

    // Handle left mouse button release for selection (not press, to avoid conflict with drag)
    if mouse_input.just_released(MouseButton::Left) && !drag_state.is_dragging {
        debug!("🔍 SELECT: Mouse released, was_dragging={}", drag_state.is_dragging);
        // This is a click, not a drag
        let Ok(window) = windows.single() else {
            return;
        };

        let Some(cursor_position) = window.cursor_position() else {
            return;
        };

        // Convert screen coordinates to world coordinates
        let Ok((camera, camera_transform)) = camera_query.single() else {
            return;
        };

        let Ok(world_position) = camera
            .viewport_to_world_2d(camera_transform, cursor_position)
        else {
            return;
        };

        debug!("🔍 SELECT TOOL: Click at world position {:?}", world_position);

        // Check if shift is held for multi-selection
        let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

        // Debug: Count selectable entities
        let selectable_count = selectable_query.iter().count();
        debug!("🔍 SELECT DEBUG: Found {} selectable entities", selectable_count);

        // Find the closest selectable entity within a threshold
        // Note: This threshold is in world units - adjust based on typical zoom levels
        let selection_threshold = 20.0; // Increased for debugging
        let mut closest_entity = None;
        let mut closest_distance = f32::MAX;

        for (entity, transform, is_selected) in selectable_query.iter() {
            let distance = transform.translation().truncate().distance(world_position);

            if distance < selection_threshold && distance < closest_distance {
                closest_distance = distance;
                closest_entity = Some((entity, is_selected.is_some()));
            }
        }

        // Handle selection based on what we found
        if let Some((entity, is_already_selected)) = closest_entity {
            debug!("🔍 SELECT TOOL: Found entity {:?} at distance {:.2}", entity, closest_distance);

            if shift_held {
                // Toggle selection with shift
                if is_already_selected {
                    commands.entity(entity).remove::<crate::editing::selection::components::Selected>();
                    debug!("🔍 SELECT TOOL: Deselected entity {:?} (shift-click)", entity);
                } else {
                    commands.entity(entity).insert(crate::editing::selection::components::Selected);
                    debug!("🔍 SELECT TOOL: Added entity {:?} to selection (shift-click)", entity);
                }
            } else {
                // Without shift: always clear all selections first, then select the clicked entity
                // This ensures clicking on empty space (no entity found) will deselect everything

                // Clear all existing selections
                for (clear_entity, _, is_selected) in selectable_query.iter() {
                    if is_selected.is_some() {
                        commands.entity(clear_entity).remove::<crate::editing::selection::components::Selected>();
                    }
                }

                // Select the clicked entity
                commands.entity(entity).insert(crate::editing::selection::components::Selected);
                debug!("🔍 SELECT TOOL: Selected entity {:?} (cleared others first)", entity);
            }
        } else {
            // No entity found near click position - this is empty space
            if !shift_held {
                // Clear all selections when clicking on empty space (without shift)
                let mut cleared_count = 0;
                for (entity, _, is_selected) in selectable_query.iter() {
                    if is_selected.is_some() {
                        commands.entity(entity).remove::<crate::editing::selection::components::Selected>();
                        cleared_count += 1;
                    }
                }
                debug!("🔍 SELECT TOOL: Cleared {} selections (clicked empty space)", cleared_count);
            } else {
                debug!("🔍 SELECT TOOL: Clicked empty space with shift - keeping current selection");
            }
        }
    }
}

/// Startup system to confirm select tool plugin loaded
fn select_tool_startup_log() {
    debug!("🔍 SelectToolPlugin successfully initialized");
}

/// System to handle select tool activation when tool state changes to select
fn handle_select_tool_activation(
    tool_state: Res<crate::tools::ToolState>,
    mut commands: Commands,
    mut select_mode: ResMut<SelectModeActive>,
) {
    if tool_state.is_active(crate::tools::ToolId::Select) {
        if !select_mode.0 {
            debug!("🔍 SELECT_DEBUG: Activating select tool via ToolState");
            select_mode.0 = true;
            commands.insert_resource(crate::io::input::InputMode::Select);
        }
    } else if select_mode.0 {
        // Deactivate when tool changes
        debug!("🔍 SELECT_DEBUG: Deactivating select tool");
        select_mode.0 = false;
    }
}

/// System to ensure select mode is properly synced with tool state
fn sync_select_mode_with_tool_state(
    tool_state: Res<crate::tools::ToolState>,
    mut select_mode: ResMut<SelectModeActive>,
) {
    let should_be_active = tool_state.is_active(crate::tools::ToolId::Select);

    if select_mode.0 != should_be_active {
        select_mode.0 = should_be_active;
        debug!(
            "[SELECT TOOL] Syncing select mode to match tool state: {}",
            should_be_active
        );
    }
}

// DISABLED: Conflicts with existing selection system in /src/editing/selection/
#[allow(dead_code)]
fn handle_select_tool_drag(
    tool_state: Res<crate::tools::ToolState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::rendering::cameras::DesignCamera>>,
    mut drag_state: ResMut<SelectToolDragState>,
    mut commands: Commands,
    selectable_query: Query<
        (Entity, &GlobalTransform),
        With<crate::editing::selection::components::Selectable>,
    >,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Only process if select tool is active
    if !tool_state.is_active(crate::tools::ToolId::Select) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        // If cursor left window, end drag
        if drag_state.is_dragging {
            drag_state.end_drag();
        }
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(world_position) = camera
        .viewport_to_world_2d(camera_transform, cursor_position)
    else {
        return;
    };

    // Start potential drag on mouse down (but don't mark as dragging yet)
    if mouse_input.just_pressed(MouseButton::Left) && !drag_state.is_dragging {
        // Just record the start position, don't start dragging yet
        drag_state.start_position = world_position;
        drag_state.current_position = world_position;
        debug!("🔍 SELECT: Mouse down at {:?}", world_position);
    }

    // Check if we should start dragging (mouse held and moved beyond threshold)
    if mouse_input.pressed(MouseButton::Left) {
        if !drag_state.is_dragging {
            let drag_threshold = 5.0; // World units
            if drag_state.start_position.distance(world_position) > drag_threshold {
                // Now we're actually dragging
                drag_state.is_dragging = true;
                debug!("🔍 SELECT: Started marquee selection (exceeded threshold)");
            }
        }

        // Update current position whether dragging or not
        drag_state.current_position = world_position;
    }

    // End drag and select entities within marquee
    if mouse_input.just_released(MouseButton::Left) {
        if drag_state.is_dragging {
            // This was a drag - select entities within marquee
            let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

            if !shift_held {
                // Clear existing selection
                for (entity, _) in selectable_query.iter() {
                    commands.entity(entity).remove::<crate::editing::selection::components::Selected>();
                }
            }

            // Select entities within marquee bounds
            for (entity, transform) in selectable_query.iter() {
                let point = transform.translation().truncate();
                if drag_state.contains_point(point) {
                    commands.entity(entity).insert(crate::editing::selection::components::Selected);
                    debug!("🔍 SELECT: Entity {:?} selected by marquee", entity);
                }
            }

            drag_state.end_drag();
        } else {
            // Reset positions even if we didn't drag
            drag_state.start_position = Vec2::ZERO;
            drag_state.current_position = Vec2::ZERO;
        }
    }
}

// DISABLED: The existing selection system already has marquee rendering
#[allow(dead_code)]
fn render_selection_marquee(
    tool_state: Res<crate::tools::ToolState>,
    drag_state: Res<SelectToolDragState>,
    mut commands: Commands,
    existing_marquee: Query<Entity, With<crate::editing::selection::components::SelectionRect>>,
) {
    // Only process if select tool is active
    if !tool_state.is_active(crate::tools::ToolId::Select) {
        // Clean up any existing marquee if tool is not active
        for entity in existing_marquee.iter() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }

    if drag_state.is_dragging {
        // Don't create marquee for tiny drags
        let drag_threshold = 5.0;
        let drag_distance = drag_state.start_position.distance(drag_state.current_position);
        if drag_distance < drag_threshold {
            // Remove marquee if drag is too small
            for entity in existing_marquee.iter() {
                commands.entity(entity).despawn_recursive();
            }
            return;
        }

        // Update or create the selection rect component
        if let Some(entity) = existing_marquee.iter().next() {
            // Update existing marquee
            commands.entity(entity).insert(
                crate::editing::selection::components::SelectionRect {
                    start: drag_state.start_position,
                    end: drag_state.current_position,
                }
            );
        } else {
            // Create new marquee entity
            commands.spawn(
                crate::editing::selection::components::SelectionRect {
                    start: drag_state.start_position,
                    end: drag_state.current_position,
                }
            );
        }

        trace!("🔍 SELECT: Updated marquee from {:?} to {:?} (distance: {})",
            drag_state.start_position, drag_state.current_position, drag_distance);
    } else {
        // Clean up marquee when not dragging
        for entity in existing_marquee.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Debug system to track select tool state
fn debug_select_tool_state(
    select_mode: Res<SelectModeActive>,
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
) {
    if select_mode.is_changed() || current_tool.is_changed() {
        debug!(
            "🔍 SELECT_DEBUG: SelectModeActive = {}, CurrentTool = {:?}",
            select_mode.0,
            current_tool.get_current()
        );
    }
}
