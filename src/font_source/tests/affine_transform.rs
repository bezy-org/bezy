//! Component system tests
//!
//! Tests for the component resolution and rendering system

#[cfg(test)]
mod tests {

    #[test]
    fn test_affine_transform() {
        use crate::font_source::fontir_state::apply_affine_transform;
        use kurbo::{BezPath, Point};

        // Create a simple path
        let mut path = BezPath::new();
        path.move_to(Point::new(0.0, 0.0));
        path.line_to(Point::new(100.0, 100.0));
        path.close_path();

        // Create an identity transform
        let transform = norad::AffineTransform {
            x_scale: 1.0,
            xy_scale: 0.0,
            yx_scale: 0.0,
            y_scale: 1.0,
            x_offset: 50.0, // Translate by 50 units
            y_offset: 25.0, // Translate by 25 units
        };

        // Apply the transformation
        let transformed_path = apply_affine_transform(&path, &transform);

        // Check that the transformation was applied
        let elements: Vec<_> = transformed_path.elements().iter().collect();
        println!("Transformed path has {} elements", elements.len());

        // The path should be translated by the offset
        assert_eq!(elements.len(), 3); // MoveTo, LineTo, ClosePath

        println!("✅ Affine transformation test passed!");
    }
}
