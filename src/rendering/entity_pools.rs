//! Entity pooling system for text editor performance optimization
//!
//! This module provides entity pooling to eliminate the expensive despawn/spawn cycles
//! that currently happen every frame in the text editor. Instead of destroying and
//! recreating entities, we reuse existing entities by updating their components.

use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use bevy::time::common_conditions::on_timer;
use std::collections::HashMap;

/// Resource to manage entity pools for different types of rendering elements (internal)
#[derive(Resource, Default)]
pub(crate) struct EntityPools {
    /// Pool for outline entities (one pool per sort entity)
    pub outline_pools: HashMap<Entity, OutlineEntityPool>,
    /// Pool for metrics entities (one pool per sort entity)  
    pub metrics_pools: HashMap<Entity, MetricsEntityPool>,
    /// Pool for cursor entities (shared pool)
    pub cursor_pool: CursorEntityPool,
}

/// Pool for outline entities associated with a specific sort (internal)
#[derive(Default)]
pub(crate) struct OutlineEntityPool {
    /// Entities currently available for reuse
    pub available: Vec<Entity>,
    /// Entities currently in use (being rendered)
    pub in_use: Vec<Entity>,
}

/// Pool for metrics entities associated with a specific sort
#[derive(Default)]
pub(crate) struct MetricsEntityPool {
    /// Entities currently available for reuse
    pub available: Vec<Entity>,
    /// Entities currently in use (being rendered)
    pub in_use: Vec<Entity>,
}

/// Pool for cursor entities (shared across all cursors)
#[derive(Default)]
pub(crate) struct CursorEntityPool {
    /// Entities currently available for reuse
    pub available: Vec<Entity>,
    /// Entities currently in use (being rendered)
    pub in_use: Vec<Entity>,
}

/// Component to mark entities as pooled (to distinguish from regular entities)
#[derive(Component)]
pub(crate) struct PooledEntity {
    pub entity_type: PooledEntityType,
}

/// Types of pooled entities
#[derive(Debug, Clone, PartialEq)]
pub enum PooledEntityType {
    Outline,
    Metrics,
    Cursor,
}

impl EntityPools {
    /// Get or create an outline entity pool for a specific sort
    pub fn get_outline_pool(&mut self, sort_entity: Entity) -> &mut OutlineEntityPool {
        self.outline_pools.entry(sort_entity).or_default()
    }

    /// Get or create a metrics entity pool for a specific sort
    pub fn get_metrics_pool(&mut self, sort_entity: Entity) -> &mut MetricsEntityPool {
        self.metrics_pools.entry(sort_entity).or_default()
    }

