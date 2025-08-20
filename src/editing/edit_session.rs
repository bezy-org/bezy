#![allow(deprecated)]

use crate::editing::selection::components::Selected;
use bevy::prelude::*;
use std::collections::HashMap;

/// A session for editing a glyph
#[derive(Component, Debug, Clone, Default)]
pub struct EditSession {
    /// The selection state
    pub selection_count: usize,
    /// Point positions (entity ID -> position)
    pub point_positions: HashMap<Entity, Vec2>,
}

impl EditSession {
    /// Check if the selection is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.selection_count == 0
    }

    /// Nudge the selected points by the given amount
    #[allow(dead_code)]
    pub fn nudge_selection(&mut self, nudge: bevy::prelude::Vec2) {
        if self.is_empty() {
            return;
        }

        debug!("Nudging selection by {:?}", nudge);
        // Apply nudge to all stored points
        for position in self.point_positions.values_mut() {
            *position += nudge;
        }
    }
}

/// Plugin to register the EditSession component and systems
pub struct EditSessionPlugin;

impl Plugin for EditSessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_edit_session).add_systems(
            Update,
            (
                update_edit_session,
                sync_transforms_with_session,
                debug_print_edit_sessions,
            ),
        );
    }
}

/// System to create the initial edit session
fn create_edit_session(mut commands: Commands) {
    debug!("Creating initial edit session");
    commands.spawn(EditSession::default());
}

/// System to update the edit session with current point positions
fn update_edit_session(
    mut edit_sessions: Query<&mut EditSession>,
    selected_points: Query<(Entity, &Transform), With<Selected>>,
    all_points: Query<
        (Entity, &Transform),
        With<crate::editing::selection::components::Selectable>,
    >,
) {
    if let Ok(mut session) = edit_sessions.get_single_mut() {
        // Update selection count
        session.selection_count = selected_points.iter().count();

        // Clear the previous session's position map to avoid stale entries
        session.point_positions.clear();

        // First track all points for complete snapshots
        for (entity, transform) in all_points.iter() {
            session.point_positions.insert(
                entity,
                Vec2::new(transform.translation.x, transform.translation.y),
            );
        }
    }
}

/// System to update transforms from edit session after undo/redo
fn sync_transforms_with_session(
    edit_sessions: Query<&EditSession, Changed<EditSession>>,
    mut transforms: Query<(&mut Transform, Entity)>,
) {
    // Only run this system when the EditSession has been explicitly changed
    // (e.g., through undo/redo operations)
    if let Ok(session) = edit_sessions.get_single() {
        // Only process if we have stored point positions
        if !session.point_positions.is_empty() {
            // Count how many transforms we updated
            let mut updated_count = 0;

            for (mut transform, entity) in transforms.iter_mut() {
                if let Some(position) = session.point_positions.get(&entity) {
                    // Update transform position from stored position
                    transform.translation.x = position.x;
                    transform.translation.y = position.y;
                    updated_count += 1;
                }
            }

            if updated_count > 0 {
                debug!("Synced {} transforms from EditSession", updated_count);
            }
        }
    }
}

/// Debug system to print out all entities with the EditSession component
fn debug_print_edit_sessions(edit_sessions: Query<Entity, With<EditSession>>) {
    if !edit_sessions.is_empty() {
        debug!(
            "Found {} EditSession entities: {:?}",
            edit_sessions.iter().count(),
            edit_sessions.iter().collect::<Vec<_>>()
        );
    }
}
