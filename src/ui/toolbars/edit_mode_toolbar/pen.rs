//! # Pen Tool
//!
//! The pen tool allows users to draw vector paths by clicking points in sequence.
//! Click to place points, click near the start point to close the path, or right-click
//! to finish an open path. Hold Shift for axis-aligned drawing, press Escape to cancel.
//!
//! The tool converts placed points into UFO contours that are saved to the font file.

use super::EditModeSystem;
use crate::core::io::input::{
    helpers, InputEvent, InputMode, InputState, ModifierState,
};
use crate::core::io::pointer::PointerInfo;
use crate::core::settings::BezySettings;
use crate::core::state::AppState;
use crate::editing::edit_type::EditType;
use crate::editing::selection::components::{
    GlyphPointReference, PointType, Selectable, Selected,
};
use crate::editing::selection::systems::AppStateChanged;
use crate::editing::selection::{
    DragPointState, DragSelectionState, SelectionState,
};
use crate::editing::sort::ActiveSortState;
use crate::geometry::design_space::DPoint;
use crate::systems::sort_manager::SortPointEntity;
use crate::systems::ui_interaction::UiHoverState;
use crate::ui::toolbars::edit_mode_toolbar::select::SelectModeActive;
use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};
use crate::ui::themes::{CurrentTheme, ToolbarBorderRadius};
use crate::ui::theme::*;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use kurbo::BezPath;
use norad::{Contour, ContourPoint};

pub struct PenTool;

impl EditTool for PenTool {
    fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
        "pen"
    }

    fn name(&self) -> &'static str {
        "Pen"
    }

    fn icon(&self) -> &'static str {
        "\u{E011}"
    }

    fn shortcut_key(&self) -> Option<char> {
        Some('p')
    }

    fn default_order(&self) -> i32 {
        20 // After select, primary drawing tool
    }

    fn description(&self) -> &'static str {
        "Draw paths and contours"
    }

    fn update(&self, commands: &mut Commands) {
        // Ensure pen mode is active
        commands.insert_resource(PenModeActive(true));
        commands.insert_resource(SelectModeActive(false));
        commands.insert_resource(crate::core::io::input::InputMode::Pen);
    }

    fn on_enter(&self) {
        info!("Entered Pen tool");
    }

    fn on_exit(&self) {
        info!("Exited Pen tool");
    }
}

// ================================================================
// CONSTANTS
// ================================================================

/// Distance threshold for closing a path by clicking near the start point
const CLOSE_PATH_THRESHOLD: f32 = 16.0;
/// Size of drawn points in the preview
const POINT_PREVIEW_SIZE: f32 = 4.0;
const CURSOR_INDICATOR_SIZE: f32 = 4.0;

// ================================================================
// PLUGIN SETUP
// ================================================================

/// Bevy plugin that sets up the pen tool
///
/// This plugin initializes the pen tool's state resources and registers
/// all the systems needed for pen functionality:
/// - Mouse input handling for placing points
/// - Keyboard shortcuts (Escape to cancel)
/// - Visual preview rendering of the current path
/// - Cleanup when switching away from pen mode
pub struct PenModePlugin;

impl Plugin for PenModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PenToolState>()
            .init_resource::<PenModeActive>()
            .init_resource::<PenDrawingMode>() // Default is Regular
            .add_systems(Startup, register_pen_tool)
            // Business logic is handled by /src/tools/pen.rs PenToolPlugin
            // This UI module only handles toolbar integration
            .add_systems(
                Update,
                (
                    reset_pen_mode_when_inactive,
                    toggle_pen_submenu_visibility,
                    handle_pen_submenu_selection,
                )
            )
            .add_systems(PostStartup, spawn_pen_submenu);
    }
}

fn register_pen_tool(mut tool_registry: ResMut<ToolRegistry>) {
    tool_registry.register_tool(Box::new(PenTool));
}

// ================================================================
// RESOURCES AND STATE
// ================================================================

