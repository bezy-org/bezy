//! Shared metrics rendering functionality
//!
//! This module contains shared functions for rendering font metrics that can be used
//! by both the main metrics system and individual sorts.

#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use crate::core::state::font_metrics::FontMetrics;
use crate::core::state::fontir_app_state::FontIRMetrics;
use crate::rendering::entity_pools::{update_metrics_entity, EntityPools};
use crate::rendering::zoom_aware_scaling::CameraResponsiveScale;
use crate::ui::themes::CurrentTheme;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use kurbo::ParamCurve;

/// Component to mark entities as metrics line visual elements
#[derive(Component)]
pub struct MetricsLine {
    pub line_type: MetricsLineType,
}

/// Component that establishes a relationship between metrics and their sort
/// This is the idiomatic Bevy pattern for entity relationships
#[derive(Component)]
pub struct MetricsFor(pub Entity);

/// Types of metrics line elements
#[derive(Debug, Clone)]
pub enum MetricsLineType {
    Baseline,
    XHeight,
    CapHeight,
    Ascender,
    Descender,
    AdvanceWidth,
    BoundingBox,
}

/// Resource to track metrics line entities
#[derive(Resource, Default)]
pub struct MetricsLineEntities {
    pub lines: std::collections::HashMap<Entity, Vec<Entity>>, // sort_entity -> line entities
}

/// Cache for expensive glyph metrics calculations
#[derive(Resource, Default)]
pub struct GlyphMetricsCache {
    /// Cache glyph advance widths to avoid repeated FontIR lookups
    pub advance_widths: std::collections::HashMap<String, f32>,
    /// Cache font metrics to avoid repeated extraction
    pub font_metrics: Option<FontIRMetrics>,
    /// Track when font changed to invalidate cache
    pub last_font_generation: u64,
}

impl GlyphMetricsCache {
    /// Get cached advance width or calculate and cache it
    pub fn get_advance_width(
        &mut self,
        glyph_name: &str,
        fontir_state: &crate::core::state::FontIRAppState,
    ) -> f32 {
        if let Some(&width) = self.advance_widths.get(glyph_name) {
            return width;
        }

        let width = fontir_state.get_glyph_advance_width(glyph_name);
        self.advance_widths.insert(glyph_name.to_string(), width);
        debug!("Cached advance width for glyph '{}': {}", glyph_name, width);
        width
    }

    /// Get cached font metrics or extract and cache them
    pub fn get_font_metrics(
        &mut self,
        fontir_state: &crate::core::state::FontIRAppState,
    ) -> &FontIRMetrics {
        // For now, skip generation tracking since FontIRAppState doesn't expose it
        // TODO: Add generation tracking when FontIRAppState API supports it

        if self.font_metrics.is_none() {
            self.font_metrics = Some(fontir_state.get_font_metrics());
            debug!(
                "Cached font metrics: UPM={}",
                self.font_metrics.as_ref().unwrap().units_per_em
            );
        }

        self.font_metrics.as_ref().unwrap()
    }

    /// Clear all cached data (useful for debugging)
    pub fn clear(&mut self) {
        self.advance_widths.clear();
        self.font_metrics = None;
        self.last_font_generation = 0;
        debug!("Cleared all glyph metrics cache");
    }
}

/// Component to mark entities as preview metrics (temporary entities)
#[derive(Component)]
pub struct PreviewMetricsLine;

/// Component to mark entities as preview glyph outline elements
#[derive(Component)]
pub struct PreviewGlyphOutline;

/// Resource to track preview metrics entities that need to be cleaned up
#[derive(Resource, Default)]
pub struct PreviewMetricsEntities {
    pub entities: Vec<Entity>,
}

/// Resource to store current preview state for mesh-based preview system
#[derive(Resource, Default)]
pub struct PreviewMetricsState {
    pub active: bool,
    pub position: Vec2,
    pub glyph_name: String,
    pub advance_width: f32,
    pub color: Color,
}

/// Z-levels for metrics lines
const METRICS_LINE_Z: f32 = 5.0; // Behind glyph editing elements

