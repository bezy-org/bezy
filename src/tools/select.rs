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
            .init_resource::<SelectToolDragState>()
            .add_systems(Startup, select_tool_startup_log)
            .add_systems(
                Update,
                (
                    handle_select_tool_activation,
                    sync_select_mode_with_tool_state,
                    handle_select_tool_input, // Direct input handling
                    handle_select_tool_drag, // Marquee selection
                    render_selection_marquee, // Visual feedback
                    debug_select_tool_state,
                )
                .chain()
                .run_if(resource_exists::<crate::tools::ToolState>),
            );
    }
}

/// Direct input handling for select tool that bypasses input_consumer.rs for better performance
fn handle_select_tool_input(
    tool_state: Res<crate::tools::ToolState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<bevy::core_pipeline::core_2d::Camera2d>>,
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

    // Handle left mouse button release for selection (not press, to avoid conflict with drag)
    if mouse_input.just_released(MouseButton::Left) {
        // Only handle as click if we weren't dragging
        let drag_threshold = 5.0;
        if drag_state.start_position.distance(drag_state.current_position) < drag_threshold {
            let Ok(window) = windows.get_single() else {
                return;
            };

        let Some(cursor_position) = window.cursor_position() else {
            return;
        };

        // Convert screen coordinates to world coordinates
        let Ok((camera, camera_transform)) = camera_query.get_single() else {
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

        // Find the closest selectable entity within a threshold
        let selection_threshold = 10.0; // World units
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
                // Clear all selections first (unless clicking on already selected item)
                if !is_already_selected {
                    for (entity, _, is_selected) in selectable_query.iter() {
                        if is_selected.is_some() {
                            commands.entity(entity).remove::<crate::editing::selection::components::Selected>();
                        }
                    }
                }

                // Select the clicked entity
                commands.entity(entity).insert(crate::editing::selection::components::Selected);
                debug!("🔍 SELECT TOOL: Selected entity {:?}", entity);
            }
        } else if !shift_held {
            // Clicked on empty space - clear selection
            for (entity, _, is_selected) in selectable_query.iter() {
                if is_selected.is_some() {
                    commands.entity(entity).remove::<crate::editing::selection::components::Selected>();
                }
            }
            debug!("🔍 SELECT TOOL: Cleared selection (clicked empty space)");
        }
        } // End of drag_threshold check
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

/// Handle drag selection (marquee)
fn handle_select_tool_drag(
    tool_state: Res<crate::tools::ToolState>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<bevy::core_pipeline::core_2d::Camera2d>>,
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

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        // If cursor left window, end drag
        if drag_state.is_dragging {
            drag_state.end_drag();
        }
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Ok(world_position) = camera
        .viewport_to_world_2d(camera_transform, cursor_position)
    else {
        return;
    };

    // Start drag on mouse down
    if mouse_input.just_pressed(MouseButton::Left) && !drag_state.is_dragging {
        drag_state.start_drag(world_position);
    }

    // Update drag position
    if mouse_input.pressed(MouseButton::Left) && drag_state.is_dragging {
        drag_state.update_drag(world_position);

        // Check if this is actually a drag (not just a click)
        let drag_threshold = 5.0;
        if drag_state.start_position.distance(drag_state.current_position) > drag_threshold {
            // We're dragging - prevent the click selection
            // This is handled by checking drag_state in handle_select_tool_input
        }
    }

    // End drag and select entities within marquee
    if mouse_input.just_released(MouseButton::Left) && drag_state.is_dragging {
        let drag_distance = drag_state.start_position.distance(drag_state.current_position);
        let drag_threshold = 5.0;

        if drag_distance > drag_threshold {
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
        }

        drag_state.end_drag();
    }
}

/// Render the selection marquee
fn render_selection_marquee(
    tool_state: Res<crate::tools::ToolState>,
    drag_state: Res<SelectToolDragState>,
    mut gizmos: Gizmos,
) {
    // Only render if select tool is active and dragging
    if !tool_state.is_active(crate::tools::ToolId::Select) || !drag_state.is_dragging {
        return;
    }

    // Don't draw marquee for tiny drags
    let drag_threshold = 5.0;
    if drag_state.start_position.distance(drag_state.current_position) < drag_threshold {
        return;
    }

    let (min, max) = drag_state.get_bounds();

    // Draw marquee rectangle
    let color = Color::srgba(0.3, 0.6, 1.0, 0.3); // Semi-transparent blue
    let border_color = Color::srgba(0.3, 0.6, 1.0, 0.8);

    // Draw filled rect (as lines for now since Bevy doesn't have filled rect gizmo)
    gizmos.line_2d(Vec2::new(min.x, min.y), Vec2::new(max.x, min.y), border_color);
    gizmos.line_2d(Vec2::new(max.x, min.y), Vec2::new(max.x, max.y), border_color);
    gizmos.line_2d(Vec2::new(max.x, max.y), Vec2::new(min.x, max.y), border_color);
    gizmos.line_2d(Vec2::new(min.x, max.y), Vec2::new(min.x, min.y), border_color);
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