/// Resource to track if pen mode is currently active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct PenModeActive(pub bool);

/// Input consumer for pen tool
#[derive(Resource)]
pub struct PenInputConsumer;

impl crate::systems::input_consumer::InputConsumer for PenInputConsumer {
    fn should_handle_input(
        &self,
        event: &InputEvent,
        input_state: &InputState,
    ) -> bool {
        // Only handle input if pen mode is active
        if !helpers::is_input_mode(input_state, InputMode::Pen) {
            return false;
        }

        // Handle mouse events
        matches!(
            event,
            InputEvent::MouseClick { .. }
                | InputEvent::MouseDrag { .. }
                | InputEvent::MouseRelease { .. }
        )
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick {
                button,
                position,
                modifiers,
            } => {
                if *button == MouseButton::Left {
                    debug!("Pen: Processing mouse click at {:?} with modifiers {:?}", position, modifiers);
                    // TODO: Implement pen click handling
                }
            }
            InputEvent::MouseDrag {
                button,
                start_position,
                current_position,
                delta: _,
                modifiers,
            } => {
                if *button == MouseButton::Left {
                    debug!("Pen: Processing mouse drag from {:?} to {:?} with modifiers {:?}",
                          start_position, current_position, modifiers);
                    // TODO: Implement pen drag handling
                }
            }
            InputEvent::MouseRelease {
                button,
                position,
                modifiers,
            } => {
                if *button == MouseButton::Left {
                    info!("Pen: Processing mouse release at {:?} with modifiers {:?}", position, modifiers);
                    // TODO: Implement pen release handling
                }
            }
            _ => {}
        }
    }
}

/// The main state manager for the pen tool
///
/// The pen tool works like this:
/// 1. Start in Ready state - waiting for first click
/// 2. First click starts a new path and moves to Drawing state
/// 3. Subsequent clicks add points to the current path
/// 4. Click near start point to close the path
/// 5. Right-click to finish an open path
/// 6. Escape cancels the current path
#[derive(Resource)]
pub struct PenToolState {
    /// Whether the pen tool is currently active
    pub active: bool,
    /// The current drawing state (Ready or Drawing)
    pub state: PenState,
    /// The path being constructed (using kurbo for geometry)
    pub current_path: Option<BezPath>,
    /// Points that have been placed in the current path
    pub points: Vec<Vec2>,
}

impl Default for PenToolState {
    fn default() -> Self {
        Self {
            active: true,
            state: PenState::Ready,
            current_path: None,
            points: Vec::new(),
        }
    }
}

/// The two states the pen tool can be in
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PenState {
    /// Ready to start a new path (no points placed yet)
    #[default]
    Ready,
    /// Currently drawing a path (at least one point placed)
    Drawing,
}

/// Pen drawing modes for the submenu
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Resource)]
pub enum PenDrawingMode {
    /// Regular pen tool (default)
    #[default]
    Regular,
    /// Hyperbezier pen tool for smooth curves
    Hyperbezier,
}

impl PenDrawingMode {
    /// Get the icon for each pen submenu mode
    pub fn get_icon(&self) -> &'static str {
        match self {
            PenDrawingMode::Regular => "\u{E011}", // Pen nib icon
            PenDrawingMode::Hyperbezier => "\u{E012}", // Spiral icon
        }
    }

    /// Get the name for each pen submenu mode
    pub fn get_name(&self) -> &'static str {
        match self {
            PenDrawingMode::Regular => "Pen",
            PenDrawingMode::Hyperbezier => "Hyper",
        }
    }

    /// Get the description for each pen submenu mode
    pub fn get_description(&self) -> &'static str {
        match self {
            PenDrawingMode::Regular => "Draw regular Bézier curves",
            PenDrawingMode::Hyperbezier => "Draw smooth hyperbezier curves",
        }
    }
}