/// Helper to spawn a mesh-based metrics line
fn spawn_metrics_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    color: Color,
    sort_entity: Entity,
    line_type: MetricsLineType,
    camera_scale: &CameraResponsiveScale,
) -> Entity {
    let line_width = camera_scale.adjusted_line_width(); // Camera-responsive width
    let line_mesh = crate::rendering::mesh_utils::create_line_mesh(start, end, line_width);

    commands
        .spawn((
            MetricsLine { line_type },
            MetricsFor(sort_entity), // Component relationship pattern
            Mesh2d(meshes.add(line_mesh)),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
            Transform::from_xyz(
                (start.x + end.x) * 0.5,
                (start.y + end.y) * 0.5,
                METRICS_LINE_Z,
            ),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id()
}

/// ENTITY POOLING: Get or update a metrics line entity from the pool
#[allow(dead_code)]
fn get_or_update_metrics_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    entity_pools: &mut ResMut<EntityPools>,
    start: Vec2,
    end: Vec2,
    color: Color,
    sort_entity: Entity,
    line_type: MetricsLineType,
    camera_scale: &CameraResponsiveScale,
) -> Entity {
    // Get an entity from the pool
    let entity = entity_pools.get_metrics_entity(commands, sort_entity);

    // Create the mesh and material
    let line_width = camera_scale.adjusted_line_width();
    let line_mesh = crate::rendering::mesh_utils::create_line_mesh(start, end, line_width);
    let mesh_handle = meshes.add(line_mesh);
    let material_handle = materials.add(ColorMaterial::from_color(color));

    // Calculate transform
    let transform = Transform::from_xyz(
        (start.x + end.x) * 0.5,
        (start.y + end.y) * 0.5,
        METRICS_LINE_Z,
    );

    // Update the entity using the helper function
    let metrics_component = MetricsLine { line_type };

    update_metrics_entity(
        commands,
        entity,
        mesh_handle,
        material_handle,
        transform,
        metrics_component,
    );

    // Add the MetricsFor relationship component
    if let Ok(mut entity_commands) = commands.get_entity(entity) {
        entity_commands.insert(MetricsFor(sort_entity));
    }

    debug!(
        "Updated pooled metrics entity {:?} for sort {:?}",
        entity, sort_entity
    );
    entity
}

/// System to render mesh-based metrics lines for all active sorts
pub fn render_mesh_metrics_lines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut metrics_entities: ResMut<MetricsLineEntities>,
    _entity_pools: ResMut<EntityPools>,
    mut metrics_cache: ResMut<GlyphMetricsCache>,
    // NO CHANGE DETECTION: Active sorts must always render metrics for visibility
    // EXCLUDE buffer sorts to prevent overlap with active_buffer_sort_query
    sort_query: Query<
        (Entity, &Transform, &crate::editing::sort::Sort),
        (
            With<crate::editing::sort::ActiveSort>,
            Without<crate::systems::sorts::sort_entities::BufferSortIndex>,
        ),
    >,
    // Add query for ACTIVE text buffer sorts (text roots)
    // NO CHANGE DETECTION: Active buffer sorts must always render metrics for visibility
    active_buffer_sort_query: Query<
        (Entity, &Transform, &crate::editing::sort::Sort),
        (
            With<crate::systems::sorts::sort_entities::BufferSortIndex>,
            With<crate::editing::sort::ActiveSort>,
        ),
    >,
    // Add query for INACTIVE text buffer sorts (typed characters)
    // NO CHANGE DETECTION: Inactive buffer sorts must always render metrics for visibility
    inactive_buffer_sort_query: Query<
        (Entity, &Transform, &crate::editing::sort::Sort),
        (
            With<crate::systems::sorts::sort_entities::BufferSortIndex>,
            With<crate::editing::sort::InactiveSort>,
        ),
    >,
    _existing_metrics: Query<Entity, With<MetricsLine>>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    camera_scale: Res<CameraResponsiveScale>,
    presentation_mode: Option<Res<crate::ui::edit_mode_toolbar::PresentationMode>>,
    theme: Res<crate::ui::themes::CurrentTheme>,
) {
    // Check presentation mode state
    let presentation_active = presentation_mode.as_ref().is_some_and(|pm| pm.active);
    let presentation_changed = presentation_mode.as_ref().is_some_and(|pm| pm.is_changed());

    // Hide metrics in presentation mode OR if presentation mode just activated
    if presentation_active || presentation_changed {
        if presentation_active {
            debug!("🎭 Metrics hidden for presentation mode - clearing all metrics entities");
            // Despawn ALL metrics entities from the tracking resource
            for (sort_entity, line_entities) in metrics_entities.lines.drain() {
                debug!(
                    "🎭 Clearing {} metrics entities for sort {:?}",
                    line_entities.len(),
                    sort_entity
                );
                for line_entity in line_entities {
                    if let Ok(mut entity_commands) = commands.get_entity(line_entity) {
                        entity_commands.despawn();
                    }
                }
            }
            // Also despawn any orphaned metrics not in the tracking
            for entity in _existing_metrics.iter() {
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.despawn();
                }
            }
            metrics_entities.lines.clear();
        }
        // Skip rendering if in presentation mode
        if presentation_active {
            return;
        }
    }

    // CHANGE DETECTION: Early return if no sorts have changed and presentation mode hasn't changed
    let sort_count = sort_query.iter().count();
    let active_buffer_count = active_buffer_sort_query.iter().count();
    let inactive_buffer_count = inactive_buffer_sort_query.iter().count();

    if sort_count == 0
        && active_buffer_count == 0
        && inactive_buffer_count == 0
        && !presentation_changed
    {
        debug!("Metrics rendering skipped - no changed sorts");
        return;
    }

    // SELECTIVE PROCESSING: Collect only the sorts that actually changed
    use std::collections::HashSet;
    let mut changed_sort_entities = HashSet::new();
    let mut changed_active_sorts = Vec::new();
    let mut changed_active_buffer_sorts = Vec::new();
    let mut changed_inactive_buffer_sorts = Vec::new();

    // Collect changed sorts by type for selective processing
    for (sort_entity, sort_transform, sort) in sort_query.iter() {
        if changed_sort_entities.insert(sort_entity) {
            changed_active_sorts.push((sort_entity, sort_transform, sort));
            debug!(
                "METRICS DEBUG: Sort {:?} collected as ACTIVE (green metrics)",
                sort_entity
            );
        } else {
            warn!(
                "METRICS DEBUG: Sort {:?} DUPLICATE in active query - this causes z-fighting!",
                sort_entity
            );
        }
    }
    for (sort_entity, sort_transform, sort) in active_buffer_sort_query.iter() {
        if changed_sort_entities.insert(sort_entity) {
            changed_active_buffer_sorts.push((sort_entity, sort_transform, sort));
            debug!(
                "METRICS DEBUG: Sort {:?} collected as ACTIVE BUFFER (green metrics)",
                sort_entity
            );
        } else {
            warn!("METRICS DEBUG: Sort {:?} DUPLICATE in active buffer query - this causes z-fighting!", sort_entity);
        }
    }
    for (sort_entity, sort_transform, sort) in inactive_buffer_sort_query.iter() {
        if changed_sort_entities.insert(sort_entity) {
            changed_inactive_buffer_sorts.push((sort_entity, sort_transform, sort));
            debug!(
                "METRICS DEBUG: Sort {:?} collected as INACTIVE BUFFER (gray metrics)",
                sort_entity
            );
        } else {
            warn!("METRICS DEBUG: Sort {:?} DUPLICATE in inactive buffer query - this causes z-fighting!", sort_entity);
        }
    }

    // COMPREHENSIVE CLEAR: Despawn AND remove metrics entities for all sorts being processed to prevent z-fighting
    // Since we removed change detection, we need to clear all metrics to avoid conflicts
    let changed_sort_entities: Vec<Entity> = changed_sort_entities.into_iter().collect();
    for &sort_entity in &changed_sort_entities {
        if let Some(line_entities) = metrics_entities.lines.remove(&sort_entity) {
            debug!(
                "METRICS DEBUG: Clearing {} metrics entities for sort {:?}",
                line_entities.len(),
                sort_entity
            );
            for line_entity in line_entities {
                if let Ok(mut entity_commands) = commands.get_entity(line_entity) {
                    entity_commands.despawn();
                } else {
                    debug!(
                        "Skipping despawn for non-existent metrics line entity {:?}",
                        line_entity
                    );
                }
            }
        }
    }

    // ADDITIONAL CLEAR: Also clear any metrics that might be stale from state transitions
    let all_active_entities: Vec<Entity> = sort_query.iter().map(|(e, _, _)| e).collect();
    let all_active_buffer_entities: Vec<Entity> =
        active_buffer_sort_query.iter().map(|(e, _, _)| e).collect();
    let all_inactive_buffer_entities: Vec<Entity> = inactive_buffer_sort_query
        .iter()
        .map(|(e, _, _)| e)
        .collect();

    for entity in all_active_entities
        .iter()
        .chain(all_active_buffer_entities.iter())
        .chain(all_inactive_buffer_entities.iter())
    {
        if let Some(line_entities) = metrics_entities.lines.remove(entity) {
            for line_entity in line_entities {
                if let Ok(mut entity_commands) = commands.get_entity(line_entity) {
                    entity_commands.despawn();
                } else {
                    debug!(
                        "Skipping despawn for non-existent metrics line entity {:?}",
                        line_entity
                    );
                }
            }
        }
    }

    debug!("Selective metrics rendering: {} active, {} active_buffer, {} inactive_buffer (total changed: {})", 
           changed_active_sorts.len(), changed_active_buffer_sorts.len(),
           changed_inactive_buffer_sorts.len(), changed_sort_entities.len());

    // DEBUG: Log details about the different sort types being processed
    if !changed_active_sorts.is_empty() {
        debug!(
            "🔵 METRICS RENDER: Processing {} active sorts (green metrics)",
            changed_active_sorts.len()
        );
    }
    if !changed_active_buffer_sorts.is_empty() {
        debug!(
            "🟢 METRICS RENDER: Processing {} active buffer sorts (green metrics)",
            changed_active_buffer_sorts.len()
        );
    }
    if !changed_inactive_buffer_sorts.is_empty() {
        debug!(
            "🔘 METRICS RENDER: Processing {} inactive buffer sorts (gray metrics)",
            changed_inactive_buffer_sorts.len()
        );
    }

    if let Some(fontir_state) = fontir_app_state {
        // Cache font metrics once for the entire frame
        let fontir_metrics = {
            let cached_metrics = metrics_cache.get_font_metrics(&fontir_state);
            cached_metrics.clone() // Clone the metrics to avoid borrow conflicts
        };

        // SELECTIVE RENDERING: Only process changed active sorts
        for (sort_entity, sort_transform, sort) in changed_active_sorts {
            let position = sort_transform.translation.truncate();
            // Use cached advance width lookup instead of expensive FontIR call
            let advance_width = metrics_cache.get_advance_width(&sort.glyph_name, &fontir_state);
            // Since query filters for ActiveSort, all sorts here are active - use active color
            // Theme is now available as a parameter
            let color = theme.theme().sort_active_metrics_color();

            let mut line_entities = Vec::new();

            // Extract metrics values
            let upm = fontir_metrics.units_per_em;
            let ascender = fontir_metrics.ascender.unwrap_or(upm * 0.8);
            let descender = fontir_metrics.descender.unwrap_or(upm * -0.2);
            let x_height = fontir_metrics.x_height.unwrap_or(upm * 0.5);
            let cap_height = fontir_metrics.cap_height.unwrap_or(upm * 0.7);

            // Baseline (most important)
            let baseline_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                position,
                Vec2::new(position.x + advance_width, position.y),
                color,
                sort_entity,
                MetricsLineType::Baseline,
                &camera_scale,
            );
            line_entities.push(baseline_entity);

            // x-height
            let x_height_y = position.y + x_height;
            let x_height_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, x_height_y),
                Vec2::new(position.x + advance_width, x_height_y),
                color,
                sort_entity,
                MetricsLineType::XHeight,
                &camera_scale,
            );
            line_entities.push(x_height_entity);

            // cap-height
            let cap_height_y = position.y + cap_height;
            let cap_height_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, cap_height_y),
                Vec2::new(position.x + advance_width, cap_height_y),
                color,
                sort_entity,
                MetricsLineType::CapHeight,
                &camera_scale,
            );
            line_entities.push(cap_height_entity);

            // ascender
            let ascender_y = position.y + ascender;
            let ascender_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, ascender_y),
                Vec2::new(position.x + advance_width, ascender_y),
                color,
                sort_entity,
                MetricsLineType::Ascender,
                &camera_scale,
            );
            line_entities.push(ascender_entity);

            // descender
            let descender_y = position.y + descender;
            let descender_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, descender_y),
                Vec2::new(position.x + advance_width, descender_y),
                color,
                sort_entity,
                MetricsLineType::Descender,
                &camera_scale,
            );
            line_entities.push(descender_entity);

            // Advance width line (vertical)
            let advance_width_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x + advance_width, descender_y),
                Vec2::new(position.x + advance_width, ascender_y),
                color,
                sort_entity,
                MetricsLineType::AdvanceWidth,
                &camera_scale,
            );
            line_entities.push(advance_width_entity);

            // Draw bounding box lines (4 lines for rectangle)
            let top_left = Vec2::new(position.x, position.y + upm);
            let bottom_right = Vec2::new(position.x + advance_width, descender_y);

            // Top line
            let top_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                top_left,
                Vec2::new(bottom_right.x, top_left.y),
                color,
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(top_entity);

            // Right line
            let right_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(bottom_right.x, top_left.y),
                bottom_right,
                color,
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(right_entity);

            // Bottom line
            let bottom_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                bottom_right,
                Vec2::new(top_left.x, bottom_right.y),
                color,
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(bottom_entity);

            // Left line
            let left_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(top_left.x, bottom_right.y),
                top_left,
                color,
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(left_entity);

            metrics_entities.lines.insert(sort_entity, line_entities);
        }

        // SELECTIVE RENDERING: Only process changed active buffer sorts
        for (sort_entity, sort_transform, sort) in changed_active_buffer_sorts {
            let position = sort_transform.translation.truncate();
            // Use cached advance width lookup for active buffer sorts
            let advance_width = metrics_cache.get_advance_width(&sort.glyph_name, &fontir_state);
            // Theme is now available as a parameter
            let color = theme.theme().sort_active_metrics_color(); // Green for active buffer sorts (text roots)

            debug!(
                "🟢 RENDERING METRICS for active buffer sort {:?} at ({:.1}, {:.1})",
                sort_entity, position.x, position.y
            );

            let mut line_entities = Vec::new();

            // Extract metrics values
            let upm = fontir_metrics.units_per_em;
            let ascender = fontir_metrics.ascender.unwrap_or(upm * 0.8);
            let descender = fontir_metrics.descender.unwrap_or(upm * -0.2);
            let x_height = fontir_metrics.x_height.unwrap_or(upm * 0.5);
            let cap_height = fontir_metrics.cap_height.unwrap_or(upm * 0.7);

            // Baseline (most important)
            let baseline_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                position,
                Vec2::new(position.x + advance_width, position.y),
                color,
                sort_entity,
                MetricsLineType::Baseline,
                &camera_scale,
            );
            line_entities.push(baseline_entity);

            // x-height
            let x_height_y = position.y + x_height;
            let x_height_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, x_height_y),
                Vec2::new(position.x + advance_width, x_height_y),
                color,
                sort_entity,
                MetricsLineType::XHeight,
                &camera_scale,
            );
            line_entities.push(x_height_entity);

            // cap-height
            let cap_height_y = position.y + cap_height;
            let cap_height_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, cap_height_y),
                Vec2::new(position.x + advance_width, cap_height_y),
                color,
                sort_entity,
                MetricsLineType::CapHeight,
                &camera_scale,
            );
            line_entities.push(cap_height_entity);

            // ascender
            let ascender_y = position.y + ascender;
            let ascender_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, ascender_y),
                Vec2::new(position.x + advance_width, ascender_y),
                color,
                sort_entity,
                MetricsLineType::Ascender,
                &camera_scale,
            );
            line_entities.push(ascender_entity);

            // descender
            let descender_y = position.y + descender;
            let descender_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, descender_y),
                Vec2::new(position.x + advance_width, descender_y),
                color,
                sort_entity,
                MetricsLineType::Descender,
                &camera_scale,
            );
            line_entities.push(descender_entity);

            // Advance width line (vertical)
            let advance_width_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x + advance_width, descender_y),
                Vec2::new(position.x + advance_width, ascender_y),
                color,
                sort_entity,
                MetricsLineType::AdvanceWidth,
                &camera_scale,
            );
            line_entities.push(advance_width_entity);

            // Draw bounding box lines (4 lines for rectangle) with more transparency
            let top_left = Vec2::new(position.x, position.y + upm);
            let bottom_right = Vec2::new(position.x + advance_width, descender_y);

            // Top line
            let top_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                top_left,
                Vec2::new(bottom_right.x, top_left.y),
                color,
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(top_entity);

            // Right line
            let right_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(bottom_right.x, top_left.y),
                bottom_right,
                color,
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(right_entity);

            // Bottom line
            let bottom_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                bottom_right,
                Vec2::new(top_left.x, bottom_right.y),
                color,
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(bottom_entity);

            // Left line
            let left_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(top_left.x, bottom_right.y),
                top_left,
                color,
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(left_entity);

            debug!(
                "🟢 METRICS STORED: {} metrics entities for active buffer sort {:?}",
                line_entities.len(),
                sort_entity
            );
            metrics_entities.lines.insert(sort_entity, line_entities);
        }

        // SELECTIVE RENDERING: Only process changed inactive buffer sorts (HUGE PERFORMANCE WIN)
        for (sort_entity, sort_transform, sort) in changed_inactive_buffer_sorts {
            let position = sort_transform.translation.truncate();
            // Use cached advance width lookup for inactive buffer sorts
            let advance_width = metrics_cache.get_advance_width(&sort.glyph_name, &fontir_state);
            // Theme is now available as a parameter
            let color = theme.theme().sort_inactive_metrics_color(); // Gray for inactive buffer sorts (typed characters)

            debug!(
                "🔘 RENDERING METRICS for inactive buffer sort {:?} at ({:.1}, {:.1})",
                sort_entity, position.x, position.y
            );

            let mut line_entities = Vec::new();

            // Extract metrics values
            let upm = fontir_metrics.units_per_em;
            let ascender = fontir_metrics.ascender.unwrap_or(upm * 0.8);
            let descender = fontir_metrics.descender.unwrap_or(upm * -0.2);
            let x_height = fontir_metrics.x_height.unwrap_or(upm * 0.5);
            let cap_height = fontir_metrics.cap_height.unwrap_or(upm * 0.7);

            // Baseline (most important)
            let baseline_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                position,
                Vec2::new(position.x + advance_width, position.y),
                color,
                sort_entity,
                MetricsLineType::Baseline,
                &camera_scale,
            );
            line_entities.push(baseline_entity);

            // x-height
            let x_height_y = position.y + x_height;
            let x_height_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, x_height_y),
                Vec2::new(position.x + advance_width, x_height_y),
                color,
                sort_entity,
                MetricsLineType::XHeight,
                &camera_scale,
            );
            line_entities.push(x_height_entity);

            // cap-height
            let cap_height_y = position.y + cap_height;
            let cap_height_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, cap_height_y),
                Vec2::new(position.x + advance_width, cap_height_y),
                color,
                sort_entity,
                MetricsLineType::CapHeight,
                &camera_scale,
            );
            line_entities.push(cap_height_entity);

            // ascender
            let ascender_y = position.y + ascender;
            let ascender_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, ascender_y),
                Vec2::new(position.x + advance_width, ascender_y),
                color,
                sort_entity,
                MetricsLineType::Ascender,
                &camera_scale,
            );
            line_entities.push(ascender_entity);

            // descender
            let descender_y = position.y + descender;
            let descender_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x, descender_y),
                Vec2::new(position.x + advance_width, descender_y),
                color,
                sort_entity,
                MetricsLineType::Descender,
                &camera_scale,
            );
            line_entities.push(descender_entity);

            // Advance width line (vertical)
            let advance_width_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(position.x + advance_width, descender_y),
                Vec2::new(position.x + advance_width, ascender_y),
                color,
                sort_entity,
                MetricsLineType::AdvanceWidth,
                &camera_scale,
            );
            line_entities.push(advance_width_entity);

            // Draw bounding box lines (4 lines for rectangle) with more transparency
            let top_left = Vec2::new(position.x, position.y + upm);
            let bottom_right = Vec2::new(position.x + advance_width, descender_y);

            // Top line
            let top_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                top_left,
                Vec2::new(bottom_right.x, top_left.y),
                color,
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(top_entity);

            // Right line
            let right_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(bottom_right.x, top_left.y),
                bottom_right,
                color,
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(right_entity);

            // Bottom line
            let bottom_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                bottom_right,
                Vec2::new(top_left.x, bottom_right.y),
                color,
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(bottom_entity);

            // Left line
            let left_entity = spawn_metrics_line(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(top_left.x, bottom_right.y),
                top_left,
                color,
                sort_entity,
                MetricsLineType::BoundingBox,
                &camera_scale,
            );
            line_entities.push(left_entity);

            debug!(
                "🔘 METRICS STORED: {} metrics entities for inactive buffer sort {:?}",
                line_entities.len(),
                sort_entity
            );
            metrics_entities.lines.insert(sort_entity, line_entities);
        }
    }
}

