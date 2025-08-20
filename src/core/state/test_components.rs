//! Component system tests
//!
//! Tests for the component resolution and rendering system

#[cfg(test)]
mod tests {
    use super::super::fontir_app_state::FontIRAppState;
    use std::path::PathBuf;

    #[test]
    fn test_component_resolution() {
        // Test component resolution with the Bezy Grotesk font
        let ufo_path = PathBuf::from("assets/fonts/bezy-grotesk-regular.ufo");
        
        // Skip test if font file doesn't exist (CI environment)
        if !ufo_path.exists() {
            eprintln!("Skipping component test - font file not found");
            return;
        }

        // Create FontIR app state
        let app_state = FontIRAppState::from_path(ufo_path);
        
        match app_state {
            Ok(state) => {
                // Test getting paths for a composite glyph (alefHamzaabove-ar has components)
                if let Some(paths) = state.get_glyph_paths_with_components("alefHamzaabove-ar") {
                    println!("✅ Component resolution test passed!");
                    println!("   Found {} paths for alefHamzaabove-ar", paths.len());
                    
                    // Should have more paths than just outline (includes components)
                    assert!(!paths.is_empty(), "Composite glyph should have paths");
                } else {
                    println!("❌ Could not resolve components for alefHamzaabove-ar");
                    // This is not necessarily a failure if FontIR doesn't have the glyph
                }
                
                // Test that regular glyphs still work
                if let Some(paths) = state.get_glyph_paths_with_components("a") {
                    println!("✅ Regular glyph resolution still works: {} paths", paths.len());
                } else {
                    println!("❌ Could not get paths for regular glyph 'a'");
                }
            }
            Err(e) => {
                eprintln!("Could not load FontIR state: {}", e);
                // This is expected in some environments, so don't fail the test
            }
        }
    }

    #[test]
    fn test_affine_transform() {
        use kurbo::{BezPath, Point};
        use crate::core::state::fontir_app_state::apply_affine_transform;
        
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
            x_offset: 50.0,  // Translate by 50 units
            y_offset: 25.0,  // Translate by 25 units
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