/// Component to mark pen submenu buttons
#[derive(Component)]
pub struct PenSubMenuButton;

/// Component to identify which pen mode button this is
#[derive(Component)]
pub struct PenModeButton {
    pub mode: PenDrawingMode,
}

// ================================================================
// EDIT MODE IMPLEMENTATION
// ================================================================

/// Pen mode for drawing vector paths in glyphs
pub struct PenMode;

impl EditModeSystem for PenMode {
    fn update(&self, commands: &mut Commands) {
        commands.insert_resource(PenModeActive(true));
        commands.insert_resource(SelectModeActive(false));
    }

    fn on_enter(&self) {
        info!("Entering Pen Mode");
    }

    fn on_exit(&self) {
        info!("Exiting Pen Mode");
    }
}

// ================================================================
// MODE MANAGEMENT
// ================================================================

/// Handles cleanup when switching away from pen mode
///
/// If the user was in the middle of drawing a path, this system
/// will automatically commit it before switching modes.
pub fn reset_pen_mode_when_inactive(
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    mut commands: Commands,
    mut pen_state: ResMut<PenToolState>,
    mut app_state_changed: EventWriter<AppStateChanged>,
) {
    // Early return if still in pen mode
    if current_tool.get_current() == Some("pen") {
        return;
    }

    // Save any work in progress before switching modes
    try_commit_current_path(&pen_state, &mut app_state_changed);

    // Clean up and deactivate pen mode
    deactivate_pen_mode(&mut pen_state, &mut commands);
}

/// Attempts to commit the current path if it has enough points to be drawable
fn try_commit_current_path(
    pen_state: &PenToolState,
    app_state_changed: &mut EventWriter<AppStateChanged>,
) {
    if !is_path_drawable(pen_state) {
        return;
    }

    if let Some(_contour) =
        create_contour_from_points(&pen_state.points, Vec2::ZERO)
    {
        app_state_changed.write(AppStateChanged);
        info!("Auto-committing path when switching modes");
    }
}

/// Checks if the current path has enough points to create a drawable contour
fn is_path_drawable(pen_state: &PenToolState) -> bool {
    pen_state.state == PenState::Drawing && pen_state.points.len() >= 2
}

/// Resets pen state and marks pen mode as inactive
fn deactivate_pen_mode(
    pen_state: &mut ResMut<PenToolState>,
    commands: &mut Commands,
) {
    **pen_state = PenToolState::default();
    pen_state.active = false;
    commands.insert_resource(PenModeActive(false));
}

// ================================================================
// MOUSE INPUT HANDLING
// ================================================================

/// Main system for handling mouse interactions with the pen tool
#[allow(clippy::too_many_arguments)]
pub fn handle_pen_mouse_events(
    mut commands: Commands,
    pointer_info: Res<PointerInfo>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pen_state: ResMut<PenToolState>,
    pen_mode: Option<Res<PenModeActive>>,
    glyph_navigation: Res<crate::core::state::GlyphNavigation>,
    mut app_state: ResMut<crate::core::state::AppState>,
    mut app_state_changed: EventWriter<AppStateChanged>,
    text_editor_state: Option<Res<crate::core::state::TextEditorState>>,
    ui_hover_state: Res<UiHoverState>,
    settings: Res<BezySettings>,
) {
    if !is_pen_mode_active(&pen_mode) || ui_hover_state.is_hovering_ui {
        return;
    }

    // Get the active sort from text_editor_state
    let active_sort_info = text_editor_state
        .as_ref()
        .and_then(|state| state.get_active_sort());

    if mouse_button_input.just_pressed(MouseButton::Left) {
        let cursor_pos = pointer_info.design.to_raw();
        handle_left_click(
            &mut commands,
            &keyboard,
            &mut pen_state,
            &glyph_navigation,
            &mut app_state,
            &mut app_state_changed,
            active_sort_info,
            text_editor_state.as_deref(),
            cursor_pos,
            &settings,
        );
    }

    if mouse_button_input.just_pressed(MouseButton::Right) {
        handle_right_click(
            &mut commands,
            &mut pen_state,
            &glyph_navigation,
            &mut app_state,
            &mut app_state_changed,
            active_sort_info,
            text_editor_state.as_deref(),
        );
    }
}

