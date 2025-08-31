//! Keyboard shortcut handling for selection

use crate::core::io::input::ModifierState;
use crate::editing::edit_type::EditType;
use crate::editing::selection::components::{
    GlyphPointReference, PointType, Selectable, Selected, SelectionState,
};
use crate::editing::selection::nudge::{EditEvent, NudgeState};
use bevy::input::ButtonInput;
use bevy::prelude::*;

/// System to handle selection shortcuts (Ctrl+A for select all, etc.)
#[allow(clippy::too_many_arguments)]
pub fn handle_selection_shortcuts(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    selected_query: Query<Entity, With<Selected>>,
    selectable_query: Query<Entity, With<Selectable>>,
    mut selection_state: ResMut<SelectionState>,
    mut event_writer: EventWriter<EditEvent>,
    select_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive>>,
    knife_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>>,
    text_editor_state: Option<Res<crate::core::state::TextEditorState>>,
) {
    // Skip processing shortcuts if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            return;
        }
    }

    // Only process shortcuts when in select mode
    if let Some(select_mode) = select_mode {
        if !select_mode.0 {
            return;
        }
    }

    // Only allow selection shortcuts when there's an active sort in text editor
    if let Some(text_editor_state) = text_editor_state.as_ref() {
        if text_editor_state.get_active_sort().is_none() {
            return;
        }
    } else {
        return;
    }

    // Handle Escape key to clear selection
    if keyboard_input.just_pressed(KeyCode::Escape) {
        for entity in &selected_query {
            commands.entity(entity).remove::<Selected>();
        }
        selection_state.selected.clear();
        debug!("Cleared selection");
    }

    // Handle Ctrl+A (select all)
    let ctrl_pressed = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight)
        || keyboard_input.pressed(KeyCode::SuperLeft)
        || keyboard_input.pressed(KeyCode::SuperRight);

    if ctrl_pressed && keyboard_input.just_pressed(KeyCode::KeyA) {
        debug!("Select all shortcut pressed");

        // Clear current selection
        for entity in &selected_query {
            commands.entity(entity).remove::<Selected>();
        }
        selection_state.selected.clear();

        // Select all selectable entities
        for entity in &selectable_query {
            selection_state.selected.insert(entity);
            commands.entity(entity).insert(Selected);
        }

        debug!("Selected all {} entities", selection_state.selected.len());

        // Send edit event
        event_writer.write(EditEvent {
            edit_type: EditType::Normal,
        });
    }
}

/// System to handle key releases for nudging
pub fn handle_key_releases(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut nudge_state: ResMut<NudgeState>,
) {
    // Reset nudging state if no arrow keys are pressed
    let arrow_keys = [
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
    ];

    let any_arrow_pressed = arrow_keys.iter().any(|key| keyboard_input.pressed(*key));

    if !any_arrow_pressed {
        nudge_state.is_nudging = false;
    }
}

/// Handle key press for selection shortcuts
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn handle_selection_key_press(
    commands: &mut Commands,
    key: &KeyCode,
    modifiers: &ModifierState,
    selectable_query: &Query<
        (
            Entity,
            &GlobalTransform,
            Option<&GlyphPointReference>,
            Option<&PointType>,
        ),
        With<Selectable>,
    >,
    selected_query: &Query<(Entity, &Transform), With<Selected>>,
    selection_state: &mut ResMut<SelectionState>,
    event_writer: &mut EventWriter<EditEvent>,
    active_sort_entity: Entity,
    sort_point_entities: &Query<&crate::systems::sort_manager::SortPointEntity>,
) {
    match key {
        KeyCode::KeyA => {
            if modifiers.ctrl {
                // Ctrl+A: Select all points in the active sort
                debug!("Select all shortcut triggered for active sort");
                let mut selected_count = 0;

                for (entity, _, _, _) in selectable_query.iter() {
                    // Check if this entity belongs to the active sort
                    if let Ok(sort_point_entity) = sort_point_entities.get(entity) {
                        if sort_point_entity.sort_entity != active_sort_entity {
                            continue; // Skip points that don't belong to the active sort
                        }
                    } else {
                        continue; // Skip entities that aren't sort points
                    }

                    if !selection_state.selected.contains(&entity) {
                        selection_state.selected.insert(entity);
                        commands.entity(entity).insert(Selected);
                        selected_count += 1;
                    }
                }

                event_writer.write(EditEvent {
                    edit_type: EditType::Normal,
                });
                debug!("Selected all {} points in active sort", selected_count);
            }
        }
        KeyCode::Escape => {
            // Escape: Clear selection
            debug!("Escape key pressed - clearing selection");
            for (entity, _) in selected_query.iter() {
                commands.entity(entity).remove::<Selected>();
            }
            selection_state.selected.clear();
            event_writer.write(EditEvent {
                edit_type: EditType::Normal,
            });
        }
        _ => {}
    }
}
