//! Selection-related events and resources

use bevy::prelude::*;

/// Event to signal that app state has changed
#[derive(Event, Debug, Clone)]
pub struct AppStateChanged;

/// A resource to hold the world position of a handled click.
/// This prevents multiple systems from reacting to the same click event.
#[derive(Resource)]
pub struct ClickWorldPosition;

/// Constants for selection
pub const SELECTION_MARGIN: f32 = 16.0; // Distance in pixels for selection hit testing
