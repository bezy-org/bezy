//! BezPath editing utilities
//!
//! This module provides utilities for editing kurbo::BezPath structures
//! in a way that's compatible with font editing operations.

use bevy::prelude::*;
use kurbo::{BezPath, PathEl, Point, Vec2};

/// A reference to a specific point in a BezPath
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct PathPointRef {
    /// Which path element this point belongs to
    pub element_index: usize,
    /// Which point within the element (0 = main, 1 = control1, 2 = control2)
    pub point_index: usize,
}

/// Represents an editable point extracted from a BezPath
#[derive(Debug, Clone)]
pub struct EditablePoint {
    pub position: Point,
    pub point_type: PathPointType,
    pub reference: PathPointRef,
}

/// Type of point in a path
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum PathPointType {
    OnCurve,  // Move, Line, or end point of Curve/Quad
    OffCurve, // Control point
}

/// Extract all editable points from a BezPath
pub fn extract_editable_points(path: &BezPath) -> Vec<EditablePoint> {
    let mut points = Vec::new();

    for (elem_idx, element) in path.elements().iter().enumerate() {
        match element {
            PathEl::MoveTo(pt) => {
                points.push(EditablePoint {
                    position: *pt,
                    point_type: PathPointType::OnCurve,
                    reference: PathPointRef {
                        element_index: elem_idx,
                        point_index: 0,
                    },
                });
            }
            PathEl::LineTo(pt) => {
                points.push(EditablePoint {
                    position: *pt,
                    point_type: PathPointType::OnCurve,
                    reference: PathPointRef {
                        element_index: elem_idx,
                        point_index: 0,
                    },
                });
            }
            PathEl::CurveTo(c1, c2, pt) => {
                // Control points
                points.push(EditablePoint {
                    position: *c1,
                    point_type: PathPointType::OffCurve,
                    reference: PathPointRef {
                        element_index: elem_idx,
                        point_index: 0,
                    },
                });
                points.push(EditablePoint {
                    position: *c2,
                    point_type: PathPointType::OffCurve,
                    reference: PathPointRef {
                        element_index: elem_idx,
                        point_index: 1,
                    },
                });
                // End point
                points.push(EditablePoint {
                    position: *pt,
                    point_type: PathPointType::OnCurve,
                    reference: PathPointRef {
                        element_index: elem_idx,
                        point_index: 2,
                    },
                });
            }
            PathEl::QuadTo(c, pt) => {
                // Control point
                points.push(EditablePoint {
                    position: *c,
                    point_type: PathPointType::OffCurve,
                    reference: PathPointRef {
                        element_index: elem_idx,
                        point_index: 0,
                    },
                });
                // End point
                points.push(EditablePoint {
                    position: *pt,
                    point_type: PathPointType::OnCurve,
                    reference: PathPointRef {
                        element_index: elem_idx,
                        point_index: 1,
                    },
                });
            }
            PathEl::ClosePath => {
                // No points to extract
            }
        }
    }

    points
}

/// Update a specific point in a BezPath
pub fn update_path_point(
    path: &mut BezPath,
    reference: PathPointRef,
    new_position: Point,
) -> Result<(), String> {
    let mut elements: Vec<PathEl> = path.elements().to_vec();

    if reference.element_index >= elements.len() {
        return Err("Element index out of bounds".to_string());
    }

    let old_element = &elements[reference.element_index];
    let new_element = match (old_element, reference.point_index) {
        (PathEl::MoveTo(_), 0) => PathEl::MoveTo(new_position),
        (PathEl::LineTo(_), 0) => PathEl::LineTo(new_position),
        (PathEl::CurveTo(_, c2, pt), 0) => {
            PathEl::CurveTo(new_position, *c2, *pt)
        }
        (PathEl::CurveTo(c1, _, pt), 1) => {
            PathEl::CurveTo(*c1, new_position, *pt)
        }
        (PathEl::CurveTo(c1, c2, _), 2) => {
            PathEl::CurveTo(*c1, *c2, new_position)
        }
        (PathEl::QuadTo(_, pt), 0) => PathEl::QuadTo(new_position, *pt),
        (PathEl::QuadTo(c, _), 1) => PathEl::QuadTo(*c, new_position),
        _ => return Err("Invalid point index for element type".to_string()),
    };

    elements[reference.element_index] = new_element;

    // Rebuild the path
    *path = BezPath::from_vec(elements);
    Ok(())
}

/// Move a point by a delta
pub fn nudge_path_point(
    path: &mut BezPath,
    reference: PathPointRef,
    delta: Vec2,
) -> Result<(), String> {
    let points = extract_editable_points(path);

    // Find the point
    let point = points
        .iter()
        .find(|p| p.reference == reference)
        .ok_or("Point not found")?;

    let new_position = point.position + delta;
    update_path_point(path, reference, new_position)
}

/// Find the nearest point to a given position
pub fn find_nearest_point(
    path: &BezPath,
    position: Point,
    max_distance: f64,
) -> Option<PathPointRef> {
    let points = extract_editable_points(path);

    let mut nearest = None;
    let mut min_dist = max_distance;

    for point in points {
        let dist = (point.position - position).hypot();
        if dist < min_dist {
            min_dist = dist;
            nearest = Some(point.reference);
        }
    }

    nearest
}

/// Convert multiple BezPaths to a single path with multiple contours
pub fn paths_to_multi_contour(paths: &[BezPath]) -> BezPath {
    let mut result = BezPath::new();

    for path in paths {
        for element in path.elements() {
            result.push(*element);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_points() {
        let mut path = BezPath::new();
        path.move_to(Point::new(0.0, 0.0));
        path.line_to(Point::new(100.0, 0.0));
        path.curve_to(
            Point::new(100.0, 50.0),
            Point::new(50.0, 100.0),
            Point::new(0.0, 100.0),
        );

        let points = extract_editable_points(&path);
        assert_eq!(points.len(), 5); // move + line + 2 controls + curve end
    }

    #[test]
    fn test_update_point() {
        let mut path = BezPath::new();
        path.move_to(Point::new(0.0, 0.0));
        path.line_to(Point::new(100.0, 0.0));

        let reference = PathPointRef {
            element_index: 1,
            point_index: 0,
        };

        update_path_point(&mut path, reference, Point::new(200.0, 50.0))
            .unwrap();

        let points = extract_editable_points(&path);
        assert_eq!(points[1].position, Point::new(200.0, 50.0));
    }
}
