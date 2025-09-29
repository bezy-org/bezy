//! Knife Tool - Path cutting and slicing tool
//!
//! This tool allows users to cut paths by drawing a line across them.
//! The tool shows a preview of the cutting line and intersection points.

use crate::core::state::AppState;
use crate::editing::selection::events::AppStateChanged;
use crate::ui::edit_mode_toolbar::{EditTool, ToolRegistry};
use crate::ui::theme::*;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use kurbo::{BezPath, ParamCurve, ParamCurveNearest, PathEl, Point, Shape};

// Simple path operations are defined at the end of this file

// Use KnifeModeActive from tools::knife and re-export it
pub use crate::tools::knife::KnifeModeActive;

pub struct KnifeTool;

impl EditTool for KnifeTool {
    fn id(&self) -> crate::ui::edit_mode_toolbar::ToolId {
        "knife"
    }

    fn name(&self) -> &'static str {
        "Knife"
    }

    fn icon(&self) -> &'static str {
        "\u{E013}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('k')
    }

    fn default_order(&self) -> i32 {
        110 // Advanced tool, later in toolbar
    }

    fn description(&self) -> &'static str {
        "Cut and slice paths"
    }

    fn update(&self, commands: &mut Commands) {
        debug!(
            "🔪 KNIFE_TOOL: update() called - setting knife mode active and input mode to Knife"
        );
        commands.insert_resource(KnifeModeActive(true));
        commands.insert_resource(crate::io::input::InputMode::Knife);
    }

    fn on_enter(&self) {
        debug!("Entered Knife tool");
    }

    fn on_exit(&self) {
        debug!("Exited Knife tool");
    }
}

/// The state of the knife gesture
#[derive(Debug, Clone, PartialEq, Default, Copy)]
pub enum KnifeGestureState {
    /// Ready to start cutting
    #[default]
    Ready,
    /// Currently dragging a cut line
    Cutting { start: Vec2, current: Vec2 },
}

/// Resource to track the state of the knife tool
#[derive(Resource, Default)]
pub struct KnifeToolState {
    /// The current gesture state
    pub gesture: KnifeGestureState,
    /// Whether shift key is pressed (for axis-aligned cuts)
    pub shift_locked: bool,
    /// Intersection points for visualization
    pub intersections: Vec<Vec2>,
}

impl KnifeToolState {
    pub fn new() -> Self {
        Self {
            gesture: KnifeGestureState::Ready,
            shift_locked: false,
            intersections: Vec::new(),
        }
    }

    /// Get the cutting line with axis locking if shift is pressed
    pub fn get_cutting_line(&self) -> Option<(Vec2, Vec2)> {
        match self.gesture {
            KnifeGestureState::Cutting { start, current } => {
                let actual_end = if self.shift_locked {
                    // Apply axis constraint for shift key
                    let delta = current - start;
                    if delta.x.abs() > delta.y.abs() {
                        // Horizontal line
                        Vec2::new(current.x, start.y)
                    } else {
                        // Vertical line
                        Vec2::new(start.x, current.y)
                    }
                } else {
                    current
                };
                Some((start, actual_end))
            }
            KnifeGestureState::Ready => None,
        }
    }
}

/// Plugin for the knife tool
pub struct KnifeToolPlugin;

impl Plugin for KnifeToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KnifeModeActive>()
            .init_resource::<KnifeToolState>()
            .init_resource::<KnifeCalculationCache>()
            .add_systems(Startup, register_knife_tool)
            .add_systems(
                Update,
                (
                    manage_knife_mode_state,
                    handle_knife_mouse_events.after(manage_knife_mode_state),
                    render_knife_preview.after(handle_knife_mouse_events),
                    handle_fontir_knife_cutting,
                ),
            );
    }
}

fn register_knife_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(KnifeTool));
}

/// Handle mouse events for the knife tool
#[allow(clippy::too_many_arguments)]
pub fn handle_knife_mouse_events(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut knife_state: ResMut<KnifeToolState>,
    knife_mode: Option<Res<KnifeModeActive>>,
    _app_state_changed: EventWriter<crate::editing::selection::events::AppStateChanged>,
    // Query for active sort to get its position
    active_sort_query: Query<
        (Entity, &crate::editing::sort::Sort, &Transform),
        With<crate::editing::sort::ActiveSort>,
    >,
) {
    // Check if knife mode is active
    let knife_is_active = if let Some(knife_mode) = knife_mode {
        knife_mode.0
    } else {
        false
    };

    // IMPORTANT: Knife tool requires an active sort to work (like pen and shapes tools)
    let active_sort = if let Ok((sort_entity, sort, sort_transform)) = active_sort_query.single() {
        Some((sort_entity, sort, sort_transform))
    } else {
        None
    };

    // Early exit if knife tool is not active, no active sort, or other conditions
    let Some((_sort_entity, _sort, sort_transform)) = active_sort else {
        if knife_is_active {
            // Only show this message when knife tool is actually trying to be used
            if mouse_button_input.just_pressed(MouseButton::Left) {
                debug!("🔪 Knife tool: Cannot cut without an active sort. Please select a glyph first.");
            }
        }
        return;
    };

    if !knife_is_active {
        return;
    }
    let sort_position = sort_transform.translation.truncate();

    let Ok(window) = windows.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Convert cursor position to world coordinates, then to sort-relative coordinates
    if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
        // Convert to sort-relative coordinates
        let sort_relative_position = world_position - sort_position;

        // Update shift lock state
        knife_state.shift_locked =
            keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

        // Handle mouse button press
        if mouse_button_input.just_pressed(MouseButton::Left) {
            knife_state.gesture = KnifeGestureState::Cutting {
                start: sort_relative_position,
                current: sort_relative_position,
            };
            knife_state.intersections.clear();
            debug!(
                "🔪 KNIFE_DEBUG: Started cutting at sort-relative {:?}, world {:?}, sort pos {:?}",
                sort_relative_position, world_position, sort_position
            );
        }

        // Handle mouse movement during cutting
        if let KnifeGestureState::Cutting { start, .. } = knife_state.gesture {
            knife_state.gesture = KnifeGestureState::Cutting {
                start,
                current: sort_relative_position,
            };

            // Intersections will be calculated by the render system
            debug!(
                "🔪 KNIFE_DEBUG: Dragging to sort-relative {:?}",
                sort_relative_position
            );
        }

        // Handle mouse button release
        if mouse_button_input.just_released(MouseButton::Left) {
            if let Some((_start, _end)) = knife_state.get_cutting_line() {
                // The actual cutting is handled by handle_fontir_knife_cutting system
                debug!("🔪 KNIFE_DEBUG: Mouse released - cutting will be handled by FontIR system");
            }

            // Reset state
            knife_state.gesture = KnifeGestureState::Ready;
            knife_state.intersections.clear();
        }
    }
}

/// Handle keyboard events for the knife tool
pub fn handle_knife_keyboard_events(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut knife_state: ResMut<KnifeToolState>,
    knife_mode: Option<Res<KnifeModeActive>>,
) {
    // Only handle events when in knife mode
    if let Some(knife_mode) = knife_mode {
        if !knife_mode.0 {
            return;
        }
    } else {
        return;
    }

    // Handle Escape key to cancel current cut
    if keyboard.just_pressed(KeyCode::Escape) {
        knife_state.gesture = KnifeGestureState::Ready;
        knife_state.intersections.clear();
        debug!("Cancelled knife cut");
    }
}

/// System to manage knife mode activation/deactivation
pub fn manage_knife_mode_state(
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
    mut knife_state: ResMut<KnifeToolState>,
    knife_mode: Option<Res<KnifeModeActive>>,
) {
    let is_knife_active = current_tool.get_current() == Some("knife");
    let current_mode = knife_mode.as_ref().map(|m| m.0).unwrap_or(false);

    if is_knife_active && !current_mode {
        // Knife tool is active but mode is not set - activate it
        commands.insert_resource(KnifeModeActive(true));
        debug!("🔪 MANAGE_KNIFE_MODE: Activating knife mode");
    } else if !is_knife_active && current_mode {
        // Knife tool is not active but mode is set - deactivate it
        *knife_state = KnifeToolState::new();
        commands.insert_resource(KnifeModeActive(false));
        debug!("🔪 MANAGE_KNIFE_MODE: Deactivating knife mode");
    }
}

