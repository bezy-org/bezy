use bevy::prelude::*;

/// Resource to track if the mouse is currently over any UI element
#[derive(Resource, Default)]
pub struct UiHoverState {
    pub is_hovering_ui: bool,
}

/// Plugin to add UI hover detection systems
pub struct UiInteractionPlugin;

impl Plugin for UiInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiHoverState>()
            .add_systems(Update, detect_ui_hover);
    }
}

/// System to detect when the mouse is hovering over any UI element
/// This will set the UiHoverState.is_hovering_ui to true when hovering UI
pub fn detect_ui_hover(
    interaction_query: Query<(&Interaction, Option<&Node>)>,
    mut ui_hover_state: ResMut<UiHoverState>,
) {
    // Reset hover state at the beginning of each frame
    ui_hover_state.is_hovering_ui = false;

    // Check if any UI element is being hovered or interacted with
    for (interaction, node) in interaction_query.iter() {
        // Make sure we're only considering actual UI elements (with Node component)
        if node.is_some() && matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
            ui_hover_state.is_hovering_ui = true;
            return;
        }
    }
}
