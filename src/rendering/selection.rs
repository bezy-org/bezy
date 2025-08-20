//! Selection rendering systems
//!
//! This module handles all visual rendering for selection-related features:
//! - Selected point highlighting
//! - Selection marquee/rectangle
//! - Hover effects
//! - Control handle rendering for selected points
//!
//! All actual selection logic (what is selected, hit testing, etc.)
//! remains in the editing/selection module.

use crate::core::state::{AppState, FontIRAppState, TextEditorState};
use crate::editing::selection::components::{
    GlyphPointReference, Hovered, PointType, Selected, SelectionRect,
};
use crate::editing::selection::nudge::NudgeState;
use crate::editing::selection::{DragPointState, DragSelectionState};
use crate::editing::sort::ActiveSort;
use crate::systems::sort_manager::SortPointEntity;
use crate::ui::theme::*;
use crate::ui::themes::CurrentTheme;
use bevy::prelude::*;

/// Renders the selection marquee rectangle during drag selection
pub fn render_selection_marquee(
    mut gizmos: Gizmos,
    drag_state: Res<DragSelectionState>,
    marquee_query: Query<&SelectionRect>,
    theme: Res<CurrentTheme>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
) {
    // Only render marquee when in select mode
    if current_tool.get_current() != Some("select") {
        return;
    }

    if !drag_state.is_dragging {
        return;
    }

    // Try to find the selection rect from the query first
    if let Some(rect) = marquee_query.iter().next() {
        info!(
            "[render_selection_marquee] Drawing marquee: start={:?}, end={:?}",
            rect.start, rect.end
        );
        let start = rect.start;
        let end = rect.end;
        let color = theme.action_color();

        // Four corners
        let p1 = Vec2::new(start.x, start.y);
        let p2 = Vec2::new(end.x, start.y);
        let p3 = Vec2::new(end.x, end.y);
        let p4 = Vec2::new(start.x, end.y);

        // gizmos, start, end, color, dash_length, gap_length
        draw_dashed_line(&mut gizmos, p1, p2, color, 8.0, 4.0);
        draw_dashed_line(&mut gizmos, p2, p3, color, 8.0, 4.0);
        draw_dashed_line(&mut gizmos, p3, p4, color, 8.0, 4.0);
        draw_dashed_line(&mut gizmos, p4, p1, color, 8.0, 4.0);
    }
}

/// Helper to draw a dashed line between two points
fn draw_dashed_line(
    gizmos: &mut Gizmos,
    start: Vec2,
    end: Vec2,
    color: Color,
    dash_length: f32,
    gap_length: f32,
) {
    let total_length = (end - start).length();
    let direction = (end - start).normalize_or_zero();
    let mut current_length = 0.0;
    let mut draw = true;
    let mut current_position = start;

    while current_length < total_length {
        let segment_length = if draw { dash_length } else { gap_length };
        let next_length = (current_length + segment_length).min(total_length);
        let segment_end_position = start + direction * next_length;

        if draw {
            gizmos.line_2d(current_position, segment_end_position, color);
        }

        current_position = segment_end_position;
        current_length = next_length;
        draw = !draw;
    }
}

