//! FontIR adapter for Bevy compatibility
//!
//! This module provides a bridge between fontir's data structures and
//! Bevy's requirements for thread-safe resources. It wraps FontIR types
//! to make them Send + Sync and provides conversion utilities.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use bevy::prelude::*;
use kurbo::BezPath;

use crate::core::state::{
    ContourData, FontData, OutlineData, PointData, PointTypeData,
};

/// Thread-safe wrapper around FontIR data
///
/// This wrapper makes FontIR data compatible with Bevy's ECS system
/// by ensuring it implements Send + Sync.
#[derive(Resource, Clone)]
pub struct FontIRData {
    /// Font file path
    pub path: Option<PathBuf>,
    /// Placeholder for FontIR integration
    #[allow(dead_code)]
    placeholder: String,
}

impl FontIRData {
    /// Create a new FontIRData placeholder
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            placeholder: "FontIR integration in progress".to_string(),
        }
    }

    /// Convert FontIR data to current Bezy format for backward compatibility
    pub fn to_bezy_font_data(&self) -> FontData {
        // For now, return empty font data as a placeholder
        // TODO: Implement actual FontIR to Bezy conversion
        FontData {
            glyphs: HashMap::new(),
            path: self.path.clone(),
        }
    }

    /// Convert kurbo::BezPath to our outline format
    /// This is a utility function for future implementation
    #[allow(dead_code)]
    fn convert_bezpath_to_outline(bez_path: &BezPath) -> OutlineData {
        let mut contours = Vec::new();
        let mut current_contour = Vec::new();

        // Convert BezPath elements to our point format
        for element in bez_path.elements() {
            match element {
                kurbo::PathEl::MoveTo(pt) => {
                    // Start new contour
                    if !current_contour.is_empty() {
                        contours.push(ContourData {
                            points: current_contour,
                        });
                        current_contour = Vec::new();
                    }
                    current_contour.push(PointData {
                        x: pt.x,
                        y: pt.y,
                        point_type: PointTypeData::Move,
                    });
                }
                kurbo::PathEl::LineTo(pt) => {
                    current_contour.push(PointData {
                        x: pt.x,
                        y: pt.y,
                        point_type: PointTypeData::Line,
                    });
                }
                kurbo::PathEl::CurveTo(p1, p2, p3) => {
                    // Add off-curve control points and the curve point
                    current_contour.push(PointData {
                        x: p1.x,
                        y: p1.y,
                        point_type: PointTypeData::OffCurve,
                    });
                    current_contour.push(PointData {
                        x: p2.x,
                        y: p2.y,
                        point_type: PointTypeData::OffCurve,
                    });
                    current_contour.push(PointData {
                        x: p3.x,
                        y: p3.y,
                        point_type: PointTypeData::Curve,
                    });
                }
                kurbo::PathEl::QuadTo(p1, p2) => {
                    // Add off-curve control point and quadratic curve point
                    current_contour.push(PointData {
                        x: p1.x,
                        y: p1.y,
                        point_type: PointTypeData::OffCurve,
                    });
                    current_contour.push(PointData {
                        x: p2.x,
                        y: p2.y,
                        point_type: PointTypeData::QCurve,
                    });
                }
                kurbo::PathEl::ClosePath => {
                    // Close current contour - no additional point needed
                }
            }
        }

        // Add final contour if it exists
        if !current_contour.is_empty() {
            contours.push(ContourData {
                points: current_contour,
            });
        }

        OutlineData { contours }
    }
}

/// Load a designspace or UFO font using FontIR
pub fn load_font_with_fontir(path: PathBuf) -> Result<FontIRData> {
    use fontir::source::Source;
    use ufo2fontir::source::DesignSpaceIrSource;

    // Try to load as designspace first, then fallback to UFO
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    match extension {
        "designspace" => {
            info!("Loading designspace file: {:?}", path);
            let _ir_source = DesignSpaceIrSource::new(&path)?;
            // TODO: Extract FontIR data from source
            Ok(FontIRData::new(Some(path)))
        }
        "ufo" => {
            info!("Loading single UFO file: {:?}", path);
            // For single UFO, we'd need to create a minimal designspace
            // TODO: Implement single UFO loading
            Ok(FontIRData::new(Some(path)))
        }
        _ => {
            anyhow::bail!("Unsupported file format: {}", extension);
        }
    }
}

/// Test compatibility with Bevy ECS
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fontir_data_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FontIRData>();
    }

    #[test]
    fn test_fontir_data_bevy_resource() {
        fn assert_resource<T: Resource>() {}
        assert_resource::<FontIRData>();
    }
}