/// System to manage mesh-based preview metrics
pub fn manage_preview_metrics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut preview_entities: ResMut<PreviewMetricsEntities>,
    preview_state: Res<PreviewMetricsState>,
    fontir_app_state: Option<Res<crate::core::state::FontIRAppState>>,
    camera_scale: Res<CameraResponsiveScale>,
) {
    // Clean up existing preview entities
    for entity in preview_entities.entities.drain(..) {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        } else {
            debug!(
                "Skipping despawn for non-existent preview entity {:?}",
                entity
            );
        }
    }

    // Only create new preview if active and we have fontir state
    if !preview_state.active || preview_state.glyph_name.is_empty() {
        return;
    }

    let Some(fontir_state) = fontir_app_state.as_ref() else {
        return;
    };

    let font_metrics = fontir_state.get_font_metrics();
    let upm = font_metrics.units_per_em;
    let ascender = font_metrics.ascender.unwrap_or(upm * 0.8);
    let descender = font_metrics.descender.unwrap_or(upm * -0.2);
    let x_height = font_metrics.x_height.unwrap_or(upm * 0.5);
    let cap_height = font_metrics.cap_height.unwrap_or(upm * 0.7);

    let temp_entity = Entity::from_raw(0); // Placeholder for preview
    let position = preview_state.position;
    let advance_width = preview_state.advance_width;
    let color = preview_state.color;

    // Baseline (most important)
    let baseline_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        position,
        Vec2::new(position.x + advance_width, position.y),
        color,
        temp_entity,
        MetricsLineType::Baseline,
        &camera_scale,
    );
    preview_entities.entities.push(baseline_entity);

    // x-height
    let x_height_y = position.y + x_height;
    let x_height_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(position.x, x_height_y),
        Vec2::new(position.x + advance_width, x_height_y),
        color,
        temp_entity,
        MetricsLineType::XHeight,
        &camera_scale,
    );
    preview_entities.entities.push(x_height_entity);

    // cap-height
    let cap_height_y = position.y + cap_height;
    let cap_height_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(position.x, cap_height_y),
        Vec2::new(position.x + advance_width, cap_height_y),
        color,
        temp_entity,
        MetricsLineType::CapHeight,
        &camera_scale,
    );
    preview_entities.entities.push(cap_height_entity);

    // ascender
    let ascender_y = position.y + ascender;
    let ascender_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(position.x, ascender_y),
        Vec2::new(position.x + advance_width, ascender_y),
        color,
        temp_entity,
        MetricsLineType::Ascender,
        &camera_scale,
    );
    preview_entities.entities.push(ascender_entity);

    // descender
    let descender_y = position.y + descender;
    let descender_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(position.x, descender_y),
        Vec2::new(position.x + advance_width, descender_y),
        color,
        temp_entity,
        MetricsLineType::Descender,
        &camera_scale,
    );
    preview_entities.entities.push(descender_entity);

    // Advance width line (vertical)
    let advance_width_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(position.x + advance_width, descender_y),
        Vec2::new(position.x + advance_width, ascender_y),
        color,
        temp_entity,
        MetricsLineType::AdvanceWidth,
        &camera_scale,
    );
    preview_entities.entities.push(advance_width_entity);

    // Draw bounding box lines (4 lines for rectangle)
    let top_left = Vec2::new(position.x, position.y + upm);
    let bottom_right = Vec2::new(position.x + advance_width, descender_y);

    // Top line
    let top_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        top_left,
        Vec2::new(bottom_right.x, top_left.y),
        color.with_alpha(0.7),
        temp_entity,
        MetricsLineType::BoundingBox,
        &camera_scale,
    );
    preview_entities.entities.push(top_entity);

    // Right line
    let right_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(bottom_right.x, top_left.y),
        bottom_right,
        color.with_alpha(0.7),
        temp_entity,
        MetricsLineType::BoundingBox,
        &camera_scale,
    );
    preview_entities.entities.push(right_entity);

    // Bottom line
    let bottom_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        bottom_right,
        Vec2::new(top_left.x, bottom_right.y),
        color.with_alpha(0.7),
        temp_entity,
        MetricsLineType::BoundingBox,
        &camera_scale,
    );
    preview_entities.entities.push(bottom_entity);

    // Left line
    let left_entity = spawn_preview_metrics_line(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(top_left.x, bottom_right.y),
        top_left,
        color.with_alpha(0.7),
        temp_entity,
        MetricsLineType::BoundingBox,
        &camera_scale,
    );
    preview_entities.entities.push(left_entity);

    // Add glyph outline preview
    if let Some(glyph_paths) = fontir_state.get_glyph_paths_with_edits(&preview_state.glyph_name) {
        for path in &glyph_paths {
            let outline_entities = create_preview_glyph_outline(
                &mut commands,
                &mut meshes,
                &mut materials,
                path,
                position,
                color.with_alpha(0.6), // Slightly transparent for preview
                &camera_scale,
            );
            preview_entities.entities.extend(outline_entities);
        }
    }
}

