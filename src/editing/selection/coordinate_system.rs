use crate::geometry::design_space::DPoint;
use bevy::prelude::*;

/// Centralized coordinate system for selection operations
/// This module provides a single source of truth for all coordinate transformations
/// to prevent bugs like coordinate system mismatches.
pub struct SelectionCoordinateSystem;

impl SelectionCoordinateSystem {
    /// Convert a design space point to the coordinate system used by selectable entities
    /// This is the ONLY place where this conversion should happen
    ///
    /// The entities are positioned at their absolute sort positions in the text buffer,
    /// so we need to convert design space coordinates to the same coordinate system.
    pub fn design_to_entity_coordinates(design_point: &DPoint) -> Vec2 {
        // The design space coordinates are already in the same coordinate system as entities
        // Both use the same world coordinate system, so no conversion is needed
        design_point.to_raw()
    }

    /// Convert entity coordinates back to design space
    pub fn entity_to_design_coordinates(entity_point: &Vec2) -> DPoint {
        // The entity coordinates are already in design space, so no conversion is needed
        DPoint::from_raw(*entity_point)
    }

    /// Check if a point in entity coordinates is inside a rectangle in design coordinates
    /// Both coordinates are in the same world coordinate system, so we can compare directly
    pub fn is_point_in_rectangle(point: &Vec2, rect_start: &DPoint, rect_end: &DPoint) -> bool {
        // Convert design space coordinates to Vec2 for comparison
        let rect_start_vec = rect_start.to_raw();
        let rect_end_vec = rect_end.to_raw();

        // Create a rectangle from the corners
        let rect = Rect::from_corners(rect_start_vec, rect_end_vec);

        // Check if the point is inside the rectangle
        rect.contains(*point)
    }

    /// Get the coordinate ranges for debugging
    pub fn debug_coordinate_ranges(
        entities: &[(Entity, Vec2)],
        rect_start: &DPoint,
        rect_end: &DPoint,
    ) -> String {
        if entities.is_empty() {
            return "No entities to check".to_string();
        }

        let min_x = entities
            .iter()
            .map(|(_, pos)| pos.x)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max_x = entities
            .iter()
            .map(|(_, pos)| pos.x)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let min_y = entities
            .iter()
            .map(|(_, pos)| pos.y)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max_y = entities
            .iter()
            .map(|(_, pos)| pos.y)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let rect_entity_start = Self::design_to_entity_coordinates(rect_start);
        let rect_entity_end = Self::design_to_entity_coordinates(rect_end);

        format!(
            "Entity ranges: X({:.1} to {:.1}), Y({:.1} to {:.1}) | Rect entity coords: start({:.1}, {:.1}), end({:.1}, {:.1})",
            min_x, max_x, min_y, max_y,
            rect_entity_start.x, rect_entity_start.y,
            rect_entity_end.x, rect_entity_end.y
        )
    }
}
