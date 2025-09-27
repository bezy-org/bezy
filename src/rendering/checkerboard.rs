//! Checkerboard background grid system for the Bezy font editor GUI.
//!
//! This module handles the creation and visibility management of the
//! checkerboard grid that provides visual context in the design space.
//! Uses dynamic rendering to only show squares visible to the camera.
//!
//! ## CRITICAL: Zoom Logic
//!
//! The grid scaling follows this relationship:
//! - **ZOOMED OUT** (large projection scale) → **LARGE grid squares** (better performance)
//! - **ZOOMED IN** (small projection scale) → **SMALL grid squares** (more detail)
//!
//! This prevents performance issues when viewing large areas while maintaining
//! detail when editing at close zoom levels. It also helps keep the grid usable
//! (not too small).

use crate::rendering::cameras::DesignCamera;
use crate::ui::themes::CurrentTheme;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};
use bevy_pancam::PanCam;
use std::collections::HashSet;

/// Z-coordinate for checkerboard squares (slightly behind other elements)
const CHECKERBOARD_Z_LEVEL: f32 = 0.1;

/// Minimum zoom level where checkerboard is visible (very zoomed out)
/// Lower values = more zoomed out before hiding checkerboard
const MIN_VISIBILITY_ZOOM: f32 = 0.01;

/// When camera moves, how much does it need to move to trigger a grid respawn
const GRID_SIZE_CHANGE_THRESHOLD: f32 = 1.25;
const MAX_SQUARES_PER_FRAME: usize = 2000;


// Resources -------------------

/// Resource to control whether the checkerboard is enabled
/// This allows complete disabling for performance when not needed
#[derive(Resource, Clone)]
pub struct CheckerboardEnabled {
    pub enabled: bool,
}

impl Default for CheckerboardEnabled {
    fn default() -> Self {
        Self {
            enabled: true, // Will be initialized with theme value in system
        }
    }
}

// Components ------------------

/// Component to identify checkerboard squares
///
/// This is used to query and modify only checkerboard sprites
/// when toggling visibility or performing other operations.
#[derive(Component)]
pub struct CheckerboardSquare {
    /// Grid coordinates of this square
    pub grid_pos: IVec2,
}

/// Resource to track currently spawned checkerboard squares
#[derive(Resource, Default)]
pub struct CheckerboardState {
    /// Set of grid positions that currently have spawned squares
    spawned_squares: HashSet<IVec2>,
    /// Last camera position and scale to detect significant changes
    last_camera_state: Option<(Vec2, f32)>,
    /// Last grid size used to detect when we need to respawn all squares
    last_grid_size: Option<f32>,
    /// Last window size to detect window resize events
    last_window_size: Option<Vec2>,
}

// Systems -----------------

/// Calculates the appropriate grid size based on zoom level
///
/// CRITICAL: This function implements the correct zoom-to-grid-size relationship:
/// - When ZOOMED OUT (large projection scale): LARGE grid squares (better performance, fewer squares)
/// - When ZOOMED IN (small projection scale): SMALL grid squares (more detail)
///
/// Bevy's OrthographicProjection.scale represents how much world space is visible:
/// - LARGER scale = more world space visible = more ZOOMED OUT
/// - SMALLER scale = less world space visible = more ZOOMED IN
pub fn calculate_dynamic_grid_size(zoom_scale: f32, theme: &CurrentTheme) -> f32 {
    let base_size = theme.theme().checkerboard_default_unit_size();

    // CORRECTED LOGIC: Higher zoom_scale (more zoomed out) = larger grid squares
    // This prevents performance issues when viewing large areas
    let zoom_thresholds = [
        (15.0, 64.0), // Very zoomed out: 2048 units per square
        (8.0, 32.0),  // Zoomed out: 1024 units per square
        (4.0, 16.0),  // Moderately zoomed out: 512 units per square
        (2.0, 8.0),   // Slightly zoomed out: 256 units per square
        (1.5, 4.0),   // Just zoomed out: 128 units per square
        (1.2, 2.0),   // Barely zoomed out: 64 units per square
        (0.0, 1.0),   // Zoomed in or normal: 32 units per square (base size)
    ];

    // Find the appropriate scale multiplier based on current zoom
    // We iterate from highest zoom (most zoomed out) to lowest
    let scale_multiplier = zoom_thresholds
        .iter()
        .find(|(threshold, _)| zoom_scale >= *threshold)
        .map(|(_, multiplier)| *multiplier)
        .unwrap_or(1.0); // Default to base size for very zoomed in

    base_size * scale_multiplier
}

