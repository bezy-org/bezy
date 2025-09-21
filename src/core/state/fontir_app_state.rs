//! FontIR-based application state
//!
//! This module provides the new AppState that directly uses FontIR structures
//! instead of custom data types. This enables multi-format support and
//! variable font handling.

use anyhow::Result;
use bevy::prelude::*;
use fontdrasil::coords::NormalizedLocation;
use fontdrasil::orchestration::Access;
use fontdrasil::types::GlyphName;
use fontir::ir::{Glyph as FontIRGlyph, GlyphInstance};
use fontir::orchestration::{Context, Flags, WorkId};
use fontir::paths::Paths;
use fontir::source::Source;
use kurbo::{Affine, BezPath, PathEl, Point};
use norad::designspace::DesignSpaceDocument;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, warn};
use ufo2fontir::source::DesignSpaceIrSource;

/// Font metrics extracted from FontIR
#[derive(Debug, Clone)]
pub struct FontIRMetrics {
    pub units_per_em: f32,
    pub ascender: Option<f32>,
    pub descender: Option<f32>,
    pub line_gap: Option<f32>,
    pub x_height: Option<f32>,
    pub cap_height: Option<f32>,
}

/// Mutable working copy of a glyph instance for high-performance editing
#[derive(Clone, Debug)]
pub struct EditableGlyphInstance {
    pub width: f64,
    pub height: Option<f64>,
    pub vertical_origin: Option<f64>,
    pub contours: Vec<BezPath>,
    /// Track if this instance has been modified from the original
    pub is_dirty: bool,
}

impl From<&GlyphInstance> for EditableGlyphInstance {
    fn from(instance: &GlyphInstance) -> Self {
        Self {
            width: instance.width,
            height: instance.height,
            vertical_origin: instance.vertical_origin,
            contours: instance.contours.clone(),
            is_dirty: false,
        }
    }
}

/// Type of font source being edited
#[derive(Debug, Clone, PartialEq)]
pub enum SourceType {
    /// Single UFO file
    SingleUfo,
    /// Designspace with multiple masters
    Designspace { master_count: usize },
}

/// The main application state using FontIR
#[derive(Resource, Clone)]
pub struct FontIRAppState {
    /// The FontIR source (handles UFO, designspace, etc.)
    pub source: Arc<DesignSpaceIrSource>,

    /// FontIR context containing processed font data
    pub context: Option<Arc<Context>>,

    /// Cached original glyph data (immutable, for reference)
    /// Maps glyph name to FontIR glyph
    pub glyph_cache: HashMap<String, Arc<FontIRGlyph>>,

    /// Working copies of glyphs being edited (mutable, for performance)
    /// Only contains glyphs that have been opened for editing
    /// Maps (glyph_name, location) to editable instance
    pub working_copies: HashMap<(String, NormalizedLocation), EditableGlyphInstance>,

    /// Currently selected glyph name
    pub current_glyph: Option<String>,

    /// Currently selected design space location
    /// For variable fonts, this determines which instance we're editing
    pub current_location: NormalizedLocation,

    /// Path to the source file
    pub source_path: PathBuf,

    /// Type of source (single UFO or designspace)
    pub source_type: SourceType,

    /// Kerning groups data loaded from UFO groups.plist
    /// Maps group name (e.g. "public.kern1.a") to list of glyph names
    pub kerning_groups: HashMap<String, Vec<String>>,
}

impl FontIRAppState {
    /// Create a new FontIR-based app state from a font file
    pub fn from_path(path: PathBuf) -> Result<Self> {
        // Load the source (works with .ufo or .designspace)
        let source = Arc::new(DesignSpaceIrSource::new(&path)?);

        // Detect source type based on file extension
        let source_type = if path.extension().and_then(|s| s.to_str()) == Some("designspace") {
            // Try to count masters from the designspace
            match DesignSpaceDocument::load(&path) {
                Ok(ds) => SourceType::Designspace {
                    master_count: ds.sources.len(),
                },
                Err(_) => SourceType::Designspace { master_count: 0 },
            }
        } else {
            // Assume it's a single UFO
            SourceType::SingleUfo
        };

        // Initialize with default location
        // Note: We'll use fallback to first available instance in glyph lookup
        // since exact location matching is complex with variable fonts
        let current_location = NormalizedLocation::default();

        let mut app_state = Self {
            source,
            context: None,
            glyph_cache: HashMap::new(),
            working_copies: HashMap::new(),
            current_glyph: Some("a".to_string()), // Default to 'a' to match GlyphNavigation
            current_location,
            source_path: path.clone(),
            source_type,
            kerning_groups: HashMap::new(),
        };

        // Load glyphs into cache
        if let Err(e) = app_state.load_glyphs() {
            warn!("Failed to load glyphs during FontIR initialization: {}", e);
        }

        // Load kerning groups from UFO
        if let Err(e) = app_state.load_kerning_groups() {
            warn!(
                "Failed to load kerning groups during FontIR initialization: {}",
                e
            );
        }

        Ok(app_state)
    }

    /// Check if this is a single UFO source (not a designspace)
    pub fn is_single_ufo(&self) -> bool {
        matches!(self.source_type, SourceType::SingleUfo)
    }

    /// Set the current glyph
    pub fn set_current_glyph(&mut self, glyph_name: Option<String>) {
        self.current_glyph = glyph_name;
    }

    /// Get a glyph by name
    pub fn get_glyph(&self, name: &str) -> Option<&FontIRGlyph> {
        self.glyph_cache.get(name).map(|g| g.as_ref())
    }

    /// Get the current glyph's path at the current location
    pub fn get_current_glyph_paths(&self) -> Option<Vec<BezPath>> {
        let glyph_name = self.current_glyph.as_ref()?;
        self.get_glyph_paths(glyph_name)
    }

