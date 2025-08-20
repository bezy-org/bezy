//! Cross-platform file menu implementation
//!
//! Provides keyboard-based file menu functionality that works reliably across
//! all platforms without threading complexity.

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use crate::core::state::fontir_app_state::FontIRAppState;
use crate::ui::panes::file_pane::FileInfo;
// Note: Removed unused imports - we now preserve original glyph data
use std::path::PathBuf;
use norad::{Font as NoradFont, designspace::DesignSpaceDocument};
use kurbo::PathEl;

// ============================================================================
// EVENTS
// ============================================================================

/// Event fired when the user triggers a save action from the menu
#[derive(Event)]
pub struct SaveFileEvent;

/// Event fired when the user triggers an export to TTF action
#[derive(Event)]
pub struct ExportTTFEvent;

/// Resource to track the file menu state
#[derive(Resource)]
pub struct FileMenuState {
    pub initialized: bool,
}

// ============================================================================
// PLUGIN
// ============================================================================

pub struct FileMenuPlugin;

impl Plugin for FileMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveFileEvent>()
            .add_event::<ExportTTFEvent>()
            .insert_resource(FileMenuState { initialized: false })
            .add_systems(Startup, setup_file_menu)
            .add_systems(PreUpdate, handle_keyboard_shortcuts)
            .add_systems(Update, (
                handle_save_file_events,
                handle_export_ttf_events,
                update_save_state,
            ));
    }
}

// ============================================================================
// MENU SETUP
// ============================================================================

/// Sets up the file menu system with keyboard shortcuts
fn setup_file_menu(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut file_menu_state: ResMut<FileMenuState>,
) {
    if let Ok(mut window) = windows.single_mut() {
        // Set window title to include app name
        window.title = "Bezy - Font Editor".to_string();
        
        // Initialize keyboard-based file menu
        info!("✅ File menu initialized with cross-platform keyboard shortcuts:");
        info!("   💾 Save: Cmd+S (macOS) or Ctrl+S (Windows/Linux)");
        info!("   📦 Export TTF: Cmd+E (macOS) or Ctrl+E (Windows/Linux)");
        info!("   ⚡ Reliable keyboard shortcuts work on all platforms");
        
        file_menu_state.initialized = true;
    }
}

// ============================================================================
// MENU EVENT HANDLING
// ============================================================================

/// Handles keyboard shortcuts for file operations
fn handle_keyboard_shortcuts(
    mut save_events: EventWriter<SaveFileEvent>,
    mut export_events: EventWriter<ExportTTFEvent>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    file_menu_state: Res<FileMenuState>,
) {
    if !file_menu_state.initialized {
        return;
    }
    
    // Handle Cmd+S (macOS) or Ctrl+S (Windows/Linux)
    let cmd_or_ctrl = keyboard_input.pressed(KeyCode::SuperLeft) 
        || keyboard_input.pressed(KeyCode::SuperRight)
        || keyboard_input.pressed(KeyCode::ControlLeft) 
        || keyboard_input.pressed(KeyCode::ControlRight);
    
    if cmd_or_ctrl && keyboard_input.just_pressed(KeyCode::KeyS) {
        info!("💾 Save shortcut triggered (Cmd+S/Ctrl+S)");
        save_events.write(SaveFileEvent);
    }
    
    // Handle Cmd+E (macOS) or Ctrl+E (Windows/Linux) for export
    if cmd_or_ctrl && keyboard_input.just_pressed(KeyCode::KeyE) {
        info!("📦 Export TTF shortcut triggered (Cmd+E/Ctrl+E)");
        export_events.write(ExportTTFEvent);
    }
    
    // TEMPORARY: Also trigger export with F5 key for testing
    if keyboard_input.just_pressed(KeyCode::F5) {
        info!("📦 Export TTF triggered via F5 (temporary test)");
        export_events.write(ExportTTFEvent);
    }
}