/// Check if pen mode is currently active
fn is_pen_mode_active(pen_mode: &Option<Res<PenModeActive>>) -> bool {
    pen_mode.as_ref().is_some_and(|mode| mode.0)
}

/// Handle left mouse button clicks
#[allow(clippy::too_many_arguments)]
fn handle_left_click(
    _commands: &mut Commands,
    keyboard: &Res<ButtonInput<KeyCode>>,
    pen_state: &mut ResMut<PenToolState>,
    glyph_navigation: &Res<crate::core::state::GlyphNavigation>,
    app_state: &mut ResMut<crate::core::state::AppState>,
    app_state_changed: &mut EventWriter<AppStateChanged>,
    active_sort_info: Option<(usize, &crate::core::state::SortEntry)>,
    text_editor_state: Option<&crate::core::state::TextEditorState>,
    cursor_pos: Vec2,
    settings: &BezySettings,
) {
    let final_position =
        calculate_final_position(cursor_pos, keyboard, pen_state, settings);

    if pen_state.state == PenState::Ready {
        start_new_path(pen_state, final_position);
    } else if should_close_path(pen_state, final_position) {
        close_current_path(
            pen_state,
            glyph_navigation,
            app_state,
            app_state_changed,
            active_sort_info,
            text_editor_state,
        );
    } else {
        add_point_to_path(pen_state, final_position);
    }
}

/// Calculate the final position after applying snap-to-grid and axis locking
fn calculate_final_position(
    cursor_pos: Vec2,
    keyboard: &Res<ButtonInput<KeyCode>>,
    pen_state: &PenToolState,
    settings: &BezySettings,
) -> Vec2 {
    // Apply snap to grid first
    let snapped_pos = settings.apply_grid_snap(cursor_pos);

    // Apply axis locking if shift is held and we have points
    let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);

    if shift_pressed && !pen_state.points.is_empty() {
        let last_point = pen_state.points.last().unwrap();
        axis_lock_position(snapped_pos, *last_point)
    } else {
        snapped_pos
    }
}

/// Start a new path with the first point
fn start_new_path(pen_state: &mut ResMut<PenToolState>, position: Vec2) {
    let mut path = BezPath::new();
    path.move_to((position.x as f64, position.y as f64));

    pen_state.current_path = Some(path);
    pen_state.points.push(position);
    pen_state.state = PenState::Drawing;

    info!("Started new path at: {:?}", position);
}

/// Check if we should close the path (clicked near start point)
fn should_close_path(pen_state: &PenToolState, position: Vec2) -> bool {
    if pen_state.points.len() <= 1 {
        return false;
    }

    let start_point = pen_state.points[0];
    let distance = start_point.distance(position);
    distance < CLOSE_PATH_THRESHOLD
}

/// Close the current path and add it to the glyph
fn close_current_path(
    pen_state: &mut ResMut<PenToolState>,
    glyph_navigation: &Res<crate::core::state::GlyphNavigation>,
    app_state: &mut ResMut<crate::core::state::AppState>,
    app_state_changed: &mut EventWriter<AppStateChanged>,
    active_sort_info: Option<(usize, &crate::core::state::SortEntry)>,
    text_editor_state: Option<&crate::core::state::TextEditorState>,
) {
    if !pen_state.points.is_empty() {
        let active_sort_offset = if let (Some((sort_index, _)), Some(state)) =
            (active_sort_info, text_editor_state)
        {
            state
                .get_sort_visual_position(sort_index)
                .unwrap_or(Vec2::ZERO)
        } else {
            Vec2::ZERO
        };

        if let Some(contour) =
            create_contour_from_points(&pen_state.points, active_sort_offset)
        {
            add_contour_to_glyph(
                contour,
                glyph_navigation,
                app_state,
                app_state_changed,
                true,
                active_sort_info,
            );
        }
    }

    // Reset for next path
    reset_pen_state(pen_state);
}