/// Updates the checkerboard based on camera position and zoom
///
/// This system dynamically spawns and despawns checkerboard squares
/// based on what's visible to the camera, providing better performance.
pub fn update_checkerboard(
    mut commands: Commands,
    mut state: ResMut<CheckerboardState>,
    camera_query: Query<(&Transform, &Projection, Option<&PanCam>), With<DesignCamera>>,
    square_query: Query<(Entity, &CheckerboardSquare)>,
    mut checkerboard_enabled: ResMut<CheckerboardEnabled>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    presentation_mode: Option<Res<crate::ui::edit_mode_toolbar::PresentationMode>>,
    theme: Res<CurrentTheme>,
) {
    // Initialize checkerboard enabled with theme value if needed
    if checkerboard_enabled.is_added() {
        checkerboard_enabled.enabled = theme.theme().checkerboard_enabled_by_default();
    }

    // Force checkerboard rebuild when theme changes (to pick up new colors)
    // Only in debug mode to avoid performance impact in release builds
    #[cfg(debug_assertions)]
    if theme.is_changed() && !state.spawned_squares.is_empty() {
        despawn_all_squares(&mut commands, &mut state, &square_query);
        return; // Let next frame rebuild with new colors
    }
    // If checkerboard is disabled OR we're in presentation mode, despawn all squares and return early
    let presentation_active = presentation_mode.is_some_and(|pm| pm.active);
    if !checkerboard_enabled.enabled || presentation_active {
        if presentation_active {
            debug!("🎭 Checkerboard hidden for presentation mode");
        }
        despawn_all_squares(&mut commands, &mut state, &square_query);
        return;
    }

    let Ok((camera_transform, projection, _pancam_opt)) = camera_query.single() else {
        return;
    };

    // Get camera scale from OrthographicProjection (this is what PanCam
    // actually modifies for zoom)
    let projection_scale = match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => 1.0, // Default for non-orthographic projections
    };
    let transform_scale = camera_transform.scale.x;

    // Use projection scale for grid calculation (this is the real zoom level)
    let camera_scale = projection_scale;
    let current_grid_size = calculate_dynamic_grid_size(camera_scale, &theme);

    // Debug logging to help troubleshoot zoom issues (only log when scale
    // changes significantly)
    let significant_scale_change = state.last_camera_state.is_none_or(|(_, last_scale)| {
        (camera_scale / last_scale - 1.0).abs() > 0.05 // Log if scale
                                                       // changes by >5%
    });

    if significant_scale_change {
        debug!(
            "Camera debug: projection_scale={:.3}, transform_scale={:.3}, \
               using={:.3}, transform=({:.1}, {:.1}, {:.1}), grid_size={:.0}",
            projection_scale,
            transform_scale,
            camera_scale,
            camera_transform.translation.x,
            camera_transform.translation.y,
            camera_transform.translation.z,
            current_grid_size
        );

        // Also show what zoom threshold this should trigger
        let expected_grid_size = calculate_dynamic_grid_size(camera_scale, &theme);
        debug!(
            "Grid size calculation: zoom={:.3} → expected_size={:.0}, \
               last_size={:?}",
            camera_scale, expected_grid_size, state.last_grid_size
        );
    }

    // Hide checkerboard if zoom is outside visible range
    if !is_checkerboard_visible(camera_scale, &theme) {
        let max_zoom = theme.theme().checkerboard_max_zoom_visible();
        debug!(
            "Checkerboard not visible at current zoom scale: {:.3} \
               (range: {:.3} to {:.1})",
            camera_scale, MIN_VISIBILITY_ZOOM, max_zoom
        );
        despawn_all_squares(&mut commands, &mut state, &square_query);
        return;
    }

    // Debug logging for checkerboard (only log once per grid size change)

    if state.last_grid_size.is_none()
        || state.last_grid_size.map_or(true, |size| size != current_grid_size)
        || significant_scale_change
    {
        debug!(
            "Checkerboard: camera_scale={:.3}, grid_size={:.0} units, \
               camera_pos=({:.1}, {:.1})",
            camera_scale,
            current_grid_size,
            camera_transform.translation.x,
            camera_transform.translation.y
        );

        // Calculate and show the actual grid level being used
        let base_size = theme.theme().checkerboard_default_unit_size();
        let scale_multiplier = current_grid_size / base_size;
        if scale_multiplier == 1.0 {
            debug!(
                "  → Grid: Base level ({:.0} units) at zoom ≥ 1.0",
                base_size
            );
        } else {
            // Show the zoom threshold for this level
            let zoom_threshold = match scale_multiplier as i32 {
                2 => "1.2",
                4 => "1.5",
                8 => "2.0",
                16 => "4.0",
                32 => "8.0",
                64 => "15.0",
                _ => "very high",
            };
            debug!(
                "  → Grid: {}x scale ({:.0} units) at zoom ≥ {}",
                scale_multiplier, current_grid_size, zoom_threshold
            );
        }
        debug!(
            "  → Checkerboard visible: {}",
            is_checkerboard_visible(camera_scale, &theme)
        );
    }

    // Check if grid size changed significantly - if so, respawn all squares
    let grid_size_changed = state.last_grid_size.is_none_or(|last_size| {
        // Use a more responsive threshold than just doubling/halving
        // This prevents sudden jumps but still triggers when grid size changes meaningfully
        let ratio = current_grid_size / last_size;
        let should_change =
            ratio >= GRID_SIZE_CHANGE_THRESHOLD || ratio <= (1.0 / GRID_SIZE_CHANGE_THRESHOLD);

        // Debug why grid size change isn't happening
        if !should_change && significant_scale_change {
            debug!(
                "Grid size NOT changing: current={:.0}, last={:.0}, \
               ratio={:.2}, threshold={:.1}",
                current_grid_size, last_size, ratio, GRID_SIZE_CHANGE_THRESHOLD
            );
        }

        should_change
    });

    if grid_size_changed {
        debug!(
            "🔄 GRID SIZE CHANGED! Respawning all squares: \
               old={:?} → new={:.0}",
            state.last_grid_size, current_grid_size
        );
        // Clear all existing squares and state
        despawn_all_squares(&mut commands, &mut state, &square_query);
        state.last_grid_size = Some(current_grid_size);
        state.last_camera_state = None; // Force camera update
                                        // Don't update visible squares this frame - let the next frame handle
                                        // spawning
        return;
    }

    // Get actual window size for visible area calculation
    let window_size = if let Ok(window) = window_query.single() {
        Vec2::new(window.resolution.width(), window.resolution.height())
    } else {
        // Fallback to theme constants if window query fails
        Vec2::new(theme.theme().window_width(), theme.theme().window_height())
    };

    // Check if window size changed (e.g., fullscreen toggle)
    let window_size_changed = state.last_window_size.is_none_or(|last_size| {
        let size_diff = (window_size - last_size).length();
        size_diff > 1.0 // If window size changed by more than 1 pixel
    });

    if window_size_changed {
        debug!(
            "🖥️ WINDOW SIZE CHANGED! Forcing checkerboard recalculation: \
               old={:?} → new=({:.0}x{:.0})",
            state.last_window_size, window_size.x, window_size.y
        );
        // Clear all existing squares and force recalculation
        despawn_all_squares(&mut commands, &mut state, &square_query);
        state.last_window_size = Some(window_size);
        state.last_camera_state = None; // Force camera update
                                        // Don't update visible squares this frame - let the next frame handle spawning
        return;
    }

    // Update window size tracking
    state.last_window_size = Some(window_size);

    update_visible_squares(
        &mut commands,
        &mut state,
        camera_transform,
        camera_scale,
        &square_query,
        current_grid_size,
        window_size,
        &theme,
    );
}