/// Renders visual feedback for selected entities
pub fn render_selected_entities(
    mut gizmos: Gizmos,
    selected_query: Query<(&GlobalTransform, &PointType), With<Selected>>,
    drag_point_state: Res<DragPointState>,
    knife_mode: Option<
        Res<crate::ui::toolbars::edit_mode_toolbar::knife::KnifeModeActive>,
    >,
    _nudge_state: Res<NudgeState>,
) {
    // Always render selected points - no dual-mode rendering

    let selected_count = selected_query.iter().count();
    if selected_count > 0 {
        debug!("Selection: Rendering {} selected entities", selected_count);
    }

    // Skip rendering if knife mode is active
    if let Some(knife_mode) = knife_mode {
        if knife_mode.0 {
            return;
        }
    }

    let selection_color = SELECTED_POINT_COLOR;
    let is_dragging = drag_point_state.is_dragging;

    // Render each selected point with selection styling
    for (transform, point_type) in &selected_query {
        let position = transform.translation().truncate();
        let position_3d = Vec3::new(
            position.x,
            position.y,
            transform.translation().z + SELECTION_Z_DEPTH_OFFSET,
        );
        let position_2d = position_3d.truncate();

        // Draw selection indicator with same shape and size as unselected points
        let point_radius = if point_type.is_on_curve {
            if USE_SQUARE_FOR_ON_CURVE {
                let adjusted_radius =
                    ON_CURVE_POINT_RADIUS * ON_CURVE_SQUARE_ADJUSTMENT;
                gizmos.rect_2d(
                    position_2d,
                    Vec2::splat(adjusted_radius * 2.0),
                    selection_color,
                );
                adjusted_radius
            } else {
                gizmos.circle_2d(
                    position_2d,
                    ON_CURVE_POINT_RADIUS,
                    selection_color,
                );
                ON_CURVE_POINT_RADIUS
            }
        } else {
            // Off-curve point - always a circle
            gizmos.circle_2d(
                position_2d,
                OFF_CURVE_POINT_RADIUS,
                selection_color,
            );
            OFF_CURVE_POINT_RADIUS
        };

        // Draw cross at the point, sized to match point radius
        let line_size = point_radius;
        gizmos.line_2d(
            Vec2::new(position_2d.x - line_size, position_2d.y),
            Vec2::new(position_2d.x + line_size, position_2d.y),
            selection_color,
        );
        gizmos.line_2d(
            Vec2::new(position_2d.x, position_2d.y - line_size),
            Vec2::new(position_2d.x, position_2d.y + line_size),
            selection_color,
        );

        // Make lines thicker when dragging
        if is_dragging {
            let offset = 0.5;
            gizmos.line_2d(
                Vec2::new(position_2d.x - line_size, position_2d.y + offset),
                Vec2::new(position_2d.x + line_size, position_2d.y + offset),
                selection_color,
            );
            gizmos.line_2d(
                Vec2::new(position_2d.x + offset, position_2d.y - line_size),
                Vec2::new(position_2d.x + offset, position_2d.y + line_size),
                selection_color,
            );
        }
    }
}

/// Renders all point entities (not just selected ones)
/// This is used to visualize all points in the active sort for debugging
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn render_all_point_entities(
    _gizmos: Gizmos,
    point_entities: Query<
        (Entity, &GlobalTransform, &PointType),
        (With<SortPointEntity>, Without<Selected>),
    >,
    selected_query: Query<Entity, With<Selected>>,
    _nudge_state: Res<NudgeState>,
    camera_query: Query<
        (&Camera, &GlobalTransform, &Projection),
        With<Camera2d>,
    >,
) {
    // Always render all points - no dual-mode rendering

    let point_count = point_entities.iter().count();

    // Debug camera information only when we have points to render
    if point_count > 0 {
        if let Ok((_camera, camera_transform, projection)) =
            camera_query.single()
        {
            let camera_pos = camera_transform.translation();
            let camera_scale = match projection {
                Projection::Orthographic(ortho) => ortho.scale,
                _ => 1.0,
            };
            debug!(
                "[render_all_point_entities] Camera pos: ({:.1}, {:.1}, {:.1}), scale: {:.3}",
                camera_pos.x, camera_pos.y, camera_pos.z, camera_scale
            );
        }
    }

    for (i, (entity, transform, point_type)) in
        point_entities.iter().enumerate()
    {
        // Skip if selected (already rendered by render_selected_entities)
        if selected_query.get(entity).is_ok() {
            continue;
        }

        let position = transform.translation().truncate();

        if i < 5 {
            info!("[render_all_point_entities] Rendering point {} at ({:.1}, {:.1}), is_on_curve={}", 
                i, position.x, position.y, point_type.is_on_curve);
        }

        // Point rendering is now handled by mesh-based system in point_backgrounds.rs
        // This includes point backgrounds, center dots, and properly-sized outlines
    }
}