/// Helper function to spawn preview metrics lines with special component
fn spawn_preview_metrics_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    color: Color,
    sort_entity: Entity,
    line_type: MetricsLineType,
    camera_scale: &CameraResponsiveScale,
) -> Entity {
    // Use the existing spawn_metrics_line function but add PreviewMetricsLine component
    let entity = spawn_metrics_line(
        commands,
        meshes,
        materials,
        start,
        end,
        color,
        sort_entity,
        line_type,
        camera_scale,
    );

    // Add the preview component for identification with entity existence check
    if let Ok(mut entity_commands) = commands.get_entity(entity) {
        entity_commands.insert(PreviewMetricsLine);
    } else {
        debug!(
            "Skipping PreviewMetricsLine component insertion for non-existent entity {:?}",
            entity
        );
    }

    entity
}

/// Create preview glyph outline entities from a kurbo path with dashed lines
fn create_preview_glyph_outline(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    path: &kurbo::BezPath,
    position: Vec2,
    color: Color,
    camera_scale: &CameraResponsiveScale,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    let line_width = camera_scale.adjusted_line_width();
    let preview_z = 6.0; // Between metrics (5.0) and glyph outlines (8.0)

    // Dashing parameters (same as selection marquee)
    let dash_length = 8.0;
    let gap_length = 4.0;

    // Convert kurbo path to line segments for mesh rendering
    let mut current_pos: Option<kurbo::Point> = None;
    let mut segment_start: Option<kurbo::Point> = None;

    // Flatten the path into line segments
    let _tolerance = 0.5; // Tessellation tolerance
    for element in path.iter() {
        match element {
            kurbo::PathEl::MoveTo(pt) => {
                current_pos = Some(pt);
                segment_start = Some(pt);
            }
            kurbo::PathEl::LineTo(pt) => {
                if let Some(start) = current_pos {
                    let start_world =
                        Vec2::new(start.x as f32 + position.x, start.y as f32 + position.y);
                    let end_world = Vec2::new(pt.x as f32 + position.x, pt.y as f32 + position.y);

                    // Create dashed line segments
                    let dashed_entities = create_dashed_line_meshes(
                        commands,
                        meshes,
                        materials,
                        start_world,
                        end_world,
                        line_width,
                        dash_length,
                        gap_length,
                        color,
                        preview_z,
                    );
                    entities.extend(dashed_entities);
                }
                current_pos = Some(pt);
            }
            kurbo::PathEl::QuadTo(cp, pt) => {
                if let Some(start) = current_pos {
                    // Tessellate quadratic curve into line segments
                    let quad = kurbo::QuadBez::new(start, cp, pt);
                    let mut prev_pt = start;

                    // Simple tessellation - could be improved
                    for i in 1..=8 {
                        let t = i as f64 / 8.0;
                        let curr_pt = quad.eval(t);

                        let start_world =
                            Vec2::new(prev_pt.x as f32 + position.x, prev_pt.y as f32 + position.y);
                        let end_world =
                            Vec2::new(curr_pt.x as f32 + position.x, curr_pt.y as f32 + position.y);

                        // Create dashed line segments for tessellated curves
                        let dashed_entities = create_dashed_line_meshes(
                            commands,
                            meshes,
                            materials,
                            start_world,
                            end_world,
                            line_width,
                            dash_length,
                            gap_length,
                            color,
                            preview_z,
                        );
                        entities.extend(dashed_entities);
                        prev_pt = curr_pt;
                    }
                }
                current_pos = Some(pt);
            }
            kurbo::PathEl::CurveTo(cp1, cp2, pt) => {
                if let Some(start) = current_pos {
                    // Tessellate cubic curve into line segments
                    let cubic = kurbo::CubicBez::new(start, cp1, cp2, pt);
                    let mut prev_pt = start;

                    // Simple tessellation - could be improved
                    for i in 1..=12 {
                        let t = i as f64 / 12.0;
                        let curr_pt = cubic.eval(t);

                        let start_world =
                            Vec2::new(prev_pt.x as f32 + position.x, prev_pt.y as f32 + position.y);
                        let end_world =
                            Vec2::new(curr_pt.x as f32 + position.x, curr_pt.y as f32 + position.y);

                        // Create dashed line segments for tessellated curves
                        let dashed_entities = create_dashed_line_meshes(
                            commands,
                            meshes,
                            materials,
                            start_world,
                            end_world,
                            line_width,
                            dash_length,
                            gap_length,
                            color,
                            preview_z,
                        );
                        entities.extend(dashed_entities);
                        prev_pt = curr_pt;
                    }
                }
                current_pos = Some(pt);
            }
            kurbo::PathEl::ClosePath => {
                if let (Some(current), Some(start)) = (current_pos, segment_start) {
                    // Close the path by connecting back to the start
                    let start_world =
                        Vec2::new(current.x as f32 + position.x, current.y as f32 + position.y);
                    let end_world =
                        Vec2::new(start.x as f32 + position.x, start.y as f32 + position.y);

                    // Create dashed line segments for closing path
                    let dashed_entities = create_dashed_line_meshes(
                        commands,
                        meshes,
                        materials,
                        start_world,
                        end_world,
                        line_width,
                        dash_length,
                        gap_length,
                        color,
                        preview_z,
                    );
                    entities.extend(dashed_entities);
                }
                current_pos = segment_start;
            }
        }
    }

    entities
}

