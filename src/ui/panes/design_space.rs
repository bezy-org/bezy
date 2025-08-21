//! Design space plugin for UI panes
//!
//! This plugin provides the DesignSpacePlugin for UI functionality.
//! The actual coordinate types (DPoint, DVec2) have been moved to geometry::design_space.

use bevy::prelude::*;

/// Plugin for design space UI functionality
///
/// This plugin is kept for compatibility with existing UI code.
/// The coordinate types (DPoint, DVec2) are now available in geometry::design_space.
pub struct DesignSpacePlugin;

impl Plugin for DesignSpacePlugin {
    fn build(&self, _app: &mut App) {
        // The ViewPort resource is deprecated and no longer initialized here
        // Coordinate types are now available via geometry::design_space
    }
}