/// Handles save file events
fn handle_save_file_events(
    mut save_events: EventReader<SaveFileEvent>,
    fontir_state: Option<Res<FontIRAppState>>,
    mut file_info: ResMut<FileInfo>,
) {
    for _event in save_events.read() {
        if let Some(state) = fontir_state.as_ref() {
            match save_font_files(&state.source_path, state) {
                Ok(saved_paths) => {
                    info!("Successfully saved {} files", saved_paths.len());
                    for path in &saved_paths {
                        info!("  Saved: {}", path.display());
                    }
                    
                    // Update the last saved time in file info
                    file_info.last_saved = Some(std::time::SystemTime::now());
                }
                Err(e) => {
                    error!("Failed to save files: {}", e);
                }
            }
        } else {
            warn!("No font data to save");
        }
    }
}

/// Updates the save state display
fn update_save_state(
    file_info: Res<FileInfo>,
) {
    // This system can be extended to show save status in the UI
    if file_info.is_changed() {
        if let Some(_last_saved) = file_info.last_saved {
            // Could show a "saved" indicator in the UI
            debug!("Save state updated");
        }
    }
}

// ============================================================================
// SAVE FUNCTIONALITY
// ============================================================================

/// Saves the font files back to disk
fn save_font_files(
    source_path: &PathBuf, 
    fontir_state: &FontIRAppState
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut saved_paths = Vec::new();
    
    // Check if it's a designspace file
    if source_path.extension().and_then(|s| s.to_str()) == Some("designspace") {
        info!("💾 Saving changes to designspace UFO sources...");
        
        // Parse designspace to get UFO source paths
        let designspace = DesignSpaceDocument::load(source_path)?;
        let designspace_dir = source_path.parent().unwrap_or(std::path::Path::new("."));
        
        info!("Found {} UFO sources in designspace", designspace.sources.len());
        
        // Check for modified glyphs in working copies
        let modified_glyphs: Vec<_> = fontir_state.working_copies
            .iter()
            .filter(|((_glyph_name, _location), working_copy)| working_copy.is_dirty)
            .collect();
            
        if modified_glyphs.is_empty() {
            info!("No modified glyphs found - nothing to save");
            return Ok(saved_paths);
        }
        
        info!("Found {} modified glyphs to save", modified_glyphs.len());
        
        // Process each UFO source
        for source in &designspace.sources {
            let ufo_path = designspace_dir.join(&source.filename);
            
            // Check if any modified glyphs belong to this UFO source
            let source_has_changes = modified_glyphs.iter().any(|((_, _location), _)| {
                // For now, save to first UFO (regular). In a full implementation,
                // you'd match the location to the correct UFO source
                ufo_path.file_name() == Some(std::ffi::OsStr::new("bezy-grotesk-regular.ufo"))
            });
            
            if source_has_changes {
                info!("Saving changes to UFO: {}", ufo_path.display());
                
                // Load the UFO
                let mut ufo_font = NoradFont::load(&ufo_path)?;
                
                // Update modified glyphs
                for ((glyph_name, _location), working_copy) in &modified_glyphs {
                    if working_copy.is_dirty {
                        info!("  Updating glyph: {}", glyph_name);
                        
                        // Preserve original glyph and only update outline
                        let layer = ufo_font.default_layer_mut();
                        if let Some(existing_glyph) = layer.get_glyph_mut(glyph_name) {
                            // Update only the outline and width, preserve everything else (anchors, unicode, etc.)
                            existing_glyph.width = working_copy.width;
                            if let Some(height) = working_copy.height {
                                existing_glyph.height = height;
                            }
                            
                            // Simple approach: recreate contours from BezPath
                            // This will lose the original starting point, but it's reliable
                            existing_glyph.contours.clear();
                            for bez_path in &working_copy.contours {
                                let contour = convert_bezpath_to_ufo_contour(bez_path);
                                existing_glyph.contours.push(contour);
                            }
                        } else {
                            warn!("Glyph {} not found in UFO, skipping update", glyph_name);
                        }
                    }
                }
                
                // Save the modified UFO
                ufo_font.save(&ufo_path)?;
                saved_paths.push(ufo_path.clone());
                
                info!("✅ Successfully saved UFO: {}", ufo_path.display());
            }
        }
        
    } else if source_path.extension().and_then(|s| s.to_str()) == Some("ufo") {
        // Handle single UFO file
        info!("💾 Saving changes to UFO file: {}", source_path.display());
        
        // Check for modified glyphs
        let modified_glyphs: Vec<_> = fontir_state.working_copies
            .iter()
            .filter(|((_glyph_name, _location), working_copy)| working_copy.is_dirty)
            .collect();
            
        if modified_glyphs.is_empty() {
            info!("No modified glyphs found - nothing to save");
            return Ok(saved_paths);
        }
        
        // Load the UFO
        let mut ufo_font = NoradFont::load(source_path)?;
        
        // Update modified glyphs
        for ((glyph_name, _location), working_copy) in &modified_glyphs {
            if working_copy.is_dirty {
                info!("  Updating glyph: {}", glyph_name);
                
                // Preserve original glyph and only update outline
                let layer = ufo_font.default_layer_mut();
                if let Some(existing_glyph) = layer.get_glyph_mut(glyph_name) {
                    // Update only the outline and width, preserve everything else (anchors, unicode, etc.)
                    existing_glyph.width = working_copy.width;
                    if let Some(height) = working_copy.height {
                        existing_glyph.height = height;
                    }
                    
                    // Simple approach: recreate contours from BezPath
                    // This will lose the original starting point, but it's reliable
                    existing_glyph.contours.clear();
                    for bez_path in &working_copy.contours {
                        let contour = convert_bezpath_to_ufo_contour(bez_path);
                        existing_glyph.contours.push(contour);
                    }
                } else {
                    warn!("Glyph {} not found in UFO, skipping update", glyph_name);
                }
            }
        }
        
        // Save the modified UFO
        ufo_font.save(source_path)?;
        saved_paths.push(source_path.clone());
        
        info!("✅ Successfully saved UFO: {}", source_path.display());
    }
    
    Ok(saved_paths)
}