/// Resource to track visual update state for performance
#[derive(Resource, Default)]
pub struct KnifeVisualUpdateTracker {
    pub needs_update: bool,
    pub last_gesture_state: Option<KnifeGestureState>,
}

/// Cache for knife tool calculations to avoid repeated computation
#[derive(Resource, Default)]
pub struct KnifeCalculationCache {
    pub last_cutting_line: Option<(Vec2, Vec2)>,
    pub cached_intersections: Vec<Vec2>,
    pub last_glyph: Option<String>,
}

/// Render the knife tool preview
#[allow(clippy::too_many_arguments)]
pub fn render_knife_preview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    knife_state: Res<KnifeToolState>, // Use main knife tool state instead of input consumer
    knife_mode: Option<Res<KnifeModeActive>>,
    camera_scale: Res<crate::rendering::zoom_aware_scaling::CameraResponsiveScale>,
    mut knife_entities: Local<Vec<Entity>>,
    theme: Res<crate::ui::themes::CurrentTheme>,
    current_tool: Res<crate::ui::edit_mode_toolbar::CurrentTool>,
    mut update_tracker: Local<Option<KnifeGestureState>>,
    fontir_state: Option<Res<crate::core::state::FontIRAppState>>,
    mut calc_cache: Local<KnifeCalculationCache>,
    // Query for active sort to get its position for preview rendering
    active_sort_query: Query<
        (Entity, &crate::editing::sort::Sort, &Transform),
        With<crate::editing::sort::ActiveSort>,
    >,
) {
    // Check if tool is active
    let is_knife_active = current_tool.get_current() == Some("knife")
        && knife_mode.as_ref().map(|m| m.0).unwrap_or(false);

    // Knife tool requires an active sort to work
    let active_sort = if let Ok((sort_entity, sort, sort_transform)) = active_sort_query.single() {
        Some((sort_entity, sort, sort_transform))
    } else {
        None
    };

    // Don't render if no active sort
    if is_knife_active && active_sort.is_none() {
        // Clean up any existing knife entities
        for entity in knife_entities.drain(..) {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.despawn();
            }
        }
        return;
    }

    let sort_position = if let Some((_, _, sort_transform)) = active_sort {
        sort_transform.translation.truncate()
    } else {
        Vec2::ZERO // Fallback, but we return early above if no active sort
    };

    // Only update if gesture state has changed or knife tool became active
    let gesture_changed = update_tracker.as_ref() != Some(&knife_state.gesture);
    let needs_update = gesture_changed || (!knife_entities.is_empty() && !is_knife_active);

    if !needs_update {
        return; // Early exit for performance
    }

    // Update tracking state
    *update_tracker = Some(knife_state.gesture);

    // Clean up previous knife entities
    for entity in knife_entities.drain(..) {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }

    // Check if knife tool is active
    if current_tool.get_current() != Some("knife") {
        return;
    }

    // Also check knife mode resource
    if let Some(knife_mode) = knife_mode {
        if !knife_mode.0 {
            return;
        }
    } else {
        return;
    }

    // Draw the cutting line
    if let Some((start, end)) = knife_state.get_cutting_line() {
        // Convert from sort-relative to world coordinates for rendering
        let world_start = start + sort_position;
        let world_end = end + sort_position;

        debug!(
            "🔪 RENDER_KNIFE_PREVIEW: Drawing cutting line from sort-relative {:?}-{:?} to world {:?}-{:?}",
            start, end, world_start, world_end
        );
        let line_color = theme.theme().knife_line_color();

        // Create dashed line effect with a single batched mesh for performance
        let _direction = (world_end - world_start).normalize();
        let _total_length = world_start.distance(world_end);
        let dash_length = theme.theme().knife_dash_length() * camera_scale.scale_factor();
        let gap_length = theme.theme().knife_gap_length() * camera_scale.scale_factor();
        let _segment_length = dash_length + gap_length;
        let line_width = camera_scale.adjusted_line_width();

        // Batch all dashes into a single mesh (use world coordinates)
        let dashed_line_entity = spawn_dashed_line_batched(
            &mut commands,
            &mut meshes,
            &mut materials,
            world_start,
            world_end,
            dash_length,
            gap_length,
            line_width,
            line_color,
            18.0, // z-order (below intersection points but above other elements)
        );
        knife_entities.push(dashed_line_entity);

        // Draw start point (yellow circle like measure tool) - use world coordinates
        let point_color = theme.theme().selected_color(); // Use yellow selection color
        let point_size = camera_scale.adjusted_size(4.0);
        let start_entity = spawn_knife_point_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            world_start,
            point_size,
            point_color,
            19.0, // z-order above line but below intersection points
        );
        knife_entities.push(start_entity);

        // Draw end point (yellow circle like measure tool) - use world coordinates
        let end_entity = spawn_knife_point_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            world_end,
            point_size,
            point_color,
            19.0, // z-order above line but below intersection points
        );
        knife_entities.push(end_entity);

        debug!(
            "🔪 RENDER_KNIFE_PREVIEW: Created {} visual entities for knife preview",
            knife_entities.len()
        );
    } else {
        // Log when we're not drawing
        if matches!(knife_state.gesture, KnifeGestureState::Ready) {
            debug!("🔪 RENDER_KNIFE_PREVIEW: No cutting line to draw (Ready state)");
        }
    }

    // Calculate and draw intersection points from actual glyph data
    if let Some((start, end)) = knife_state.get_cutting_line() {
        // Check if we need to recalculate intersections
        let current_glyph = fontir_state
            .as_ref()
            .and_then(|fs| fs.current_glyph.clone());

        let needs_recalc = calc_cache.last_cutting_line != Some((start, end))
            || calc_cache.last_glyph != current_glyph;

        if needs_recalc {
            // Update cache with new intersections
            calc_cache.cached_intersections =
                calculate_real_intersections(start, end, &fontir_state);
            calc_cache.last_cutting_line = Some((start, end));
            calc_cache.last_glyph = current_glyph;
        }

        let intersection_color = theme.theme().selected_color(); // Use yellow selection color

        for &intersection in &calc_cache.cached_intersections {
            // Convert intersection from sort-relative to world coordinates
            let world_intersection = intersection + sort_position;

            // Create yellow filled circles for intersection points (like measure tool)
            let intersection_size = camera_scale.adjusted_size(6.0); // Same size as measure tool
            let circle_entity = spawn_knife_point_mesh(
                &mut commands,
                &mut meshes,
                &mut materials,
                world_intersection,
                intersection_size,
                intersection_color,
                20.0, // z-order above everything else
            );
            knife_entities.push(circle_entity);
        }
    }
}

/// Calculate real intersections between knife line and current glyph contours
fn calculate_real_intersections(
    start: Vec2,
    end: Vec2,
    fontir_state: &Option<Res<crate::core::state::FontIRAppState>>,
) -> Vec<Vec2> {
    let mut intersections = Vec::new();

    // Convert cutting line to kurbo Line for intersection testing
    let cutting_line = kurbo::Line::new(
        kurbo::Point::new(start.x as f64, start.y as f64),
        kurbo::Point::new(end.x as f64, end.y as f64),
    );

    // Try FontIR state first (preferred)
    if let Some(fontir_state) = fontir_state {
        if let Some(ref current_glyph) = fontir_state.current_glyph {
            if let Some(paths) = fontir_state.get_glyph_paths_with_edits(current_glyph) {
                debug!(
                    "🔪 CALCULATE_REAL_INTERSECTIONS: Found {} paths for glyph '{}'",
                    paths.len(),
                    current_glyph
                );
                for path in &paths {
                    let path_intersections = find_path_intersections_simple(path, &cutting_line);
                    for intersection in path_intersections {
                        intersections.push(Vec2::new(intersection.x as f32, intersection.y as f32));
                    }
                }
                debug!(
                    "🔪 CALCULATE_REAL_INTERSECTIONS: Total intersections found: {}",
                    intersections.len()
                );
                return intersections;
            } else {
                debug!(
                    "🔪 CALCULATE_REAL_INTERSECTIONS: No paths found for glyph '{}'",
                    current_glyph
                );
            }
        } else {
            debug!("🔪 CALCULATE_REAL_INTERSECTIONS: No current glyph selected");
        }
    } else {
        debug!("🔪 CALCULATE_REAL_INTERSECTIONS: No FontIR state available");
    }

    intersections
}

