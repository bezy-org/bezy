//! Text buffer management system
//!
//! Manages text buffer entities and their relationship with sort entities.
//! Handles buffer creation, cursor management, and buffer-sort synchronization.

use crate::core::state::text_editor::buffer::BufferId;
use crate::core::state::text_editor::{
    ActiveTextBuffer, BufferCursor, BufferMember, BufferSystemSet, SortLayoutMode, TextBuffer,
    TextEditorState,
};
use crate::editing::sort::{ActiveSort, Sort};
use bevy::prelude::*;

/// Visual marker component for text buffer entities
/// This creates a small page icon to show where text flows begin
#[derive(Component, Debug)]
pub struct BufferVisualMarker {
    /// The layout mode of this buffer (for styling the marker)
    pub layout_mode: SortLayoutMode,
}

/// Plugin for text buffer management
pub struct TextBufferManagerPlugin;

impl Plugin for TextBufferManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveTextBuffer>()
            .configure_sets(
                Update,
                (
                    BufferSystemSet::UpdateBuffers,
                    BufferSystemSet::SyncMembership,
                    BufferSystemSet::RenderBuffers,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    create_missing_buffer_entities.in_set(BufferSystemSet::UpdateBuffers),
                    sync_buffer_membership.in_set(BufferSystemSet::SyncMembership),
                    update_active_buffer.in_set(BufferSystemSet::UpdateBuffers),
                    // TEMPORARILY DISABLED: Buffer visual markers (page icons with direction arrows)
                    // render_buffer_markers.in_set(BufferSystemSet::RenderBuffers),
                ),
            );
    }
}

/// Creates a new text buffer entity with the specified parameters
pub fn create_text_buffer(
    commands: &mut Commands,
    id: BufferId,
    layout_mode: SortLayoutMode,
    root_position: Vec2,
    initial_cursor_position: usize,
) -> Entity {
    let buffer_entity = commands
        .spawn((
            TextBuffer::new(id, layout_mode.clone(), root_position),
            BufferCursor::new(initial_cursor_position),
            BufferVisualMarker {
                layout_mode: layout_mode.clone(),
            },
            Name::new(format!("TextBuffer-{:?}-{:?}", id.0, layout_mode)),
        ))
        .id();

    debug!(
        "📝 BUFFER CREATED: Entity {:?}, BufferId {:?}, layout: {:?}, position: ({:.1}, {:.1}), cursor: {}",
        buffer_entity, id.0, layout_mode, root_position.x, root_position.y, initial_cursor_position
    );

    buffer_entity
}

/// Links a sort entity to a text buffer as a member
pub fn add_sort_to_buffer(
    commands: &mut Commands,
    sort_entity: Entity,
    buffer_entity: Entity,
    buffer_index: usize,
) {
    commands
        .entity(sort_entity)
        .insert(BufferMember::new(buffer_entity, buffer_index));

    debug!(
        "🔗 BUFFER MEMBERSHIP: Sort {:?} added to buffer {:?} at index {}",
        sort_entity, buffer_entity, buffer_index
    );
}