    /// Get a glyph's path by name at the current location
    pub fn get_glyph_paths(&self, glyph_name: &str) -> Option<Vec<BezPath>> {
        // First try to get from FontIR context using the helper method
        if let Some(ref context) = self.context {
            let glyph_name_typed: GlyphName = glyph_name.into();
            let work_id = WorkId::Glyph(glyph_name_typed);

            // Use try_get since we're not sure if the glyph exists
            if let Some(glyph) = context.glyphs.try_get(&work_id) {
                // Try to get the instance at our current location
                if let Some(instance) = glyph.sources().get(&self.current_location) {
                    return Some(instance.contours.clone());
                }

                // Fallback: Use the first available instance if exact location doesn't exist
                if let Some((_location, instance)) = glyph.sources().iter().next() {
                    debug!("get_glyph_paths: Using first available instance for glyph '{}' with {} contours", glyph_name, instance.contours.len());
                    return Some(instance.contours.clone());
                }
            }
        }

        // Fall back to cached glyph data
        if let Some(glyph) = self.get_glyph(glyph_name) {
            // Try to get the instance at our current location
            if let Some(instance) = glyph.sources().get(&self.current_location) {
                return Some(instance.contours.clone());
            }

            // Fallback: Use the first available instance if exact location doesn't exist
            if let Some((_location, instance)) = glyph.sources().iter().next() {
                debug!("get_glyph_paths: Using first available cached instance for glyph '{}' with {} contours", glyph_name, instance.contours.len());
                return Some(instance.contours.clone());
            }
        }

        // Final fallback - return test shapes for common glyphs to verify the system works
        warn!("get_glyph_paths: No real glyph data found for '{}', falling back to placeholder shapes", glyph_name);
        self.create_fallback_glyph_path(glyph_name)
    }

    /// Get a mutable path element by indices
    /// Returns (path_index, element_index) for the found element
    pub fn find_path_element(
        &self,
        contour_idx: usize,
        point_idx: usize,
    ) -> Option<(usize, usize)> {
        let paths = self.get_current_glyph_paths()?;

        if contour_idx >= paths.len() {
            return None;
        }

        let path = &paths[contour_idx];
        let elements: Vec<_> = path.elements().iter().collect();

        if point_idx < elements.len() {
            Some((contour_idx, point_idx))
        } else {
            None
        }
    }

    /// Get or create a working copy of a glyph instance for editing
    /// This is optimized for zero-lag performance during editing
    pub fn get_or_create_working_copy(
        &mut self,
        glyph_name: &str,
    ) -> Option<&mut EditableGlyphInstance> {
        let location = self.current_location.clone();
        let key = (glyph_name.to_string(), location.clone());

        // Fast path: working copy already exists
        if self.working_copies.contains_key(&key) {
            debug!(
                "*** FontIR: REUSING existing working copy for glyph '{}'",
                glyph_name
            );
            return self.working_copies.get_mut(&key);
        }

        // Slow path: create working copy from original FontIR data
        if let Some(fontir_glyph) = self.glyph_cache.get(glyph_name) {
            // Get the appropriate instance for our location
            // Try to find the exact location match first
            let instance = fontir_glyph
                .sources()
                .iter()
                .find(|(loc, _)| **loc == location)
                .map(|(_, inst)| inst)
                .or_else(|| {
                    // Fallback to first available instance if exact match not found
                    fontir_glyph.sources().iter().next().map(|(_, inst)| inst)
                });

            if let Some(instance) = instance {
                let working_copy = EditableGlyphInstance::from(instance);
                debug!(
                    "FontIR: Created new working copy for glyph '{}' with {} contours at location {:?}",
                    glyph_name,
                    working_copy.contours.len(),
                    location
                );
                self.working_copies.insert(key.clone(), working_copy);
                return self.working_copies.get_mut(&key);
            } else {
                warn!("FontIR: No instances found for glyph '{}'", glyph_name);
            }
        } else {
            warn!("FontIR: Glyph '{}' not found in cache", glyph_name);
        }

        None
    }

    /// Update a point position in a FontIR glyph (high-performance implementation)
    pub fn update_point_position(
        &mut self,
        glyph_name: &str,
        contour_idx: usize,
        point_idx: usize,
        new_x: f64,
        new_y: f64,
    ) -> Result<bool> {
        use bevy::log::debug;

        debug!("FontIR: Updating point position for glyph '{}', contour {}, point {} to ({:.1}, {:.1})", 
              glyph_name, contour_idx, point_idx, new_x, new_y);

        // Get or create the working copy for this glyph
        if self.get_or_create_working_copy(glyph_name).is_some() {
            let location = self.current_location.clone();
            let key = (glyph_name.to_string(), location);

            if let Some(working_copy) = self.working_copies.get_mut(&key) {
                // Check bounds
                if contour_idx >= working_copy.contours.len() {
                    warn!(
                        "FontIR: Contour index {} out of bounds for glyph '{}'",
                        contour_idx, glyph_name
                    );
                    return Ok(false);
                }

                // Update the point in the working copy BezPath
                if Self::update_point_in_bezpath_static(
                    &mut working_copy.contours[contour_idx],
                    point_idx,
                    new_x,
                    new_y,
                ) {
                    working_copy.is_dirty = true;
                    debug!(
                        "FontIR: Successfully updated point {} in working copy",
                        point_idx
                    );
                    return Ok(true);
                }
            }
        }

        debug!(
            "FontIR: Could not update point - glyph '{}' not found or invalid indices",
            glyph_name
        );
        Ok(false)
    }

