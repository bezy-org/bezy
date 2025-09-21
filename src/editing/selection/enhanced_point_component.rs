//! Enhanced point component that supports full UFO specification
//!
//! This module provides an enhanced point component that can gradually replace
//! the current simplified PointType component while maintaining backward compatibility.

use crate::core::state::ufo_point::{UfoPoint, UfoPointType};
use bevy::prelude::*;

/// Enhanced point component that fully supports UFO specification
/// This can gradually replace the current simplified PointType component
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct EnhancedPointType {
    /// Full UFO point data
    pub ufo_point: UfoPoint,
    /// Cached on-curve status for performance (avoids repeated computation)
    pub is_on_curve: bool,
    /// Legacy compatibility field
    pub legacy_type: crate::editing::selection::components::PointType,
}

impl EnhancedPointType {
    /// Create a new enhanced point type from UFO point data
    pub fn new(ufo_point: UfoPoint) -> Self {
        let is_on_curve = ufo_point.is_on_curve();
        let legacy_type = crate::editing::selection::components::PointType { is_on_curve };

        Self {
            ufo_point,
            is_on_curve,
            legacy_type,
        }
    }

    /// Create from coordinates and point type
    pub fn from_coords(x: f64, y: f64, point_type: UfoPointType) -> Self {
        Self::new(UfoPoint::new(x, y, point_type))
    }

    /// Create a move point
    pub fn move_to(x: f64, y: f64) -> Self {
        Self::new(UfoPoint::move_to(x, y))
    }

    /// Create a line point
    pub fn line_to(x: f64, y: f64) -> Self {
        Self::new(UfoPoint::line_to(x, y))
    }

    /// Create an off-curve control point
    pub fn off_curve(x: f64, y: f64) -> Self {
        Self::new(UfoPoint::off_curve(x, y))
    }

    /// Create a cubic curve point
    pub fn curve_to(x: f64, y: f64) -> Self {
        Self::new(UfoPoint::curve_to(x, y))
    }

    /// Create a quadratic curve point
    pub fn qcurve_to(x: f64, y: f64) -> Self {
        Self::new(UfoPoint::qcurve_to(x, y))
    }

    /// Set the smooth flag
    pub fn with_smooth(mut self, smooth: bool) -> Self {
        self.ufo_point = self.ufo_point.with_smooth(smooth);
        self
    }

    /// Set the point name
    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.ufo_point = self.ufo_point.with_name(name);
        self
    }

    /// Set the point identifier
    pub fn with_identifier<S: Into<String>>(mut self, identifier: S) -> Self {
        self.ufo_point = self.ufo_point.with_identifier(identifier);
        self
    }

    /// Update the UFO point data and refresh caches
    pub fn update_ufo_point(&mut self, ufo_point: UfoPoint) {
        self.is_on_curve = ufo_point.is_on_curve();
        self.legacy_type.is_on_curve = self.is_on_curve;
        self.ufo_point = ufo_point;
    }

    /// Get the point coordinates
    pub fn coords(&self) -> (f64, f64) {
        (self.ufo_point.x, self.ufo_point.y)
    }

    /// Set the point coordinates
    pub fn set_coords(&mut self, x: f64, y: f64) {
        self.ufo_point.x = x;
        self.ufo_point.y = y;
    }

    /// Check if the point is smooth
    pub fn is_smooth(&self) -> bool {
        self.ufo_point.is_smooth()
    }

    /// Get the point name if any
    pub fn name(&self) -> Option<&str> {
        self.ufo_point.name.as_deref()
    }

    /// Get the point identifier if any
    pub fn identifier(&self) -> Option<&str> {
        self.ufo_point.identifier.as_deref()
    }

    /// Get the UFO point type
    pub fn ufo_type(&self) -> Option<UfoPointType> {
        self.ufo_point.point_type
    }

    /// For backward compatibility with existing systems
    pub fn as_legacy(&self) -> &crate::editing::selection::components::PointType {
        &self.legacy_type
    }
}

/// Conversion from legacy PointType
impl From<crate::editing::selection::components::PointType> for EnhancedPointType {
    fn from(legacy: crate::editing::selection::components::PointType) -> Self {
        // We need coordinates to create a proper UFO point, so we'll use defaults
        // In a real migration, coordinates would come from Transform or other sources
        let point_type = if legacy.is_on_curve {
            UfoPointType::Line // Default to line for on-curve points
        } else {
            UfoPointType::OffCurve
        };

        Self::from_coords(0.0, 0.0, point_type)
    }
}

/// System to gradually migrate from PointType to EnhancedPointType
pub fn migrate_point_types(
    mut commands: Commands,
    legacy_points: Query<
        (
            Entity,
            &crate::editing::selection::components::PointType,
            &Transform,
        ),
        Without<EnhancedPointType>,
    >,
) {
    for (entity, legacy_type, transform) in legacy_points.iter() {
        let coords = transform.translation.truncate();
        let point_type = if legacy_type.is_on_curve {
            UfoPointType::Line
        } else {
            UfoPointType::OffCurve
        };

        let enhanced = EnhancedPointType::from_coords(coords.x as f64, coords.y as f64, point_type);

        // Add the enhanced component while keeping the legacy one for compatibility
        commands.entity(entity).insert(enhanced);

        debug!(
            "Migrated point entity {:?} to enhanced type: on_curve={}",
            entity, legacy_type.is_on_curve
        );
    }
}

/// Plugin to enable enhanced point type migration
pub struct EnhancedPointTypePlugin;

impl Plugin for EnhancedPointTypePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EnhancedPointType>().add_systems(
            Update,
            migrate_point_types.run_if(
                // Only run migration when we have legacy points without enhanced types
                |legacy_points: Query<
                    (),
                    (
                        With<crate::editing::selection::components::PointType>,
                        Without<EnhancedPointType>,
                    ),
                >| !legacy_points.is_empty(),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_point_creation() {
        let point = EnhancedPointType::curve_to(100.0, 200.0)
            .with_smooth(true)
            .with_name("anchor_top");

        assert_eq!(point.coords(), (100.0, 200.0));
        assert!(point.is_on_curve);
        assert!(point.is_smooth());
        assert_eq!(point.name(), Some("anchor_top"));
        assert_eq!(point.ufo_type(), Some(UfoPointType::Curve));
    }

    #[test]
    fn test_legacy_compatibility() {
        let enhanced = EnhancedPointType::off_curve(50.0, 75.0);
        let legacy = enhanced.as_legacy();

        assert!(!legacy.is_on_curve);
        assert!(!enhanced.is_on_curve);
    }

    #[test]
    fn test_coordinate_updates() {
        let mut point = EnhancedPointType::line_to(10.0, 20.0);
        assert_eq!(point.coords(), (10.0, 20.0));

        point.set_coords(30.0, 40.0);
        assert_eq!(point.coords(), (30.0, 40.0));
    }
}
