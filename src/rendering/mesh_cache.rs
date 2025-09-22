//! Mesh caching system for performance optimization
//!
//! This module provides caching for expensive mesh generation operations, particularly
//! the tessellation of bezier curves to triangles for filled glyph rendering.
//! By caching generated meshes per glyph, we avoid repeated tessellation overhead.

use bevy::prelude::*;
use std::collections::HashMap;

/// Resource for caching generated meshes to avoid expensive tessellation operations
#[derive(Resource, Default)]
pub struct GlyphMeshCache {
    /// Cache for filled glyph meshes: glyph_name -> mesh handle
    pub filled_meshes: HashMap<String, Handle<Mesh>>,
    /// Cache for outline meshes: glyph_name -> vec of mesh handles for segments
    pub outline_meshes: HashMap<String, Vec<Handle<Mesh>>>,
    /// Cache for metrics line meshes: glyph_name -> vec of mesh handles for metrics
    pub metrics_meshes: HashMap<String, Vec<Handle<Mesh>>>,
    /// Cache invalidation tracking: font generation counter for cache busting
    pub font_generation: u64,
    /// Statistics for monitoring cache performance
    pub stats: MeshCacheStats,
}

/// Statistics for mesh cache performance monitoring
#[derive(Default, Debug)]
pub struct MeshCacheStats {
    /// Number of cache hits for filled meshes
    pub filled_hits: u64,
    /// Number of cache misses for filled meshes (tessellation required)
    pub filled_misses: u64,
    /// Number of cache hits for outline meshes
    pub outline_hits: u64,
    /// Number of cache misses for outline meshes
    pub outline_misses: u64,
    /// Number of cache hits for metrics meshes
    pub metrics_hits: u64,
    /// Number of cache misses for metrics meshes
    pub metrics_misses: u64,
}

impl GlyphMeshCache {
    /// Get a cached filled mesh for a glyph, or None if not cached
    pub fn get_filled_mesh(&mut self, glyph_name: &str) -> Option<Handle<Mesh>> {
        if let Some(mesh_handle) = self.filled_meshes.get(glyph_name) {
            self.stats.filled_hits += 1;
            debug!(
                "Mesh cache HIT for filled glyph '{}' (hits: {})",
                glyph_name, self.stats.filled_hits
            );
            Some(mesh_handle.clone())
        } else {
            self.stats.filled_misses += 1;
            debug!(
                "Mesh cache MISS for filled glyph '{}' (misses: {})",
                glyph_name, self.stats.filled_misses
            );
            None
        }
    }

    /// Cache a filled mesh for a glyph
    pub fn cache_filled_mesh(&mut self, glyph_name: String, mesh_handle: Handle<Mesh>) {
        debug!("Caching filled mesh for glyph '{}'", glyph_name);
        self.filled_meshes.insert(glyph_name, mesh_handle);
    }

    /// Get cached outline meshes for a glyph, or None if not cached
    pub fn get_outline_meshes(&mut self, glyph_name: &str) -> Option<Vec<Handle<Mesh>>> {
        if let Some(mesh_handles) = self.outline_meshes.get(glyph_name) {
            self.stats.outline_hits += 1;
            debug!(
                "Mesh cache HIT for outline glyph '{}' ({} segments)",
                glyph_name,
                mesh_handles.len()
            );
            Some(mesh_handles.clone())
        } else {
            self.stats.outline_misses += 1;
            debug!("Mesh cache MISS for outline glyph '{}'", glyph_name);
            None
        }
    }

    /// Cache outline meshes for a glyph
    pub fn cache_outline_meshes(&mut self, glyph_name: String, mesh_handles: Vec<Handle<Mesh>>) {
        debug!(
            "Caching {} outline meshes for glyph '{}'",
            mesh_handles.len(),
            glyph_name
        );
        self.outline_meshes.insert(glyph_name, mesh_handles);
    }