/// Checks if checkerboard should be visible at the current zoom level
///
/// The checkerboard is always visible within our defined zoom range
/// since the dynamic scaling system ensures it's always useful.
fn is_checkerboard_visible(zoom_scale: f32, theme: &CurrentTheme) -> bool {
    // Always show checkerboard within our zoom limits
    // The dynamic grid scaling ensures it's always useful
    let max_zoom = theme.theme().checkerboard_max_zoom_visible();
    (0.05..=max_zoom).contains(&zoom_scale)
}

/// Updates which squares are visible based on camera position
fn update_visible_squares(
    commands: &mut Commands,
    state: &mut CheckerboardState,
    camera_transform: &Transform,
    camera_scale: f32,
    square_query: &Query<(Entity, &CheckerboardSquare)>,
    current_grid_size: f32,
    window_size: Vec2,
    theme: &CurrentTheme,
) {
    let camera_pos = camera_transform.translation.truncate();

    // Skip update if camera hasn't moved significantly
    if let Some((last_pos, last_scale)) = state.last_camera_state {
        let pos_diff = (camera_pos - last_pos).length();
        let scale_diff = (camera_scale - last_scale).abs();

        // Only update if moved more than one grid unit or zoom changed >5%
        // Reduced zoom threshold from 10% to 5% for more responsive grid changes
        if pos_diff < current_grid_size && scale_diff < last_scale * 0.05 {
            return;
        }
    }

    // Update camera state
    state.last_camera_state = Some((camera_pos, camera_scale));

    // Calculate visible area
    let visible_area = calculate_visible_area(
        camera_transform,
        camera_scale,
        current_grid_size,
        window_size,
        theme,
    );
    let needed_squares = get_needed_squares(&visible_area, current_grid_size);

    // Debug logging for visible area (only when grid size changes)
    if state.last_grid_size.is_none() || state.last_grid_size.unwrap() != current_grid_size {
        debug!(
            "Design space grid: visible=({:.0}, {:.0}) to ({:.0}, {:.0}), \
               {} squares",
            visible_area.min.x,
            visible_area.min.y,
            visible_area.max.x,
            visible_area.max.y,
            needed_squares.len()
        );
    }

    // Despawn squares that are no longer needed
    despawn_unneeded_squares(commands, state, square_query, &needed_squares);

    // Spawn new squares that are needed
    spawn_needed_squares(commands, state, &needed_squares, current_grid_size, theme);
}