/// System to handle actual path cutting with FontIR integration
#[allow(clippy::too_many_arguments)]
pub fn handle_fontir_knife_cutting(
    mut fontir_state: Option<ResMut<crate::core::state::FontIRAppState>>,
    mut knife_state: ResMut<KnifeToolState>, // Use main knife state instead of consumer
    mut app_state_changed: EventWriter<crate::editing::selection::events::AppStateChanged>,
    _keyboard: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    // Check if we just finished a cutting gesture
    if mouse_input.just_released(MouseButton::Left) {
        if let Some(ref mut fontir_state) = fontir_state {
            if let Some((start, end)) = knife_state.get_cutting_line() {
                perform_fontir_cut(start, end, fontir_state, &mut app_state_changed);

                // Reset the knife gesture state after successful cut
                knife_state.gesture = KnifeGestureState::Ready;
                knife_state.intersections.clear();
                debug!("🔪 KNIFE CUTTING: Gesture state reset after successful cut");
            }
        }
    }
}

/// Perform cutting with FontIR working copies using glyph-level multi-contour approach
fn perform_fontir_cut(
    start: Vec2,
    end: Vec2,
    fontir_state: &mut crate::core::state::FontIRAppState,
    app_state_changed: &mut EventWriter<crate::editing::selection::events::AppStateChanged>,
) {
    debug!("Performing FontIR knife cut from {:?} to {:?}", start, end);

    // Convert cutting line to kurbo Line
    let cutting_line = kurbo::Line::new(
        kurbo::Point::new(start.x as f64, start.y as f64),
        kurbo::Point::new(end.x as f64, end.y as f64),
    );

    if let Some(ref current_glyph) = fontir_state.current_glyph.clone() {
        // Get or create a working copy using the proper method (like pen and shapes tools)
        if let Some(working_copy) = fontir_state.get_or_create_working_copy(current_glyph) {
            // NEW APPROACH: Glyph-level multi-contour cutting
            match perform_multi_contour_cut(&working_copy.contours, &cutting_line) {
                Ok(new_contours) => {
                    working_copy.contours = new_contours;
                    working_copy.is_dirty = true;
                    app_state_changed.write(crate::editing::selection::events::AppStateChanged);
                    debug!(
                        "FontIR knife cut completed - glyph now has {} contours",
                        working_copy.contours.len()
                    );
                }
                Err(reason) => {
                    debug!("FontIR knife cut failed: {}", reason);
                }
            }
        }
    } else {
        debug!("FontIR knife cut completed - no current glyph selected");
    }
}

