//! Mesh-based sort handle rendering
//!
//! This module provides mesh-based handle rendering for sorts.
//! All gizmo-based rendering has been removed in favor of camera-responsive mesh rendering.

#![allow(clippy::too_many_arguments)]
#![allow(clippy::uninlined_format_args)]

use crate::core::state::FontIRAppState;
use crate::editing::selection::components::Selected;
use crate::rendering::zoom_aware_scaling::CameraResponsiveScale;
use crate::systems::sort_manager::SortPointEntity;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortRenderStyle {
    TextBuffer,
    Freeform,
}

/// Component to mark entities as sort handle visual elements
#[derive(Component)]
pub struct SortHandle {
    pub sort_entity: Entity,
    pub handle_type: SortHandleType,
}

/// Types of sort handle elements
#[derive(Debug, Clone)]
pub enum SortHandleType {
    Square,
    Circle,
    SelectionIndicator,
}

/// Resource to track sort handle entities
#[derive(Resource, Default)]
pub struct SortHandleEntities {
    pub handles: HashMap<Entity, Vec<Entity>>, // sort_entity -> handle entities
}

/// Resource to track dragging state for sort handles
#[derive(Resource, Default)]
pub struct SortHandleDragState {
    pub dragging_sort: Option<Entity>,
    pub drag_offset: Vec2,
    pub initial_position: Vec2,
}