/// Calculates the area visible to the camera
#[allow(static_mut_refs)]
fn calculate_visible_area(
    camera_transform: &Transform,
    camera_scale: f32,
    current_grid_size: f32,
    window_size: Vec2,
    theme: &CurrentTheme,
) -> Rect {
    let camera_pos = camera_transform.translation.truncate();

    // Calculate screen coverage in world space
    let screen_width = window_size.x;
    let screen_height = window_size.y;

    // CRITICAL: Always ensure complete screen coverage first
    // This is the minimum area needed to cover the entire screen
    let min_screen_half_width = (screen_width * camera_scale) / 2.0;
    let min_screen_half_height = (screen_height * camera_scale) / 2.0;

    // Add padding to extend beyond screen edges for smooth scrolling
    // Use fixed padding that's proportional to grid size but not too conservative
    let edge_padding = current_grid_size * 2.0; // Always at least 2 grid squares beyond edges

    // Calculate conservative coverage for performance, but never less than screen coverage
    let base_size = theme.theme().checkerboard_default_unit_size();
    let grid_scale_factor = current_grid_size / base_size;
    let coverage_multiplier = 1.2; // VISIBLE_AREA_COVERAGE_MULTIPLIER equivalent
    let performance_coverage_multiplier = coverage_multiplier / grid_scale_factor.sqrt();

    // Performance-based coverage
    let perf_half_width = (screen_width * camera_scale * performance_coverage_multiplier) / 2.0;
    let perf_half_height = (screen_height * camera_scale * performance_coverage_multiplier) / 2.0;

    // ENSURE COMPLETE COVERAGE: Use the larger of screen coverage or performance coverage
    let final_half_width = (min_screen_half_width + edge_padding).max(perf_half_width);
    let final_half_height = (min_screen_half_height + edge_padding).max(perf_half_height);

    // Only log when grid size changes to reduce spam
    // TODO: Refactor to use OnceCell or Lazy for safer static access
    #[allow(static_mut_refs)]
    static mut LAST_LOGGED_GRID_SIZE: Option<f32> = None;
    unsafe {
        if LAST_LOGGED_GRID_SIZE.is_none() || LAST_LOGGED_GRID_SIZE.map_or(true, |size| size != current_grid_size) {
            debug!(
                "✅ Screen coverage: window=({:.0}x{:.0}), camera_scale={:.3}, \
                   min_screen=({:.0}, {:.0}), final=({:.0}, {:.0}), \
                   grid_size={:.0}, padding={:.0}",
                screen_width,
                screen_height,
                camera_scale,
                min_screen_half_width,
                min_screen_half_height,
                final_half_width,
                final_half_height,
                current_grid_size,
                edge_padding
            );
            LAST_LOGGED_GRID_SIZE = Some(current_grid_size);
        }
    }

    Rect::from_center_half_size(camera_pos, Vec2::new(final_half_width, final_half_height))
}

