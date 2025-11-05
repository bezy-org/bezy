//! Startup layout configuration for the viewport
//!
//! This module handles the initial layout of sorts and camera positioning
//! when the application starts. This includes creating default sorts for
//! first-time users and positioning the camera for optimal initial view.
//!
//! Future: This will be expanded to create a grid of glyph sorts instead
//! of just a single default sort.

use crate::core::state::text_editor::{SortData, SortKind};
use crate::core::state::TextEditorState;
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
    mut text_editor_state: ResMut<TextEditorState>,
    mut commands: Commands,
    cli_args: Res<crate::core::config::CliArgs>,
    app_state: Option<Res<crate::core::state::AppState>>,
) {
    // Only create default layout if no sorts exist yet
    if !text_editor_state.buffer.is_empty() {
        return;
    }

    // Check if default buffer creation is disabled via CLI flag
    if cli_args.no_default_buffer {
        debug!("Skipping default LTR buffer creation due to --no-default-buffer flag");
        debug!("Ready for isolated text flow testing - use text tool to place sorts manually");
        return;
    }

    // Default to 'a' glyph
    let glyph_name = "a".to_string();

    debug!(
        "Creating startup layout with default LTR text sort for glyph '{}'",
        glyph_name
    );

    // Get actual advance width from font data
    let advance_width = if let Some(app_state) = app_state.as_ref() {
        let width = app_state
            .workspace
            .font
            .glyphs
            .get(&glyph_name)
            .map(|g| g.advance_width as f32)
            .unwrap_or_else(|| {
                warn!("Glyph '{}' not found in font data, using fallback width 500.0", glyph_name);
                500.0
            });
        warn!("🎯 STARTUP LAYOUT: Using advance_width={:.1} for glyph '{}'", width, glyph_name);
        width
    } else {
        warn!("AppState not available, using fallback advance width 500.0");
        500.0
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
    let visual_center_y = 328.0; // <-- ADJUST THIS VALUE: Higher = camera moves up

    // Insert a marker resource to trigger camera centering in the next frame
    commands.insert_resource(CenterCameraOnDefaultSort {
        center_x: visual_center_x,
        center_y: visual_center_y,
        advance_width,
    });

    debug!(
        "Startup layout created. Camera will center at ({}, {})",
        visual_center_x, visual_center_y
    );
}

/// Helper function to create a single sort at a specific position
/// This is separated out to make it easy to create multiple sorts in a grid later
fn create_default_sort_at_position(
    text_editor_state: &mut TextEditorState,
    position: Vec2,
    glyph_name: &str,
    advance_width: f32,
) {
    use crate::core::state::text_editor::buffer::BufferId;
    use crate::core::state::text_editor::{SortData, SortKind, SortLayoutMode};

    warn!(
        "🔍 STARTUP LAYOUT: create_default_sort_at_position called - buffer has {} sorts BEFORE insert",
        text_editor_state.buffer.len()
    );

    // Create a new buffer ID for this LTR text flow
    let buffer_id = BufferId::new();

    let sort = SortData {
        kind: SortKind::Glyph {
            codepoint: Some('a'), // Default to 'a'
            glyph_name: glyph_name.to_string(),
            advance_width, // Use actual advance width for cursor positioning
        },
        is_active: true,                      // Make it active and ready to edit
        layout_mode: SortLayoutMode::LTRText, // LTR text mode for typing
        root_position: position,
        buffer_cursor_position: Some(1),
        buffer_id: Some(buffer_id), // Assign unique buffer ID for isolation
    };

    debug!(
        "📍 STARTUP: Created sort with buffer_id: {:?} (cursor now managed by buffer entities)",
        sort.buffer_id
    );

    // Add to the text editor buffer
    let insert_index = text_editor_state.buffer.len();
    warn!(
        "🔍 STARTUP LAYOUT: Inserting sort '{}' at index {} (buffer_id: {:?})",
        glyph_name, insert_index, buffer_id.0
    );
    text_editor_state.buffer.insert(insert_index, sort);

    warn!(
        "🔍 STARTUP LAYOUT: Insertion complete - buffer now has {} sorts",
        text_editor_state.buffer.len()
    );

    // Mark as changed using Bevy's change detection
    // The ResMut wrapper automatically tracks changes when we modify the resource

    debug!(
        "Created default sort '{}' at position ({:.1}, {:.1})",
        glyph_name, position.x, position.y
    );
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

            debug!(
                "Centered camera on startup layout at ({}, {})",
                center.center_x, center.center_y
            );
        }

        // Remove the resource so this only happens once
        commands.remove_resource::<CenterCameraOnDefaultSort>();
    }
}

/// System to migrate existing sorts to use correct advance widths from font data
/// This fixes sorts that were created with the old hardcoded 500.0 value
pub fn migrate_sort_advance_widths(
    mut text_editor_state: ResMut<TextEditorState>,
    app_state: Option<Res<crate::core::state::AppState>>,
    mut has_run: Local<bool>,
) {
    // Only run once
    if *has_run {
        return;
    }

    // Only migrate if we have font data - keep trying until font loads
    let Some(app_state) = app_state.as_ref() else {
        debug!("Waiting for font data to migrate advance widths...");
        return;  // Don't mark as done, keep trying
    };

    let mut updated_count = 0;

    // Iterate through all sorts by index and fix their advance_widths
    for i in 0..text_editor_state.buffer.len() {
        if let Some(sort) = text_editor_state.buffer.get_mut(i) {
            if let SortKind::Glyph { glyph_name, advance_width, .. } = &mut sort.kind {
                // Check if this sort has the old hardcoded value
                if (*advance_width - 500.0).abs() < 0.1 {
                    // Try to get the correct advance width from font data
                    if let Some(glyph_data) = app_state.workspace.font.glyphs.get(glyph_name.as_str()) {
                        let correct_width = glyph_data.advance_width as f32;
                        // Only update if the correct width is different
                        if (correct_width - 500.0).abs() > 0.1 {
                            warn!(
                                "🔧 MIGRATION: Updating glyph '{}' advance_width: {:.1} → {:.1}",
                                glyph_name, *advance_width, correct_width
                            );
                            *advance_width = correct_width;
                            updated_count += 1;
                        }
                    }
                }
            }
        }
    }

    if updated_count > 0 {
        warn!("✅ MIGRATION: Updated {} sorts with correct advance widths", updated_count);
    } else {
        debug!("Migration check complete - no sorts needed updating");
    }

    // Mark as done after successful migration check
    *has_run = true;
}