/// Add a point to the current path
fn add_point_to_path(pen_state: &mut ResMut<PenToolState>, position: Vec2) {
    if let Some(ref mut path) = pen_state.current_path {
        path.line_to((position.x as f64, position.y as f64));
        pen_state.points.push(position);
        info!("Added point to path: {:?}", position);
    }
}

/// Handle right mouse button clicks (finish open path)
fn handle_right_click(
    _commands: &mut Commands,
    pen_state: &mut ResMut<PenToolState>,
    glyph_navigation: &Res<crate::core::state::GlyphNavigation>,
    app_state: &mut ResMut<crate::core::state::AppState>,
    app_state_changed: &mut EventWriter<AppStateChanged>,
    active_sort_info: Option<(usize, &crate::core::state::SortEntry)>,
    text_editor_state: Option<&crate::core::state::TextEditorState>,
) {
    if pen_state.state == PenState::Drawing && pen_state.points.len() >= 2 {
        info!("Finishing open path with right click");

        let active_sort_offset = if let (Some((sort_index, _)), Some(state)) =
            (active_sort_info, text_editor_state)
        {
            state
                .get_sort_visual_position(sort_index)
                .unwrap_or(Vec2::ZERO)
        } else {
            Vec2::ZERO
        };

        if let Some(contour) =
            create_contour_from_points(&pen_state.points, active_sort_offset)
        {
            add_contour_to_glyph(
                contour,
                glyph_navigation,
                app_state,
                app_state_changed,
                false,
                active_sort_info,
            );
        }

        reset_pen_state(pen_state);
    }
}

/// Add a contour to the current glyph using the current thread-safe architecture
fn add_contour_to_glyph(
    contour: Contour,
    glyph_navigation: &Res<crate::core::state::GlyphNavigation>,
    app_state: &mut ResMut<crate::core::state::AppState>,
    app_state_changed: &mut EventWriter<AppStateChanged>,
    is_closed: bool,
    active_sort_info: Option<(usize, &crate::core::state::SortEntry)>,
) {
    let glyph_name = if let Some((_sort_index, sort_entry)) = active_sort_info {
        info!(
            "PEN TOOL: Using active sort glyph: {}",
            sort_entry.kind.glyph_name()
        );
        sort_entry.kind.glyph_name().to_string()
    } else {
        let Some(glyph_name) = glyph_navigation.current_glyph.clone() else {
            warn!("PEN TOOL: No glyph found in navigation and no active sort");
            return;
        };
        info!("PEN TOOL: Using glyph navigation glyph: {}", glyph_name);
        glyph_name
    };

    info!(
        "PEN TOOL: Adding contour with {} points to glyph {}",
        contour.points.len(),
        glyph_name
    );

    // For now, we'll create a simplified implementation that works with the current architecture
    // TODO: This needs to be properly implemented when the full glyph editing system is ready
    // The current architecture uses thread-safe data structures and doesn't have direct norad access

    // Convert the norad contour to our thread-safe ContourData
    let contour_data =
        crate::core::state::font_data::ContourData::from_norad_contour(
            &contour,
        );

    // Check if the glyph exists in our thread-safe data
    if let Some(glyph_data) =
        app_state.workspace.font.glyphs.get_mut(&glyph_name)
    {
        // Get or create the outline data
        let outline_data = glyph_data.outline.get_or_insert_with(|| {
            crate::core::state::font_data::OutlineData {
                contours: Vec::new(),
            }
        });

        // Add the new contour
        outline_data.contours.push(contour_data);

        let path_type = if is_closed { "closed" } else { "open" };
        let source = if active_sort_info.is_some() {
            "active sort"
        } else {
            "glyph navigation"
        };
        info!("PEN TOOL: Successfully added {} contour to glyph {} (from {}). Total contours now: {}", 
               path_type, glyph_name, source, outline_data.contours.len());

        // Notify that the app state has changed
        app_state_changed.write(AppStateChanged);
    } else {
        warn!(
            "PEN TOOL: Could not find glyph '{}' in app state to add contour",
            glyph_name
        );
    }
}