/// Gets the set of grid positions that need checkerboard squares
#[allow(static_mut_refs)]
fn get_needed_squares(visible_area: &Rect, current_grid_size: f32) -> HashSet<IVec2> {
    let mut needed = HashSet::new();

    // Calculate grid bounds for visible area with extra padding to ensure edge coverage
    // Using floor/ceil with -1/+1 extension ensures we always cover screen edges
    let min_x = (visible_area.min.x / current_grid_size).floor() as i32 - 1;
    let max_x = (visible_area.max.x / current_grid_size).ceil() as i32 + 1;
    let min_y = (visible_area.min.y / current_grid_size).floor() as i32 - 1;
    let max_y = (visible_area.max.y / current_grid_size).ceil() as i32 + 1;

    // Debug log grid bounds occasionally to verify coverage
    // TODO: Refactor to use OnceCell or Lazy for safer static access
    #[allow(static_mut_refs)]
    static mut BOUNDS_LOG_COUNT: u32 = 0;
    unsafe {
        BOUNDS_LOG_COUNT += 1;
        if BOUNDS_LOG_COUNT % 100 == 1 {
            // Log every 100th call
            debug!(
                "🔢 Grid bounds: visible_area=({:.0},{:.0} to {:.0},{:.0}), \
                   grid_bounds=({},{} to {},{}), grid_size={:.0}",
                visible_area.min.x,
                visible_area.min.y,
                visible_area.max.x,
                visible_area.max.y,
                min_x,
                min_y,
                max_x,
                max_y,
                current_grid_size
            );
        }
    }

    // Add squares in checkerboard pattern
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            // Only add squares that should be visible in checkerboard pattern
            if (x + y) % 2 == 0 {
                needed.insert(IVec2::new(x, y));
            }
        }
    }

    // Performance safety check - if too many squares, warn but still return them
    // The grid size should have been calculated to prevent this, but this is a safeguard
    if needed.len() > MAX_SQUARES_PER_FRAME {
        warn!(
            "⚠️  Many squares needed: {} (max recommended: {}). \
               Grid size {:.0} may be too small for current zoom level.",
            needed.len(),
            MAX_SQUARES_PER_FRAME,
            current_grid_size
        );
    }

    needed
}

