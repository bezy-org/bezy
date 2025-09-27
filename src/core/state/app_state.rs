//! Core application state structures
//!
//! This module contains the main AppState and Workspace structures
//! that manage the overall font editing session.

use anyhow::Context;
use bevy::prelude::*;
use std::path::PathBuf;

use crate::core::errors::{validate_ufo_path, BezyContext, BezyResult};
use crate::core::state::font_data::{FontData, PointData};
use crate::core::state::font_metrics::FontInfo;

/// The main application state - thread-safe for Bevy
#[derive(Resource, Default, Clone)]
pub struct AppState {
    /// The current font editing workspace
    pub workspace: Workspace,
}

/// Represents a font editing session with thread-safe data
#[derive(Clone, Default)]
pub struct Workspace {
    /// Thread-safe font data extracted from norad
    pub font: FontData,
    /// Information about the font (name, metrics, etc.)
    pub info: FontInfo,
    /// The currently selected glyph (if any)
    pub selected: Option<String>,
}

impl AppState {
    /// Load a font from a UFO file path
    ///
    /// This method loads a UFO font file and converts it into our optimized
    /// internal representation for real-time editing.
    pub fn load_font_from_path(&mut self, path: PathBuf) -> BezyResult<()> {
        // Validate the UFO path
        validate_ufo_path(&path)?;

        // Load the font using norad
        let font = norad::Font::load(&path).with_file_context("load", &path)?;

        // Extract data into our thread-safe structures
        self.workspace.font = FontData::from_norad_font(&font, Some(path));
        self.workspace.info = FontInfo::from_norad_font(&font);

        debug!(
            "Successfully loaded UFO font with {} glyphs",
            self.workspace.font.glyphs.len()
        );
        Ok(())
    }

    /// Save the current font to its file path
    ///
    /// This method converts our internal representation back to UFO format
    /// and saves it to the file path that was used to load the font.
    pub fn save_font(&self) -> BezyResult<()> {
        let path = self
            .workspace
            .font
            .path
            .as_ref()
            .context("No file path set - use Save As first")?;

        // Convert our internal data back to norad and save
        let norad_font = self.workspace.font.to_norad_font(&self.workspace.info);
        norad_font.save(path).with_file_context("save", path)?;

        debug!("Saved font to {:?}", path);
        Ok(())
    }

    /// Save the font to a new path
    ///
    /// This method saves the font to a new location and updates the internal
    /// path reference for future save operations.
    pub fn save_font_as(&mut self, path: PathBuf) -> BezyResult<()> {
        // Convert our internal data back to norad and save
        let norad_font = self.workspace.font.to_norad_font(&self.workspace.info);
        norad_font.save(&path).with_file_context("save", &path)?;

        // Update our stored path
        self.workspace.font.path = Some(path.clone());

        debug!("Saved font to new location: {:?}", path);
        Ok(())
    }

    /// Get a display name for the current font
    pub fn get_font_display_name(&self) -> String {
        self.workspace.get_font_display_name()
    }

    /// Get a mutable reference to a specific point in a glyph
    pub fn get_point_mut(
        &mut self,
        glyph_name: &str,
        contour_idx: usize,
        point_idx: usize,
    ) -> Option<&mut PointData> {
        self.workspace
            .font
            .glyphs
            .get_mut(glyph_name)?
            .outline
            .as_mut()?
            .contours
            .get_mut(contour_idx)?
            .points
            .get_mut(point_idx)
    }


    /// Set the position of a point
    pub fn set_point_position(
        &mut self,
        glyph_name: &str,
        contour_idx: usize,
        point_idx: usize,
        x: f64,
        y: f64,
    ) -> bool {
        if let Some(point) = self.get_point_mut(glyph_name, contour_idx, point_idx) {
            point.x = x;
            point.y = y;
            true
        } else {
            false
        }
    }

}

impl Workspace {
    /// Get a display name for the font
    pub fn get_font_display_name(&self) -> String {
        self.get_font_name()
    }

    /// Get a display name combining family and style names  
    pub fn get_font_name(&self) -> String {
        let parts: Vec<&str> = [&self.info.family_name, &self.info.style_name]
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.as_str())
            .collect();

        if parts.is_empty() {
            "Untitled Font".to_string()
        } else {
            parts.join(" ")
        }
    }
}
