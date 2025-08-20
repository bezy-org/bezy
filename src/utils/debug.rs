//! Debug utilities for font rendering and metrics
//!
//! This module contains debug functions that were previously scattered
//! throughout the rendering code. It provides clean separation between
//! debug functionality and core rendering logic.

use crate::core::state::{AppState, FontMetrics};
use bevy::prelude::*;

/// Debug font metrics information
///
/// Logs key metrics from the font info for troubleshooting.
#[allow(dead_code)]
pub fn debug_font_metrics(app_state: &AppState) {
    debug!("=== Font Metrics Debug ===");
    let metrics = &app_state.workspace.info.metrics;
    debug!("Units per em: {}", metrics.units_per_em);
    debug!("X-height: {:?}", metrics.x_height);
    debug!("Cap-height: {:?}", metrics.cap_height);
    debug!("Ascender: {:?}", metrics.ascender);
    debug!("Descender: {:?}", metrics.descender);
}

/// Debug layer information
///
/// Logs information about font layers for troubleshooting.
#[allow(dead_code)]
pub fn debug_layer_info(app_state: &AppState) {
    let glyph_count = app_state.workspace.font.glyphs.len();
    debug!("Font found with {} glyphs", glyph_count);

    // Log first few glyph names for reference
    let glyph_names: Vec<String> = app_state
        .workspace
        .font
        .glyphs
        .keys()
        .take(5)
        .cloned()
        .collect();

    if !glyph_names.is_empty() {
        debug!("Sample glyphs: {}", glyph_names.join(", "));
    }
}

/// Debug placeholder glyph creation
///
/// Logs when a placeholder glyph is created due to missing glyphs.
#[allow(dead_code)]
pub fn debug_placeholder_creation(metrics: &FontMetrics) {
    warn!(
        "Creating placeholder glyph - no real glyphs found. \
         Width will be {} units (40% of UPM: {})",
        (metrics.units_per_em as f32 * 0.4) as i32,
        metrics.units_per_em as i32
    );
}

/// Debug missing glyph information
///
/// Logs when a specific glyph cannot be found.
#[allow(dead_code)]
pub fn debug_missing_glyph(glyph_name: &str) {
    warn!("Glyph '{}' not found in font", glyph_name);
}

/// Debug codepoint lookup
///
/// Logs information about codepoint-to-glyph mapping attempts.
#[allow(dead_code)]
pub fn debug_codepoint_lookup(codepoint: &str, found: bool) {
    if found {
        debug!("Successfully found glyph for codepoint U+{}", codepoint);
    } else {
        warn!("No glyph found for codepoint U+{}", codepoint);
    }
}
