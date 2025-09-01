//! Knife Tool - Path cutting and slicing tool
//!
//! This tool allows users to cut paths by drawing a line across them.
//! The tool shows a preview of the cutting line and intersection points.

#![allow(unused_variables)]

use crate::core::state::AppState;
#[allow(unused_imports)]
use crate::core::state::GlyphNavigation;
use crate::editing::selection::events::AppStateChanged;
use crate::ui::theme::*;
use crate::ui::edit_mode_toolbar::{EditTool, ToolRegistry};
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};
use kurbo::{BezPath, ParamCurve, PathEl, Point};

// Simple path operations are defined at the end of this file

/// Resource to track if knife mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct KnifeModeActive(pub bool);

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
        info!("🔪 KNIFE_TOOL: update() called - setting knife mode active and input mode to Knife");
        commands.insert_resource(KnifeModeActive(true));
        commands.insert_resource(crate::core::io::input::InputMode::Knife);
    }

    fn on_enter(&self) {
        info!("Entered Knife tool");
    }

    fn on_exit(&self) {
        info!("Exited Knife tool");
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
    app_state_changed: EventWriter<crate::editing::selection::events::AppStateChanged>,
    // Query for active sort to get its position
    active_sort_query: Query<(Entity, &crate::editing::sort::Sort, &Transform), With<crate::editing::sort::ActiveSort>>,
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
    if !knife_is_active || active_sort.is_none() {
        if knife_is_active && active_sort.is_none() {
            // Only show this message when knife tool is actually trying to be used
            if mouse_button_input.just_pressed(MouseButton::Left) {
                info!("🔪 Knife tool: Cannot cut without an active sort. Please select a glyph first.");
            }
        }
        return;
    }

    let (_sort_entity, _sort, sort_transform) = active_sort.unwrap();
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
            info!("🔪 KNIFE_DEBUG: Started cutting at sort-relative {:?}, world {:?}, sort pos {:?}", 
                  sort_relative_position, world_position, sort_position);
        }

        // Handle mouse movement during cutting
        if let KnifeGestureState::Cutting { start, .. } = knife_state.gesture {
            knife_state.gesture = KnifeGestureState::Cutting {
                start,
                current: sort_relative_position,
            };

            // Intersections will be calculated by the render system
            debug!("🔪 KNIFE_DEBUG: Dragging to sort-relative {:?}", sort_relative_position);
        }

        // Handle mouse button release
        if mouse_button_input.just_released(MouseButton::Left) {
            if let Some((_start, _end)) = knife_state.get_cutting_line() {
                // The actual cutting is handled by handle_fontir_knife_cutting system
                info!("🔪 KNIFE_DEBUG: Mouse released - cutting will be handled by FontIR system");
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
        info!("Cancelled knife cut");
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
        info!("🔪 MANAGE_KNIFE_MODE: Activating knife mode");
    } else if !is_knife_active && current_mode {
        // Knife tool is not active but mode is set - deactivate it
        *knife_state = KnifeToolState::new();
        commands.insert_resource(KnifeModeActive(false));
        info!("🔪 MANAGE_KNIFE_MODE: Deactivating knife mode");
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
    active_sort_query: Query<(Entity, &crate::editing::sort::Sort, &Transform), With<crate::editing::sort::ActiveSort>>,
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
        let direction = (world_end - world_start).normalize();
        let total_length = world_start.distance(world_end);
        let dash_length = theme.theme().knife_dash_length() * camera_scale.scale_factor;
        let gap_length = theme.theme().knife_gap_length() * camera_scale.scale_factor;
        let segment_length = dash_length + gap_length;
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

        // Draw start point (green circle) - use world coordinates
        let start_color = theme.theme().knife_start_point_color();
        let point_size = camera_scale.adjusted_point_size(4.0);
        let point_entity = spawn_knife_point_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            world_start,
            point_size,
            start_color,
            19.0, // z-order above line but below intersection points
        );
        knife_entities.push(point_entity);

        // Draw end point (orange cross) - use world coordinates
        let end_color = theme.theme().action_color();
        let cross_size = theme.theme().knife_cross_size() * camera_scale.scale_factor;
        let cross_width = camera_scale.adjusted_line_width();

        // Horizontal line of cross
        let cross_h_entity = spawn_knife_line_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec2::new(world_end.x - cross_size, world_end.y),
            Vec2::new(world_end.x + cross_size, world_end.y),
            cross_width,
            end_color,
            19.0, // z-order above line but below intersection points
        );
        knife_entities.push(cross_h_entity);

        // Vertical line of cross
        let cross_v_entity = spawn_knife_line_mesh(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec2::new(world_end.x, world_end.y - cross_size),
            Vec2::new(world_end.x, world_end.y + cross_size),
            cross_width,
            end_color,
            19.0, // z-order above line but below intersection points
        );
        knife_entities.push(cross_v_entity);

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

        let intersection_color = theme.theme().knife_intersection_color();

        for &intersection in &calc_cache.cached_intersections {
            // Convert intersection from sort-relative to world coordinates
            let world_intersection = intersection + sort_position;
            
            let cross_size = theme.theme().knife_cross_size() * camera_scale.scale_factor;
            let cross_width = camera_scale.adjusted_line_width();

            // Create X mark with two diagonal lines (use world coordinates)
            // Diagonal line from top-left to bottom-right
            let diagonal1_entity = spawn_knife_line_mesh(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(world_intersection.x - cross_size, world_intersection.y + cross_size),
                Vec2::new(world_intersection.x + cross_size, world_intersection.y - cross_size),
                cross_width,
                intersection_color,
                20.0, // z-order above everything else
            );
            knife_entities.push(diagonal1_entity);

            // Diagonal line from top-right to bottom-left
            let diagonal2_entity = spawn_knife_line_mesh(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec2::new(world_intersection.x + cross_size, world_intersection.y + cross_size),
                Vec2::new(world_intersection.x - cross_size, world_intersection.y - cross_size),
                cross_width,
                intersection_color,
                20.0, // z-order above everything else
            );
            knife_entities.push(diagonal2_entity);
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
                info!(
                    "🔪 CALCULATE_REAL_INTERSECTIONS: No paths found for glyph '{}'",
                    current_glyph
                );
            }
        } else {
            info!("🔪 CALCULATE_REAL_INTERSECTIONS: No current glyph selected");
        }
    } else {
        info!("🔪 CALCULATE_REAL_INTERSECTIONS: No FontIR state available");
    }

    intersections
}


