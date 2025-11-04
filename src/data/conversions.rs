//! UFO format conversion utilities
//!
//! This module contains conversion logic between our internal thread-safe
//! data structures and the norad UFO format. This is pure data transformation
//! logic - serialization and deserialization between equivalent representations.

use crate::core::state::{
    ComponentData, ContourData, FontData, FontInfo, GlyphData, OutlineData, PointData,
    PointTypeData,
};
use kurbo::{BezPath, Point};
use norad::Font;
use std::path::PathBuf;

impl GlyphData {
    /// Convert from norad glyph to our thread-safe version
    pub fn from_norad_glyph(norad_glyph: &norad::Glyph) -> Self {
        let outline = if !norad_glyph.contours.is_empty() {
            Some(OutlineData::from_norad_contours(&norad_glyph.contours))
        } else {
            None
        };

        // Convert components from norad format
        let components = norad_glyph
            .components
            .iter()
            .map(ComponentData::from_norad_component)
            .collect();

        Self {
            name: norad_glyph.name().to_string(),
            advance_width: norad_glyph.width,
            advance_height: Some(norad_glyph.height),
            unicode_values: norad_glyph.codepoints.iter().collect(),
            outline,
            components,
        }
    }

    /// Convert back to norad glyph
    pub fn to_norad_glyph(&self) -> norad::Glyph {
        let mut glyph = norad::Glyph::new(&self.name);
        glyph.width = self.advance_width;
        glyph.height = self.advance_height.unwrap_or(0.0);

        // Convert Vec<char> to Codepoints
        for &codepoint in &self.unicode_values {
            glyph.codepoints.insert(codepoint);
        }

        if let Some(outline_data) = &self.outline {
            glyph.contours = outline_data.to_norad_contours();
        }

        // Convert components back to norad format
        glyph.components = self
            .components
            .iter()
            .map(ComponentData::to_norad_component)
            .collect();

        glyph
    }
}

impl ComponentData {
    /// Convert from norad component to our thread-safe version
    pub fn from_norad_component(norad_component: &norad::Component) -> Self {
        Self {
            base_glyph: norad_component.base.to_string(),
            transform: [
                norad_component.transform.x_scale,
                norad_component.transform.xy_scale,
                norad_component.transform.yx_scale,
                norad_component.transform.y_scale,
                norad_component.transform.x_offset,
                norad_component.transform.y_offset,
            ],
        }
    }

    /// Convert back to norad component
    pub fn to_norad_component(&self) -> norad::Component {
        let base_name: norad::Name = self
            .base_glyph
            .parse()
            .unwrap_or_else(|_| "a".parse().expect("'a' should always be a valid glyph name")); // Fallback to 'a' if invalid name

        let transform = norad::AffineTransform {
            x_scale: self.transform[0],
            xy_scale: self.transform[1],
            yx_scale: self.transform[2],
            y_scale: self.transform[3],
            x_offset: self.transform[4],
            y_offset: self.transform[5],
        };

        norad::Component::new(base_name, transform, None)
    }
}

impl OutlineData {
    pub fn from_norad_contours(norad_contours: &[norad::Contour]) -> Self {
        let contours = norad_contours
            .iter()
            .map(ContourData::from_norad_contour)
            .collect();

        Self { contours }
    }

    pub fn to_norad_contours(&self) -> Vec<norad::Contour> {
        self.contours
            .iter()
            .map(ContourData::to_norad_contour)
            .collect()
    }

    pub fn to_bezpaths(&self) -> Vec<BezPath> {
        self.contours
            .iter()
            .map(|contour| contour.to_bezpath())
            .collect()
    }
}

impl ContourData {
    pub fn from_norad_contour(norad_contour: &norad::Contour) -> Self {
        let points = norad_contour
            .points
            .iter()
            .map(PointData::from_norad_point)
            .collect();

        Self { points }
    }

    pub fn to_norad_contour(&self) -> norad::Contour {
        let points = self.points.iter().map(PointData::to_norad_point).collect();

        // Use constructor with required arguments
        norad::Contour::new(points, None)
    }

