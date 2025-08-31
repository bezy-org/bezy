#![allow(unused_imports)]

use crate::core::state::AppState;
use crate::editing::edit_type::EditType;
use crate::editing::selection::systems::*;
use crate::editing::UndoPlugin;
use bevy::prelude::*;

pub mod components;
pub mod coordinate_system;
pub mod entity_management;
pub mod events;
pub mod input;
pub mod nudge;
pub mod systems;
pub mod utils;

pub use components::*;
pub use entity_management::*;
pub use events::{AppStateChanged, ClickWorldPosition, SELECTION_MARGIN};
pub use input::*;
pub use nudge::*;
pub use utils::{clear_selection_on_app_change, update_hover_state};

use std::collections::HashMap;

/// Resource to track the drag selection state
#[derive(Resource, Default)]
pub struct DragSelectionState {
    /// Whether a drag selection is in progress
    pub is_dragging: bool,
    /// The start position of the drag selection (in design space)
    pub start_position: Option<crate::geometry::design_space::DPoint>,
    /// The current position of the drag selection (in design space)
    pub current_position: Option<crate::geometry::design_space::DPoint>,
    /// Whether this is a multi-select operation (shift is held)
    pub is_multi_select: bool,
    /// The previous selection before the drag started
    pub previous_selection: Vec<Entity>,
    pub selection_rect_entity: Option<Entity>,
}

/// Resource to track the state of dragging points
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct DragPointState {
    /// Whether a point drag is in progress
    pub is_dragging: bool,
    /// The start position of the drag
    pub start_position: Option<Vec2>,
    /// The current position of the drag
    pub current_position: Option<Vec2>,
    /// The entities being dragged
    #[reflect(ignore)]
    pub dragged_entities: Vec<Entity>,
    /// The original positions of the dragged entities
    #[reflect(ignore)]
    pub original_positions: HashMap<Entity, Vec2>,
}

/// Plugin to add selection functionality to the font editor
pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add events
            .add_event::<AppStateChanged>()
            .add_event::<EditEvent>()
            .register_type::<EditType>()
            .register_type::<NudgeState>()
            // Register components
            .register_type::<Selectable>()
            .register_type::<Selected>()
            .register_type::<Hovered>()
            .register_type::<SelectionRect>()
            .register_type::<PointType>()
            .register_type::<GlyphPointReference>()
            // Register resources
            .init_resource::<SelectionState>()
            .init_resource::<DragSelectionState>()
            .init_resource::<DragPointState>()
            // TEMP FIX: Manually initialize SelectModeActive since it's not being created
            .insert_resource(crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive(true))
            // Configure system sets for proper ordering
            .configure_sets(
                Update,
                (SelectionSystemSet::Input, SelectionSystemSet::Processing).chain(),
            )
            .configure_sets(PostUpdate, (SelectionSystemSet::Render,))
            // Input systems - the process_selection_input_events system handles the actual selection logic
            // It's called by the centralized input consumer system when in select mode
            .add_systems(Update, input::process_selection_input_events)
            .add_systems(Update, input::handle_point_drag)
            // Processing systems
            .add_systems(
                Update,
                (
                    // TEMP DISABLED: Causing performance lag during text input
                    // sync_selected_components,
                    // entity_management::update_glyph_data_from_selection,
                    clear_selection_on_app_change,
                    entity_management::cleanup_click_resource,
                )
                    .in_set(SelectionSystemSet::Processing)
                    .after(SelectionSystemSet::Input),
            )
            // Add the new ECS-based point management systems
            // TEMP DISABLED: Causing performance lag during text input
            // .add_systems(
            //     Update,
            //     (
            //         // entity_management::spawn_active_sort_points, // DISABLED: Causes duplicate point entities
            //         entity_management::despawn_inactive_sort_points,
            //         entity_management::sync_point_positions_to_sort,
            //     )
            //         .after(entity_management::update_glyph_data_from_selection),
            // )
            // Rendering systems - moved to PostUpdate to run after transform propagation
            .add_systems(
                PostUpdate,
                (
                    crate::rendering::selection::render_selection_marquee,
                    // DISABLED: These are now handled by the unified glyph editing system
                    // crate::rendering::selection::render_selected_entities,
                    // crate::rendering::selection::render_all_point_entities,
                    // crate::rendering::selection::render_control_handles,
                    utils::debug_print_selection_rects, // TEMP: debug system
                )
                    .in_set(SelectionSystemSet::Render),
            )
            // Add the nudge plugin
            .add_plugins(NudgePlugin);

        // Register debug validation system only in debug builds
        #[cfg(debug_assertions)]
        app.add_systems(
            PostUpdate,
            utils::debug_validate_point_entity_uniqueness.after(SelectionSystemSet::Render),
        );
    }
}

/// System sets for Selection
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SelectionSystemSet {
    #[allow(dead_code)]
    Input,
    #[allow(dead_code)]
    Processing,
    #[allow(dead_code)]
    Render,
}

/// System to ensure Selected components are synchronized with SelectionState
pub fn sync_selected_components(
    mut commands: Commands,
    selection_state: Res<SelectionState>,
    selected_entities: Query<Entity, With<Selected>>,
    entities: Query<Entity>,
) {
    // Only log when there are changes to synchronize to avoid spam
    if !selection_state.selected.is_empty() || selected_entities.iter().count() > 0 {
        debug!(
            "Synchronizing Selected components with SelectionState (current: {})",
            selection_state.selected.len()
        );
    }

    // First, ensure all entities in the selection_state have the Selected component
    for &entity in &selection_state.selected {
        // Only add the component if the entity is valid
        if entities.contains(entity) && !selected_entities.contains(entity) {
            commands.entity(entity).insert(Selected);
            info!(
                "Adding Selected component to entity {:?} from selection state",
                entity
            );
        }
    }

    // Then, ensure all entities with the Selected component are in the selection_state
    for entity in &selected_entities {
        if !selection_state.selected.contains(&entity) {
            commands.entity(entity).remove::<Selected>();
            info!(
                "Removing Selected component from entity {:?} not in selection state",
                entity
            );
        }
    }
}

#[allow(dead_code)]
fn selection_drag_active(drag_state: Res<DragSelectionState>) -> bool {
    drag_state.is_dragging
}