/// Perform multi-contour cutting using Runebender's unified approach
/// Treats all segments from all contours as one unified sequence
fn perform_multi_contour_cut(
    contours: &[kurbo::BezPath],
    cutting_line: &kurbo::Line,
) -> Result<Vec<kurbo::BezPath>, String> {
    debug!(
        "🔪 MULTI_CONTOUR_CUT: Analyzing {} contours with Runebender-style unified cutting",
        contours.len()
    );
    debug!(
        "🔪 CUTTING_LINE: from {:?} to {:?}",
        cutting_line.p0, cutting_line.p1
    );

    // Step 1: Find ALL intersections across ALL segments using Runebender's approach
    // This is the key insight: flat_map all segments from all contours
    let mut all_intersections = Vec::new();

    for (contour_idx, contour) in contours.iter().enumerate() {
        debug!(
            "🔪 CONTOUR_{}: Processing contour with {} elements",
            contour_idx,
            contour.elements().len()
        );

        // Debug: Print contour bounds and key points
        let bounds = contour.bounding_box();
        debug!("🔪 CONTOUR_{}: Bounding box: {:?}", contour_idx, bounds);

        let hits = find_path_intersections_with_parameters(contour, cutting_line);
        debug!(
            "🔪 CONTOUR_{}: Found {} intersections",
            contour_idx,
            hits.len()
        );

        // Debug: Print intersection details
        for (hit_idx, hit) in hits.iter().enumerate() {
            debug!(
                "🔪 CONTOUR_{}_HIT_{}: point={:?}, t={:.3}, segment_idx={}",
                contour_idx, hit_idx, hit.point, hit.t, hit.segment_idx
            );
        }

        // Store intersections with their source contour info
        for hit in hits {
            let pos_on_cutting_line = {
                let cutting_dir = (cutting_line.p1 - cutting_line.p0).normalize();
                (hit.point - cutting_line.p0).dot(cutting_dir)
            };

            debug!(
                "🔪 UNIFIED_INTERSECTION: contour={}, point={:?}, pos_on_line={:.3}",
                contour_idx, hit.point, pos_on_cutting_line
            );

            all_intersections.push(UnifiedIntersection {
                contour_idx,
                hit,
                pos_on_cutting_line,
            });
        }
    }

    // Step 2: Check if we have enough intersections
    if all_intersections.len() < 2 {
        let error_msg = format!(
            "Need at least 2 intersections total to cut glyph, found {}. This is likely the root cause of the failure.",
            all_intersections.len()
        );
        debug!("🔪 ERROR: {}", error_msg);
        return Err(error_msg);
    }

    // Step 3: Sort all intersections by position along cutting line (Runebender approach)
    all_intersections.sort_by(|a, b| {
        a.pos_on_cutting_line
            .partial_cmp(&b.pos_on_cutting_line)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    debug!(
        "🔪 SORTED_INTERSECTIONS: {} total intersections sorted by position along cutting line",
        all_intersections.len()
    );

    // Debug: Print sorted intersection order
    for (i, intersection) in all_intersections.iter().enumerate() {
        debug!(
            "🔪 SORTED_{}: contour={}, point={:?}, pos={:.3}",
            i, intersection.contour_idx, intersection.hit.point, intersection.pos_on_cutting_line
        );
    }

    // Step 4: Use Runebender's unified slicing approach
    debug!("🔪 PROCEEDING: to unified path slicing");
    perform_unified_path_slicing(contours, &all_intersections, cutting_line)
}

/// Unified intersection with contour context (simpler than previous complex bridging)
#[derive(Debug, Clone)]
struct UnifiedIntersection {
    contour_idx: usize,
    hit: Hit,
    pos_on_cutting_line: f64,
}

/// Perform unified path slicing inspired by Runebender's approach
fn perform_unified_path_slicing(
    contours: &[kurbo::BezPath],
    sorted_intersections: &[UnifiedIntersection],
    cutting_line: &kurbo::Line,
) -> Result<Vec<kurbo::BezPath>, String> {
    debug!(
        "🔪 UNIFIED_SLICING: Starting with {} sorted intersections across {} contours",
        sorted_intersections.len(),
        contours.len()
    );

    let mut result_contours = Vec::new();
    let mut processed_contours = std::collections::HashSet::new();

    // Group intersections by contour for processing
    let mut contour_intersection_map: std::collections::HashMap<usize, Vec<&UnifiedIntersection>> =
        std::collections::HashMap::new();

    for intersection in sorted_intersections {
        contour_intersection_map
            .entry(intersection.contour_idx)
            .or_default()
            .push(intersection);
    }

    debug!(
        "🔪 INTERSECTION_GROUPING: {} contours have intersections",
        contour_intersection_map.len()
    );

    // Debug: Print intersection distribution
    for (contour_idx, intersections) in &contour_intersection_map {
        debug!(
            "🔪 CONTOUR_{}_INTERSECTIONS: {} intersections found",
            contour_idx,
            intersections.len()
        );
    }

    // Process each contour with its intersections
    for (contour_idx, contour_intersections) in &contour_intersection_map {
        let contour_idx = *contour_idx;
        if processed_contours.contains(&contour_idx) {
            debug!("🔪 SKIP_CONTOUR_{}: Already processed", contour_idx);
            continue;
        }

        let contour = &contours[contour_idx];

        debug!(
            "🔪 PROCESS_CONTOUR_{}: {} intersections",
            contour_idx,
            contour_intersections.len()
        );

        if contour_intersections.len() >= 2 {
            // This contour can be cut normally
            debug!(
                "🔪 CONTOUR_{}: Can be cut normally (≥2 intersections)",
                contour_idx
            );

            let hits: Vec<Hit> = contour_intersections
                .iter()
                .map(|ui| ui.hit.clone())
                .collect();

            debug!(
                "🔪 CONTOUR_{}: Calling slice_path_at_hits with {} hits",
                contour_idx,
                hits.len()
            );
            let sliced_paths = slice_path_at_hits(contour, &hits);

            if sliced_paths.len() > 1 {
                debug!(
                    "🔪 CONTOUR_{}: Successfully sliced into {} pieces",
                    contour_idx,
                    sliced_paths.len()
                );
                result_contours.extend(sliced_paths);
            } else {
                debug!(
                    "🔪 CONTOUR_{}: Slicing failed, keeping original contour",
                    contour_idx
                );
                result_contours.push(contour.clone());
            }
        } else if contour_intersections.len() == 1 {
            // Single intersection - this is the key challenge for cross-contour cutting
            debug!(
                "🔪 CONTOUR_{}: Single intersection - implementing cross-contour connection logic",
                contour_idx
            );

            // Don't process single intersections immediately - collect them for cross-contour bridging
            debug!(
                "🔪 CONTOUR_{}: Deferring single-intersection contour for cross-contour processing",
                contour_idx
            );
        } else {
            // No intersections - keep original
            debug!(
                "🔪 CONTOUR_{}: No intersections, keeping original",
                contour_idx
            );
            result_contours.push(contour.clone());
        }

        processed_contours.insert(contour_idx);
    }

    // CROSS-CONTOUR BRIDGING: Handle single intersections by connecting them across contours
    let single_intersection_contours: Vec<(usize, &UnifiedIntersection)> = contour_intersection_map
        .iter()
        .filter_map(|(contour_idx, intersections)| {
            if intersections.len() == 1 && !processed_contours.contains(contour_idx) {
                Some((*contour_idx, intersections[0]))
            } else {
                None
            }
        })
        .collect();

    debug!(
        "🔪 CROSS_CONTOUR_BRIDGING: Found {} contours with single intersections",
        single_intersection_contours.len()
    );

    if single_intersection_contours.len() >= 2 {
        // We can create cross-contour connections
        debug!(
            "🔪 CROSS_CONTOUR_BRIDGING: Attempting to connect {} single-intersection contours",
            single_intersection_contours.len()
        );

        // Sort single-intersection contours by their position along cutting line
        let mut sorted_single_intersections = single_intersection_contours;
        sorted_single_intersections.sort_by(|a, b| {
            a.1.pos_on_cutting_line
                .partial_cmp(&b.1.pos_on_cutting_line)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Create cross-contour bridges between adjacent single intersections
        let bridged_contours =
            create_cross_contour_bridges(contours, &sorted_single_intersections, cutting_line);

        match bridged_contours {
            Ok(bridges) => {
                debug!(
                    "🔪 CROSS_CONTOUR_SUCCESS: Created {} bridged contours",
                    bridges.len()
                );
                result_contours.extend(bridges);

                // Mark these contours as processed
                for (contour_idx, _) in sorted_single_intersections {
                    processed_contours.insert(contour_idx);
                }
            }
            Err(error) => {
                debug!(
                    "🔪 CROSS_CONTOUR_FAILED: {}, keeping original contours",
                    error
                );

                // Fall back to keeping original contours
                for (contour_idx, _) in sorted_single_intersections {
                    result_contours.push(contours[contour_idx].clone());
                    processed_contours.insert(contour_idx);
                }
            }
        }
    } else if single_intersection_contours.len() == 1 {
        // Only one contour with single intersection - split it at the intersection point
        let (contour_idx, intersection) = single_intersection_contours[0];
        debug!(
            "🔪 SINGLE_SPLIT: Only one contour with single intersection, splitting at intersection point"
        );

        let contour = &contours[contour_idx];
        let split_paths = split_path_at_single_point(contour, &intersection.hit);
        result_contours.extend(split_paths);
        processed_contours.insert(contour_idx);
    }

    // Add any unprocessed contours
    for (idx, contour) in contours.iter().enumerate() {
        if !processed_contours.contains(&idx) {
            debug!("🔪 UNPROCESSED_CONTOUR_{}: Adding to results", idx);
            result_contours.push(contour.clone());
        }
    }

    if result_contours.is_empty() {
        let error_msg = "No valid contours produced after unified slicing".to_string();
        debug!("🔪 ERROR: {}", error_msg);
        Err(error_msg)
    } else {
        debug!(
            "🔪 UNIFIED_SLICING_COMPLETE: {} input -> {} output contours",
            contours.len(),
            result_contours.len()
        );

        // Debug: Print what we're returning
        for (i, result_contour) in result_contours.iter().enumerate() {
            debug!(
                "🔪 RESULT_CONTOUR_{}: {} elements, bounding_box={:?}",
                i,
                result_contour.elements().len(),
                result_contour.bounding_box()
            );
        }

        Ok(result_contours)
    }
}

/// Create cross-contour bridges between contours that have single intersections
/// This is the key insight from Runebender: connect single intersections across different contours
fn create_cross_contour_bridges(
    contours: &[kurbo::BezPath],
    sorted_single_intersections: &[(usize, &UnifiedIntersection)],
    cutting_line: &kurbo::Line,
) -> Result<Vec<kurbo::BezPath>, String> {
    debug!(
        "🔗 BRIDGE_CREATION: Creating bridges between {} single-intersection contours",
        sorted_single_intersections.len()
    );

    if sorted_single_intersections.len() < 2 {
        return Err(
            "Need at least 2 single intersections to create cross-contour bridges".to_string(),
        );
    }

    let mut bridged_contours = Vec::new();

    // Process pairs of adjacent single intersections
    for pair in sorted_single_intersections.windows(2) {
        if pair.len() != 2 {
            continue;
        }

        let (contour_idx_1, intersection_1) = pair[0];
        let (contour_idx_2, intersection_2) = pair[1];

        debug!(
            "🔗 BRIDGE_PAIR: Connecting contour {} (intersection at {:?}) with contour {} (intersection at {:?})",
            contour_idx_1, intersection_1.hit.point,
            contour_idx_2, intersection_2.hit.point
        );

        let contour_1 = &contours[contour_idx_1];
        let contour_2 = &contours[contour_idx_2];

        // Create a bridge between these two contours
        match create_single_cross_contour_bridge(
            contour_1,
            &intersection_1.hit,
            contour_2,
            &intersection_2.hit,
            cutting_line,
        ) {
            Ok(bridge) => {
                debug!(
                    "🔗 BRIDGE_SUCCESS: Created bridge between contours {} and {}",
                    contour_idx_1, contour_idx_2
                );
                bridged_contours.push(bridge);
            }
            Err(error) => {
                debug!(
                    "🔗 BRIDGE_FAILED: Failed to create bridge between contours {} and {}: {}",
                    contour_idx_1, contour_idx_2, error
                );
                return Err(format!("Cross-contour bridge creation failed: {error}"));
            }
        }
    }

    debug!(
        "🔗 BRIDGE_COMPLETE: Successfully created {} cross-contour bridges",
        bridged_contours.len()
    );

    Ok(bridged_contours)
}

/// Create a single bridge connecting two contours at their intersection points
/// This is the core cross-contour connection logic inspired by Runebender's approach
fn create_single_cross_contour_bridge(
    contour_1: &kurbo::BezPath,
    intersection_1: &Hit,
    contour_2: &kurbo::BezPath,
    intersection_2: &Hit,
    _cutting_line: &kurbo::Line,
) -> Result<kurbo::BezPath, String> {
    debug!(
        "🌉 SINGLE_BRIDGE: Creating bridge from {:?} to {:?}",
        intersection_1.point, intersection_2.point
    );

    // Start building the bridged contour
    let mut bridged_path = kurbo::BezPath::new();

    // Start at intersection_1
    bridged_path.move_to(intersection_1.point);

    // Add the portion of contour_1 from intersection_1 around to where it would naturally end
    add_contour_portion_from_intersection(
        &mut bridged_path,
        contour_1,
        intersection_1,
        true, // forward direction
    )?;

    // Add bridge line from contour_1 back to intersection_1, then to intersection_2
    // This creates the "cutting line" portion of the bridge
    bridged_path.line_to(intersection_2.point);

    // Add the portion of contour_2 from intersection_2 around to where it would naturally end
    add_contour_portion_from_intersection(
        &mut bridged_path,
        contour_2,
        intersection_2,
        true, // forward direction
    )?;

    // Close the path to create a complete contour
    bridged_path.close_path();

    debug!(
        "🌉 SINGLE_BRIDGE_SUCCESS: Created bridge with {} elements",
        bridged_path.elements().len()
    );

    Ok(bridged_path)
}

/// Add a portion of a contour to the bridged path, starting from an intersection point
fn add_contour_portion_from_intersection(
    bridged_path: &mut kurbo::BezPath,
    contour: &kurbo::BezPath,
    intersection: &Hit,
    forward: bool,
) -> Result<(), String> {
    let segments = path_to_segments(contour);

    if intersection.segment_idx >= segments.len() {
        return Err(format!(
            "Intersection segment index {} out of bounds (contour has {} segments)",
            intersection.segment_idx,
            segments.len()
        ));
    }

    let mut current_started = true; // bridged_path already has a starting point

    if forward {
        // Add remainder of the intersection segment
        if intersection.segment_idx < segments.len() {
            let segment = &segments[intersection.segment_idx];
            let remainder = extract_subsegment(segment, intersection.t, 1.0);
            add_segment_to_path(bridged_path, &remainder, &mut current_started);
        }

        // Add all subsequent segments
        for segment in segments.iter().skip(intersection.segment_idx + 1) {
            add_segment_to_path(bridged_path, segment, &mut current_started);
        }

        // Add all segments from the beginning up to the intersection
        for segment in segments.iter().take(intersection.segment_idx) {
            add_segment_to_path(bridged_path, segment, &mut current_started);
        }

        // Add beginning of intersection segment up to the intersection point
        if intersection.segment_idx < segments.len() {
            let segment = &segments[intersection.segment_idx];
            let beginning = extract_subsegment(segment, 0.0, intersection.t);
            add_segment_to_path(bridged_path, &beginning, &mut current_started);
        }
    } else {
        // Reverse direction - add beginning of intersection segment first
        if intersection.segment_idx < segments.len() {
            let segment = &segments[intersection.segment_idx];
            let beginning = extract_subsegment(segment, 0.0, intersection.t);
            add_segment_to_path(bridged_path, &beginning, &mut current_started);
        }

        // Add all segments before intersection in reverse order
        for seg_idx in (0..intersection.segment_idx).rev() {
            add_segment_to_path(bridged_path, &segments[seg_idx], &mut current_started);
        }

        // Add all segments after intersection in reverse order
        for seg_idx in ((intersection.segment_idx + 1)..segments.len()).rev() {
            add_segment_to_path(bridged_path, &segments[seg_idx], &mut current_started);
        }

        // Add remainder of intersection segment
        if intersection.segment_idx < segments.len() {
            let segment = &segments[intersection.segment_idx];
            let remainder = extract_subsegment(segment, intersection.t, 1.0);
            add_segment_to_path(bridged_path, &remainder, &mut current_started);
        }
    }

    Ok(())
}

/// Split a path at a single intersection point by breaking it into an open path
fn split_path_at_single_point(path: &kurbo::BezPath, hit: &Hit) -> Vec<kurbo::BezPath> {
    // For single intersection, we'll break the closed path at the intersection point
    // This creates an open path that can be useful for complex multi-contour cuts

    let segments = path_to_segments(path);
    if hit.segment_idx >= segments.len() {
        return vec![path.clone()];
    }

    let mut result_path = kurbo::BezPath::new();

    // Start from the intersection point
    result_path.move_to(hit.point);
    let mut path_started = true;

    // Add the remainder of the segment after the intersection
    if hit.segment_idx < segments.len() {
        let segment = &segments[hit.segment_idx];
        let remainder = extract_subsegment(segment, hit.t, 1.0);
        add_segment_to_path(&mut result_path, &remainder, &mut path_started);
    }

    // Add all subsequent segments
    for (seg_idx, segment) in segments.iter().enumerate() {
        if seg_idx > hit.segment_idx {
            add_segment_to_path(&mut result_path, segment, &mut path_started);
        }
    }

    // Add all segments before the intersection
    for (seg_idx, segment) in segments.iter().enumerate() {
        if seg_idx < hit.segment_idx {
            add_segment_to_path(&mut result_path, segment, &mut path_started);
        }
    }

    // Add the beginning of the segment up to the intersection
    if hit.segment_idx < segments.len() {
        let segment = &segments[hit.segment_idx];
        let beginning = extract_subsegment(segment, 0.0, hit.t);
        add_segment_to_path(&mut result_path, &beginning, &mut path_started);
    }

    // Close the path to maintain proper glyph topology
    if !result_path.elements().is_empty() {
        result_path.close_path();
        vec![result_path]
    } else {
        vec![path.clone()]
    }
}

/// Spawn a batched dashed line mesh for better performance
#[allow(clippy::too_many_arguments)]
fn spawn_dashed_line_batched(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    dash_length: f32,
    gap_length: f32,
    width: f32,
    color: Color,
    z: f32,
) -> Entity {
    use bevy::render::mesh::{Indices, PrimitiveTopology};

    let direction = (end - start).normalize();
    let perpendicular = Vec2::new(-direction.y, direction.x) * width * 0.5;
    let total_length = start.distance(end);
    let segment_length = dash_length + gap_length;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut vertex_count = 0u32;

    // Generate all dash segments in a single mesh
    let mut current_pos = 0.0;
    while current_pos < total_length {
        let dash_start = start + direction * current_pos;
        let dash_end_pos = (current_pos + dash_length).min(total_length);
        let dash_end = start + direction * dash_end_pos;

        // Add vertices for this dash segment
        vertices.push([
            dash_start.x - perpendicular.x,
            dash_start.y - perpendicular.y,
            z,
        ]);
        vertices.push([
            dash_start.x + perpendicular.x,
            dash_start.y + perpendicular.y,
            z,
        ]);
        vertices.push([
            dash_end.x + perpendicular.x,
            dash_end.y + perpendicular.y,
            z,
        ]);
        vertices.push([
            dash_end.x - perpendicular.x,
            dash_end.y - perpendicular.y,
            z,
        ]);

        // Add indices for this dash segment
        indices.extend_from_slice(&[
            vertex_count,
            vertex_count + 1,
            vertex_count + 2,
            vertex_count,
            vertex_count + 2,
            vertex_count + 3,
        ]);

        vertex_count += 4;
        current_pos += segment_length;
    }

    if vertices.is_empty() {
        // Create a dummy entity if no dashes were created
        return commands.spawn_empty().id();
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(ColorMaterial::from(color));

    commands
        .spawn((
            Mesh2d(mesh_handle),
            MeshMaterial2d(material_handle),
            Transform::from_translation(Vec3::new(0.0, 0.0, z)),
        ))
        .id()
}

/// Spawn a point (circle) mesh for the knife tool
fn spawn_knife_point_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec2,
    radius: f32,
    color: Color,
    z: f32,
) -> Entity {
    use bevy::render::mesh::{Indices, PrimitiveTopology};

    // Create circle using triangle fan
    let segments = 16;
    let mut vertices = vec![[position.x, position.y, z]]; // Center vertex
    let mut indices = Vec::new();

    // Create vertices around the circle
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        let x = position.x + radius * angle.cos();
        let y = position.y + radius * angle.sin();
        vertices.push([x, y, z]);
    }

    // Create triangle indices
    for i in 0..segments {
        indices.extend_from_slice(&[0, i + 1, i + 2]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(Indices::U32(indices));

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(ColorMaterial::from(color));

    commands
        .spawn((
            Mesh2d(mesh_handle),
            MeshMaterial2d(material_handle),
            Transform::from_translation(Vec3::new(0.0, 0.0, z)),
        ))
        .id()
}

// ============================================================================
// SIMPLE PATH OPERATIONS FOR KNIFE TOOL
// ============================================================================

/// Find simple intersections between a cutting line and a path
fn find_path_intersections_simple(path: &BezPath, cutting_line: &kurbo::Line) -> Vec<Point> {
    let mut intersections = Vec::new();
    let mut current_point = Point::ZERO;

    for element in path.elements() {
        match element {
            PathEl::MoveTo(pt) => {
                current_point = *pt;
            }
            PathEl::LineTo(end) => {
                let segment = kurbo::Line::new(current_point, *end);
                if let Some(intersection_point) =
                    line_line_intersection_simple(cutting_line, &segment)
                {
                    intersections.push(intersection_point);
                }
                current_point = *end;
            }
            PathEl::CurveTo(c1, c2, end) => {
                let curve = kurbo::CubicBez::new(current_point, *c1, *c2, *end);
                let curve_intersections = curve_line_intersections_simple(&curve, cutting_line);
                intersections.extend(curve_intersections);
                current_point = *end;
            }
            PathEl::QuadTo(c, end) => {
                let curve = kurbo::QuadBez::new(current_point, *c, *end);
                let curve_intersections = quad_line_intersections_simple(&curve, cutting_line);
                intersections.extend(curve_intersections);
                current_point = *end;
            }
            PathEl::ClosePath => {
                if let Some(start_point) = get_path_start_point_inline(path) {
                    let segment = kurbo::Line::new(current_point, start_point);
                    if let Some(intersection_point) =
                        line_line_intersection_simple(cutting_line, &segment)
                    {
                        intersections.push(intersection_point);
                    }
                }
            }
        }
    }

    intersections.dedup_by(|a, b| a.distance(*b) < 5.0);
    intersections
}

/// Hit structure to track intersection details
#[derive(Debug, Clone)]
struct Hit {
    pub point: Point,
    pub t: f64,
    pub segment_idx: usize,
}

/// Find intersections with parameter information for accurate slicing
fn find_path_intersections_with_parameters(path: &BezPath, cutting_line: &kurbo::Line) -> Vec<Hit> {
    let mut hits = Vec::new();
    let mut current_point = Point::ZERO;
    let mut segment_idx = 0;

    debug!(
        "🔍 INTERSECTION_SEARCH: Starting search through {} path elements",
        path.elements().len()
    );
    debug!(
        "🔍 CUTTING_LINE: from {:?} to {:?}",
        cutting_line.p0, cutting_line.p1
    );

    for (element_idx, element) in path.elements().iter().enumerate() {
        match element {
            PathEl::MoveTo(pt) => {
                current_point = *pt;
                debug!("🔍 ELEMENT_{}: MoveTo({:?})", element_idx, pt);
            }
            PathEl::LineTo(end) => {
                let segment = kurbo::Line::new(current_point, *end);
                debug!(
                    "🔍 ELEMENT_{}: LineTo from {:?} to {:?}",
                    element_idx, current_point, end
                );

                if let Some(intersection) =
                    line_line_intersection_with_parameter(&segment, cutting_line)
                {
                    debug!(
                        "🔍 LINE_INTERSECTION: Found at {:?}, t={:.3}",
                        intersection.0, intersection.1
                    );
                    hits.push(Hit {
                        point: intersection.0,
                        t: intersection.1,
                        segment_idx,
                    });
                } else {
                    debug!("🔍 LINE_NO_INTERSECTION: Line segment does not intersect cutting line");
                }
                current_point = *end;
                segment_idx += 1;
            }
            PathEl::CurveTo(c1, c2, end) => {
                let curve = kurbo::CubicBez::new(current_point, *c1, *c2, *end);
                debug!(
                    "🔍 ELEMENT_{}: CurveTo from {:?} via {:?},{:?} to {:?}",
                    element_idx, current_point, c1, c2, end
                );

                let curve_hits =
                    curve_line_intersections_with_parameters(&curve, cutting_line, segment_idx);
                debug!(
                    "🔍 CURVE_INTERSECTIONS: Found {} intersections",
                    curve_hits.len()
                );

                for (i, hit) in curve_hits.iter().enumerate() {
                    debug!("🔍 CURVE_HIT_{}: point={:?}, t={:.3}", i, hit.point, hit.t);
                }

                hits.extend(curve_hits);
                current_point = *end;
                segment_idx += 1;
            }
            PathEl::QuadTo(c, end) => {
                let curve = kurbo::QuadBez::new(current_point, *c, *end);
                debug!(
                    "🔍 ELEMENT_{}: QuadTo from {:?} via {:?} to {:?}",
                    element_idx, current_point, c, end
                );

                let curve_hits =
                    quad_line_intersections_with_parameters(&curve, cutting_line, segment_idx);
                debug!(
                    "🔍 QUAD_INTERSECTIONS: Found {} intersections",
                    curve_hits.len()
                );

                hits.extend(curve_hits);
                current_point = *end;
                segment_idx += 1;
            }
            PathEl::ClosePath => {
                if let Some(start_point) = get_path_start_point_inline(path) {
                    let segment = kurbo::Line::new(current_point, start_point);
                    debug!(
                        "🔍 ELEMENT_{}: ClosePath from {:?} to {:?}",
                        element_idx, current_point, start_point
                    );

                    if let Some(intersection) =
                        line_line_intersection_with_parameter(&segment, cutting_line)
                    {
                        debug!(
                            "🔍 CLOSE_INTERSECTION: Found at {:?}, t={:.3}",
                            intersection.0, intersection.1
                        );
                        hits.push(Hit {
                            point: intersection.0,
                            t: intersection.1,
                            segment_idx,
                        });
                    } else {
                        debug!("🔍 CLOSE_NO_INTERSECTION: ClosePath segment does not intersect cutting line");
                    }
                }
                segment_idx += 1;
            }
        }
    }

    debug!(
        "🔍 RAW_HITS: Found {} raw intersections before deduplication",
        hits.len()
    );

    // Remove duplicate hits with better tolerance
    let original_count = hits.len();
    hits.dedup_by(|a, b| a.point.distance(b.point) < 1.0);
    if hits.len() < original_count {
        debug!(
            "🔍 DEDUPLICATION: Removed {} duplicates, {} remaining",
            original_count - hits.len(),
            hits.len()
        );
    }

    // Sort hits by their position along the knife cutting line for better ordering
    hits.sort_by(|a, b| {
        // Calculate position along cutting line for better ordering
        let cutting_dir = (cutting_line.p1 - cutting_line.p0).normalize();
        let a_proj = (a.point - cutting_line.p0).dot(cutting_dir);
        let b_proj = (b.point - cutting_line.p0).dot(cutting_dir);
        a_proj
            .partial_cmp(&b_proj)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    debug!(
        "🔍 FINAL_HITS: Returning {} sorted intersections",
        hits.len()
    );
    for (i, hit) in hits.iter().enumerate() {
        debug!(
            "🔍 FINAL_HIT_{}: point={:?}, t={:.3}, segment_idx={}",
            i, hit.point, hit.t, hit.segment_idx
        );
    }

    hits
}

/// Slice path at specific hit points using Runebender's recursive algorithm
/// This can handle any number of intersection points, minimum 2 per contour
fn slice_path_at_hits(path: &BezPath, hits: &[Hit]) -> Vec<BezPath> {
    if hits.is_empty() {
        return vec![path.clone()];
    }

    if hits.len() < 2 {
        debug!(
            "Individual contour requires at least 2 intersection points to cut, found {}",
            hits.len()
        );
        return vec![path.clone()];
    }

    // Sort hits by their position along the original path
    let mut sorted_hits = hits.to_vec();
    sorted_hits.sort_by(|a, b| {
        a.segment_idx
            .cmp(&b.segment_idx)
            .then(a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal))
    });

    debug!(
        "Cutting path at {} intersection points using recursive algorithm",
        sorted_hits.len()
    );

    // Use recursive slicing algorithm (similar to Runebender)
    slice_path_recursively(path, &sorted_hits)
}

/// Recursive path slicing algorithm based on Runebender's approach
/// This can handle any number of intersections by recursively slicing paths
fn slice_path_recursively(path: &BezPath, hits: &[Hit]) -> Vec<BezPath> {
    if hits.len() < 2 {
        return vec![path.clone()];
    }

    if hits.len() == 2 {
        // Base case: exactly 2 intersections - slice normally
        return slice_path_with_two_hits(path, &hits[0], &hits[1]);
    }

    // More than 2 intersections - recursive approach
    // Take first two intersections and slice, then recursively slice the remaining parts
    let first_hit = &hits[0];
    let second_hit = &hits[1];

    // Slice path at first two intersections
    let sliced_paths = slice_path_with_two_hits(path, first_hit, second_hit);

    if sliced_paths.len() != 2 {
        // If slicing failed, try with remaining intersections
        return slice_path_recursively(path, &hits[1..]);
    }

    let mut result_paths = Vec::new();

    // Check if remaining intersections affect either of the sliced paths
    let remaining_hits = &hits[2..];

    for sliced_path in sliced_paths {
        // Find which remaining hits belong to this path segment
        let relevant_hits: Vec<Hit> = remaining_hits
            .iter()
            .filter(|hit| path_contains_hit(&sliced_path, hit))
            .cloned()
            .collect();

        if relevant_hits.is_empty() {
            // No more intersections in this path
            result_paths.push(sliced_path);
        } else {
            // Recursively slice this path with its relevant intersections
            let further_sliced = slice_path_recursively(&sliced_path, &relevant_hits);
            result_paths.extend(further_sliced);
        }
    }

    debug!(
        "Recursive slicing: {} intersections -> {} final paths",
        hits.len(),
        result_paths.len()
    );

    result_paths
}

/// Check if a path segment likely contains a hit point (rough approximation)
fn path_contains_hit(path: &BezPath, hit: &Hit) -> bool {
    // Simple heuristic: check if the hit point is close to any part of the path
    let hit_point = hit.point;
    const TOLERANCE: f64 = 5.0; // Tighter tolerance for better accuracy

    let mut current_point = kurbo::Point::ZERO;

    for element in path.elements() {
        match element {
            PathEl::MoveTo(pt) => {
                current_point = *pt;
                if hit_point.distance(current_point) < TOLERANCE {
                    return true;
                }
            }
            PathEl::LineTo(end) => {
                let line = kurbo::Line::new(current_point, *end);
                let closest = line.nearest(hit_point, 0.01);
                let closest_point = line.eval(closest.t);
                if hit_point.distance(closest_point) < TOLERANCE {
                    return true;
                }
                current_point = *end;
            }
            PathEl::CurveTo(c1, c2, end) => {
                let curve = kurbo::CubicBez::new(current_point, *c1, *c2, *end);
                let closest = curve.nearest(hit_point, 0.01);
                let closest_point = curve.eval(closest.t);
                if hit_point.distance(closest_point) < TOLERANCE {
                    return true;
                }
                current_point = *end;
            }
            PathEl::QuadTo(c, end) => {
                let curve = kurbo::QuadBez::new(current_point, *c, *end);
                let closest = curve.nearest(hit_point, 0.01);
                let closest_point = curve.eval(closest.t);
                if hit_point.distance(closest_point) < TOLERANCE {
                    return true;
                }
                current_point = *end;
            }
            PathEl::ClosePath => {
                if let Some(start) = get_path_start_point_inline(path) {
                    let line = kurbo::Line::new(current_point, start);
                    let closest = line.nearest(hit_point, 0.01);
                    let closest_point = line.eval(closest.t);
                    if hit_point.distance(closest_point) < TOLERANCE {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Slice a path with exactly two intersections (base case for recursion)
fn slice_path_with_two_hits(path: &BezPath, first_hit: &Hit, second_hit: &Hit) -> Vec<BezPath> {
    // Convert path to segments for easier processing
    let segments = path_to_segments(path);

    // Create two complete closed contours
    let mut path1 = BezPath::new(); // Path from first hit to second hit
    let mut path2 = BezPath::new(); // Path from second hit back to first hit

    let mut _path1_started = false;
    let mut _path2_started = false;

    // Build path1: from first intersection to second intersection
    path1.move_to(first_hit.point);
    _path1_started = true;

    for (seg_idx, segment) in segments.iter().enumerate() {
        if seg_idx < first_hit.segment_idx {
            // Before first intersection - ignore
            continue;
        } else if seg_idx == first_hit.segment_idx {
            // Segment containing first intersection
            if seg_idx == second_hit.segment_idx {
                // Both intersections in same segment
                let subseg = extract_subsegment(segment, first_hit.t, second_hit.t);
                add_segment_to_path(&mut path1, &subseg, &mut _path1_started);
            } else {
                // Start from first intersection to end of segment
                let subseg = extract_subsegment(segment, first_hit.t, 1.0);
                add_segment_to_path(&mut path1, &subseg, &mut _path1_started);
            }
        } else if seg_idx > first_hit.segment_idx && seg_idx < second_hit.segment_idx {
            // Between intersections - add entire segment
            add_segment_to_path(&mut path1, segment, &mut _path1_started);
        } else if seg_idx == second_hit.segment_idx {
            // Segment containing second intersection - end here
            let subseg = extract_subsegment(segment, 0.0, second_hit.t);
            add_segment_to_path(&mut path1, &subseg, &mut _path1_started);
            break;
        }
    }

    // Close path1 - the cutting line is implicit in the close_path() operation
    if !path1.elements().is_empty() {
        path1.close_path();
    }

    // Build path2: from second intersection, around the rest, back to first intersection
    // This path takes the "long way around" the original contour
    path2.move_to(second_hit.point);
    _path2_started = true;

    // Start from the second intersection and go to the end of that segment
    if second_hit.segment_idx < segments.len() && first_hit.segment_idx != second_hit.segment_idx {
        let segment = &segments[second_hit.segment_idx];
        let subseg = extract_subsegment(segment, second_hit.t, 1.0);
        add_segment_to_path(&mut path2, &subseg, &mut _path2_started);
    }

    // Add all segments after the second intersection
    for (seg_idx, segment) in segments.iter().enumerate() {
        if seg_idx > second_hit.segment_idx {
            add_segment_to_path(&mut path2, segment, &mut _path2_started);
        }
    }

    // Add all segments before the first intersection (completing the loop around)
    for (seg_idx, segment) in segments.iter().enumerate() {
        if seg_idx < first_hit.segment_idx {
            add_segment_to_path(&mut path2, segment, &mut _path2_started);
        }
    }

    // Add the final segment up to the first intersection
    if first_hit.segment_idx < segments.len() && first_hit.segment_idx != second_hit.segment_idx {
        let segment = &segments[first_hit.segment_idx];
        let subseg = extract_subsegment(segment, 0.0, first_hit.t);
        add_segment_to_path(&mut path2, &subseg, &mut _path2_started);
    }

    // Close path2 - the cutting line is implicit in the close_path() operation
    if !path2.elements().is_empty() {
        path2.close_path();
    }

    let mut result_paths = Vec::new();
    if !path1.elements().is_empty() {
        result_paths.push(path1);
    }
    if !path2.elements().is_empty() {
        result_paths.push(path2);
    }

    debug!(
        "Successfully split closed contour into {} closed contours",
        result_paths.len()
    );
    result_paths
}

/// Represent a path segment for processing
#[derive(Debug, Clone)]
enum PathSegment {
    Line(kurbo::Line),
    Quad(kurbo::QuadBez),
    Cubic(kurbo::CubicBez),
}

/// Convert a BezPath to a vector of segments
fn path_to_segments(path: &BezPath) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut current_point = Point::ZERO;
    let mut start_point = Point::ZERO;

    for element in path.elements() {
        match element {
            PathEl::MoveTo(pt) => {
                current_point = *pt;
                start_point = *pt;
            }
            PathEl::LineTo(end) => {
                segments.push(PathSegment::Line(kurbo::Line::new(current_point, *end)));
                current_point = *end;
            }
            PathEl::CurveTo(c1, c2, end) => {
                segments.push(PathSegment::Cubic(kurbo::CubicBez::new(
                    current_point,
                    *c1,
                    *c2,
                    *end,
                )));
                current_point = *end;
            }
            PathEl::QuadTo(c, end) => {
                segments.push(PathSegment::Quad(kurbo::QuadBez::new(
                    current_point,
                    *c,
                    *end,
                )));
                current_point = *end;
            }
            PathEl::ClosePath => {
                // Add a line back to the start if needed
                if current_point.distance(start_point) > 1e-6 {
                    segments.push(PathSegment::Line(kurbo::Line::new(
                        current_point,
                        start_point,
                    )));
                }
            }
        }
    }

    segments
}

/// Extract a subsegment from a PathSegment
fn extract_subsegment(segment: &PathSegment, t0: f64, t1: f64) -> PathSegment {
    match segment {
        PathSegment::Line(line) => {
            let p0 = line.eval(t0);
            let p1 = line.eval(t1);
            PathSegment::Line(kurbo::Line::new(p0, p1))
        }
        PathSegment::Cubic(cubic) => {
            // Use kurbo's subsegment method for cubic curves
            let subseg = cubic.subsegment(t0..t1);
            PathSegment::Cubic(subseg)
        }
        PathSegment::Quad(quad) => {
            // Use kurbo's subsegment method for quadratic curves
            let subseg = quad.subsegment(t0..t1);
            PathSegment::Quad(subseg)
        }
    }
}

/// Add a segment to a BezPath
fn add_segment_to_path(path: &mut BezPath, segment: &PathSegment, started: &mut bool) {
    match segment {
        PathSegment::Line(line) => {
            if !*started {
                path.move_to(line.p0);
                *started = true;
            }
            path.line_to(line.p1);
        }
        PathSegment::Cubic(cubic) => {
            if !*started {
                path.move_to(cubic.p0);
                *started = true;
            }
            path.curve_to(cubic.p1, cubic.p2, cubic.p3);
        }
        PathSegment::Quad(quad) => {
            if !*started {
                path.move_to(quad.p0);
                *started = true;
            }
            path.quad_to(quad.p1, quad.p2);
        }
    }
}

/// Line-line intersection with parameter information
fn line_line_intersection_with_parameter(
    line1: &kurbo::Line,
    line2: &kurbo::Line,
) -> Option<(Point, f64)> {
    let p1 = line1.p0;
    let p2 = line1.p1;
    let p3 = line2.p0;
    let p4 = line2.p1;

    let denom = (p1.x - p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x - p4.x);

    if denom.abs() < 1e-10 {
        return None;
    }

    let t = ((p1.x - p3.x) * (p3.y - p4.y) - (p1.y - p3.y) * (p3.x - p4.x)) / denom;
    let u = -((p1.x - p2.x) * (p1.y - p3.y) - (p1.y - p2.y) * (p1.x - p3.x)) / denom;

    if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&t) {
        let point = Point::new(p1.x + t * (p2.x - p1.x), p1.y + t * (p2.y - p1.y));
        Some((point, t))
    } else {
        None
    }
}

/// Curve-line intersections with parameter information
fn curve_line_intersections_with_parameters(
    curve: &kurbo::CubicBez,
    line: &kurbo::Line,
    segment_idx: usize,
) -> Vec<Hit> {
    let mut hits = Vec::new();
    let curve_seg = kurbo::PathSeg::Cubic(*curve);
    let curve_intersections = curve_seg.intersect_line(*line);

    for intersection in curve_intersections {
        // Calculate the intersection point using segment_t
        let point = curve.eval(intersection.segment_t);
        hits.push(Hit {
            point,
            t: intersection.segment_t,
            segment_idx,
        });
    }

    hits
}

/// Quad-line intersections with parameter information
fn quad_line_intersections_with_parameters(
    curve: &kurbo::QuadBez,
    line: &kurbo::Line,
    segment_idx: usize,
) -> Vec<Hit> {
    let mut hits = Vec::new();
    let curve_seg = kurbo::PathSeg::Quad(*curve);
    let curve_intersections = curve_seg.intersect_line(*line);

    for intersection in curve_intersections {
        // Calculate the intersection point using segment_t
        let point = curve.eval(intersection.segment_t);
        hits.push(Hit {
            point,
            t: intersection.segment_t,
            segment_idx,
        });
    }

    hits
}

fn line_line_intersection_simple(line1: &kurbo::Line, line2: &kurbo::Line) -> Option<Point> {
    let p1 = line1.p0;
    let p2 = line1.p1;
    let p3 = line2.p0;
    let p4 = line2.p1;

    let denom = (p1.x - p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x - p4.x);

    if denom.abs() < 1e-10 {
        return None;
    }

    let t = ((p1.x - p3.x) * (p3.y - p4.y) - (p1.y - p3.y) * (p3.x - p4.x)) / denom;
    let u = -((p1.x - p2.x) * (p1.y - p3.y) - (p1.y - p2.y) * (p1.x - p3.x)) / denom;

    // IMPORTANT: Check that both t and u are within [0,1] for line segment intersection
    // t is the parameter for line1 (cutting line), u is for line2 (path segment)
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some(Point::new(
            p1.x + t * (p2.x - p1.x),
            p1.y + t * (p2.y - p1.y),
        ))
    } else {
        None
    }
}

fn curve_line_intersections_simple(curve: &kurbo::CubicBez, line: &kurbo::Line) -> Vec<Point> {
    // Use kurbo's built-in intersection method for accurate mathematical intersection
    let mut intersections = Vec::new();

    // Convert curve to PathSeg for intersection testing
    let curve_seg = kurbo::PathSeg::Cubic(*curve);

    // Find intersections using kurbo's accurate intersection algorithm
    let curve_intersections = curve_seg.intersect_line(*line);

    for intersection in curve_intersections {
        // Calculate the intersection point using segment_t
        let point = curve.eval(intersection.segment_t);

        // IMPORTANT: Check if intersection point lies within the knife line segment
        // kurbo finds intersections with infinite line, but we want line segment only
        if point_lies_on_line_segment(point, line) {
            intersections.push(point);
        }
    }

    // Remove duplicates with smaller tolerance for accuracy
    intersections.dedup_by(|a, b| a.distance(*b) < 0.5);
    intersections
}

fn quad_line_intersections_simple(curve: &kurbo::QuadBez, line: &kurbo::Line) -> Vec<Point> {
    // Use kurbo's built-in intersection method for accurate mathematical intersection
    let mut intersections = Vec::new();

    // Convert curve to PathSeg for intersection testing
    let curve_seg = kurbo::PathSeg::Quad(*curve);

    // Find intersections using kurbo's accurate intersection algorithm
    let curve_intersections = curve_seg.intersect_line(*line);

    for intersection in curve_intersections {
        // Calculate the intersection point using segment_t
        let point = curve.eval(intersection.segment_t);

        // IMPORTANT: Check if intersection point lies within the knife line segment
        // kurbo finds intersections with infinite line, but we want line segment only
        if point_lies_on_line_segment(point, line) {
            intersections.push(point);
        }
    }

    // Remove duplicates with smaller tolerance for accuracy
    intersections.dedup_by(|a, b| a.distance(*b) < 0.5);
    intersections
}

/// Check if a point lies on a line segment (not just the infinite line)
fn point_lies_on_line_segment(point: Point, line: &kurbo::Line) -> bool {
    // Calculate the parameter t for the point on the line
    let dx = line.p1.x - line.p0.x;
    let dy = line.p1.y - line.p0.y;

    // Handle near-vertical and near-horizontal lines appropriately
    let t = if dx.abs() > dy.abs() {
        // Use x coordinate for parameter calculation
        if dx.abs() < 1e-10 {
            // Vertical line
            return (point.x - line.p0.x).abs() < 1e-6
                && point.y >= line.p0.y.min(line.p1.y) - 1e-6
                && point.y <= line.p0.y.max(line.p1.y) + 1e-6;
        }
        (point.x - line.p0.x) / dx
    } else {
        // Use y coordinate for parameter calculation
        if dy.abs() < 1e-10 {
            // Horizontal line
            return (point.y - line.p0.y).abs() < 1e-6
                && point.x >= line.p0.x.min(line.p1.x) - 1e-6
                && point.x <= line.p0.x.max(line.p1.x) + 1e-6;
        }
        (point.y - line.p0.y) / dy
    };

    // Check if t is within [0, 1] (point lies on line segment)
    (0.0..=1.0).contains(&t)
}

fn get_path_start_point_inline(path: &BezPath) -> Option<Point> {
    for element in path.elements() {
        if let PathEl::MoveTo(pt) = element {
            return Some(*pt);
        }
    }
    None
}
