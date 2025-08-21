//! Startup layout configuration for the viewport
//!
//! This module handles the initial layout of sorts and camera positioning
//! when the application starts. This includes creating default sorts for
//! first-time users and positioning the camera for optimal initial view.
//!
//! Future: This will be expanded to create a grid of glyph sorts instead
//! of just a single default sort.

use crate::core::state::{FontIRAppState, TextEditorState};
use bevy::prelude::*;

/// Resource to trigger camera centering on the default sort
#[derive(Resource)]
pub struct CenterCameraOnDefaultSort {
    pub center_x: f32,
    pub center_y: f32,
    pub advance_width: f32,
}

/// System to create the initial viewport layout with default sorts
/// This creates a single default sort for now, but will be expanded
/// to create a grid of sorts in the future
pub fn create_startup_layout(
    fontir_state: Option<Res<FontIRAppState>>,
    mut text_editor_state: ResMut<TextEditorState>,
    mut commands: Commands,
    cli_args: Res<crate::core::cli::CliArgs>,
) {
    // Only create default layout if no sorts exist yet
    if !text_editor_state.buffer.is_empty() {
        return;
    }

    // Check if default buffer creation is disabled via CLI flag
    if cli_args.no_default_buffer {
        info!("Skipping default LTR buffer creation due to --no-default-buffer flag");
        info!("Ready for isolated text flow testing - use text tool to place sorts manually");
        return;
    }

    // Get the current glyph from FontIR state or use 'a' as fallback
    let glyph_name = if let Some(state) = &fontir_state {
        state.current_glyph.clone().unwrap_or_else(|| "a".to_string())
    } else {
        "a".to_string()
    };

    info!("Creating startup layout with default LTR text sort for glyph '{}'", glyph_name);

    // Get advance width from FontIR if available
    let advance_width = if let Some(state) = &fontir_state {
        state.get_glyph_advance_width(&glyph_name)
    } else {
        500.0 // Default fallback
    };

    // Create a default LTR text sort at the origin with cursor ready for typing
    // Future: This will be replaced with a grid of sorts
    create_default_sort_at_position(
        &mut text_editor_state,
        Vec2::ZERO,
        &glyph_name,
        advance_width,
    );

    // Calculate camera position to center on the default sort
    // TEMPORARY: Center camera on the visual center of the default glyph
    // TO REVERT: Simply comment out the camera centering resource creation below
    let visual_center_x = advance_width / 2.25;
    
    // For vertical centering, estimate the visual center of lowercase 'a'
    // MANUAL ADJUSTMENT: Change this value to move camera up/down
    let visual_center_y = 328.0;  // <-- ADJUST THIS VALUE: Higher = camera moves up
    
    // Insert a marker resource to trigger camera centering in the next frame
    commands.insert_resource(CenterCameraOnDefaultSort {
        center_x: visual_center_x,
        center_y: visual_center_y,
        advance_width,
    });
    
    info!("Startup layout created. Camera will center at ({}, {})", 
          visual_center_x, visual_center_y);
}

/// Helper function to create a single sort at a specific position
/// This is separated out to make it easy to create multiple sorts in a grid later
fn create_default_sort_at_position(
    text_editor_state: &mut TextEditorState,
    position: Vec2,
    glyph_name: &str,
    advance_width: f32,
) {
    use crate::core::state::text_editor::{SortEntry, SortKind, SortLayoutMode};
    use crate::core::state::text_editor::buffer::BufferId;
    
    // Create a new buffer ID for this LTR text flow
    let buffer_id = BufferId::new();
    
    let sort = SortEntry {
        kind: SortKind::Glyph {
            codepoint: Some('a'), // Default to 'a'
            glyph_name: glyph_name.to_string(),
            advance_width,
        },
        is_active: true, // Make it active and ready to edit
        layout_mode: SortLayoutMode::LTRText,  // LTR text mode for typing
        root_position: position,
        is_buffer_root: true, // This is a text root so cursor can appear
        buffer_cursor_position: Some(1), // Cursor after the first character
        buffer_id: Some(buffer_id), // Assign unique buffer ID for isolation
    };

    // Add to the text editor buffer
    let insert_index = text_editor_state.buffer.len();
    text_editor_state.buffer.insert(insert_index, sort);
    
    // Mark as changed using Bevy's change detection
    // The ResMut wrapper automatically tracks changes when we modify the resource
    
    info!("Created default sort '{}' at position ({:.1}, {:.1})", 
          glyph_name, position.x, position.y);
}

/// System to center the camera on the startup layout
/// This runs once after the startup layout is created
pub fn center_camera_on_startup_layout(
    mut commands: Commands,
    center_resource: Option<Res<CenterCameraOnDefaultSort>>,
    mut camera_query: Query<&mut Transform, With<crate::rendering::cameras::DesignCamera>>,
) {
    if let Some(center) = center_resource {
        // Center the camera on the visual center of the default sort
        for mut transform in camera_query.iter_mut() {
            transform.translation.x = center.center_x;
            transform.translation.y = center.center_y;
            
            info!("Centered camera on startup layout at ({}, {})", 
                  center.center_x, center.center_y);
        }
        
        // Remove the resource so this only happens once
        commands.remove_resource::<CenterCameraOnDefaultSort>();
    }
}

// Future expansion ideas for post v1.0:
//
// /// Configuration for the startup glyph grid
// pub struct GridConfig {
//     pub rows: usize,
//     pub cols: usize,
//     pub spacing: f32,
//     pub glyphs: Vec<String>,
// }
//
// /// Create a grid of glyph sorts at startup
// fn create_glyph_grid(
//     text_editor_state: &mut TextEditorState,
//     config: &GridConfig,
//     fontir_state: Option<&FontIRAppState>,
// ) {
//     // Implementation for creating a grid of sorts
//     // Each sort would be positioned based on grid coordinates
//     // Camera would center on the entire grid
// }