/// Reset pen state to ready for next path
fn reset_pen_state(pen_state: &mut ResMut<PenToolState>) {
    pen_state.current_path = None;
    pen_state.points.clear();
    pen_state.state = PenState::Ready;
}

// ================================================================
// KEYBOARD INPUT HANDLING
// ================================================================

/// Handle keyboard shortcuts for the pen tool
pub fn handle_pen_keyboard_events(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pen_state: ResMut<PenToolState>,
    pen_mode: Option<Res<PenModeActive>>,
) {
    if !is_pen_mode_active(&pen_mode) {
        return;
    }

    // Escape key cancels the current path
    if keyboard.just_pressed(KeyCode::Escape) {
        reset_pen_state(&mut pen_state);
        info!("Cancelled current path with Escape key");
    }
}

// ================================================================
// VISUAL PREVIEW RENDERING
// ================================================================

/// Render visual preview of the pen tool's current state
///
/// This shows:
/// - Placed points (yellow circles, green for start point)
/// - Lines connecting placed points (white)
/// - Preview line from last point to cursor (translucent white)
/// - Close indicator when near start point (green highlight)
/// - Current cursor position (small white circle)
pub fn render_pen_preview(
    mut gizmos: Gizmos,
    pointer_info: Res<PointerInfo>,
    keyboard: Res<ButtonInput<KeyCode>>,
    pen_state: Res<PenToolState>,
    pen_mode: Option<Res<PenModeActive>>,
    settings: Res<BezySettings>,
) {
    if !is_pen_mode_active(&pen_mode) {
        return;
    }

    draw_placed_points_and_lines(&mut gizmos, &pen_state);
    let cursor_pos = pointer_info.design.to_raw();
    draw_preview_elements(
        &mut gizmos,
        &pen_state,
        cursor_pos,
        &keyboard,
        &settings,
    );
}

/// Draw all the points that have been placed and lines between them
fn draw_placed_points_and_lines(gizmos: &mut Gizmos, pen_state: &PenToolState) {
    let point_color = Color::srgb(1.0, 1.0, 0.0); // Yellow
    let start_point_color = Color::srgb(0.0, 1.0, 0.5); // Green
    let line_color = Color::srgba(1.0, 1.0, 1.0, 0.9); // White

    for (i, point) in pen_state.points.iter().enumerate() {
        // Draw point (start point gets special color)
        let color = if i == 0 {
            start_point_color
        } else {
            point_color
        };
        gizmos.circle_2d(*point, POINT_PREVIEW_SIZE, color);

        // Draw line to next point
        if i < pen_state.points.len() - 1 {
            gizmos.line_2d(*point, pen_state.points[i + 1], line_color);
        }
    }
}

/// Draw preview elements like the next line segment and axis-locked guides
fn draw_preview_elements(
    gizmos: &mut Gizmos,
    pen_state: &PenToolState,
    cursor_pos: Vec2,
    keyboard: &Res<ButtonInput<KeyCode>>,
    settings: &BezySettings,
) {
    // Draw cursor indicator
    gizmos.circle_2d(
        cursor_pos,
        CURSOR_INDICATOR_SIZE,
        Color::srgb(0.0, 1.0, 0.0),
    );

    if let Some(&last_point) = pen_state.points.last() {
        // Calculate the final position for the preview, same logic as for placing points
        let final_pos =
            calculate_final_position(cursor_pos, keyboard, pen_state, settings);

        // Draw line from last point to cursor's final position
        gizmos.line_2d(last_point, final_pos, Color::srgb(0.0, 1.0, 0.0));

        // Draw a circle at the final position
        gizmos.circle_2d(
            final_pos,
            POINT_PREVIEW_SIZE,
            Color::srgb(0.0, 1.0, 0.0),
        );

        // If close to the start point, draw a circle to indicate path closing
        draw_close_indicator_if_needed(
            gizmos, pen_state, cursor_pos, last_point,
        );
    }
}