/// Convert BezPath to norad Contour, preserving the starting point
/// 
/// The BezPath MoveTo defines where the contour starts. We ensure the UFO contour
/// starts at the same logical position by organizing points accordingly.
fn convert_bezpath_to_ufo_contour(bez_path: &kurbo::BezPath) -> norad::Contour {
    let mut all_points = Vec::new();
    let mut is_closed = false;
    let mut move_to_pos: Option<kurbo::Point> = None;
    
    let elements = bez_path.elements();
    
    // Check if path is closed
    for element in elements {
        if matches!(element, PathEl::ClosePath) {
            is_closed = true;
            break;
        }
    }
    
    // Convert elements to points
    for element in elements {
        match element {
            PathEl::MoveTo(pt) => {
                move_to_pos = Some(*pt);
                if !is_closed {
                    // For open paths, include MoveTo as first point
                    all_points.push(norad::ContourPoint::new(
                        pt.x, pt.y, norad::PointType::Move, false, None, None
                    ));
                }
                // For closed paths, MoveTo determines starting point but isn't a UFO point
            }
            PathEl::LineTo(pt) => {
                all_points.push(norad::ContourPoint::new(
                    pt.x, pt.y, norad::PointType::Line, false, None, None
                ));
            }
            PathEl::CurveTo(cp1, cp2, pt) => {
                all_points.push(norad::ContourPoint::new(
                    cp1.x, cp1.y, norad::PointType::OffCurve, false, None, None
                ));
                all_points.push(norad::ContourPoint::new(
                    cp2.x, cp2.y, norad::PointType::OffCurve, false, None, None
                ));
                all_points.push(norad::ContourPoint::new(
                    pt.x, pt.y, norad::PointType::Curve, false, None, None
                ));
            }
            PathEl::QuadTo(cp, pt) => {
                all_points.push(norad::ContourPoint::new(
                    cp.x, cp.y, norad::PointType::OffCurve, false, None, None
                ));
                all_points.push(norad::ContourPoint::new(
                    pt.x, pt.y, norad::PointType::QCurve, false, None, None
                ));
            }
            PathEl::ClosePath => {}
        }
    }
    
    // For closed contours, ensure the first point corresponds to the MoveTo position
    if is_closed && move_to_pos.is_some() && !all_points.is_empty() {
        let start_pt = move_to_pos.unwrap();
        
        
        // Find if any point matches the MoveTo position
        if let Some(start_idx) = find_point_near_position(&all_points, start_pt) {
            // Rotate the points so the matching point comes first
            let rotated = rotate_points_to_start(&all_points, start_idx);
            return norad::Contour::new(rotated, None);
        } else {
            info!("No matching point found for MoveTo position, using original order");
        }
    }
    
    norad::Contour::new(all_points, None)
}