    /// Get cached metrics meshes for a glyph, or None if not cached
    pub fn get_metrics_meshes(&mut self, glyph_name: &str) -> Option<Vec<Handle<Mesh>>> {
        if let Some(mesh_handles) = self.metrics_meshes.get(glyph_name) {
            self.stats.metrics_hits += 1;
            debug!(
                "Mesh cache HIT for metrics glyph '{}' ({} lines)",
                glyph_name,
                mesh_handles.len()
            );
            Some(mesh_handles.clone())
        } else {
            self.stats.metrics_misses += 1;
            debug!("Mesh cache MISS for metrics glyph '{}'", glyph_name);
            None
        }
    }

    /// Cache metrics meshes for a glyph
    pub fn cache_metrics_meshes(&mut self, glyph_name: String, mesh_handles: Vec<Handle<Mesh>>) {
        debug!(
            "Caching {} metrics meshes for glyph '{}'",
            mesh_handles.len(),
            glyph_name
        );
        self.metrics_meshes.insert(glyph_name, mesh_handles);
    }

    /// Invalidate all cached meshes (call when font data changes)
    pub fn invalidate_all(&mut self) {
        debug!("Invalidating all mesh caches due to font change (filled: {}, outline: {}, metrics: {})", 
            self.filled_meshes.len(), self.outline_meshes.len(), self.metrics_meshes.len());

        self.filled_meshes.clear();
        self.outline_meshes.clear();
        self.metrics_meshes.clear();
        self.font_generation += 1;
    }

    /// Invalidate cache for a specific glyph (call when glyph data changes)
    pub fn invalidate_glyph(&mut self, glyph_name: &str) {
        debug!("Invalidating mesh cache for glyph '{}'", glyph_name);
        self.filled_meshes.remove(glyph_name);
        self.outline_meshes.remove(glyph_name);
        self.metrics_meshes.remove(glyph_name);
    }

    /// Get cache statistics for performance monitoring
    pub fn get_stats(&self) -> &MeshCacheStats {
        &self.stats
    }

    /// Get cache hit rate for filled meshes (0.0 to 1.0)
    pub fn filled_hit_rate(&self) -> f32 {
        let total = self.stats.filled_hits + self.stats.filled_misses;
        if total == 0 {
            0.0
        } else {
            self.stats.filled_hits as f32 / total as f32
        }
    }

    /// Get cache hit rate for outline meshes (0.0 to 1.0)
    pub fn outline_hit_rate(&self) -> f32 {
        let total = self.stats.outline_hits + self.stats.outline_misses;
        if total == 0 {
            0.0
        } else {
            self.stats.outline_hits as f32 / total as f32
        }
    }

    /// Get cache hit rate for metrics meshes (0.0 to 1.0)
    pub fn metrics_hit_rate(&self) -> f32 {
        let total = self.stats.metrics_hits + self.stats.metrics_misses;
        if total == 0 {
            0.0
        } else {
            self.stats.metrics_hits as f32 / total as f32
        }
    }

    /// Get total number of cached meshes across all types
    pub fn total_cached_count(&self) -> usize {
        self.filled_meshes.len() + self.outline_meshes.len() + self.metrics_meshes.len()
    }
}

/// Plugin for mesh caching system
pub struct MeshCachingPlugin;

impl Plugin for MeshCachingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlyphMeshCache>().add_systems(
            Update,
            (
                log_cache_stats.run_if(bevy::time::common_conditions::on_timer(
                    std::time::Duration::from_secs(10),
                )),
            ),
        );
    }
}

/// System to log cache statistics periodically for performance monitoring
fn log_cache_stats(cache: Res<GlyphMeshCache>) {
    let stats = cache.get_stats();
    debug!(
        "Mesh Cache Stats: Filled(hit:{}/miss:{}, rate:{:.1}%), Outline(hit:{}/miss:{}, rate:{:.1}%), Metrics(hit:{}/miss:{}, rate:{:.1}%), Total cached: {}",
        stats.filled_hits, stats.filled_misses, cache.filled_hit_rate() * 100.0,
        stats.outline_hits, stats.outline_misses, cache.outline_hit_rate() * 100.0,
        stats.metrics_hits, stats.metrics_misses, cache.metrics_hit_rate() * 100.0,
        cache.total_cached_count()
    );
}