/// If the cursor is close to the start point, draw a special indicator
fn draw_close_indicator_if_needed(
    gizmos: &mut Gizmos,
    pen_state: &PenToolState,
    cursor_pos: Vec2,
    _last_point: Vec2,
) {
    if let Some(&first_point) = pen_state.points.first() {
        if cursor_pos.distance(first_point) < CLOSE_PATH_THRESHOLD {
            gizmos.circle_2d(
                first_point,
                CLOSE_PATH_THRESHOLD,
                Color::srgba(1.0, 0.0, 0.0, 0.5),
            );
        }
    }
}

// ================================================================
// UTILITY FUNCTIONS
// ================================================================

/// Lock a position to horizontal or vertical axis relative to another point
/// (used when shift is held to constrain movement)
fn axis_lock_position(pos: Vec2, relative_to: Vec2) -> Vec2 {
    let dxy = pos - relative_to;
    if dxy.x.abs() > dxy.y.abs() {
        Vec2::new(pos.x, relative_to.y)
    } else {
        Vec2::new(relative_to.x, pos.y)
    }
}

/// Create a UFO contour from a list of points
fn create_contour_from_points(
    points: &[Vec2],
    active_sort_offset: Vec2,
) -> Option<Contour> {
    if points.len() < 2 {
        return None;
    }

    info!(
        "PEN TOOL: Creating contour with active_sort_offset: ({:.1}, {:.1})",
        active_sort_offset.x, active_sort_offset.y
    );

    let mut contour_points = Vec::new();

    for point in points {
        // Convert from world coordinates to glyph-local coordinates
        let glyph_local_point = *point - active_sort_offset;

        info!(
            "  - Converting point: world({:.1}, {:.1}) -> local({:.1}, {:.1})",
            point.x, point.y, glyph_local_point.x, glyph_local_point.y
        );

        contour_points.push(ContourPoint::new(
            glyph_local_point.x as f64,
            glyph_local_point.y as f64,
            crate::core::state::font_data::PointTypeData::Line
                .to_norad_point_type(), // Convert our internal type to norad for I/O
            false, // not smooth
            None,  // no name
            None,  // no identifier
        ));
    }

    Some(Contour::new(contour_points, None))
}

// ================================================================
// PEN SUBMENU SYSTEMS (following text tool pattern)
// ================================================================

/// Helper function to spawn a single pen mode button using the unified system
fn spawn_pen_mode_button(
    parent: &mut ChildSpawnerCommands,
    mode: PenDrawingMode,
    asset_server: &Res<AssetServer>,
    theme: &Res<CurrentTheme>,
) {
    // Use the unified toolbar button creation system for consistent styling with hover text
    crate::ui::toolbars::edit_mode_toolbar::ui::create_toolbar_button_with_hover_text(
        parent,
        mode.get_icon(),
        Some(mode.get_name()), // Show the mode name on hover
        (PenSubMenuButton, PenModeButton { mode }),
        asset_server,
        theme,
    );
}