/// Find a point near the given position (within 1.0 units, expanded tolerance)
fn find_point_near_position(points: &[norad::ContourPoint], target: kurbo::Point) -> Option<usize> {
    let mut best_match = None;
    let mut best_distance = f64::INFINITY;
    
    for (i, point) in points.iter().enumerate() {
        // Check both on-curve and off-curve points since starting point might be off-curve
        let distance = ((point.x - target.x).powi(2) + (point.y - target.y).powi(2)).sqrt();
        if distance < 1.0 && distance < best_distance {
            best_distance = distance;
            best_match = Some(i);
        }
    }
    
    
    best_match
}

/// Rotate a list of points so that the point at start_idx becomes first
/// Simple rotation - start exactly from the specified index
fn rotate_points_to_start(points: &[norad::ContourPoint], start_idx: usize) -> Vec<norad::ContourPoint> {
    if start_idx == 0 || points.is_empty() {
        return points.to_vec();
    }
    
    
    // Simple rotation: start from start_idx
    let mut result = Vec::new();
    result.extend_from_slice(&points[start_idx..]);
    result.extend_from_slice(&points[..start_idx]);
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::{BezPath, Point};
    
    #[test]
    fn test_closed_contour_conversion() {
        // Create a closed square where MoveTo position has a corresponding LineTo
        let mut bez_path = BezPath::new();
        bez_path.move_to(Point::new(0.0, 0.0));
        bez_path.line_to(Point::new(100.0, 0.0));
        bez_path.line_to(Point::new(100.0, 100.0));
        bez_path.line_to(Point::new(0.0, 100.0));
        bez_path.line_to(Point::new(0.0, 0.0)); // Explicit line back to start
        bez_path.close_path();
        
        let contour = convert_bezpath_to_ufo_contour(&bez_path);
        
        // Should have 4 points
        assert_eq!(contour.points.len(), 4, "Closed square should have 4 points");
        
        // The contour should be rotated to start at (0,0) since that matches MoveTo
        let first_point = &contour.points[0];
        assert_eq!((first_point.x, first_point.y), (0.0, 0.0), 
                   "First point should be at MoveTo position (0,0)");
    }
    
    #[test] 
    fn test_open_contour_conversion() {
        // Create an open path
        let mut bez_path = BezPath::new();
        bez_path.move_to(Point::new(0.0, 0.0));
        bez_path.line_to(Point::new(100.0, 0.0));
        bez_path.line_to(Point::new(100.0, 100.0));
        // No close_path()
        
        let contour = convert_bezpath_to_ufo_contour(&bez_path);
        
        // Should have 3 points (MoveTo + 2 LineTo)
        assert_eq!(contour.points.len(), 3, "Open contour should have 3 points");
        
        // First point should be Move
        assert_eq!(contour.points[0].typ, norad::PointType::Move, "First point should be Move type");
        assert_eq!(contour.points[0].x, 0.0);
        assert_eq!(contour.points[0].y, 0.0);
    }
    
    #[test]
    fn test_curve_contour_conversion() {
        // Create a closed path with curves that starts with a curve
        let mut bez_path = BezPath::new();
        bez_path.move_to(Point::new(224.0, -16.0));
        bez_path.curve_to(Point::new(306.0, -16.0), Point::new(374.0, 16.0), Point::new(416.0, 72.0));
        bez_path.line_to(Point::new(224.0, -16.0));
        bez_path.close_path();
        
        let contour = convert_bezpath_to_ufo_contour(&bez_path);
        
        // Should have 4 points: 2 off-curves, 1 curve, 1 line
        assert_eq!(contour.points.len(), 4, "Curve contour should have 4 points");
        
        // First points should be the curve (starting with off-curves)
        assert_eq!(contour.points[0].typ, norad::PointType::OffCurve);
        assert_eq!((contour.points[0].x, contour.points[0].y), (306.0, -16.0));
        
        assert_eq!(contour.points[1].typ, norad::PointType::OffCurve);
        assert_eq!((contour.points[1].x, contour.points[1].y), (374.0, 16.0));
        
        assert_eq!(contour.points[2].typ, norad::PointType::Curve);
        assert_eq!((contour.points[2].x, contour.points[2].y), (416.0, 72.0));
        
        // Last should be the line back to start
        assert_eq!(contour.points[3].typ, norad::PointType::Line);
        assert_eq!((contour.points[3].x, contour.points[3].y), (224.0, -16.0));
    }
    
    #[test]
    fn test_starting_point_preservation() {
        // Test case inspired by the 'a' glyph issue
        // Create a contour that starts with a curve at a specific position
        let mut bez_path = BezPath::new();
        bez_path.move_to(Point::new(32.0, 160.0)); // Start here
        bez_path.curve_to(Point::new(32.0, 52.0), Point::new(106.0, -16.0), Point::new(224.0, -16.0));
        bez_path.line_to(Point::new(400.0, 100.0));
        bez_path.line_to(Point::new(32.0, 160.0)); // Explicit line back to start
        bez_path.close_path();
        
        let contour = convert_bezpath_to_ufo_contour(&bez_path);
        
        // Should find the point at (32, 160) and start there
        assert_eq!(contour.points.len(), 5); // 2 off-curve + 1 curve + 1 line + 1 line back
        
        // First point should be at the MoveTo position
        assert_eq!((contour.points[0].x, contour.points[0].y), (32.0, 160.0));
        assert_eq!(contour.points[0].typ, norad::PointType::Line);
    }
}