    /// Update a specific point in a BezPath (optimized for performance)
    fn update_point_in_bezpath_static(
        path: &mut BezPath,
        point_idx: usize,
        new_x: f64,
        new_y: f64,
    ) -> bool {
        let elements: Vec<PathEl> = path.elements().to_vec();
        let mut current_point_idx = 0;
        let mut new_elements = Vec::new();

        for element in elements {
            match element {
                PathEl::MoveTo(_pt) => {
                    if current_point_idx == point_idx {
                        new_elements.push(PathEl::MoveTo(Point::new(new_x, new_y)));
                    } else {
                        new_elements.push(element);
                    }
                    current_point_idx += 1;
                }
                PathEl::LineTo(_pt) => {
                    if current_point_idx == point_idx {
                        new_elements.push(PathEl::LineTo(Point::new(new_x, new_y)));
                    } else {
                        new_elements.push(element);
                    }
                    current_point_idx += 1;
                }
                PathEl::CurveTo(c1, c2, pt) => {
                    let mut new_c1 = c1;
                    let mut new_c2 = c2;
                    let mut new_pt = pt;

                    // Check each control point and endpoint
                    if current_point_idx == point_idx {
                        new_c1 = Point::new(new_x, new_y);
                    } else if current_point_idx + 1 == point_idx {
                        new_c2 = Point::new(new_x, new_y);
                    } else if current_point_idx + 2 == point_idx {
                        new_pt = Point::new(new_x, new_y);
                    }

                    new_elements.push(PathEl::CurveTo(new_c1, new_c2, new_pt));
                    current_point_idx += 3;
                }
                PathEl::QuadTo(c, pt) => {
                    let mut new_c = c;
                    let mut new_pt = pt;

                    if current_point_idx == point_idx {
                        new_c = Point::new(new_x, new_y);
                    } else if current_point_idx + 1 == point_idx {
                        new_pt = Point::new(new_x, new_y);
                    }

                    new_elements.push(PathEl::QuadTo(new_c, new_pt));
                    current_point_idx += 2;
                }
                PathEl::ClosePath => {
                    new_elements.push(element);
                    // ClosePath doesn't increment point index
                }
            }
        }

        // Rebuild the BezPath with updated elements
        if current_point_idx > point_idx {
            *path = BezPath::from_vec(new_elements);
            return true;
        }

        false
    }

    /// Update a path element (for advanced point editing)
    pub fn update_path_element(
        &mut self,
        glyph_name: &str,
        contour_idx: usize,
        element_idx: usize,
        new_element: PathEl,
    ) -> Result<()> {
        use bevy::log::warn;

        debug!(
            "FontIR: Update path element for glyph '{}', contour {}, element {}",
            glyph_name, contour_idx, element_idx
        );

        // Same issue as above - Arc<FontIRGlyph> is not easily mutable
        warn!("FontIR path element mutation not yet implemented");
        warn!("This requires architectural changes to support mutable FontIR data");

        // For now, log what we would do:
        debug!(
            "Would update {:?} at contour {}, element {}",
            new_element, contour_idx, element_idx
        );

        Ok(())
    }

    /// Check if this FontIR state can handle mutations
    pub fn supports_mutations(&self) -> bool {
        // Now we support mutations via working copies!
        true
    }

    /// Get glyph paths, preferring working copies over original FontIR data
    /// This ensures live rendering shows the edited version
    pub fn get_glyph_paths_with_edits(&self, glyph_name: &str) -> Option<Vec<BezPath>> {
        let location = &self.current_location;
        let key = (glyph_name.to_string(), location.clone());

        // Fast path: return working copy if it exists
        if let Some(working_copy) = self.working_copies.get(&key) {
            debug!(
                "*** FontIR: Using WORKING COPY for glyph '{}' (dirty: {}, {} contours)",
                glyph_name,
                working_copy.is_dirty,
                working_copy.contours.len()
            );
            return Some(working_copy.contours.clone());
        }

        // Fallback to original FontIR data
        debug!(
            "*** FontIR: Using ORIGINAL DATA for glyph '{}' (no working copy found)",
            glyph_name
        );
        self.get_glyph_paths(glyph_name)
    }

    /// Get glyph paths with component resolution for complete glyph rendering
    /// This resolves component references and applies transformations
    pub fn get_glyph_paths_with_components(&self, glyph_name: &str) -> Option<Vec<BezPath>> {
        let mut all_paths = Vec::new();

        // Get the main outline paths (if any)
        if let Some(outline_paths) = self.get_glyph_paths_with_edits(glyph_name) {
            all_paths.extend(outline_paths);
        }

        // Load component data directly from UFO source for component resolution
        if let Some(component_paths) = self.resolve_components(glyph_name) {
            all_paths.extend(component_paths);
        }

        if all_paths.is_empty() {
            None
        } else {
            Some(all_paths)
        }
    }

