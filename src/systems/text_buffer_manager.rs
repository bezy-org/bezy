//! Text buffer management system
//!
//! Manages text buffer entities and their relationship with sort entities.
//! Handles buffer creation, cursor management, and buffer-sort synchronization.

use bevy::prelude::*;
use crate::core::state::text_editor::{
    TextBuffer, BufferCursor, BufferMember, ActiveTextBuffer, BufferSystemSet,
    SortLayoutMode, TextEditorState,
};
use crate::core::state::text_editor::buffer::BufferId;
use crate::editing::sort::{Sort, ActiveSort};

/// Plugin for text buffer management
pub struct TextBufferManagerPlugin;

impl Plugin for TextBufferManagerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ActiveTextBuffer>()
            .configure_sets(Update, (
                BufferSystemSet::UpdateBuffers,
                BufferSystemSet::SyncMembership,
                BufferSystemSet::RenderBuffers,
            ).chain())
            .add_systems(Update, (
                sync_buffer_membership.in_set(BufferSystemSet::SyncMembership),
                update_active_buffer.in_set(BufferSystemSet::UpdateBuffers),
            ));
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
    let buffer_entity = commands.spawn((
        TextBuffer::new(id, layout_mode.clone(), root_position),
        BufferCursor::new(initial_cursor_position),
        Name::new(format!("TextBuffer-{:?}-{:?}", id.0, layout_mode)),
    )).id();
    
    info!(
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
    commands.entity(sort_entity).insert(BufferMember::new(buffer_entity, buffer_index));
    
    info!(
        "🔗 BUFFER MEMBERSHIP: Sort {:?} added to buffer {:?} at index {}",
        sort_entity, buffer_entity, buffer_index
    );
}

/// System to sync buffer membership and maintain consistency
pub fn sync_buffer_membership(
    _commands: Commands,
    text_editor_state: Option<Res<TextEditorState>>,
    buffer_query: Query<(Entity, &TextBuffer), With<TextBuffer>>,
    sort_query: Query<Entity, (With<Sort>, Without<BufferMember>)>,
    _buffer_member_query: Query<&mut BufferMember>,
) {
    let Some(text_state) = text_editor_state else { return };
    
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
    
    info!(
        "🔄 BUFFER SYNC: Found {} buffer sorts, {} buffer entities, {} unmapped sort entities",
        buffer_sorts.len(), buffer_entities.len(), sort_entities.len()
    );
}

/// System to update the active buffer based on active sorts
pub fn update_active_buffer(
    mut active_buffer: ResMut<ActiveTextBuffer>,
    active_sort_query: Query<&BufferMember, (With<ActiveSort>, Changed<ActiveSort>)>,
    _buffer_query: Query<&mut TextBuffer>,
) {
    // If an active sort changed, update the active buffer
    if let Ok(buffer_member) = active_sort_query.single() {
        let old_active = active_buffer.buffer_entity;
        active_buffer.buffer_entity = Some(buffer_member.buffer_entity);
        
        if old_active != active_buffer.buffer_entity {
            info!(
                "🎯 ACTIVE BUFFER CHANGED: {:?} -> {:?}",
                old_active, active_buffer.buffer_entity
            );
        }
    }
}

/// Helper function to get the cursor position for a buffer
pub fn get_buffer_cursor_position(
    buffer_entity: Entity,
    cursor_query: &Query<&BufferCursor>,
) -> Option<usize> {
    cursor_query.get(buffer_entity).ok().map(|cursor| cursor.position)
}

/// Helper function to set the cursor position for a buffer
pub fn set_buffer_cursor_position(
    commands: &mut Commands,
    buffer_entity: Entity,
    new_position: usize,
) {
    commands.entity(buffer_entity).insert(BufferCursor::new(new_position));
    
    info!(
        "🔍 CURSOR UPDATE: Buffer {:?} cursor set to position {}",
        buffer_entity, new_position
    );
}