pub fn spawn_pen_submenu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme: Res<CurrentTheme>,
) {
    info!("🖊️ Spawning pen submenu with Regular and Hyperbezier modes");
    info!("🖊️ Default pen mode is: {:?}", PenDrawingMode::default());
    
    let modes = [
        PenDrawingMode::Regular,
        PenDrawingMode::Hyperbezier,
    ];

    // Create the parent submenu node (left-aligned to match main toolbar)
    let submenu_node = Node {
        position_type: PositionType::Absolute,
        top: Val::Px(TOOLBAR_CONTAINER_MARGIN + 74.0),
        left: Val::Px(TOOLBAR_CONTAINER_MARGIN),  // Left-aligned to match toolbar
        flex_direction: FlexDirection::Row,
        padding: UiRect::all(Val::Px(TOOLBAR_PADDING)),
        margin: UiRect::all(Val::ZERO),
        row_gap: Val::Px(TOOLBAR_PADDING),
        display: Display::None, // Hidden by default
        ..default()
    };

    // Spawn the submenu with all buttons
    commands
        .spawn((submenu_node, Name::new("PenSubMenu")))
        .with_children(|parent| {
            for mode in modes {
                spawn_pen_mode_button(parent, mode, &asset_server, &theme);
            }
        });
        
    info!("🖊️ Pen submenu spawned successfully");
}

/// Auto-show pen submenu when pen tool is active (like text tool)
pub fn toggle_pen_submenu_visibility(
    current_tool: Option<Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>>,
    mut submenu_query: Query<(&mut Node, &Name)>,
) {
    let is_pen_tool_active = current_tool.as_ref()
        .and_then(|tool| tool.get_current()) == Some("pen");
    
    for (mut node, name) in submenu_query.iter_mut() {
        if name.as_str() == "PenSubMenu" {
            let new_display = if is_pen_tool_active {
                Display::Flex
            } else {
                Display::None
            };
            
            if node.display != new_display {
                node.display = new_display;
                info!("🖊️ Pen submenu visibility changed: tool_active={}, display={:?}", 
                      is_pen_tool_active, new_display);
            }
        }
    }
}

/// Handle pen submenu selection and visual feedback (following text tool pattern)
pub fn handle_pen_submenu_selection(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &PenModeButton,
            Entity,
        ),
        With<PenSubMenuButton>,
    >,
    mut current_mode: ResMut<PenDrawingMode>,
) {
    // Debug: Log if we find any submenu buttons
    let button_count = interaction_query.iter().len();
    if button_count > 0 {
        static mut LAST_LOG: f32 = 0.0;
        unsafe {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f32();
            if current_time - LAST_LOG > 2.0 {
                LAST_LOG = current_time;
                info!("🖊️ Pen submenu selection system: found {} buttons", button_count);
            }
        }
    }
    for (interaction, mut color, mut border_color, mode_button, _entity) in
        &mut interaction_query
    {
        let is_current_mode = *current_mode == mode_button.mode;
        
        // Debug: Log interactions for debugging
        if *interaction != Interaction::None {
            info!("🖊️ Button interaction: {:?} for mode {:?} (current: {:?})", 
                  interaction, mode_button.mode, *current_mode);
        }
        
        if *interaction == Interaction::Pressed && !is_current_mode {
            *current_mode = mode_button.mode;
            info!("🖊️ Switched to pen mode: {:?}", mode_button.mode);
        }

        // Visual feedback based on current mode
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON_COLOR.into();
                *border_color = PRESSED_BUTTON_OUTLINE_COLOR.into();
            }
            Interaction::Hovered => {
                if is_current_mode {
                    *color = PRESSED_BUTTON_COLOR.into();
                    *border_color = PRESSED_BUTTON_OUTLINE_COLOR.into();
                } else {
                    *color = HOVERED_BUTTON_COLOR.into();
                    *border_color = HOVERED_BUTTON_OUTLINE_COLOR.into();
                }
            }
            Interaction::None => {
                if is_current_mode {
                    *color = PRESSED_BUTTON_COLOR.into();
                    *border_color = PRESSED_BUTTON_OUTLINE_COLOR.into();
                } else {
                    *color = NORMAL_BUTTON_COLOR.into();
                    *border_color = NORMAL_BUTTON_OUTLINE_COLOR.into();
                }
            }
        }
    }
}
