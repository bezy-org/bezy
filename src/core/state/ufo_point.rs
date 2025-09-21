//! UFO-compliant point data structures
//!
//! This module provides data structures that fully support the UFO specification
//! for point data, including all optional attributes and metadata.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// UFO-compliant point type enumeration
/// Maps directly to the UFO specification point types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum UfoPointType {
    /// First point in an open contour
    #[serde(rename = "move")]
    Move,
    /// Draws straight line from previous point
    #[serde(rename = "line")]
    Line,
    /// Part of curve segment (control point)
    #[serde(rename = "offcurve")]
    OffCurve,
    /// Draws cubic Bézier curve
    #[serde(rename = "curve")]
    Curve,
    /// Draws quadratic curve
    #[serde(rename = "qcurve")]
    QCurve,
}

impl UfoPointType {
    /// Check if this point type is on-curve (not a control point)
    pub fn is_on_curve(&self) -> bool {
        !matches!(self, UfoPointType::OffCurve)
    }

    /// Check if this point type can have the smooth attribute
    pub fn can_be_smooth(&self) -> bool {
        self.is_on_curve()
    }
}

/// Full UFO-compliant point data structure
/// Supports all UFO specification attributes and metadata
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Reflect)]
pub struct UfoPoint {
    /// X coordinate (required)
    pub x: f64,
    /// Y coordinate (required)
    pub y: f64,
    /// Point/segment type (optional in UFO, but we make it required for consistency)
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub point_type: Option<UfoPointType>,
    /// Smooth curve flag - only valid for on-curve points
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smooth: Option<bool>,
    /// Arbitrary text label for the point
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Unique identifier within the glyph
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    /// Custom metadata storage (UFO point libraries)
    /// Note: Simplified for Bevy compatibility - use String for now
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lib: Option<String>,
}

impl UfoPoint {
    /// Create a new UFO point with coordinates and type
    pub fn new(x: f64, y: f64, point_type: UfoPointType) -> Self {
        Self {
            x,
            y,
            point_type: Some(point_type),
            smooth: None,
            name: None,
            identifier: None,
            lib: None,
        }
    }

    /// Create a move point (first point in contour)
    pub fn move_to(x: f64, y: f64) -> Self {
        Self::new(x, y, UfoPointType::Move)
    }

    /// Create a line point
    pub fn line_to(x: f64, y: f64) -> Self {
        Self::new(x, y, UfoPointType::Line)
    }

    /// Create an off-curve control point
    pub fn off_curve(x: f64, y: f64) -> Self {
        Self::new(x, y, UfoPointType::OffCurve)
    }

    /// Create a cubic curve point
    pub fn curve_to(x: f64, y: f64) -> Self {
        Self::new(x, y, UfoPointType::Curve)
    }

    /// Create a quadratic curve point
    pub fn qcurve_to(x: f64, y: f64) -> Self {
        Self::new(x, y, UfoPointType::QCurve)
    }

    /// Set the smooth flag (only valid for on-curve points)
    pub fn with_smooth(mut self, smooth: bool) -> Self {
        if self.point_type.map_or(false, |t| t.can_be_smooth()) {
            self.smooth = Some(smooth);
        }
        self
    }

    /// Set the point name
    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the point identifier
    pub fn with_identifier<S: Into<String>>(mut self, identifier: S) -> Self {
        self.identifier = Some(identifier.into());
        self
    }

    /// Add custom metadata (simplified for Bevy compatibility)
    pub fn with_lib_data<S: Into<String>>(mut self, data: S) -> Self {
        self.lib = Some(data.into());
        self
    }

    /// Check if this point is on-curve
    pub fn is_on_curve(&self) -> bool {
        self.point_type.map_or(true, |t| t.is_on_curve())
    }

    /// Check if this point is smooth
    pub fn is_smooth(&self) -> bool {
        self.smooth.unwrap_or(false) && self.is_on_curve()
    }