    /// Get a cursor entity from the pool, or create a new one if none available
    pub fn get_cursor_entity(
        &mut self,
        commands: &mut Commands,
        entity_type: PooledEntityType,
    ) -> Entity {
        // Try to reuse an available entity
        if let Some(entity) = self.cursor_pool.available.pop() {
            // Only log at trace level to reduce debug noise in hot paths
            trace!("Reusing cursor entity: {:?}", entity);
            self.cursor_pool.in_use.push(entity);
            entity
        } else {
            // Create new entity if pool is empty
            let entity = commands
                .spawn((
                    PooledEntity { entity_type },
                    // Basic transform - will be updated when entity is used
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .id();

            debug!("Created new cursor entity: {:?}", entity);
            self.cursor_pool.in_use.push(entity);
            entity
        }
    }

    /// Get an outline entity from the pool for a specific sort
    pub fn get_outline_entity(&mut self, commands: &mut Commands, sort_entity: Entity) -> Entity {
        let pool = self.get_outline_pool(sort_entity);

        // Try to reuse an available entity
        if let Some(entity) = pool.available.pop() {
            // Only log at trace level to reduce debug noise in hot paths
            trace!(
                "Reusing outline entity for sort {:?}: {:?}",
                sort_entity,
                entity
            );
            pool.in_use.push(entity);
            entity
        } else {
            // Create new entity if pool is empty
            let entity = commands
                .spawn((
                    PooledEntity {
                        entity_type: PooledEntityType::Outline,
                    },
                    // Basic transform - will be updated when entity is used
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .id();

            debug!(
                "Created new outline entity for sort {:?}: {:?}",
                sort_entity, entity
            );
            pool.in_use.push(entity);
            entity
        }
    }

    /// Get a metrics entity from the pool for a specific sort
    pub fn get_metrics_entity(&mut self, commands: &mut Commands, sort_entity: Entity) -> Entity {
        let pool = self.get_metrics_pool(sort_entity);

        // Try to reuse an available entity
        if let Some(entity) = pool.available.pop() {
            // Only log at trace level to reduce debug noise in hot paths
            trace!(
                "Reusing metrics entity for sort {:?}: {:?}",
                sort_entity,
                entity
            );
            pool.in_use.push(entity);
            entity
        } else {
            // Create new entity if pool is empty
            let entity = commands
                .spawn((
                    PooledEntity {
                        entity_type: PooledEntityType::Metrics,
                    },
                    // Basic transform - will be updated when entity is used
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .id();

            debug!(
                "Created new metrics entity for sort {:?}: {:?}",
                sort_entity, entity
            );
            pool.in_use.push(entity);
            entity
        }
    }

    /// Return cursor entities to the available pool (called at start of frame)
    pub fn return_cursor_entities(&mut self, commands: &mut Commands) {
        debug!(
            "Returning {} cursor entities to pool",
            self.cursor_pool.in_use.len()
        );

        // Hide all cursor entities when returning them to pool
        for entity in &self.cursor_pool.in_use {
            if let Ok(mut entity_commands) = commands.get_entity(*entity) {
                entity_commands.insert(Visibility::Hidden);
            }
        }

        self.cursor_pool
            .available
            .append(&mut self.cursor_pool.in_use);
    }

    /// Return outline entities for a specific sort to the available pool
    pub fn return_outline_entities(&mut self, commands: &mut Commands, sort_entity: Entity) {
        if let Some(pool) = self.outline_pools.get_mut(&sort_entity) {
            // Only log if returning significant number of entities to avoid debug noise
            if pool.in_use.len() > 5 {
                debug!(
                    "Returning {} outline entities to pool for sort {:?}",
                    pool.in_use.len(),
                    sort_entity
                );
            }
            // CRITICAL: Hide entities when returning them to pool to prevent double rendering
            for &entity in &pool.in_use {
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.insert(Visibility::Hidden);
                }
            }
            pool.available.append(&mut pool.in_use);
        }
    }

    /// Return metrics entities for a specific sort to the available pool
    pub fn return_metrics_entities(&mut self, commands: &mut Commands, sort_entity: Entity) {
        if let Some(pool) = self.metrics_pools.get_mut(&sort_entity) {
            // Only log if returning significant number of entities to avoid debug noise
            if pool.in_use.len() > 10 {
                debug!(
                    "Returning {} metrics entities to pool for sort {:?}",
                    pool.in_use.len(),
                    sort_entity
                );
            }
            // CRITICAL: Hide entities when returning them to pool to prevent double rendering
            for &entity in &pool.in_use {
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.insert(Visibility::Hidden);
                }
            }
            pool.available.append(&mut pool.in_use);
        }
    }

    /// Return all entities to pools (useful for cleanup)
    /// NOTE: This is expensive - prefer selective returns when possible
    pub fn return_all_entities(&mut self, commands: &mut Commands) {
        debug!("Returning all entities to pools");

        // Return cursor entities
        self.return_cursor_entities(commands);

        // Return outline entities for all sorts
        let sort_entities: Vec<Entity> = self.outline_pools.keys().copied().collect();
        for sort_entity in sort_entities {
            self.return_outline_entities(commands, sort_entity);
        }

        // Return metrics entities for all sorts
        let sort_entities: Vec<Entity> = self.metrics_pools.keys().copied().collect();
        for sort_entity in sort_entities {
            self.return_metrics_entities(commands, sort_entity);
        }
    }

    /// Return entities for specific sorts that have changed (more efficient)
    pub fn return_entities_for_changed_sorts(
        &mut self,
        commands: &mut Commands,
        changed_sort_entities: &[Entity],
    ) {
        if changed_sort_entities.is_empty() {
            return;
        }

        debug!(
            "Returning entities for {} changed sorts",
            changed_sort_entities.len()
        );

        // Return outline and metrics entities only for changed sorts
        for &sort_entity in changed_sort_entities {
            self.return_outline_entities(commands, sort_entity);
            self.return_metrics_entities(commands, sort_entity);
        }
    }

    /// Clean up pools for deleted sorts (remove empty pools)
    pub fn cleanup_empty_pools(&mut self) {
        // Remove outline pools that have no entities
        self.outline_pools
            .retain(|_, pool| !pool.available.is_empty() || !pool.in_use.is_empty());

        // Remove metrics pools that have no entities
        self.metrics_pools
            .retain(|_, pool| !pool.available.is_empty() || !pool.in_use.is_empty());
    }

    /// Get statistics about pool usage for monitoring
    pub fn get_pool_stats(&self) -> PoolStats {
        let mut outline_available = 0;
        let mut outline_in_use = 0;
        for pool in self.outline_pools.values() {
            outline_available += pool.available.len();
            outline_in_use += pool.in_use.len();
        }

        let mut metrics_available = 0;
        let mut metrics_in_use = 0;
        for pool in self.metrics_pools.values() {
            metrics_available += pool.available.len();
            metrics_in_use += pool.in_use.len();
        }

        PoolStats {
            outline_available,
            outline_in_use,
            metrics_available,
            metrics_in_use,
            cursor_available: self.cursor_pool.available.len(),
            cursor_in_use: self.cursor_pool.in_use.len(),
            outline_pools_count: self.outline_pools.len(),
            metrics_pools_count: self.metrics_pools.len(),
        }
    }
}

/// Statistics about entity pool usage
#[derive(Debug)]
pub(crate) struct PoolStats {
    pub outline_available: usize,
    pub outline_in_use: usize,
    pub metrics_available: usize,
    pub metrics_in_use: usize,
    pub cursor_available: usize,
    pub cursor_in_use: usize,
    pub outline_pools_count: usize,
    pub metrics_pools_count: usize,
}

/// Helper functions for updating pooled entities
/// Update an outline entity with new mesh and material
pub fn update_outline_entity(
    commands: &mut Commands,
    entity: Entity,
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    transform: Transform,
    outline_component: impl Component,
) {
    // Check if entity exists before trying to update it
    if let Ok(mut entity_commands) = commands.get_entity(entity) {
        entity_commands.insert((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            transform,
            outline_component,
            Visibility::Visible,
        ));
    } else {
        debug!(
            "Skipping update for non-existent outline entity {:?}",
            entity
        );
    }
}

/// Update a metrics entity with new mesh and material  
pub fn update_metrics_entity(
    commands: &mut Commands,
    entity: Entity,
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    transform: Transform,
    metrics_component: impl Component,
) {
    // Check if entity exists before trying to update it
    if let Ok(mut entity_commands) = commands.get_entity(entity) {
        entity_commands.insert((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            transform,
            metrics_component,
            Visibility::Visible,
        ));
    } else {
        debug!(
            "Skipping update for non-existent metrics entity {:?}",
            entity
        );
    }
}

/// Update a cursor entity with new mesh and material
pub fn update_cursor_entity(
    commands: &mut Commands,
    entity: Entity,
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    transform: Transform,
    cursor_component: impl Component,
) {
    // Check if entity exists before trying to update it
    if let Ok(mut entity_commands) = commands.get_entity(entity) {
        entity_commands.insert((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            transform,
            cursor_component,
            Visibility::Visible,
        ));
    } else {
        debug!(
            "Skipping update for non-existent cursor entity {:?}",
            entity
        );
    }
}

/// Plugin for entity pooling system
pub struct EntityPoolingPlugin;

impl Plugin for EntityPoolingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityPools>().add_systems(
            Update,
            (
                log_pool_stats.run_if(on_timer(std::time::Duration::from_secs(5))),
                cleanup_pools.run_if(on_timer(std::time::Duration::from_secs(10))),
            ),
        );
    }
}

/// System to log pool statistics periodically
fn log_pool_stats(pools: Res<EntityPools>) {
    let stats = pools.get_pool_stats();
    debug!("Entity Pool Stats: Outline(avail:{}/use:{}), Metrics(avail:{}/use:{}), Cursor(avail:{}/use:{}), Pools(outline:{}, metrics:{})", 
        stats.outline_available, stats.outline_in_use,
        stats.metrics_available, stats.metrics_in_use,
        stats.cursor_available, stats.cursor_in_use,
        stats.outline_pools_count, stats.metrics_pools_count);
}

/// System to clean up empty pools periodically
fn cleanup_pools(mut pools: ResMut<EntityPools>) {
    pools.cleanup_empty_pools();
}