/// Helper to spawn a box outline handle mesh
fn spawn_box_outline_handle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec2,
    size: f32,
    color: Color,
    line_width: f32,
    sort_entity: Entity,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    let half_size = size / 2.0;

    // Top line
    let top_mesh = Rectangle::new(size, line_width);
    let top_entity = commands
        .spawn((
            SortHandle {
                sort_entity,
                handle_type: SortHandleType::Square,
            },
            Mesh2d(meshes.add(top_mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_xyz(position.x, position.y + half_size - line_width / 2.0, 15.0),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            crate::editing::selection::components::Selectable,
        ))
        .id();
    entities.push(top_entity);

    // Bottom line
    let bottom_mesh = Rectangle::new(size, line_width);
    let bottom_entity = commands
        .spawn((
            SortHandle {
                sort_entity,
                handle_type: SortHandleType::Square,
            },
            Mesh2d(meshes.add(bottom_mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_xyz(position.x, position.y - half_size + line_width / 2.0, 15.0),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            crate::editing::selection::components::Selectable,
        ))
        .id();
    entities.push(bottom_entity);

    // Left line
    let left_mesh = Rectangle::new(line_width, size);
    let left_entity = commands
        .spawn((
            SortHandle {
                sort_entity,
                handle_type: SortHandleType::Square,
            },
            Mesh2d(meshes.add(left_mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_xyz(position.x - half_size + line_width / 2.0, position.y, 15.0),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            crate::editing::selection::components::Selectable,
        ))
        .id();
    entities.push(left_entity);

    // Right line
    let right_mesh = Rectangle::new(line_width, size);
    let right_entity = commands
        .spawn((
            SortHandle {
                sort_entity,
                handle_type: SortHandleType::Square,
            },
            Mesh2d(meshes.add(right_mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_xyz(position.x + half_size - line_width / 2.0, position.y, 15.0),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            crate::editing::selection::components::Selectable,
        ))
        .id();
    entities.push(right_entity);

    entities
}

/// System to render mesh-based sort handles for all sorts
#[allow(clippy::type_complexity)]
pub fn render_mesh_sort_handles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut handle_entities: ResMut<SortHandleEntities>,
    // Query all sorts (both active and inactive)
    sort_query: Query<(
        Entity,
        &Transform,
        &crate::editing::sort::Sort,
        Option<&crate::editing::sort::ActiveSort>,
        Option<&crate::editing::sort::InactiveSort>,
    )>,
    existing_handles: Query<Entity, With<SortHandle>>,
    selected_query: Query<Entity, With<Selected>>,
    fontir_app_state: Option<Res<FontIRAppState>>,
    camera_scale: Res<CameraResponsiveScale>,
    presentation_mode: Option<Res<crate::ui::edit_mode_toolbar::PresentationMode>>,
) {
    // Clear existing handles with entity existence checks
    for entity in existing_handles.iter() {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }
    handle_entities.handles.clear();

    // Hide sort handles in presentation mode
    let presentation_active = presentation_mode.is_some_and(|pm| pm.active);
    if presentation_active {
        info!("🎭 Sort handles hidden for presentation mode");
        return;
    }

    if let Some(fontir_state) = fontir_app_state {
        let fontir_metrics = fontir_state.get_font_metrics();
        let descender = fontir_metrics.descender.unwrap_or(-200.0);

        for (sort_entity, sort_transform, _sort, active, _inactive) in sort_query.iter() {
            let position = sort_transform.translation.truncate();

            // Position handle at lower left corner of the metrics box
            // The handle should be at the bottom (descender) and left edge (x=0 relative to sort)
            let handle_size = 32.0; // Fixed size for the handle box
            let handle_position =
                position + Vec2::new(handle_size / 2.0, descender + handle_size / 2.0);

            // Check if this sort is selected
            let is_selected = selected_query.iter().any(|e| e == sort_entity);

            // Determine the base color based on active/inactive state
            let base_color = if active.is_some() {
                crate::ui::theme::SORT_ACTIVE_METRICS_COLOR
            } else {
                crate::ui::theme::SORT_INACTIVE_METRICS_COLOR
            };

            // Override color to yellow if selected
            let handle_color = if is_selected {
                Color::srgb(1.0, 1.0, 0.0) // Yellow for selected
            } else {
                base_color // Use metrics color when not selected
            };

            // Camera-responsive line width
            let line_width = camera_scale.adjusted_line_width();

            // Create box outline handle
            let mut handle_entities_list = spawn_box_outline_handle(
                &mut commands,
                &mut meshes,
                &mut materials,
                handle_position,
                handle_size,
                handle_color,
                line_width,
                sort_entity,
            );

            // Add filled center circle for selected handles
            if is_selected {
                let center_radius = handle_size * 0.25; // Small circle in center
                let center_circle = commands
                    .spawn((
                        SortHandle {
                            sort_entity,
                            handle_type: SortHandleType::Square,
                        },
                        Mesh2d(meshes.add(Circle::new(center_radius))),
                        MeshMaterial2d(
                            materials.add(ColorMaterial::from_color(Color::srgb(1.0, 1.0, 0.0))),
                        ), // Yellow
                        Transform::from_xyz(handle_position.x, handle_position.y, 16.0), // Above the outline
                        GlobalTransform::default(),
                        Visibility::Visible,
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        crate::editing::selection::components::Selectable,
                    ))
                    .id();
                handle_entities_list.push(center_circle);
            }

            handle_entities
                .handles
                .insert(sort_entity, handle_entities_list);
        }
    }
}

/// System to handle sort selection and dragging initiation through handles
pub fn handle_sort_selection_and_drag_start(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::rendering::cameras::DesignCamera>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    handle_query: Query<(&Transform, &SortHandle), With<SortHandle>>,
    sort_query: Query<
        (
            Entity,
            &Transform,
            Option<&crate::editing::sort::ActiveSort>,
        ),
        With<crate::editing::sort::Sort>,
    >,
    _active_sorts: Query<Entity, With<crate::editing::sort::ActiveSort>>,
    selected_sorts: Query<Entity, (With<crate::editing::sort::Sort>, With<Selected>)>,
    ui_hover_state: Res<crate::systems::ui_interaction::UiHoverState>,
    mut drag_state: ResMut<SortHandleDragState>,
    mut active_sort_state: ResMut<crate::editing::sort::ActiveSortState>,
    mut text_editor_state: Option<ResMut<crate::core::state::text_editor::TextEditorState>>,
    buffer_index_query: Query<(
        Entity,
        &crate::systems::sorts::sort_entities::BufferSortIndex,
    )>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Don't process clicks when hovering over UI
    if ui_hover_state.is_hovering_ui {
        return;
    }

    // Check for left mouse click
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    // Get cursor position in world coordinates
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    // Check if click is on any handle
    let handle_size = 32.0; // Must match the size used in render_mesh_sort_handles
    let half_size = handle_size / 2.0;

    for (handle_transform, sort_handle) in handle_query.iter() {
        let handle_pos = handle_transform.translation.truncate();

        // Check if click is within the handle box
        if world_position.x >= handle_pos.x - half_size
            && world_position.x <= handle_pos.x + half_size
            && world_position.y >= handle_pos.y - half_size
            && world_position.y <= handle_pos.y + half_size
        {
            // Found a clicked handle
            let sort_entity = sort_handle.sort_entity;

            // Get the current active state of the clicked sort
            let _is_currently_active = sort_query
                .get(sort_entity)
                .map(|(_, _, active)| active.is_some())
                .unwrap_or(false);

            // Check if shift or cmd/ctrl is held for multi-selection
            let is_multi_select = keyboard_input.pressed(KeyCode::ShiftLeft)
                || keyboard_input.pressed(KeyCode::ShiftRight)
                || keyboard_input.pressed(KeyCode::SuperLeft)
                || keyboard_input.pressed(KeyCode::SuperRight)
                || keyboard_input.pressed(KeyCode::ControlLeft)
                || keyboard_input.pressed(KeyCode::ControlRight);

            // If not multi-selecting, clear selection from all other sorts
            if !is_multi_select {
                for selected_entity in selected_sorts.iter() {
                    if selected_entity != sort_entity {
                        if let Ok(mut entity_commands) = commands.get_entity(selected_entity) {
                            entity_commands.remove::<Selected>();
                        } else {
                            debug!(
                                "Skipping selection removal for non-existent entity {:?}",
                                selected_entity
                            );
                        }
                    }
                }
            }

            // Toggle or add selection based on current state
            let was_selected = selected_sorts.iter().any(|e| e == sort_entity);
            if is_multi_select && was_selected {
                // In multi-select mode, clicking a selected item deselects it
                if let Ok(mut entity_commands) = commands.get_entity(sort_entity) {
                    entity_commands.remove::<Selected>();
                    info!(
                        "Deselected sort {:?} via handle click (multi-select)",
                        sort_entity
                    );
                } else {
                    debug!(
                        "Skipping deselection for non-existent entity {:?}",
                        sort_entity
                    );
                }
                // Don't proceed with activation/dragging if deselecting
                continue;
            } else {
                // Select the clicked sort
                if let Ok(mut entity_commands) = commands.get_entity(sort_entity) {
                    entity_commands.insert(Selected);
                    info!("Selected sort {:?} via handle click", sort_entity);
                } else {
                    debug!(
                        "Skipping selection for non-existent entity {:?}",
                        sort_entity
                    );
                    continue; // Skip the rest if entity doesn't exist
                }
            }

            // CRITICAL FIX: Properly manage sort activation without affecting visual rendering

            // First, deactivate all currently active sorts to ensure only one is active
            for (entity, _, active_component) in sort_query.iter() {
                if active_component.is_some() && entity != sort_entity {
                    // Deactivate other sorts with entity existence check
                    if let Ok(mut entity_commands) = commands.get_entity(entity) {
                        entity_commands
                            .remove::<crate::editing::sort::ActiveSort>()
                            .insert(crate::editing::sort::InactiveSort);

                        // Check if this is a buffer sort and specifically track root sort deactivation
                        if let Ok(buffer_index) = buffer_index_query.get(entity) {
                            warn!("HANDLE SYSTEM: Deactivated BUFFER sort {:?} (index {}) - should show filled rendering!", entity, buffer_index.0);
                        } else {
                            info!("HANDLE DEBUG: Deactivated non-buffer sort {:?} via handle selection", entity);
                        }

                        // DEBUG: Check if this is a buffer sort (which should get filled rendering when inactive)
                        if let Ok(_entity_ref) = commands.get_entity(entity) {
                            // We can't directly query components from EntityCommands, so let's add a marker
                            info!("HANDLE DEBUG: Deactivated sort {:?} - checking if it's a buffer sort for filled rendering", entity);
                        }
                    } else {
                        debug!("Skipping deactivation for non-existent entity {:?}", entity);
                    }
                }
            }

            // Activate the clicked sort with entity existence check
            if let Ok(mut entity_commands) = commands.get_entity(sort_entity) {
                entity_commands
                    .remove::<crate::editing::sort::InactiveSort>()
                    .insert(crate::editing::sort::ActiveSort);
                info!("HANDLE DEBUG: Activated sort {:?} via handle selection (should get green metrics)", sort_entity);
            } else {
                debug!(
                    "Skipping activation for non-existent entity {:?}",
                    sort_entity
                );
                continue; // Skip the rest if entity doesn't exist
            }

            // Update the global active sort state
            active_sort_state.active_sort_entity = Some(sort_entity);

            // Update text editor state if this is a buffer sort
            if let (Some(text_editor_state), Ok((_, buffer_index))) = (
                text_editor_state.as_mut(),
                buffer_index_query.get(sort_entity),
            ) {
                // Ensure all other buffer sorts are marked as inactive in text editor state
                for i in 0..text_editor_state.buffer.len() {
                    if i != buffer_index.0 {
                        if let Some(sort_entry) = text_editor_state.buffer.get_mut(i) {
                            sort_entry.is_active = false;
                        }
                    }
                }

                // Activate the selected sort in text editor state
                if let Some(sort_entry) = text_editor_state.buffer.get_mut(buffer_index.0) {
                    sort_entry.is_active = true;
                    info!("Activated buffer sort {} via handle click", buffer_index.0);
                }
            }

            info!("Activated sort {:?} via handle click", sort_entity);

            // Start dragging
            if let Ok((_, sort_transform, _)) = sort_query.get(sort_entity) {
                let sort_position = sort_transform.translation.truncate();
                drag_state.dragging_sort = Some(sort_entity);
                drag_state.drag_offset = sort_position - world_position;
                drag_state.initial_position = sort_position;
                info!(
                    "Started dragging sort {:?} from position {:?}",
                    sort_entity, sort_position
                );
            }

            // Stop checking other handles since we found one
            break;
        }
    }
}

/// System to handle sort dragging updates
pub fn handle_sort_drag_update(
    mut sort_query: Query<&mut Transform, With<crate::editing::sort::Sort>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::rendering::cameras::DesignCamera>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    drag_state: Res<SortHandleDragState>,
    text_editor_state: Option<Res<crate::core::state::text_editor::TextEditorState>>,
    buffer_index_query: Query<(
        Entity,
        &crate::systems::sorts::sort_entities::BufferSortIndex,
    )>,
) {
    if let Some(dragging_sort) = drag_state.dragging_sort {
        // Get cursor position in world coordinates
        let Ok((camera, camera_transform)) = camera_query.single() else {
            return;
        };

        let Ok(window) = windows.single() else {
            return;
        };

        let Some(cursor_position) = window.cursor_position() else {
            return;
        };

        let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
        else {
            return;
        };

        // Update the sort's position with entity existence check
        if let Ok(mut transform) = sort_query.get_mut(dragging_sort) {
            let new_position = world_position + drag_state.drag_offset;
            let delta = new_position - transform.translation.truncate();
            transform.translation.x = new_position.x;
            transform.translation.y = new_position.y;

            // If this is a text sort, move all other text sorts in the buffer
            if let (Some(text_editor_state), Ok((_, buffer_index))) = (
                text_editor_state.as_ref(),
                buffer_index_query.get(dragging_sort),
            ) {
                if let Some(sort_entry) = text_editor_state.buffer.get(buffer_index.0) {
                    // Check if this is a text sort (LTR or RTL)
                    match sort_entry.layout_mode {
                        crate::core::state::SortLayoutMode::LTRText
                        | crate::core::state::SortLayoutMode::RTLText => {
                            // Move all other text sorts by the same delta
                            for (other_entity, other_buffer_index) in buffer_index_query.iter() {
                                if other_entity != dragging_sort {
                                    if let Some(other_sort) =
                                        text_editor_state.buffer.get(other_buffer_index.0)
                                    {
                                        // Check if the other sort is also a text sort
                                        match other_sort.layout_mode {
                                            crate::core::state::SortLayoutMode::LTRText
                                            | crate::core::state::SortLayoutMode::RTLText => {
                                                // Check entity exists before updating transform
                                                if let Ok(mut other_transform) =
                                                    sort_query.get_mut(other_entity)
                                                {
                                                    other_transform.translation.x += delta.x;
                                                    other_transform.translation.y += delta.y;
                                                } else {
                                                    debug!("Skipping transform update for non-existent entity {:?}", other_entity);
                                                }
                                            }
                                            _ => {} // Skip freeform sorts
                                        }
                                    }
                                }
                            }
                            info!("Moved text sort group by delta: {:?}", delta);
                        }
                        crate::core::state::SortLayoutMode::Freeform => {
                            // Freeform sorts move individually
                        }
                    }
                }
            }
        }
    }
}

/// System to handle sort drag release
pub fn handle_sort_drag_release(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<SortHandleDragState>,
    mut text_editor_state: Option<ResMut<crate::core::state::text_editor::TextEditorState>>,
    sort_query: Query<&Transform, With<crate::editing::sort::Sort>>,
    buffer_index_query: Query<(
        Entity,
        &crate::systems::sorts::sort_entities::BufferSortIndex,
    )>,
) {
    if drag_state.dragging_sort.is_some() && mouse_button_input.just_released(MouseButton::Left) {
        if let Some(dragging_sort) = drag_state.dragging_sort.take() {
            info!("Stopped dragging sort {:?}", dragging_sort);

            // Update the text editor state with the new position if applicable
            if let (Some(text_editor_state), Ok((_, buffer_index)), Ok(transform)) = (
                text_editor_state.as_mut(),
                buffer_index_query.get(dragging_sort),
                sort_query.get(dragging_sort),
            ) {
                if let Some(sort_entry) = text_editor_state.buffer.get_mut(buffer_index.0) {
                    let new_position = transform.translation.truncate();
                    sort_entry.root_position = new_position;
                    info!(
                        "Updated buffer sort {} position to {:?}",
                        buffer_index.0, new_position
                    );
                }
            }
        }

        // Clear drag state
        drag_state.drag_offset = Vec2::ZERO;
        drag_state.initial_position = Vec2::ZERO;
    }
}

pub struct SortHandleRenderingPlugin;

impl Plugin for SortHandleRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SortHandleEntities>()
            .init_resource::<SortHandleDragState>()
            .add_systems(Update, (
                render_mesh_sort_handles,
                // Handle selection should run before auto_activate_selected_sorts
                handle_sort_selection_and_drag_start
                    .before(crate::systems::sorts::sort_entities::auto_activate_selected_sorts),
                handle_sort_drag_update,
                handle_sort_drag_release,
            ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_render_style_distinction() {
        // Test that TextBuffer and Freeform styles are distinct
        assert_ne!(SortRenderStyle::TextBuffer, SortRenderStyle::Freeform);

        // Test that each style is equal to itself
        assert_eq!(SortRenderStyle::TextBuffer, SortRenderStyle::TextBuffer);
        assert_eq!(SortRenderStyle::Freeform, SortRenderStyle::Freeform);
    }

    #[test]
    fn test_sort_render_style_debug() {
        // Test that styles can be debug printed
        let text_style = SortRenderStyle::TextBuffer;
        let freeform_style = SortRenderStyle::Freeform;

        let debug_text = format!("{:?}", text_style);
        let debug_freeform = format!("{:?}", freeform_style);

        assert!(debug_text.contains("TextBuffer"));
        assert!(debug_freeform.contains("Freeform"));
    }
}