    pub fn to_bezpath(&self) -> BezPath {
        let mut path = BezPath::new();
        let mut pending_offcurves: Vec<Point> = Vec::new();
        let mut first_point: Option<(Point, PointTypeData)> = None;

        for (idx, point) in self.points.iter().enumerate() {
            let pt = Point::new(point.x, point.y);

            // UFO contours: first point defines start position
            if idx == 0 {
                path.move_to(pt);
                first_point = Some((pt, point.point_type));
                continue;
            }

            match point.point_type {
                PointTypeData::Move => {
                    path.move_to(pt);
                }
                PointTypeData::Line => {
                    path.line_to(pt);
                }
                PointTypeData::OffCurve => {
                    pending_offcurves.push(pt);
                }
                PointTypeData::Curve => {
                    if pending_offcurves.len() >= 2 {
                        let cp1 = pending_offcurves[pending_offcurves.len() - 2];
                        let cp2 = pending_offcurves[pending_offcurves.len() - 1];
                        path.curve_to(cp1, cp2, pt);
                        pending_offcurves.clear();
                    } else if pending_offcurves.len() == 1 {
                        let cp = pending_offcurves[0];
                        path.quad_to(cp, pt);
                        pending_offcurves.clear();
                    } else {
                        path.line_to(pt);
                    }
                }
                PointTypeData::QCurve => {
                    if !pending_offcurves.is_empty() {
                        if pending_offcurves.len() == 1 {
                            path.quad_to(pending_offcurves[0], pt);
                        } else {
                            for i in 0..pending_offcurves.len() {
                                let cp = pending_offcurves[i];
                                let end = if i == pending_offcurves.len() - 1 {
                                    pt
                                } else {
                                    let next_cp = pending_offcurves[i + 1];
                                    Point::new(
                                        (cp.x + next_cp.x) / 2.0,
                                        (cp.y + next_cp.y) / 2.0,
                                    )
                                };
                                path.quad_to(cp, end);
                            }
                        }
                        pending_offcurves.clear();
                    } else {
                        path.line_to(pt);
                    }
                }
            }
        }

        // Handle wrap-around segment back to first point
        // The first point's type defines how to reach it from the last point
        if let Some((first_pt, first_type)) = first_point {
            match first_type {
                PointTypeData::Curve => {
                    if pending_offcurves.len() >= 2 {
                        let cp1 = pending_offcurves[pending_offcurves.len() - 2];
                        let cp2 = pending_offcurves[pending_offcurves.len() - 1];
                        path.curve_to(cp1, cp2, first_pt);
                        pending_offcurves.clear();
                    } else if pending_offcurves.len() == 1 {
                        let cp = pending_offcurves[0];
                        path.quad_to(cp, first_pt);
                        pending_offcurves.clear();
                    } else {
                        path.line_to(first_pt);
                    }
                }
                PointTypeData::QCurve => {
                    if !pending_offcurves.is_empty() {
                        if pending_offcurves.len() == 1 {
                            path.quad_to(pending_offcurves[0], first_pt);
                        } else {
                            for i in 0..pending_offcurves.len() {
                                let cp = pending_offcurves[i];
                                let end = if i == pending_offcurves.len() - 1 {
                                    first_pt
                                } else {
                                    let next_cp = pending_offcurves[i + 1];
                                    Point::new(
                                        (cp.x + next_cp.x) / 2.0,
                                        (cp.y + next_cp.y) / 2.0,
                                    )
                                };
                                path.quad_to(cp, end);
                            }
                        }
                        pending_offcurves.clear();
                    } else {
                        path.line_to(first_pt);
                    }
                }
                PointTypeData::Line => {
                    path.line_to(first_pt);
                }
                PointTypeData::Move => {
                    // Move doesn't draw, just repositions
                }
                PointTypeData::OffCurve => {
                    // First point shouldn't be off-curve, but if it is, draw line
                    path.line_to(first_pt);
                }
            }
        }

        path.close_path();
        path
    }
}

impl PointData {
    pub fn from_norad_point(norad_point: &norad::ContourPoint) -> Self {
        Self {
            x: norad_point.x,
            y: norad_point.y,
            point_type: PointTypeData::from_norad_point_type(&norad_point.typ),
        }
    }

    pub fn to_norad_point(&self) -> norad::ContourPoint {
        // Use constructor with all 6 required arguments
        norad::ContourPoint::new(
            self.x, // f64 is expected
            self.y, // f64 is expected
            self.point_type.to_norad_point_type(),
            false, // smooth
            None,  // name
            None,  // identifier
        )
    }
}

impl PointTypeData {
    pub fn from_norad_point_type(norad_type: &norad::PointType) -> Self {
        match norad_type {
            norad::PointType::Move => PointTypeData::Move,
            norad::PointType::Line => PointTypeData::Line,
            norad::PointType::OffCurve => PointTypeData::OffCurve,
            norad::PointType::Curve => PointTypeData::Curve,
            norad::PointType::QCurve => PointTypeData::QCurve,
        }
    }

    pub fn to_norad_point_type(&self) -> norad::PointType {
        match self {
            PointTypeData::Move => norad::PointType::Move,
            PointTypeData::Line => norad::PointType::Line,
            PointTypeData::OffCurve => norad::PointType::OffCurve,
            PointTypeData::Curve => norad::PointType::Curve,
            PointTypeData::QCurve => norad::PointType::QCurve,
        }
    }
}

impl FontData {
    /// Extract font data from norad Font
    pub fn from_norad_font(font: &Font, path: Option<PathBuf>) -> Self {
        let mut glyphs = std::collections::HashMap::new();

        // Extract all glyphs from the default layer
        let layer = font.default_layer();

        // Iterate over glyphs in the layer
        for glyph in layer.iter() {
            let glyph_data = GlyphData::from_norad_glyph(glyph);
            glyphs.insert(glyph.name().to_string(), glyph_data);
        }

        Self { glyphs, path }
    }

    /// Convert back to a complete norad Font
    pub fn to_norad_font(&self, info: &FontInfo) -> Font {
        let mut font = Font::new();

        // Set font info using our conversion method
        font.font_info = info.to_norad_font_info();

        // Add glyphs to the default layer
        let layer = font.default_layer_mut();
        for glyph_data in self.glyphs.values() {
            let glyph = glyph_data.to_norad_glyph();
            layer.insert_glyph(glyph);
        }

        font
    }
}