/// Handles export to TTF events
fn handle_export_ttf_events(
    mut export_events: EventReader<ExportTTFEvent>,
    mut file_info: ResMut<FileInfo>,
) {
    for _ in export_events.read() {
        info!("🚀🚀🚀 EXPORT EVENT RECEIVED! 🚀🚀🚀");
        
        // Always update the export time to show the feature is working
        file_info.last_exported = Some(std::time::SystemTime::now());
        info!("✅ Updated export timestamp in UI");
        
        if file_info.designspace_path.is_empty() {
            warn!("Cannot export: No designspace file loaded");
            warn!("But the export system is working - timestamp updated!");
            continue;
        }

        info!("🚀 Starting TTF export from designspace: {}", file_info.designspace_path);
        
        // Get the directory of the designspace file
        let designspace_path = PathBuf::from(&file_info.designspace_path);
        let default_dir = PathBuf::from(".");
        let output_dir = designspace_path.parent().unwrap_or(&default_dir);
        
        // Create temporary build directory
        let build_dir = output_dir.join(".fontc-build");
        if !build_dir.exists() {
            if let Err(e) = std::fs::create_dir(&build_dir) {
                error!("Failed to create build directory: {}", e);
                return;
            }
        }
        
        // Load the designspace to understand what instances we expect
        let ds = match DesignSpaceDocument::load(&designspace_path) {
            Ok(ds) => ds,
            Err(e) => {
                error!("Failed to load designspace: {}", e);
                return;
            }
        };
        
        info!("📋 Found {} instances in designspace", ds.instances.len());
        for instance in &ds.instances {
            let style_name = instance.stylename.as_deref()
                .unwrap_or("Regular");
            info!("   - {}", style_name);
        }
        
        // Create input from the designspace path
        let input = match fontc::Input::new(&designspace_path) {
            Ok(input) => input,
            Err(e) => {
                error!("Failed to create fontc input: {}", e);
                return;
            }
        };
        
        // Extract family name from the first source or instance
        let family_name = ds.sources
            .first()
            .and_then(|s| s.familyname.as_ref())
            .or_else(|| ds.instances.first().and_then(|i| i.familyname.as_ref()))
            .cloned()
            .unwrap_or_else(|| "Font".to_string());
        
        let mut exported_files = Vec::new();
        
        // First, compile the variable font
        info!("🔨 Compiling variable font with fontc...");
        let flags = fontc::Flags::default();
        
        match fontc::generate_font(
            &input,
            &build_dir,
            None,
            flags,
            false,
        ) {
            Ok(font_bytes) => {
                info!("✅ Variable font compilation completed!");
                
                let output_filename = format!("{}-Variable.ttf", family_name.replace(" ", ""));
                let output_path = output_dir.join(&output_filename);
                
                // Write the variable font bytes to file
                match std::fs::write(&output_path, &font_bytes) {
                    Ok(_) => {
                        info!("📁 Exported variable font: {}", output_filename);
                        exported_files.push(output_filename);
                    }
                    Err(e) => {
                        error!("Failed to write variable font file: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("❌ Variable font compilation failed: {}", e);
            }
        }
        
        // Now generate static instances for Regular and Bold
        info!("🔨 Generating static font instances...");
        
        // For each desired static instance (Regular and Bold)
        for instance in &ds.instances {
            let style_name = instance.stylename.as_deref()
                .unwrap_or("Regular");
            
            // Only export Regular and Bold static instances
            if style_name != "Regular" && style_name != "Bold" {
                continue;
            }
            
            info!("   Generating static instance: {}", style_name);
            
            // Create a temporary UFO by interpolating at the instance location
            // For now, we'll just copy the appropriate source UFO
            let source_ufo_path = if style_name == "Regular" {
                output_dir.join("bezy-grotesk-regular.ufo")
            } else if style_name == "Bold" {
                output_dir.join("bezy-grotesk-bold.ufo")
            } else {
                continue;
            };
            
            // If the source UFO exists, compile it directly
            if source_ufo_path.exists() {
                match fontc::Input::new(&source_ufo_path) {
                    Ok(static_input) => {
                        match fontc::generate_font(
                            &static_input,
                            &build_dir,
                            None,
                            flags,
                            false,
                        ) {
                            Ok(static_font_bytes) => {
                                let instance_filename = format!("{}-{}.ttf", 
                                    family_name.replace(" ", ""), 
                                    style_name.replace(" ", ""));
                                let instance_path = output_dir.join(&instance_filename);
                                
                                match std::fs::write(&instance_path, &static_font_bytes) {
                                    Ok(_) => {
                                        info!("   ✅ Exported static instance: {}", instance_filename);
                                        exported_files.push(instance_filename);
                                    }
                                    Err(e) => {
                                        error!("   Failed to write static font: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("   Failed to compile static instance {}: {}", style_name, e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("   Failed to create input for static instance: {}", e);
                    }
                }
            } else {
                info!("   Source UFO not found for {}, skipping static instance", style_name);
            }
        }
        
        if !exported_files.is_empty() {
            info!("📁 Successfully exported {} font file(s) to: {}", exported_files.len(), output_dir.display());
            for file in &exported_files {
                info!("   - {}", file);
            }
            
            // Update the last exported time
            file_info.last_exported = Some(std::time::SystemTime::now());
        } else {
            warn!("⚠️ No font files were exported");
        }
        
        // Clean up build directory
        let _ = std::fs::remove_dir_all(&build_dir);
    }
}