/// System to create missing buffer entities for sorts that have buffer_id but no buffer entity
/// This fixes the startup system which creates legacy sorts without buffer entities
pub fn create_missing_buffer_entities(
    mut commands: Commands,
    text_editor_state: Option<Res<TextEditorState>>,
    buffer_query: Query<&TextBuffer, With<TextBuffer>>,
    _sort_query: Query<
        &Sort,
        (
            With<Sort>,
            With<crate::systems::sorts::sort_entities::BufferSortIndex>,
        ),
    >,
) {
    let Some(text_state) = text_editor_state else {
        return;
    };

    // Build a map of existing buffer entities by BufferId
    let mut existing_buffer_ids = std::collections::HashSet::new();
    for text_buffer in buffer_query.iter() {
        existing_buffer_ids.insert(text_buffer.id);
    }

    // Check all sorts in the text editor state for missing buffer entities
    let mut missing_buffer_ids: std::collections::HashMap<BufferId, (SortLayoutMode, Vec2)> =
        std::collections::HashMap::new();
    for (index, sort_entry) in text_state.buffer.iter().enumerate() {
        if let Some(buffer_id) = sort_entry.buffer_id {
            if !existing_buffer_ids.contains(&buffer_id) {
                missing_buffer_ids.insert(
                    buffer_id,
                    (sort_entry.layout_mode.clone(), sort_entry.root_position),
                );
                debug!("🔍 MISSING BUFFER: Found sort at buffer[{}] with buffer_id {:?} but no corresponding buffer entity", 
                      index, buffer_id.0);
            }
        }
    }

    // Create missing buffer entities
    for (buffer_id, (layout_mode, root_position)) in missing_buffer_ids {
        let buffer_entity = create_text_buffer(
            &mut commands,
            buffer_id,
            layout_mode.clone(),
            root_position,
            1, // Start cursor after the first character (matching startup behavior)
        );

        debug!(
            "🆕 AUTO-CREATED: Buffer entity {:?} for buffer_id {:?} at position ({:.1}, {:.1})",
            buffer_entity, buffer_id.0, root_position.x, root_position.y
        );
    }
}

/// System to sync buffer membership and maintain consistency
pub fn sync_buffer_membership(
    _commands: Commands,
    text_editor_state: Option<Res<TextEditorState>>,
    buffer_query: Query<(Entity, &TextBuffer), With<TextBuffer>>,
    sort_query: Query<Entity, (With<Sort>, Without<BufferMember>)>,
    _buffer_member_query: Query<&mut BufferMember>,
) {
    let Some(text_state) = text_editor_state else {
        return;
    };

    // Get all text sorts from the text editor state
    let buffer_sorts = text_state.get_text_sorts();
    if buffer_sorts.is_empty() {
        return;
    }

    // Find existing buffers by BufferId
    let mut buffer_entities = std::collections::HashMap::new();
    for (entity, text_buffer) in buffer_query.iter() {
        buffer_entities.insert(text_buffer.id, entity);
    }

    // For now, we'll need a way to map from text editor state to actual sort entities
    // This is a placeholder - in practice we need entity tracking
    let sort_entities: Vec<_> = sort_query.iter().collect();

    debug!(
        "🔄 BUFFER SYNC: Found {} buffer sorts, {} buffer entities, {} unmapped sort entities",
        buffer_sorts.len(),
        buffer_entities.len(),
        sort_entities.len()
    );
}

/// System to update the active buffer based on active sorts
pub fn update_active_buffer(
    mut active_buffer: ResMut<ActiveTextBuffer>,
    active_sort_query: Query<&BufferMember, (With<ActiveSort>, Changed<ActiveSort>)>,
    all_active_sort_query: Query<&BufferMember, With<ActiveSort>>,
    buffer_query: Query<Entity, With<TextBuffer>>,
) {
    // If an active sort changed, update the active buffer
    if let Ok(buffer_member) = active_sort_query.single() {
        let old_active = active_buffer.buffer_entity;
        active_buffer.buffer_entity = Some(buffer_member.buffer_entity);

        if old_active != active_buffer.buffer_entity {
            debug!(
                "🎯 ACTIVE BUFFER CHANGED: {:?} -> {:?}",
                old_active, active_buffer.buffer_entity
            );
        }
        return;
    }

    // If no active buffer is set but there are active sorts, try to set one
    if active_buffer.buffer_entity.is_none() {
        if let Ok(buffer_member) = all_active_sort_query.single() {
            active_buffer.buffer_entity = Some(buffer_member.buffer_entity);
            debug!(
                "🎯 AUTO-SET ACTIVE BUFFER: Set to {:?} based on existing active sort",
                active_buffer.buffer_entity
            );
            return;
        }

        // If no sorts have buffer members yet, but there's at least one buffer entity,
        // we might be in startup mode - this will be handled by the spawn_missing_sort_entities system
        if buffer_query.iter().count() > 0 {
            let first_buffer = buffer_query.iter().next();
            if let Some(buffer_entity) = first_buffer {
                active_buffer.buffer_entity = Some(buffer_entity);
                debug!(
                    "🎯 STARTUP FALLBACK: Set active buffer to first available buffer entity {:?}",
                    buffer_entity
                );
            }
        }
    }
}