/// Despawns squares that are no longer needed
fn despawn_unneeded_squares(
    commands: &mut Commands,
    state: &mut CheckerboardState,
    square_query: &Query<(Entity, &CheckerboardSquare)>,
    needed_squares: &HashSet<IVec2>,
) {
    let mut to_remove = Vec::new();

    for (entity, square) in square_query.iter() {
        if !needed_squares.contains(&square.grid_pos) {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.despawn();
                to_remove.push(square.grid_pos);
            }
        }
    }

    for pos in to_remove {
        state.spawned_squares.remove(&pos);
    }
}

/// Spawns new squares that are needed
fn spawn_needed_squares(
    commands: &mut Commands,
    state: &mut CheckerboardState,
    needed_squares: &HashSet<IVec2>,
    current_grid_size: f32,
    theme: &CurrentTheme,
) {
    let new_squares: Vec<_> = needed_squares
        .iter()
        .filter(|pos| !state.spawned_squares.contains(pos))
        .copied()
        .collect();

    if !new_squares.is_empty() {
        debug!(
            "Spawning {} new checkerboard squares (grid size: {:.1})",
            new_squares.len(),
            current_grid_size
        );
    }

    for grid_pos in new_squares {
        spawn_square(commands, grid_pos, current_grid_size, theme);
        state.spawned_squares.insert(grid_pos);
    }
}

/// Spawns a single checkerboard square at the given grid position
#[allow(static_mut_refs)]
fn spawn_square(
    commands: &mut Commands,
    grid_pos: IVec2,
    current_grid_size: f32,
    theme: &CurrentTheme,
) {
    let world_pos = grid_to_world_position(grid_pos, current_grid_size);

    // Debug log the first few squares spawned to verify design space alignment
    // TODO: Refactor to use OnceCell or Lazy for safer static access
    #[allow(static_mut_refs)]
    static mut SPAWN_COUNT: usize = 0;
    unsafe {
        if SPAWN_COUNT < 3 {
            debug!(
                "Design space square {} at grid=({}, {}), \
                   world=({:.0}, {:.0}), size={:.0}",
                SPAWN_COUNT, grid_pos.x, grid_pos.y, world_pos.x, world_pos.y, current_grid_size
            );
            SPAWN_COUNT += 1;
        }
    }

    commands.spawn((
        CheckerboardSquare { grid_pos },
        Sprite {
            color: theme.theme().checkerboard_color(),
            custom_size: Some(Vec2::splat(current_grid_size)),
            ..default()
        },
        Transform::from_translation(world_pos.extend(CHECKERBOARD_Z_LEVEL)),
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
}

/// Converts grid position to world position aligned to design space
/// The grid is positioned so that (0,0) - the font baseline/left sidebearing
/// intersection - falls exactly at the intersection of four grid squares,
/// making unit counting accurate
fn grid_to_world_position(grid_pos: IVec2, current_grid_size: f32) -> Vec2 {
    Vec2::new(
        (grid_pos.x as f32 + 0.5) * current_grid_size,
        (grid_pos.y as f32 + 0.5) * current_grid_size,
    )
}

/// Despawns all checkerboard squares
fn despawn_all_squares(
    commands: &mut Commands,
    state: &mut CheckerboardState,
    square_query: &Query<(Entity, &CheckerboardSquare)>,
) {
    for (entity, _) in square_query.iter() {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }
    state.spawned_squares.clear();

    // Also clear grid size to force recalculation
    state.last_grid_size = None;
}

#[derive(Default)]
pub struct CheckerboardPlugin;

impl Plugin for CheckerboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CheckerboardState>()
            .init_resource::<CheckerboardEnabled>()
            .add_systems(Update, update_checkerboard);
    }
}