    /// Resolve component references by loading UFO data directly
    /// This provides access to component data that FontIR doesn't currently expose
    fn resolve_components(&self, glyph_name: &str) -> Option<Vec<BezPath>> {
        // Load the UFO font directly to access component data
        // This is a temporary approach until FontIR provides better component access
        let ufo_font = if self.source_path.extension()? == "ufo" {
            match norad::Font::load(&self.source_path) {
                Ok(font) => font,
                Err(e) => {
                    warn!("Failed to load UFO for component resolution: {}", e);
                    return None;
                }
            }
        } else if self.source_path.extension()? == "designspace" {
            // For .designspace files, load the first available UFO source
            let designspace_dir = self.source_path.parent()?;

            // Load the designspace to get the sources
            let designspace = DesignSpaceDocument::load(&self.source_path).ok()?;
            let first_source = designspace.sources.first()?;
            let first_ufo_path = designspace_dir.join(&first_source.filename);

            debug!(
                "🔧 Loading UFO from designspace for component resolution: {:?}",
                first_ufo_path
            );
            match norad::Font::load(&first_ufo_path) {
                Ok(font) => {
                    debug!("✅ Successfully loaded UFO for component resolution");
                    font
                }
                Err(e) => {
                    warn!("Failed to load first UFO source from designspace for component resolution: {}", e);
                    return None;
                }
            }
        } else {
            debug!(
                "Component resolution not implemented for file type: {:?}",
                self.source_path.extension()
            );
            return None;
        };

        // Get the glyph from UFO
        let layer = ufo_font.default_layer();
        let ufo_glyph = layer.get_glyph(glyph_name)?;

        // Check if this glyph has components
        if ufo_glyph.components.is_empty() {
            debug!("📭 Glyph '{}' has no components", glyph_name);
            return None;
        }

        debug!(
            "🧩 Resolving {} components for glyph '{}'",
            ufo_glyph.components.len(),
            glyph_name
        );
        let mut component_paths = Vec::new();

        // Resolve each component
        for (i, component) in ufo_glyph.components.iter().enumerate() {
            let base_glyph_name = component.base.as_str();
            debug!(
                "🧩 Component {}/{}: base='{}', transform=[{}, {}, {}, {}, {}, {}]",
                i + 1,
                ufo_glyph.components.len(),
                base_glyph_name,
                component.transform.x_scale,
                component.transform.xy_scale,
                component.transform.yx_scale,
                component.transform.y_scale,
                component.transform.x_offset,
                component.transform.y_offset
            );

            // Get the base glyph's paths (recursively resolve components)
            if let Some(base_paths) = self.get_glyph_paths_with_components(base_glyph_name) {
                debug!(
                    "🧩 Found {} paths for component '{}'",
                    base_paths.len(),
                    base_glyph_name
                );
                for (path_idx, base_path) in base_paths.iter().enumerate() {
                    // Apply the component's transformation matrix
                    let transformed_path = apply_affine_transform(base_path, &component.transform);
                    let path_elements = transformed_path.elements().len();
                    debug!(
                        "🧩 Applied transform to path {}: {} elements",
                        path_idx, path_elements
                    );
                    component_paths.push(transformed_path);
                }
            } else {
                warn!(
                    "🧩 Component base glyph '{}' not found for glyph '{}'",
                    base_glyph_name, glyph_name
                );
            }
        }

        if component_paths.is_empty() {
            warn!("🧩 No component paths resolved for glyph '{}'", glyph_name);
            None
        } else {
            debug!(
                "🧩 Successfully resolved {} total paths for composite glyph '{}'",
                component_paths.len(),
                glyph_name
            );
            Some(component_paths)
        }
    }

    /// Load all glyphs into cache
    pub fn load_glyphs(&mut self) -> Result<()> {
        debug!("Loading glyphs from FontIR source");

        // Create a new context with proper flags and paths
        let flags = Flags::empty(); // For now, use empty flags
        let temp_dir = std::env::temp_dir();
        let paths = Paths::new(&temp_dir);
        let mut context = Context::new_root(flags, paths);

        // Execute FontIR work items to populate the context with real glyph data
        if let Err(e) = self.execute_fontir_work(&mut context) {
            warn!(
                "Failed to execute FontIR work: {}. Trying alternative direct access approach.",
                e
            );

            // Alternative approach: Try direct data extraction from DesignSpaceIrSource
            if let Err(e2) = self.try_direct_glyph_extraction() {
                warn!(
                    "Direct glyph extraction also failed: {}. Using fallback data.",
                    e2
                );
            }
        }

        self.context = Some(Arc::new(context));

        Ok(())
    }

    /// Execute FontIR work items with proper orchestration and permissions
    fn execute_fontir_work(&mut self, context: &mut Context) -> Result<()> {
        debug!("Executing FontIR work items to load real glyph data");

        // First, create and execute static metadata work
        let static_metadata_work = self.source.create_static_metadata_work()?;
        let static_read_access = static_metadata_work.read_access();
        let static_write_access = static_metadata_work.write_access();
        let static_context = context.copy_for_work(static_read_access, static_write_access);

        debug!("Executing static metadata work");
        if let Err(e) = static_metadata_work.exec(&static_context) {
            warn!("Static metadata work failed: {}", e);
            return Err(anyhow::anyhow!("Static metadata work failed: {}", e));
        }

        // Create and execute preliminary glyph order work
        let glyph_order_work = fontir::glyph::create_glyph_order_work();
        let order_read_access = glyph_order_work.read_access();
        let order_write_access = glyph_order_work.write_access();
        let order_context = context.copy_for_work(order_read_access, order_write_access);

        // Execute global metrics work
        let global_metrics_work = self.source.create_global_metric_work()?;
        let metrics_read_access = global_metrics_work.read_access();
        let metrics_write_access = global_metrics_work.write_access();
        let metrics_context = context.copy_for_work(metrics_read_access, metrics_write_access);

        debug!("Executing global metrics work");
        if let Err(e) = global_metrics_work.exec(&metrics_context) {
            warn!("Global metrics work failed: {}", e);
        }

        // Create and execute glyph IR work items
        let glyph_work_items = self.source.create_glyph_ir_work()?;
        debug!("Executing {} glyph IR work items", glyph_work_items.len());

        // Execute each glyph work item with proper permissions
        // Glyph work items need access to all previously computed data
        use fontdrasil::orchestration::AccessBuilder;

        for (i, work_item) in glyph_work_items.iter().enumerate() {
            // Glyph work needs broader read access than what's specified in the work item
            // It needs to read static metadata, global metrics, and other glyphs for components
            let broad_read_access = AccessBuilder::new()
                .variant(WorkId::StaticMetadata)
                .variant(WorkId::GlobalMetrics)
                .variant(WorkId::PreliminaryGlyphOrder)
                .variant(WorkId::ALL_GLYPHS) // Access to all glyphs for component resolution
                .build();

            let write_access = work_item.write_access();
            let work_context = context.copy_for_work(broad_read_access, write_access);

            if let Err(e) = work_item.exec(&work_context) {
                warn!("Glyph work item {} failed: {}", i, e);
                // Continue with other work items even if one fails
                continue;
            }

            // Cache the glyph data if it was successfully created
            if let Ok(glyph_name) = self.extract_glyph_name_from_work_id(&work_item.id()) {
                if let Some(glyph) = work_context.glyphs.try_get(&work_item.id()) {
                    self.glyph_cache.insert(glyph_name, glyph);
                }
            }
        }

        // Execute preliminary glyph order after glyphs are loaded
        debug!("Executing glyph order work");
        if let Err(e) = glyph_order_work.exec(&order_context) {
            warn!("Glyph order work failed: {}", e);
        }

        debug!(
            "FontIR work execution completed. Loaded {} glyphs into cache",
            self.glyph_cache.len()
        );
        Ok(())
    }