    /// Validate point according to UFO constraints
    pub fn validate(&self) -> Result<(), String> {
        // Check smooth flag is only used on on-curve points
        if self.smooth.is_some() && !self.is_on_curve() {
            return Err("Smooth attribute can only be used on on-curve points".to_string());
        }

        // Additional UFO validation rules could be added here
        Ok(())
    }
}

/// Conversion traits for compatibility with existing systems

impl From<crate::core::state::font_data::PointData> for UfoPoint {
    fn from(point: crate::core::state::font_data::PointData) -> Self {
        let point_type = match point.point_type {
            crate::core::state::font_data::PointTypeData::Move => UfoPointType::Move,
            crate::core::state::font_data::PointTypeData::Line => UfoPointType::Line,
            crate::core::state::font_data::PointTypeData::OffCurve => UfoPointType::OffCurve,
            crate::core::state::font_data::PointTypeData::Curve => UfoPointType::Curve,
            crate::core::state::font_data::PointTypeData::QCurve => UfoPointType::QCurve,
        };

        Self::new(point.x, point.y, point_type)
    }
}

impl From<UfoPoint> for crate::core::state::font_data::PointData {
    fn from(ufo_point: UfoPoint) -> Self {
        let point_type = match ufo_point.point_type.unwrap_or(UfoPointType::Line) {
            UfoPointType::Move => crate::core::state::font_data::PointTypeData::Move,
            UfoPointType::Line => crate::core::state::font_data::PointTypeData::Line,
            UfoPointType::OffCurve => crate::core::state::font_data::PointTypeData::OffCurve,
            UfoPointType::Curve => crate::core::state::font_data::PointTypeData::Curve,
            UfoPointType::QCurve => crate::core::state::font_data::PointTypeData::QCurve,
        };

        Self {
            x: ufo_point.x,
            y: ufo_point.y,
            point_type,
        }
    }
}

/// Enhanced ECS component for UFO-compliant point data
#[derive(bevy::prelude::Component, Debug, Clone)]
pub struct UfoPointComponent {
    /// Full UFO point data
    pub point: UfoPoint,
    /// Cached on-curve status for performance
    pub is_on_curve: bool,
}

impl UfoPointComponent {
    /// Create a new UFO point component
    pub fn new(point: UfoPoint) -> Self {
        let is_on_curve = point.is_on_curve();
        Self { point, is_on_curve }
    }

    /// Update the point data and refresh cache
    pub fn update_point(&mut self, point: UfoPoint) {
        self.is_on_curve = point.is_on_curve();
        self.point = point;
    }
}

impl From<UfoPoint> for UfoPointComponent {
    fn from(point: UfoPoint) -> Self {
        Self::new(point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ufo_point_creation() {
        let point = UfoPoint::new(100.0, 200.0, UfoPointType::Line);
        assert_eq!(point.x, 100.0);
        assert_eq!(point.y, 200.0);
        assert_eq!(point.point_type, Some(UfoPointType::Line));
        assert!(point.is_on_curve());
    }

    #[test]
    fn test_smooth_flag_validation() {
        // On-curve point can be smooth
        let mut point = UfoPoint::line_to(100.0, 200.0).with_smooth(true);
        assert!(point.validate().is_ok());
        assert!(point.is_smooth());

        // Off-curve point cannot be smooth
        point.point_type = Some(UfoPointType::OffCurve);
        assert!(point.validate().is_err());
        assert!(!point.is_smooth());
    }

    #[test]
    fn test_builder_pattern() {
        let point = UfoPoint::curve_to(50.0, 75.0)
            .with_name("anchor_top")
            .with_identifier("point_001")
            .with_smooth(true)
            .with_lib_data("custom_metadata");

        assert_eq!(point.name, Some("anchor_top".to_string()));
        assert_eq!(point.identifier, Some("point_001".to_string()));
        assert_eq!(point.smooth, Some(true));
        assert_eq!(point.lib, Some("custom_metadata".to_string()));
    }
}