/// Create dashed line mesh entities between two points
fn create_dashed_line_meshes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    line_width: f32,
    dash_length: f32,
    gap_length: f32,
    color: Color,
    z: f32,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    let direction = (end - start).normalize();
    let total_length = start.distance(end);
    let segment_length = dash_length + gap_length;

    let mut current_pos = 0.0;
    while current_pos < total_length {
        let dash_start_pos = current_pos;
        let dash_end_pos = (current_pos + dash_length).min(total_length);

        let dash_start = start + direction * dash_start_pos;
        let dash_end = start + direction * dash_end_pos;

        // Create mesh for this dash segment
        let line_mesh =
            crate::rendering::mesh_utils::create_line_mesh(dash_start, dash_end, line_width);

        let entity = commands
            .spawn((
                PreviewGlyphOutline,
                bevy::render::mesh::Mesh2d(meshes.add(line_mesh)),
                bevy::sprite::MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
                Transform::from_xyz(
                    (dash_start.x + dash_end.x) * 0.5,
                    (dash_start.y + dash_end.y) * 0.5,
                    z,
                ),
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ))
            .id();
        entities.push(entity);

        current_pos += segment_length;
    }

    entities
}

/// System to clean up orphaned metrics entities using component relationships
/// This prevents the race condition by using change detection and explicit relationships
pub fn cleanup_orphaned_metrics(
    buffer_entities: Res<crate::systems::sorts::sort_entities::BufferSortEntities>,
    metrics_query: Query<(Entity, &MetricsFor), With<MetricsLine>>,
    sort_query: Query<Entity, With<crate::editing::sort::Sort>>,
    mut commands: Commands,
) {
    // Only run cleanup if buffer entities have changed to avoid aggressive cleanup
    if !buffer_entities.is_changed() {
        return;
    }

    let mut cleanup_count = 0;
    let mut debug_info = Vec::new();

    for (metrics_entity, metrics_for) in metrics_query.iter() {
        let sort_entity = metrics_for.0;

        // Check if the sort this metrics belongs to still exists
        let sort_exists = sort_query.get(sort_entity).is_ok();
        let sort_in_buffer = buffer_entities.entities.values().any(|&e| e == sort_entity);

        debug_info.push(format!(
            "Metrics {metrics_entity:?} -> Sort {sort_entity:?} (exists: {sort_exists}, in_buffer: {sort_in_buffer})"
        ));

        // Only cleanup if sort doesn't exist AND is not in buffer
        if !sort_exists && !sort_in_buffer {
            debug!(
                "🗑️ CLEANUP: Removing orphaned metrics entity {:?} for sort {:?} (exists: {}, in_buffer: {})",
                metrics_entity, sort_entity, sort_exists, sort_in_buffer
            );
            commands.entity(metrics_entity).despawn();
            cleanup_count += 1;
        }
    }

    if cleanup_count > 0 {
        debug!(
            "🗑️ CLEANUP: Removed {} orphaned metrics entities",
            cleanup_count
        );
        debug!("🗑️ CLEANUP DEBUG: {:?}", debug_info);
    }
}

/// Plugin for mesh-based metrics line rendering
pub struct MetricsRenderingPlugin;

impl Plugin for MetricsRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MetricsLineEntities>()
            .init_resource::<PreviewMetricsEntities>()
            .init_resource::<PreviewMetricsState>()
            .init_resource::<GlyphMetricsCache>()
            .add_systems(
                Update,
                (
                    render_mesh_metrics_lines.in_set(crate::editing::FontEditorSets::Rendering),
                    manage_preview_metrics
                        .in_set(crate::editing::FontEditorSets::Rendering)
                        .after(render_mesh_metrics_lines),
                    cleanup_orphaned_metrics.in_set(crate::editing::FontEditorSets::Cleanup),
                ),
            );
    }
}