    /// Helper to extract glyph name from WorkId
    fn extract_glyph_name_from_work_id(&self, work_id: &WorkId) -> Result<String> {
        match work_id {
            WorkId::Glyph(glyph_name) => Ok(glyph_name.to_string()),
            _ => Err(anyhow::anyhow!("Work ID is not a glyph: {:?}", work_id)),
        }
    }

    /// Alternative approach: Try to extract glyph data directly from DesignSpaceIrSource
    /// without complex orchestration. This is simpler but may not handle all edge cases.
    fn try_direct_glyph_extraction(&mut self) -> Result<()> {
        debug!("Attempting direct glyph extraction from DesignSpaceIrSource");

        // This approach tries to access the underlying UFO data directly
        // Note: This is less robust than full FontIR orchestration but may work for simple cases

        // For now, we'll attempt to create a minimal working context
        // and try to access the source data structures directly

        // Create work items but use them only to understand what glyphs exist
        match self.source.create_glyph_ir_work() {
            Ok(glyph_work_items) => {
                debug!(
                    "Found {} glyph work items to extract",
                    glyph_work_items.len()
                );

                // Try to create a simplified context with full access for testing
                let flags = Flags::empty();
                let temp_dir = std::env::temp_dir();
                let paths = Paths::new(&temp_dir);
                let context = Context::new_root(flags, paths);

                // Create a context with full access permissions for testing
                use fontdrasil::orchestration::AccessBuilder;

                // Create broad access that should cover all necessary data
                let broad_read_access = AccessBuilder::new()
                    .variant(WorkId::StaticMetadata)
                    .variant(WorkId::GlobalMetrics)
                    .variant(WorkId::PreliminaryGlyphOrder)
                    .variant(WorkId::ALL_GLYPHS)
                    .variant(WorkId::ALL_ANCHORS)
                    .build();

                let full_access_context = context.copy_for_work(broad_read_access, Access::All);

                // Try to execute just one work item to test the approach
                if let Some(test_work) = glyph_work_items.first() {
                    debug!(
                        "Testing work execution with work item: {:?}",
                        test_work.id()
                    );

                    if let Err(e) = test_work.exec(&full_access_context) {
                        warn!("Direct work execution failed: {}", e);
                        return Err(anyhow::anyhow!("Direct work execution failed: {}", e));
                    }

                    // If successful, try to extract the glyph data
                    if let Ok(glyph_name) = self.extract_glyph_name_from_work_id(&test_work.id()) {
                        if let Some(glyph) = full_access_context.glyphs.try_get(&test_work.id()) {
                            debug!(
                                "Successfully extracted glyph '{}' with {} sources",
                                glyph_name,
                                glyph.sources().len()
                            );
                            self.glyph_cache.insert(glyph_name, glyph);
                            return Ok(());
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to create glyph work items: {}", e);
                return Err(anyhow::anyhow!("Failed to create glyph work items: {}", e));
            }
        }

        Err(anyhow::anyhow!("Direct glyph extraction did not succeed"))
    }

    /// Get font metrics from FontIR source
    pub fn get_font_metrics(&self) -> FontIRMetrics {
        if let Some(ref context) = self.context {
            // Get static metadata for units_per_em
            let static_metadata = context.static_metadata.get();
            let units_per_em = static_metadata.units_per_em as f32;
            let default_location = static_metadata.default_location();

            // Get global metrics at default location
            let global_metrics = context.global_metrics.get();
            let metrics = global_metrics.at(default_location);

            let fontir_metrics = FontIRMetrics {
                units_per_em,
                ascender: Some(metrics.ascender.0 as f32),
                descender: Some(metrics.descender.0 as f32),
                line_gap: Some(metrics.os2_typo_line_gap.0 as f32),
                x_height: Some(metrics.x_height.0 as f32),
                cap_height: Some(metrics.cap_height.0 as f32),
            };

            debug!("FontIR metrics extracted: UPM={}, ascender={}, descender={}, x_height={}, cap_height={}", 
                  fontir_metrics.units_per_em,
                  fontir_metrics.ascender.unwrap_or(-1.0),
                  fontir_metrics.descender.unwrap_or(-1.0),
                  fontir_metrics.x_height.unwrap_or(-1.0),
                  fontir_metrics.cap_height.unwrap_or(-1.0));

            fontir_metrics
        } else {
            // Fallback to sensible defaults if context not available
            FontIRMetrics {
                units_per_em: 1000.0,
                ascender: Some(800.0),
                descender: Some(-200.0),
                line_gap: Some(0.0),
                x_height: Some(500.0),
                cap_height: Some(700.0),
            }
        }
    }

    /// Get glyph names available in the font
    pub fn get_glyph_names(&self) -> Vec<String> {
        // First try to get from the FontIR context
        if let Some(ref context) = self.context {
            let all_glyphs = context.glyphs.all();
            let mut names: Vec<String> = all_glyphs
                .into_iter()
                .filter_map(|(work_id, _)| {
                    if let WorkId::Glyph(glyph_name) = work_id {
                        Some(glyph_name.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            if !names.is_empty() {
                names.sort();
                return names;
            }
        }

        // Fall back to cached glyph names if available
        if !self.glyph_cache.is_empty() {
            let mut names: Vec<String> = self.glyph_cache.keys().cloned().collect();
            names.sort();
            return names;
        }

        // Final fallback - return basic test glyph names
        vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
            "f".to_string(),
            "g".to_string(),
            "h".to_string(),
            "i".to_string(),
            "j".to_string(),
            "k".to_string(),
            "l".to_string(),
            "m".to_string(),
            "n".to_string(),
            "o".to_string(),
            "p".to_string(),
            "q".to_string(),
            "r".to_string(),
            "s".to_string(),
            "t".to_string(),
            "u".to_string(),
            "v".to_string(),
            "w".to_string(),
            "x".to_string(),
            "y".to_string(),
            "z".to_string(),
        ]
    }

    /// Get advance width for a glyph
    pub fn get_glyph_advance_width(&self, glyph_name: &str) -> f32 {
        // First try to get from FontIR context
        if let Some(ref context) = self.context {
            let glyph_name_typed: GlyphName = glyph_name.into();
            let work_id = WorkId::Glyph(glyph_name_typed);

            // Use try_get since we're not sure if the glyph exists
            if let Some(glyph) = context.glyphs.try_get(&work_id) {
                debug!(
                    "get_glyph_advance_width: Found glyph '{}' in context",
                    glyph_name
                );

                // Log available locations
                let sources = glyph.sources();
                debug!(
                    "get_glyph_advance_width: Glyph '{}' has {} sources",
                    glyph_name,
                    sources.len()
                );
                for (loc, instance) in sources.iter() {
                    debug!("  Location: {:?}, width: {}", loc, instance.width);
                }

                // Get the instance at our current location
                if let Some(instance) = sources.get(&self.current_location) {
                    debug!(
                        "get_glyph_advance_width: Found at current location, width: {}",
                        instance.width
                    );
                    return instance.width as f32;
                } else {
                    debug!(
                        "get_glyph_advance_width: Not found at current location {:?}",
                        self.current_location
                    );
                    // Try first available instance
                    if let Some((loc, instance)) = sources.iter().next() {
                        debug!(
                            "get_glyph_advance_width: Using first instance at {:?}, width: {}",
                            loc, instance.width
                        );
                        return instance.width as f32;
                    }
                }
            } else {
                debug!(
                    "get_glyph_advance_width: Glyph '{}' not found in context",
                    glyph_name
                );
            }
        } else {
            debug!("get_glyph_advance_width: No context available");
        }

        // Fall back to cached glyph data
        if let Some(glyph) = self.get_glyph(glyph_name) {
            debug!(
                "get_glyph_advance_width: Found glyph '{}' in cache",
                glyph_name
            );
            let sources = glyph.sources();
            // Get the instance at our current location
            if let Some(instance) = sources.get(&self.current_location) {
                debug!(
                    "get_glyph_advance_width: Found in cache at current location, width: {}",
                    instance.width
                );
                return instance.width as f32;
            } else {
                // Try first available instance
                if let Some((loc, instance)) = sources.iter().next() {
                    debug!(
                        "get_glyph_advance_width: Using first cached instance at {:?}, width: {}",
                        loc, instance.width
                    );
                    return instance.width as f32;
                }
            }
        } else {
            debug!(
                "get_glyph_advance_width: Glyph '{}' not found in cache",
                glyph_name
            );
        }

        // Final fallback - return reasonable defaults for common glyphs
        debug!(
            "get_glyph_advance_width: Using fallback value for glyph '{}'",
            glyph_name
        );
        match glyph_name {
            "a" | "c" | "e" | "o" | "u" => 500.0,
            "b" | "d" | "h" | "k" | "l" | "p" | "q" => 550.0,
            "f" | "i" | "j" | "r" | "t" => 300.0,
            "g" | "n" | "s" | "v" | "x" | "y" | "z" => 450.0,
            "m" | "w" => 750.0,
            _ => 600.0,
        }
    }

    /// Create fallback glyph paths for testing
    fn create_fallback_glyph_path(&self, glyph_name: &str) -> Option<Vec<BezPath>> {
        match glyph_name {
            "a" => {
                let mut path = BezPath::new();
                // Create a simple "a" shape - rectangle with notch
                path.move_to((100.0, 0.0));
                path.line_to((100.0, 400.0));
                path.line_to((300.0, 400.0));
                path.line_to((300.0, 200.0));
                path.line_to((200.0, 200.0));
                path.line_to((200.0, 0.0));
                path.close_path();
                Some(vec![path])
            }
            "b" => {
                let mut path = BezPath::new();
                // Create a simple "b" shape - vertical line with bumps
                path.move_to((50.0, 0.0));
                path.line_to((50.0, 400.0));
                path.line_to((200.0, 400.0));
                path.line_to((200.0, 200.0));
                path.line_to((150.0, 200.0));
                path.line_to((150.0, 0.0));
                path.close_path();
                Some(vec![path])
            }
            "c" => {
                let mut path = BezPath::new();
                // Create a simple "c" shape - open rectangle
                path.move_to((300.0, 100.0));
                path.line_to((100.0, 100.0));
                path.line_to((100.0, 300.0));
                path.line_to((300.0, 300.0));
                Some(vec![path])
            }
            "d" => {
                let mut path = BezPath::new();
                // Create a simple "d" shape - rectangle with stem
                path.move_to((100.0, 0.0));
                path.line_to((100.0, 300.0));
                path.line_to((250.0, 300.0));
                path.line_to((250.0, 400.0));
                path.line_to((280.0, 400.0));
                path.line_to((280.0, 0.0));
                path.close_path();
                Some(vec![path])
            }
            "e" => {
                let mut path = BezPath::new();
                // Create a simple "e" shape - rectangle with horizontal line
                path.move_to((100.0, 100.0));
                path.line_to((100.0, 300.0));
                path.line_to((300.0, 300.0));
                path.line_to((300.0, 250.0));
                path.line_to((150.0, 250.0));
                path.line_to((150.0, 200.0));
                path.line_to((280.0, 200.0));
                path.line_to((280.0, 100.0));
                path.close_path();
                Some(vec![path])
            }
            "h" => {
                let mut path = BezPath::new();
                // Create a simple "h" shape - two vertical lines with crossbar
                path.move_to((50.0, 0.0));
                path.line_to((50.0, 400.0));
                path.line_to((80.0, 400.0));
                path.line_to((80.0, 220.0));
                path.line_to((200.0, 220.0));
                path.line_to((200.0, 400.0));
                path.line_to((230.0, 400.0));
                path.line_to((230.0, 0.0));
                path.line_to((200.0, 0.0));
                path.line_to((200.0, 190.0));
                path.line_to((80.0, 190.0));
                path.line_to((80.0, 0.0));
                path.close_path();
                Some(vec![path])
            }
            "l" => {
                let mut path = BezPath::new();
                // Create a simple "l" shape - vertical line
                path.move_to((140.0, 0.0));
                path.line_to((140.0, 400.0));
                path.line_to((170.0, 400.0));
                path.line_to((170.0, 0.0));
                path.close_path();
                Some(vec![path])
            }
            "o" => {
                let mut path = BezPath::new();
                // Create a simple "o" shape - hollow rectangle
                path.move_to((100.0, 100.0));
                path.line_to((100.0, 300.0));
                path.line_to((300.0, 300.0));
                path.line_to((300.0, 100.0));
                path.close_path();
                // Inner hole
                path.move_to((140.0, 140.0));
                path.line_to((260.0, 140.0));
                path.line_to((260.0, 260.0));
                path.line_to((140.0, 260.0));
                path.close_path();
                Some(vec![path])
            }
            "r" => {
                let mut path = BezPath::new();
                // Create a simple "r" shape - vertical line with top branch
                path.move_to((50.0, 0.0));
                path.line_to((50.0, 300.0));
                path.line_to((200.0, 300.0));
                path.line_to((200.0, 270.0));
                path.line_to((80.0, 270.0));
                path.line_to((80.0, 0.0));
                path.close_path();
                Some(vec![path])
            }
            "w" => {
                let mut path = BezPath::new();
                // Create a simple "w" shape - zigzag
                path.move_to((50.0, 300.0));
                path.line_to((80.0, 0.0));
                path.line_to((110.0, 0.0));
                path.line_to((140.0, 200.0));
                path.line_to((170.0, 0.0));
                path.line_to((200.0, 0.0));
                path.line_to((230.0, 300.0));
                path.line_to((200.0, 300.0));
                path.line_to((185.0, 100.0));
                path.line_to((170.0, 200.0));
                path.line_to((155.0, 100.0));
                path.line_to((140.0, 300.0));
                path.close_path();
                Some(vec![path])
            }
            // Add a few more common letters
            "s" => {
                let mut path = BezPath::new();
                // Create a simple "s" shape - stepped rectangle
                path.move_to((100.0, 100.0));
                path.line_to((100.0, 150.0));
                path.line_to((200.0, 150.0));
                path.line_to((200.0, 200.0));
                path.line_to((100.0, 200.0));
                path.line_to((100.0, 250.0));
                path.line_to((300.0, 250.0));
                path.line_to((300.0, 300.0));
                path.line_to((200.0, 300.0));
                path.line_to((200.0, 350.0));
                path.line_to((300.0, 350.0));
                path.line_to((300.0, 400.0));
                path.line_to((100.0, 400.0));
                path.close_path();
                Some(vec![path])
            }
            "t" => {
                let mut path = BezPath::new();
                // Create a simple "t" shape - T shape
                path.move_to((50.0, 350.0));
                path.line_to((50.0, 400.0));
                path.line_to((250.0, 400.0));
                path.line_to((250.0, 350.0));
                path.line_to((170.0, 350.0));
                path.line_to((170.0, 0.0));
                path.line_to((130.0, 0.0));
                path.line_to((130.0, 350.0));
                path.close_path();
                Some(vec![path])
            }
            _ => None,
        }
    }

    /// Load kerning groups data from the UFO file
    /// This loads the groups.plist data that FontIR doesn't currently expose
    pub fn load_kerning_groups(&mut self) -> Result<()> {
        let source_path = self.source_path.clone();
        if let Some(ext) = source_path.extension() {
            if ext == "ufo" {
                // Load groups directly from UFO file
                self.load_groups_from_ufo(&source_path)?;
            } else if ext == "designspace" {
                // Load groups from UFO sources referenced in designspace
                self.load_groups_from_designspace()?;
            } else {
                debug!(
                    "Skipping groups loading for unsupported file type: {:?}",
                    ext
                );
            }
        }

        debug!(
            "Successfully loaded {} kerning groups into FontIR",
            self.kerning_groups.len()
        );
        Ok(())
    }

    /// Load kerning groups from a UFO file
    fn load_groups_from_ufo(&mut self, ufo_path: &std::path::Path) -> Result<()> {
        let groups_path = ufo_path.join("groups.plist");
        if !groups_path.exists() {
            debug!("No groups.plist found at: {:?}", groups_path);
            return Ok(());
        }

        debug!("Loading kerning groups from UFO: {:?}", ufo_path);

        // Load the norad font temporarily just to access groups
        match norad::Font::load(ufo_path) {
            Ok(font) => {
                debug!(
                    "Successfully loaded UFO, found {} groups",
                    font.groups.len()
                );

                // Extract groups data
                for (group_name, glyph_names) in font.groups.iter() {
                    let names: Vec<String> = glyph_names.iter().map(|n| n.to_string()).collect();

                    // Log groups that contain 'a'
                    if names.contains(&"a".to_string()) {
                        debug!("★ Group '{}' contains 'a': {:?}", group_name, names);
                    }

                    self.kerning_groups.insert(group_name.to_string(), names);
                }
                Ok(())
            }
            Err(e) => {
                warn!("Failed to load UFO for groups data: {}", e);
                Ok(()) // Don't fail the entire initialization
            }
        }
    }

    /// Load kerning groups from designspace file by loading from source UFOs
    fn load_groups_from_designspace(&mut self) -> Result<()> {
        let source_path = self.source_path.clone();
        debug!("Loading kerning groups from designspace: {:?}", source_path);

        // Parse the designspace file to get source UFO paths
        match DesignSpaceDocument::load(&source_path) {
            Ok(designspace) => {
                debug!(
                    "Successfully loaded designspace with {} sources",
                    designspace.sources.len()
                );

                // Load groups from each source UFO
                for source in &designspace.sources {
                    let filename = &source.filename;
                    // Resolve the UFO path relative to the designspace file
                    let designspace_dir = source_path
                        .parent()
                        .unwrap_or_else(|| std::path::Path::new("."));
                    let ufo_path = designspace_dir.join(filename);

                    debug!("Loading groups from source UFO: {:?}", ufo_path);

                    // Load groups from this UFO (ignore errors for individual UFOs)
                    if let Err(e) = self.load_groups_from_ufo(&ufo_path) {
                        warn!("Failed to load groups from {}: {}", ufo_path.display(), e);
                    }
                }
                Ok(())
            }
            Err(e) => {
                warn!("Failed to load designspace for groups: {}", e);
                Ok(()) // Don't fail the entire initialization
            }
        }
    }

    /// Find which kerning groups a glyph belongs to
    /// Returns simplified group names (without "public.kern1." or "public.kern2." prefix)
    pub fn get_glyph_kerning_groups(&self, glyph_name: &str) -> (Option<String>, Option<String>) {
        let mut left_group = None;
        let mut right_group = None;

        for (group_name, glyph_list) in &self.kerning_groups {
            if glyph_list.contains(&glyph_name.to_string()) {
                if let Some(suffix) = group_name.strip_prefix("public.kern1.") {
                    left_group = Some(suffix.to_string());
                } else if let Some(suffix) = group_name.strip_prefix("public.kern2.") {
                    right_group = Some(suffix.to_string());
                }
            }
        }

        (left_group, right_group)
    }

    /// Get glyph name for a Unicode character by looking up Unicode mappings in the font
    pub fn get_glyph_name_for_unicode(&self, unicode_char: char) -> Option<String> {
        debug!(
            "🔍 FONTIR UNICODE: Looking up character '{}' (U+{:04X})",
            unicode_char, unicode_char as u32
        );

        // SIMPLE TEST: For now, just handle basic ASCII letters directly
        // This will help us verify the lookup path is working
        match unicode_char {
            'a'..='z' | 'A'..='Z' => {
                let char_name = unicode_char.to_string();
                debug!(
                    "✅ FONTIR UNICODE: Simple ASCII lookup for '{}' -> '{}'",
                    unicode_char, char_name
                );
                return Some(char_name);
            }
            _ => {
                debug!("🔍 FONTIR UNICODE: Not a simple ASCII letter, trying UFO lookup");
            }
        }

        // Load Unicode mappings from the source UFO if not already cached
        let source_path = &self.source_path;

        // Try to read the UFO and find glyphs with this Unicode value
        if source_path.extension() == Some(std::ffi::OsStr::new("ufo")) {
            if let Ok(font) = norad::Font::load(source_path) {
                // Look through all glyphs for one that maps to this Unicode character
                for glyph in font.default_layer().iter() {
                    if glyph.codepoints.contains(unicode_char) {
                        debug!(
                            "Found glyph '{}' for character '{}' (U+{:04X}) via Unicode mapping",
                            glyph.name(),
                            unicode_char,
                            unicode_char as u32
                        );
                        return Some(glyph.name().to_string());
                    }
                }
            }
        } else if source_path.extension() == Some(std::ffi::OsStr::new("designspace")) {
            // For .designspace files, load the first available UFO source
            if let Some(designspace_dir) = source_path.parent() {
                // Load the designspace to get the sources
                if let Ok(designspace) = DesignSpaceDocument::load(&source_path) {
                    if let Some(first_source) = designspace.sources.first() {
                        let first_ufo_path = designspace_dir.join(&first_source.filename);
                        if let Ok(font) = norad::Font::load(&first_ufo_path) {
                            for glyph in font.default_layer().iter() {
                                if glyph.codepoints.contains(unicode_char) {
                                    debug!("Found glyph '{}' for character '{}' (U+{:04X}) via Unicode mapping in designspace source",
                                          glyph.name(), unicode_char, unicode_char as u32);
                                    return Some(glyph.name().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        debug!(
            "❌ FONTIR UNICODE: No mapping found for character '{}' (U+{:04X})",
            unicode_char, unicode_char as u32
        );
        None
    }
}

/// Helper to convert PathEl to a point position
pub fn path_element_position(el: &PathEl) -> Option<Point> {
    match el {
        PathEl::MoveTo(pt) => Some(*pt),
        PathEl::LineTo(pt) => Some(*pt),
        PathEl::CurveTo(_, _, pt) => Some(*pt),
        PathEl::QuadTo(_, pt) => Some(*pt),
        PathEl::ClosePath => None,
    }
}

/// Helper to get control points from a path element
pub fn path_element_control_points(el: &PathEl) -> Vec<Point> {
    match el {
        PathEl::CurveTo(c1, c2, _) => vec![*c1, *c2],
        PathEl::QuadTo(c, _) => vec![*c],
        _ => vec![],
    }
}

/// Point type derived from PathEl
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontIRPointType {
    Move,
    Line,
    Curve,
    Quad,
    OffCurve,
}

impl From<&PathEl> for FontIRPointType {
    fn from(el: &PathEl) -> Self {
        match el {
            PathEl::MoveTo(_) => FontIRPointType::Move,
            PathEl::LineTo(_) => FontIRPointType::Line,
            PathEl::CurveTo(_, _, _) => FontIRPointType::Curve,
            PathEl::QuadTo(_, _) => FontIRPointType::Quad,
            PathEl::ClosePath => FontIRPointType::Move, // Treat as move for now
        }
    }
}

/// Apply an affine transformation to a BezPath
/// Converts norad::AffineTransform to kurbo::Affine and transforms the path
pub fn apply_affine_transform(path: &BezPath, transform: &norad::AffineTransform) -> BezPath {
    // Convert norad::AffineTransform to kurbo::Affine
    // norad format: [x_scale, xy_scale, yx_scale, y_scale, x_offset, y_offset]
    // kurbo format: [xx, yx, xy, yy, x, y] (matrix form)
    let affine = Affine::new([
        transform.x_scale,
        transform.yx_scale,
        transform.xy_scale,
        transform.y_scale,
        transform.x_offset,
        transform.y_offset,
    ]);

    // Apply the transformation to the path
    affine * path
}
