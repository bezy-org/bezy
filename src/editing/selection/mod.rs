#![allow(unused_imports)]

use crate::core::state::AppState;
use crate::editing::selection::systems::*;
use crate::editing::FontEditorSets;
use bevy::prelude::*;

pub mod components;
pub mod coordinate_system;
pub mod enhanced_point_component;
pub mod entity_management;
pub mod events;
pub mod input;
pub mod nudge;
pub mod point_movement;
pub mod systems;
pub mod utils;

// Explicit re-exports for public API
// Components
pub use components::{
    FontIRPointReference, GlyphPointReference, PointType, Selectable, Selected, SelectionRect,
    SelectionState,
};
// Enhanced point types
pub use enhanced_point_component::{EnhancedPointType, EnhancedPointTypePlugin, migrate_point_types};
// Entity management
pub use entity_management::{
    EnhancedPointAttributes, cleanup_click_resource, despawn_inactive_sort_points,
    spawn_active_sort_points, sync_enhanced_point_attributes, sync_point_positions_to_sort,
    update_glyph_data_from_selection,
};
// Events
pub use events::{AppStateChanged, ClickWorldPosition, SELECTION_MARGIN};
// Input handling
pub use input::mouse::{
    DoubleClickState, SelectionInputEvents, DOUBLE_CLICK_THRESHOLD_SECS,
    collect_selection_input_events, find_clicked_point, handle_selection_click,
    handle_selection_drag, handle_selection_release, handle_smooth_point_toggle,
    process_selection_input_events,
};
pub use input::drag::handle_point_drag;
pub use input::shortcuts::{
    handle_key_releases, handle_selection_key_press, handle_selection_shortcuts,
};
// Nudge functionality
pub use nudge::{
    EditEvent, NudgePlugin, NudgeState, PointCoordinates, handle_nudge_input, reset_nudge_state,
    sync_nudged_points_on_completion,
};
// Utilities
pub use utils::clear_selection_on_app_change;

use std::collections::HashMap;

/// Resource to track the drag selection state
#[derive(Resource, Default)]
pub struct DragSelectionState {
    /// Whether a drag selection is in progress
    pub is_dragging: bool,
    /// The start position of the drag selection (in design space)
    pub start_position: Option<crate::geometry::world_space::DPoint>,
    /// The current position of the drag selection (in design space)
    pub current_position: Option<crate::geometry::world_space::DPoint>,
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
            .register_type::<NudgeState>()
            // Register components
            .register_type::<Selectable>()
            .register_type::<Selected>()
            .register_type::<SelectionRect>()
            .register_type::<PointType>()
            .register_type::<GlyphPointReference>()
            // Register resources
            .init_resource::<SelectionState>()
            .init_resource::<DragSelectionState>()
            .init_resource::<DragPointState>()
            .init_resource::<DoubleClickState>()
            .init_resource::<input::mouse::SelectionInputEvents>()
            .init_resource::<entity_management::sync::EnhancedPointAttributes>()
            // SelectModeActive is now properly managed by SelectToolPlugin
            // Configure system sets for proper ordering
            // Selection systems now use FontEditorSets for better integration
            // NOTE: Input handling moved to SelectionInputConsumer in input_consumer.rs
            // to prevent event consumption conflicts
            .add_systems(Update, input::drag::handle_point_drag)
            // Processing systems
            .add_systems(
                Update,
                (
                    // TEMP DISABLED: Causing performance lag during text input
                    // sync_selected_components,
                    // DISABLED: Uses old AppState instead of FontIRAppState
                    // entity_management::update_glyph_data_from_selection,
                    entity_management::sync_enhanced_point_attributes,
                    crate::editing::smooth_curves::auto_apply_smooth_constraints,
                    crate::editing::smooth_curves::universal_smooth_constraints,
                    clear_selection_on_app_change,
                    entity_management::cleanup_click_resource,
                )
                    .in_set(FontEditorSets::Rendering),
            )
            // Rendering systems - moved to PostUpdate to run after transform propagation
            .add_systems(
                PostUpdate,
                (
                    crate::rendering::selection::render_selection_marquee,
                    utils::debug_print_selection_rects, // TEMP: debug system
                )
                    .in_set(FontEditorSets::Rendering),
            )
            // Add the nudge plugin
            .add_plugins(NudgePlugin);

        // Register debug validation system only in debug builds
        #[cfg(debug_assertions)]
        app.add_systems(
            PostUpdate,
            utils::debug_validate_point_entity_uniqueness.in_set(FontEditorSets::Cleanup),
        );
    }
}

// SelectionSystemSet removed - now using FontEditorSets for better integration

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
            debug!(
                "Adding Selected component to entity {:?} from selection state",
                entity
            );
        }
    }

    // Then, ensure all entities with the Selected component are in the selection_state
    for entity in &selected_entities {
        if !selection_state.selected.contains(&entity) {
            commands.entity(entity).remove::<Selected>();
            debug!(
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