/// Helper function to get the cursor position for a buffer
pub fn get_buffer_cursor_position(
    buffer_entity: Entity,
    cursor_query: &Query<&BufferCursor>,
) -> Option<usize> {
    cursor_query
        .get(buffer_entity)
        .ok()
        .map(|cursor| cursor.position)
}

/// Helper function to set the cursor position for a buffer
pub fn set_buffer_cursor_position(
    commands: &mut Commands,
    buffer_entity: Entity,
    new_position: usize,
) {
    commands
        .entity(buffer_entity)
        .insert(BufferCursor::new(new_position));

    debug!(
        "🔍 CURSOR UPDATE: Buffer {:?} cursor set to position {}",
        buffer_entity, new_position
    );
}

/// System to render visual markers for text buffer entities
pub fn render_buffer_markers(
    _commands: Commands,
    mut gizmos: Gizmos,
    buffer_query: Query<(Entity, &TextBuffer, &BufferVisualMarker)>,
) {
    for (_entity, text_buffer, marker) in buffer_query.iter() {
        let position = text_buffer.root_position;

        // Draw a small page icon to represent the text buffer
        let icon_size = 16.0;
        let color = match marker.layout_mode {
            SortLayoutMode::LTRText => Color::srgb(0.2, 0.6, 1.0), // Light blue for LTR
            SortLayoutMode::RTLText => Color::srgb(1.0, 0.6, 0.2), // Orange for RTL
            SortLayoutMode::Freeform => Color::srgb(0.6, 1.0, 0.2), // Green for Freeform
        };

        // Simple page icon: rectangle with a folded corner
        let half_size = icon_size / 2.0;
        let corner_size = 4.0;

        // Main page rectangle
        gizmos.rect_2d(
            position + Vec2::new(-2.0, 2.0), // Slightly offset so it doesn't overlap with sorts
            Vec2::new(icon_size * 0.8, icon_size),
            color,
        );

        // Folded corner (small triangle in top-right)
        let corner_pos =
            position + Vec2::new(half_size * 0.8 - corner_size, half_size + corner_size);
        gizmos.line_2d(corner_pos, corner_pos + Vec2::new(corner_size, 0.0), color);
        gizmos.line_2d(
            corner_pos + Vec2::new(corner_size, 0.0),
            corner_pos + Vec2::new(0.0, -corner_size),
            color,
        );
        gizmos.line_2d(corner_pos + Vec2::new(0.0, -corner_size), corner_pos, color);

        // Small text indicator based on layout mode
        let text_offset = Vec2::new(-half_size * 0.6, -half_size * 1.2);
        match marker.layout_mode {
            SortLayoutMode::LTRText => {
                // Arrow pointing right for LTR
                gizmos.line_2d(
                    position + text_offset,
                    position + text_offset + Vec2::new(8.0, 0.0),
                    color,
                );
                gizmos.line_2d(
                    position + text_offset + Vec2::new(8.0, 0.0),
                    position + text_offset + Vec2::new(4.0, -3.0),
                    color,
                );
                gizmos.line_2d(
                    position + text_offset + Vec2::new(8.0, 0.0),
                    position + text_offset + Vec2::new(4.0, 3.0),
                    color,
                );
            }
            SortLayoutMode::RTLText => {
                // Arrow pointing left for RTL
                gizmos.line_2d(
                    position + text_offset + Vec2::new(8.0, 0.0),
                    position + text_offset,
                    color,
                );
                gizmos.line_2d(
                    position + text_offset,
                    position + text_offset + Vec2::new(4.0, -3.0),
                    color,
                );
                gizmos.line_2d(
                    position + text_offset,
                    position + text_offset + Vec2::new(4.0, 3.0),
                    color,
                );
            }
            SortLayoutMode::Freeform => {
                // Simple dot for freeform
                gizmos.circle_2d(position + text_offset + Vec2::new(4.0, 0.0), 2.0, color);
            }
        }
    }
}