/// System to handle actual path cutting with FontIR integration
#[allow(clippy::too_many_arguments)]
pub fn handle_fontir_knife_cutting(
    mut fontir_state: Option<ResMut<crate::core::state::FontIRAppState>>,
    mut knife_consumer: ResMut<crate::systems::input_consumer::KnifeInputConsumer>,
    mut app_state_changed: EventWriter<crate::editing::selection::events::AppStateChanged>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    // Check if we just finished a cutting gesture
    if mouse_input.just_released(MouseButton::Left) {
        if let Some(ref mut fontir_state) = fontir_state {
            if let Some((start, end)) = knife_consumer.get_cutting_line() {
                perform_fontir_cut(start, end, fontir_state, &mut app_state_changed);

                // Reset the knife gesture state after successful cut
                knife_consumer.gesture = crate::systems::input_consumer::KnifeGestureState::Ready;
                knife_consumer.intersections.clear();
                info!("🔪 KNIFE CUTTING: Gesture state reset after successful cut");
            }
        }
    }
}

/// Perform cutting with FontIR working copies
fn perform_fontir_cut(
    start: Vec2,
    end: Vec2,
    fontir_state: &mut crate::core::state::FontIRAppState,
    app_state_changed: &mut EventWriter<crate::editing::selection::events::AppStateChanged>,
) {
    info!("Performing FontIR knife cut from {:?} to {:?}", start, end);

    // Convert cutting line to kurbo Line
    let cutting_line = kurbo::Line::new(
        kurbo::Point::new(start.x as f64, start.y as f64),
        kurbo::Point::new(end.x as f64, end.y as f64),
    );

    if let Some(ref current_glyph) = fontir_state.current_glyph.clone() {
        // Get or create a working copy using the proper method (like pen and shapes tools)
        if let Some(working_copy) = fontir_state.get_or_create_working_copy(current_glyph) {
            let mut new_contours = Vec::new();
            let mut any_cuts_made = false;

            for contour in &working_copy.contours {
                // Find intersections with the cutting line
                let hits = find_path_intersections_with_parameters(contour, &cutting_line);

                if !hits.is_empty() {
                    // Path has intersections - slice it
                    let sliced_paths = slice_path_at_hits(contour, &hits);

                    if sliced_paths.len() > 1 {
                        info!(
                            "Cut contour into {} pieces at {} intersection points",
                            sliced_paths.len(),
                            hits.len()
                        );

                        // Add all the sliced paths as new contours
                        new_contours.extend(sliced_paths);
                        any_cuts_made = true;
                    } else {
                        // Shouldn't happen, but keep original if slicing failed
                        new_contours.push(contour.clone());
                    }
                } else {
                    // Path was not cut, keep original
                    new_contours.push(contour.clone());
                }
            }

            if any_cuts_made {
                // Replace the contours with the cut versions
                working_copy.contours = new_contours;
                working_copy.is_dirty = true;
                app_state_changed.write(crate::editing::selection::events::AppStateChanged);
                info!(
                    "FontIR knife cut completed - glyph now has {} contours",
                    working_copy.contours.len()
                );
            } else {
                info!("FontIR knife cut completed - no intersections found");
            }
        }
    } else {
        info!("FontIR knife cut completed - no current glyph selected");
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

/// Spawn a line mesh for the knife tool
#[allow(clippy::too_many_arguments)]
fn spawn_knife_line_mesh(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    start: Vec2,
    end: Vec2,
    width: f32,
    color: Color,
    z: f32,
) -> Entity {
    use bevy::render::mesh::{Indices, PrimitiveTopology};

    let direction = (end - start).normalize();
    let perpendicular = Vec2::new(-direction.y, direction.x) * width * 0.5;

    // Create quad vertices for the line
    let vertices = vec![
        [start.x - perpendicular.x, start.y - perpendicular.y, z],
        [start.x + perpendicular.x, start.y + perpendicular.y, z],
        [end.x + perpendicular.x, end.y + perpendicular.y, z],
        [end.x - perpendicular.x, end.y - perpendicular.y, z],
    ];

    let indices = vec![0, 1, 2, 0, 2, 3];

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

/// Slice a path with a cutting line, returning new path segments
fn _slice_path_with_line_simple(path: &BezPath, cutting_line: &kurbo::Line) -> Vec<BezPath> {
    let hits = find_path_intersections_with_parameters(path, cutting_line);

    if hits.is_empty() {
        return vec![path.clone()];
    }

    info!("Found {} intersections, slicing path", hits.len());

    // Sort hits by segment index and parameter t
    let mut sorted_hits = hits;
    sorted_hits.sort_by(|a, b| {
        a.segment_idx
            .cmp(&b.segment_idx)
            .then(a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal))
    });

    // Slice the path at intersection points
    slice_path_at_hits(path, &sorted_hits)
}

/// Find intersections with parameter information for accurate slicing
fn find_path_intersections_with_parameters(path: &BezPath, cutting_line: &kurbo::Line) -> Vec<Hit> {
    let mut hits = Vec::new();
    let mut current_point = Point::ZERO;
    let mut segment_idx = 0;

    for element in path.elements() {
        match element {
            PathEl::MoveTo(pt) => {
                current_point = *pt;
            }
            PathEl::LineTo(end) => {
                let segment = kurbo::Line::new(current_point, *end);
                if let Some(intersection) =
                    line_line_intersection_with_parameter(&segment, cutting_line)
                {
                    hits.push(Hit {
                        point: intersection.0,
                        t: intersection.1,
                        segment_idx,
                    });
                }
                current_point = *end;
                segment_idx += 1;
            }
            PathEl::CurveTo(c1, c2, end) => {
                let curve = kurbo::CubicBez::new(current_point, *c1, *c2, *end);
                let curve_hits =
                    curve_line_intersections_with_parameters(&curve, cutting_line, segment_idx);
                hits.extend(curve_hits);
                current_point = *end;
                segment_idx += 1;
            }
            PathEl::QuadTo(c, end) => {
                let curve = kurbo::QuadBez::new(current_point, *c, *end);
                let curve_hits =
                    quad_line_intersections_with_parameters(&curve, cutting_line, segment_idx);
                hits.extend(curve_hits);
                current_point = *end;
                segment_idx += 1;
            }
            PathEl::ClosePath => {
                if let Some(start_point) = get_path_start_point_inline(path) {
                    let segment = kurbo::Line::new(current_point, start_point);
                    if let Some(intersection) =
                        line_line_intersection_with_parameter(&segment, cutting_line)
                    {
                        hits.push(Hit {
                            point: intersection.0,
                            t: intersection.1,
                            segment_idx,
                        });
                    }
                }
                segment_idx += 1;
            }
        }
    }

    // Remove duplicate hits
    hits.dedup_by(|a, b| a.point.distance(b.point) < 1.0);
    hits
}

/// Slice path at specific hit points
/// This creates two complete closed contours from one closed contour
fn slice_path_at_hits(path: &BezPath, hits: &[Hit]) -> Vec<BezPath> {
    if hits.is_empty() {
        return vec![path.clone()];
    }

    if hits.len() != 2 {
        info!(
            "Knife tool requires exactly 2 intersection points, found {}",
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

    let first_hit = &sorted_hits[0];
    let second_hit = &sorted_hits[1];

    info!(
        "Cutting path between intersection points at segments {} and {}",
        first_hit.segment_idx, second_hit.segment_idx
    );

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

    info!(
        "Successfully split closed contour into {} closed contours",
        result_paths.len()
    );
    result_paths
}

/// DEPRECATED: We no longer create connecting bridges
/// Instead, we create proper closed contours with the cutting line integrated
fn _create_connecting_bridges(_sorted_hits: &[Hit]) -> Vec<BezPath> {
    // No longer needed - we create proper closed contours instead
    vec![]
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

    if (0.0..=1.0).contains(&u) {
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
        intersections.push(point);
    }

    // Remove duplicates with smaller tolerance for accuracy
    intersections.dedup_by(|a, b| a.distance(*b) < 1.0);
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
        intersections.push(point);
    }

    // Remove duplicates with smaller tolerance for accuracy
    intersections.dedup_by(|a, b| a.distance(*b) < 1.0);
    intersections
}

fn get_path_start_point_inline(path: &BezPath) -> Option<Point> {
    for element in path.elements() {
        if let PathEl::MoveTo(pt) = element {
            return Some(*pt);
        }
    }
    None
}

/// Calculate the distance from a point to a line
fn _calculate_line_point_distance(line: &kurbo::Line, point: Point) -> f64 {
    let a = line.p1.y - line.p0.y;
    let b = line.p0.x - line.p1.x;
    let c = line.p1.x * line.p0.y - line.p0.x * line.p1.y;

    (a * point.x + b * point.y + c).abs() / (a * a + b * b).sqrt()
}
