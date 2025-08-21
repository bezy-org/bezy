use bevy::prelude::*;
use std::sync::Arc;

use crate::editing::edit_type::EditType;
use crate::editing::sort::Sort;
use crate::editing::undo::UndoState;

type UndoableState = Vec<(Entity, Sort)>;

/// Resource that holds the undo/redo stack
#[derive(Resource, Debug)]
pub struct UndoStateResource {
    /// The undo stack containing all edit session states
    pub undos: UndoState<Arc<UndoableState>>,
    /// The last edit type that was processed
    #[allow(dead_code)]
    last_edit_type: Option<EditType>,
}

impl Default for UndoStateResource {
    fn default() -> Self {
        Self {
            undos: UndoState::new(Arc::new(vec![])),
            last_edit_type: None,
        }
    }
}

impl UndoStateResource {
    /// Get the last edit type
    #[allow(dead_code)]
    pub fn last_edit_type(&self) -> Option<EditType> {
        self.last_edit_type
    }

    /// Set the last edit type
    #[allow(dead_code)]
    pub fn set_last_edit_type(&mut self, edit_type: EditType) {
        self.last_edit_type = Some(edit_type);
    }

    /// Push a new state onto the undo stack
    pub fn push_undo_state(&mut self, state: Arc<UndoableState>) {
        self.undos.push(state);
    }

    /// Update the current undo state without adding a new history item
    #[allow(dead_code)]
    pub fn update_current_undo(&mut self, state: Arc<UndoableState>) {
        self.undos.update_current(state);
    }
}

/// System to initialize the undo stack with the first state
pub fn initialize_undo_stack(
    mut undo_resource: ResMut<UndoStateResource>,
    sorts: Query<(Entity, &Sort)>,
    mut initialized: Local<bool>,
) {
    if *initialized || sorts.is_empty() {
        return;
    }

    let initial_state: Vec<(Entity, Sort)> =
        sorts.iter().map(|(e, s)| (e, s.clone())).collect();
    if !initial_state.is_empty() {
        debug!("Initializing undo stack with initial sort state");
        undo_resource.undos = UndoState::new(Arc::new(initial_state));
        *initialized = true;
    }
}

/// System to save the state of all sorts when they are changed
pub fn save_sort_state(
    mut undo_resource: ResMut<UndoStateResource>,
    changed_sorts: Query<&Sort, Changed<Sort>>,
    all_sorts: Query<(Entity, &Sort)>,
) {
    if changed_sorts.is_empty() {
        return;
    }

    let current_state: Arc<UndoableState> =
        Arc::new(all_sorts.iter().map(|(e, s)| (e, s.clone())).collect());
    undo_resource.push_undo_state(current_state);
    debug!("Saved new sort state to undo stack");
}

/// System to handle undo/redo keyboard shortcuts
pub fn handle_undo_redo_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut undo_state: ResMut<UndoStateResource>,
    mut sorts: Query<&mut Sort>,
) {
    // Check for Command/Control key
    let modifier_pressed = keyboard.pressed(KeyCode::SuperLeft)
        || keyboard.pressed(KeyCode::SuperRight)
        || keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight);

    if !modifier_pressed {
        return;
    }

    let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);

    let state_to_restore =
        if keyboard.just_pressed(KeyCode::KeyZ) && !shift_pressed {
            debug!("Undo shortcut detected (Cmd+Z)");
            undo_state.undos.undo()
        } else if keyboard.just_pressed(KeyCode::KeyZ) && shift_pressed {
            debug!("Redo shortcut detected (Cmd+Shift+Z)");
            undo_state.undos.redo()
        } else {
            None
        };

    if let Some(state) = state_to_restore {
        debug!("Restoring sort state from undo/redo stack");
        for (entity, sort_state) in state.iter() {
            if let Ok(mut sort) = sorts.get_mut(*entity) {
                *sort = sort_state.clone();
            }
        }
    }
}

/// Plugin to set up the undo/redo system
pub struct UndoPlugin;

impl Plugin for UndoPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize the undo state resource
            .init_resource::<UndoStateResource>()
            // Add the systems
            .add_systems(
                Update,
                (
                    initialize_undo_stack,
                    handle_undo_redo_shortcuts,
                    save_sort_state.after(initialize_undo_stack),
                )
                    .chain(),
            );
    }
}
