//! Point drag handling for selection

use crate::core::io::pointer::PointerInfo;
use crate::core::settings::BezySettings;
use crate::core::state::{AppState, FontIRAppState};
use crate::editing::edit_type::EditType;
use crate::editing::selection::components::{GlyphPointReference, Selected};
use crate::editing::selection::nudge::{EditEvent, PointCoordinates};
use crate::editing::selection::DragPointState;
use bevy::input::ButtonInput;
use bevy::log::{debug, warn};
use bevy::prelude::*;

/// System to handle advanced point dragging with constraints and snapping
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn handle_point_drag(
    pointer_info: Res<PointerInfo>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut drag_point_state: ResMut<DragPointState>,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &mut PointCoordinates,
            Option<&GlyphPointReference>,
            Option<&crate::systems::sort_manager::SortCrosshair>,
        ),
        With<Selected>,
    >,
    mut app_state: Option<ResMut<AppState>>,
    mut fontir_app_state: Option<ResMut<FontIRAppState>>,
    mut event_writer: EventWriter<EditEvent>,
    settings: Res<BezySettings>,
) {
    
    // Only drag if the resource says we are
    if !drag_point_state.is_dragging {
        return;
    }

    let cursor_pos = pointer_info.design.to_raw();
    drag_point_state.current_position = Some(cursor_pos);

    if let Some(start_pos) = drag_point_state.start_position {
        let total_movement = cursor_pos - start_pos;
        let mut movement = total_movement;

        // Handle constrained movement with Shift key
        if keyboard_input.pressed(KeyCode::ShiftLeft)
            || keyboard_input.pressed(KeyCode::ShiftRight)
        {
            if total_movement.x.abs() > total_movement.y.abs() {
                movement.y = 0.0; // Constrain to horizontal
            } else {
                movement.x = 0.0; // Constrain to vertical
            }
        }

        let mut updated_count = 0;

        for (
            entity,
            mut transform,
            mut coordinates,
            point_ref,
            sort_crosshair,
        ) in &mut query
        {
            if let Some(original_pos) =
                drag_point_state.original_positions.get(&entity)
            {
                let new_pos = *original_pos + movement;

                // Handle sort crosshair drag (no snapping, keep on top)
                if sort_crosshair.is_some() {
                    transform.translation.x = new_pos.x;
                    transform.translation.y = new_pos.y;
                    transform.translation.z = 25.0; // Keep crosshairs on top
                    coordinates.x = new_pos.x;
                    coordinates.y = new_pos.y;
                }
                // Handle glyph point drag (with snapping)
                else if let Some(point_ref) = point_ref {
                    // Apply grid snapping if enabled
                    let snapped_pos = settings.apply_grid_snap(new_pos);

                    transform.translation.x = snapped_pos.x;
                    transform.translation.y = snapped_pos.y;
                    transform.translation.z = 5.0; // Keep glyph points above background
                    coordinates.x = snapped_pos.x;
                    coordinates.y = snapped_pos.y;

                    // Try FontIR first, then fallback to UFO AppState
                    let mut handled = false;

                    if let Some(ref mut fontir_state) = fontir_app_state {
                        // Try to update FontIR data
                        match fontir_state.update_point_position(
                            &point_ref.glyph_name,
                            point_ref.contour_index,
                            point_ref.point_index,
                            transform.translation.x as f64,
                            transform.translation.y as f64,
                        ) {
                            Ok(was_updated) => {
                                if was_updated {
                                    updated_count += 1;
                                    handled = true;
                                }
                            }
                            Err(e) => {
                                warn!("Failed to update FontIR point: {}", e);
                            }
                        }
                    }

                    // Fallback to UFO AppState if FontIR didn't handle it
                    if !handled && app_state.is_some() {
                        if let Some(ref mut app_state) = app_state {
                            let updated = app_state.set_point_position(
                                &point_ref.glyph_name,
                                point_ref.contour_index,
                                point_ref.point_index,
                                transform.translation.x as f64,
                                transform.translation.y as f64,
                            );
                            if updated {
                                updated_count += 1;
                                handled = true;
                            }
                        }
                    }

                    // If neither FontIR nor UFO handled it, just track the Transform update
                    if !handled {
                        updated_count += 1;
                        debug!("Point update handled via Transform only (no source data update)");
                    }
                }
                // Handle other draggable entities (no snapping, normal Z layer)
                else {
                    transform.translation.x = new_pos.x;
                    transform.translation.y = new_pos.y;
                    transform.translation.z = 10.0; // Middle layer
                    coordinates.x = new_pos.x;
                    coordinates.y = new_pos.y;
                }
            }
        }

        if updated_count > 0 {
            debug!("Updated {} UFO points during drag", updated_count);

            // Send edit event
            event_writer.write(EditEvent {
                edit_type: EditType::Normal,
            });
        }
    }
}
