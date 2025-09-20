//! Input Consumer System
//!
//! This module provides the input consumer system that routes input events
//! to the appropriate handlers based on priority and current input mode.
//! It ensures that input is handled consistently and predictably across
//! the application.

use crate::core::io::input::{helpers, InputEvent, InputMode, InputState};
use crate::core::io::pointer::PointerInfo;
use crate::editing::selection::components::{
    GlyphPointReference, PointType, Selectable, Selected, SelectionRect,
};
use crate::editing::selection::{DragPointState, DragSelectionState, SelectionState};
use crate::editing::sort::manager::SortPointEntity;
use crate::editing::sort::ActiveSortState;
use crate::geometry::world_space::DPoint;
use crate::systems::ui_interaction::UiHoverState;
use bevy::prelude::*;

/// Trait for input consumers that handle specific types of input events
pub trait InputConsumer {
    /// Determine if this consumer should handle the given input event
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool;

    /// Handle the input event
    fn handle_input(&mut self, event: &InputEvent, input_state: &InputState);
}

/// Input consumer for selection functionality
#[derive(Resource, Default)]
pub struct SelectionInputConsumer {
    /// Events that need ECS processing (selection, smooth point toggle, etc.)
    pub pending_events: Vec<InputEvent>,
}

impl InputConsumer for SelectionInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        let is_mouse_event = matches!(
            event,
            InputEvent::MouseClick { .. } | InputEvent::MouseDrag { .. }
        );
        let is_select_mode = helpers::is_input_mode(input_state, InputMode::Select);

        if is_mouse_event {
            println!("[SELECTION INPUT CONSUMER] Mouse event detected. is_select_mode={}, current_mode={:?}",
                   is_select_mode, input_state.mode);
        }

        is_mouse_event && is_select_mode
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        println!(
            "[SELECTION INPUT CONSUMER] Storing event for ECS processing: {:?}",
            event
        );
        // Store the event for processing by the ECS system
        self.pending_events.push(event.clone());
    }
}

/// Input consumer for pen tool functionality
#[derive(Resource, Default)]
pub struct PenInputConsumer {
    /// Points that have been placed in the current path
    pub current_path: Vec<DPoint>,
    /// Whether the path should be closed (clicking near start point)
    pub should_close_path: bool,
    /// Whether we are currently placing a path
    pub is_drawing: bool,
}

impl InputConsumer for PenInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        let should_handle = matches!(
            event,
            InputEvent::MouseClick { .. }
                | InputEvent::MouseDrag { .. }
                | InputEvent::MouseRelease { .. }
        ) && helpers::is_input_mode(input_state, InputMode::Pen);

        // Debug: Only log when pen tool should handle input
        if should_handle && matches!(event, InputEvent::MouseClick { .. }) {
            println!("🖊️ PEN_DEBUG: Mouse click will be handled by pen tool");
        }

        should_handle
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseClick {
                button,
                position,
                modifiers: _,
            } => {
                println!(
                    "🖊️ PEN_DEBUG: Processing mouse click at ({:.1}, {:.1})",
                    position.x, position.y
                );

                if *button == bevy::input::mouse::MouseButton::Left {
                    let click_position = DPoint::new(position.x, position.y);

                    // Check if we should close the path
                    if self.current_path.len() > 2 {
                        if let Some(first_point) = self.current_path.first() {
                            let distance = click_position.to_raw().distance(first_point.to_raw());
                            println!("🖊️ PEN_DEBUG: Distance to first point: {distance:.1} (threshold: 16.0)");
                            if distance < 16.0 {
                                // CLOSE_PATH_THRESHOLD
                                self.should_close_path = true;
                                // Don't add this click as a new point since we're closing
                                println!("🖊️ PEN_DEBUG: CLOSING PATH - should_close_path={}, is_drawing={}", self.should_close_path, self.is_drawing);
                                info!("🖊️ [PEN] Closing path - clicked near start point");
                                // Mark for finalization - actual finalization happens in process_input_events
                                return;
                            }
                        }
                    }

                    // Add point to current path
                    self.current_path.push(click_position);
                    self.is_drawing = true;

                    println!(
                        "🖊️ PEN_DEBUG: Added point at ({:.1}, {:.1}), total points: {}",
                        click_position.x,
                        click_position.y,
                        self.current_path.len()
                    );
                } else if *button == bevy::input::mouse::MouseButton::Right {
                    info!("🖊️ [PEN] Right click - finishing open path");
                    if self.current_path.len() > 1 {
                        // Mark for finalization - actual finalization happens in process_input_events
                        self.is_drawing = false; // Will trigger finalization
                        println!(
                            "🖊️ PEN_DEBUG: RIGHT CLICK FINALIZATION - is_drawing={}, path_len={}",
                            self.is_drawing,
                            self.current_path.len()
                        );
                    }
                }
            }
            InputEvent::MouseDrag { .. } => {
                // For now, pen tool doesn't handle dragging
                // In the future, this could be used for handle manipulation
            }
            InputEvent::MouseRelease { .. } => {
                // Currently not needed for pen tool
            }
            _ => {}
        }
    }
}

impl PenInputConsumer {
    /// Finalize the current path and add it to the glyph
    #[allow(dead_code)]
    fn finalize_path(
        &mut self,
        fontir_app_state: &mut Option<&mut crate::core::state::FontIRAppState>,
        app_state_changed: &mut bevy::ecs::event::EventWriter<
            crate::editing::selection::events::AppStateChanged,
        >,
        active_sort_position: Vec2,
    ) {
        if self.current_path.len() < 2 {
            return;
        }

        info!(
            "🖊️ [PEN] Finalizing path with {} points (closed: {})",
            self.current_path.len(),
            self.should_close_path
        );

        // Convert world coordinates to relative coordinates for consistent storage
        let mut relative_path = Vec::new();
        for &point in &self.current_path {
            let world_pos = Vec2::new(point.x, point.y);
            let relative_pos = world_pos - active_sort_position;
            let relative_point = DPoint::new(relative_pos.x, relative_pos.y);
            relative_path.push(relative_point);
            info!(
                "🔍 PEN COORD CONVERSION: world=({:.1}, {:.1}) -> relative=({:.1}, {:.1})",
                world_pos.x, world_pos.y, relative_pos.x, relative_pos.y
            );
        }

        // Create a BezPath from the relative coordinates
        let mut bez_path = kurbo::BezPath::new();

        if let Some(&first_point) = relative_path.first() {
            info!(
                "🔍 PEN COORD DEBUG: Creating BezPath - first_relative_point=({:.1}, {:.1})",
                first_point.x, first_point.y
            );
            bez_path.move_to(kurbo::Point::new(
                first_point.x as f64,
                first_point.y as f64,
            ));

            for (i, &point) in relative_path.iter().skip(1).enumerate() {
                info!(
                    "🔍 PEN COORD DEBUG: Adding line_to relative_point[{}]=({:.1}, {:.1})",
                    i + 1,
                    point.x,
                    point.y
                );
                bez_path.line_to(kurbo::Point::new(point.x as f64, point.y as f64));
            }

            if self.should_close_path {
                bez_path.close_path();
            }
        }

        // Add the BezPath to the FontIR glyph data if available
        if let Some(ref mut fontir_state) = fontir_app_state {
            let current_glyph_name = fontir_state.current_glyph.clone();
            if let Some(current_glyph_name) = current_glyph_name {
                // Get the current location
                let location = fontir_state.current_location.clone();
                let key = (current_glyph_name.clone(), location);

                // Get or create a working copy
                let working_copy_exists = fontir_state.working_copies.contains_key(&key);

                if !working_copy_exists {
                    // Create working copy from original FontIR data
                    if let Some(fontir_glyph) = fontir_state.glyph_cache.get(&current_glyph_name) {
                        if let Some((_location, instance)) = fontir_glyph.sources().iter().next() {
                            let working_copy =
                                crate::core::state::fontir_app_state::EditableGlyphInstance::from(
                                    instance,
                                );
                            fontir_state
                                .working_copies
                                .insert(key.clone(), working_copy);
                        }
                    }
                }

                // Add the new contour to the working copy
                if let Some(working_copy) = fontir_state.working_copies.get_mut(&key) {
                    working_copy.contours.push(bez_path.clone());
                    working_copy.is_dirty = true;
                    app_state_changed.write(crate::editing::selection::events::AppStateChanged);

                    info!(
                        "🖊️ [PEN] Added contour with {} elements to glyph '{}'. Total contours: {}",
                        bez_path.elements().len(),
                        current_glyph_name,
                        working_copy.contours.len()
                    );
                } else {
                    warn!(
                        "🖊️ [PEN] Could not create working copy for glyph '{}'",
                        current_glyph_name
                    );
                }
            } else {
                warn!("🖊️ [PEN] No current glyph selected");
            }
        } else {
            warn!("🖊️ [PEN] FontIR app state not available");
        }

        info!("🖊️ [PEN] Path finalized successfully - added to FontIR data");

        // Reset state
        self.current_path.clear();
        self.is_drawing = false;
        self.should_close_path = false;
    }
}

/// Input consumer for knife tool functionality
#[derive(Resource, Default)]
pub struct KnifeInputConsumer {
    /// The current gesture state
    pub gesture: KnifeGestureState,
    /// Whether shift key is pressed (for axis-aligned cuts)
    pub shift_locked: bool,
    /// Intersection points for visualization
    pub intersections: Vec<Vec2>,
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

impl InputConsumer for KnifeInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        let is_right_event = matches!(
            event,
            InputEvent::MouseClick { .. } | InputEvent::MouseDrag { .. }
        );
        let is_knife_mode = helpers::is_input_mode(input_state, InputMode::Knife);

        if is_right_event {
            debug!("🔪 KNIFE_INPUT_CONSUMER: should_handle_input - event: {:?}, is_knife_mode: {}, current_mode: {:?}", 
                   event, is_knife_mode, input_state.mode);
        }

        is_right_event && is_knife_mode
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        debug!("🔪 KNIFE INPUT CONSUMER: Handling event: {:?}", event);

        match event {
            InputEvent::MouseClick {
                button, position, ..
            } => {
                info!(
                    "🔪 KNIFE INPUT CONSUMER: Mouse click: {:?} at {:?} - EVENT CONSUMED",
                    button, position
                );
                if button == &bevy::input::mouse::MouseButton::Left {
                    let world_position = Vec2::new(position.x, position.y);
                    self.gesture = KnifeGestureState::Cutting {
                        start: world_position,
                        current: world_position,
                    };
                    self.intersections.clear();
                    info!(
                        "🔪 KNIFE INPUT CONSUMER: Started cutting at {:?}",
                        world_position
                    );
                }
            }
            InputEvent::MouseDrag {
                current_position, ..
            } => {
                debug!(
                    "🔪 KNIFE INPUT CONSUMER: Mouse drag at {:?} - EVENT CONSUMED",
                    current_position
                );
                if let KnifeGestureState::Cutting { start, .. } = self.gesture {
                    let world_position = Vec2::new(current_position.x, current_position.y);
                    self.gesture = KnifeGestureState::Cutting {
                        start,
                        current: world_position,
                    };

                    // Update intersections for preview
                    self.update_intersections(start, world_position);
                    debug!("🔪 KNIFE INPUT CONSUMER: Dragging to {:?}", world_position);
                }
            }
            InputEvent::MouseRelease {
                button, position, ..
            } => {
                debug!(
                    "🔪 KNIFE INPUT CONSUMER: Mouse release: {:?} at {:?}",
                    button, position
                );
                if button == &bevy::input::mouse::MouseButton::Left {
                    if let KnifeGestureState::Cutting { start, current } = self.gesture {
                        info!("🔪 KNIFE INPUT CONSUMER: Knife cut gesture completed from {:?} to {:?}", start, current);
                        // Note: State reset is handled by the knife tool's cutting system
                        // to avoid race conditions between input handling and cutting logic
                    }

                    // DON'T reset state here - let the cutting system handle it
                    // This prevents race conditions where state is reset before cutting happens
                    // self.gesture = KnifeGestureState::Ready;
                    // self.intersections.clear();
                }
            }
            _ => {}
        }
    }
}

impl KnifeInputConsumer {
    /// Update intersection points for preview
    fn update_intersections(&mut self, _start: Vec2, _end: Vec2) {
        self.intersections.clear();
        // Real intersection detection will be handled by the render system
        // This is just a placeholder that gets overridden
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

/// Input consumer for shape tool functionality
#[derive(Resource, Default)]
pub struct ShapeInputConsumer;

impl InputConsumer for ShapeInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        let is_shape_event = matches!(
            event,
            InputEvent::MouseClick { .. } | InputEvent::MouseDrag { .. }
        );
        let is_shape_mode = helpers::is_input_mode(input_state, InputMode::Shape);

        // Debug: Log when we should handle input
        if is_shape_event {
            info!(
                "🔧 SHAPE INPUT CONSUMER: Mouse event - input_mode: {:?}, should_handle: {}",
                input_state.mode,
                is_shape_event && is_shape_mode
            );
        }

        is_shape_event && is_shape_mode
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        info!("🔧 SHAPE INPUT CONSUMER: Handling input event: {:?}", event);
        if let InputEvent::MouseClick {
            button,
            position,
            modifiers: _,
        } = event
        {
            info!(
                "🔧 SHAPE INPUT CONSUMER: Mouse click: {:?} at {:?} - EVENT CONSUMED",
                button, position
            );
            // Shape tool logic would go here
        }
        if let InputEvent::MouseDrag { .. } = event {
            info!("🔧 SHAPE INPUT CONSUMER: Mouse drag - EVENT CONSUMED");
        }
    }
}

/// Input consumer for hyper tool functionality
#[derive(Resource, Default)]
pub struct HyperInputConsumer;

impl InputConsumer for HyperInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        // Handle hyper tool events
        matches!(
            event,
            InputEvent::MouseClick { .. } | InputEvent::MouseDrag { .. }
        ) && helpers::is_input_mode(input_state, InputMode::Hyper)
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        if let InputEvent::MouseClick {
            button,
            position,
            modifiers: _,
        } = event
        {
            debug!("[HYPER] Mouse click: {:?} at {:?}", button, position);
            // Hyper tool logic would go here
        }
    }
}

/// Input consumer for text editing functionality
#[derive(Resource, Default)]
pub struct TextInputConsumer;

impl InputConsumer for TextInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        // Handle text input events
        matches!(
            event,
            InputEvent::KeyPress { .. } | InputEvent::TextInput { .. }
        ) && helpers::is_input_mode(input_state, InputMode::Text)
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::KeyPress { key, modifiers: _ } => {
                debug!("[TEXT] Key press: {:?}", key);
                // Text editing logic would go here
            }
            InputEvent::TextInput { text } => {
                debug!("[TEXT] Text input: '{}'", text);
                // Text input logic would go here
            }
            _ => {}
        }
    }
}

/// Input consumer for camera control functionality
#[derive(Resource, Default)]
pub struct CameraInputConsumer;

impl InputConsumer for CameraInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        // Handle camera control events (low priority)
        matches!(
            event,
            InputEvent::MouseDrag { .. } | InputEvent::MouseWheel { .. }
        ) && !helpers::is_input_mode(input_state, InputMode::Text)
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        match event {
            InputEvent::MouseDrag {
                button,
                start_position,
                current_position,
                modifiers: _,
                delta: _,
            } => {
                if *button == MouseButton::Middle {
                    debug!(
                        "[CAMERA] Middle mouse drag: from {:?} to {:?}",
                        start_position, current_position
                    );
                    // Camera pan logic would go here
                }
            }
            InputEvent::MouseWheel { delta } => {
                debug!("[CAMERA] Mouse wheel: {:?}", delta);
                // Camera zoom logic would go here
            }
            _ => {}
        }
    }
}

/// Input consumer for measurement tool functionality
#[derive(Resource, Default)]
pub struct MeasureInputConsumer {
    /// The current gesture state
    pub gesture: MeasureGestureState,
    /// Whether shift key is pressed (for axis-aligned measurements)
    pub shift_locked: bool,
    /// Intersection points for visualization
    pub intersections: Vec<Vec2>,
}

/// The state of the measure gesture
#[derive(Debug, Clone, PartialEq, Default, Copy)]
pub enum MeasureGestureState {
    /// Ready to start measuring
    #[default]
    Ready,
    /// Currently dragging a measure line
    Measuring { start: Vec2, current: Vec2 },
}

impl InputConsumer for MeasureInputConsumer {
    fn should_handle_input(&self, event: &InputEvent, input_state: &InputState) -> bool {
        let is_right_event = matches!(
            event,
            InputEvent::MouseClick { .. } | InputEvent::MouseDrag { .. }
        );
        let is_measure_mode = helpers::is_input_mode(input_state, InputMode::Measure);

        if is_right_event {
            debug!("📏 MEASURE_INPUT_CONSUMER: should_handle_input - event: {:?}, is_measure_mode: {}, current_mode: {:?}", 
                   event, is_measure_mode, input_state.mode);
        }

        is_right_event && is_measure_mode
    }

    fn handle_input(&mut self, event: &InputEvent, _input_state: &InputState) {
        info!("📏 MEASURE INPUT CONSUMER: Handling event: {:?}", event);

        match event {
            InputEvent::MouseClick {
                button, position, ..
            } => {
                info!(
                    "📏 MEASURE INPUT CONSUMER: Mouse click: {:?} at {:?} - EVENT CONSUMED",
                    button, position
                );
                if button == &bevy::input::mouse::MouseButton::Left {
                    let world_position = Vec2::new(position.x, position.y);
                    self.gesture = MeasureGestureState::Measuring {
                        start: world_position,
                        current: world_position,
                    };
                    self.intersections.clear();
                    info!(
                        "📏 MEASURE INPUT CONSUMER: Started measuring at {:?}",
                        world_position
                    );
                }
            }
            InputEvent::MouseDrag {
                current_position, ..
            } => {
                debug!(
                    "📏 MEASURE INPUT CONSUMER: Mouse drag at {:?} - EVENT CONSUMED",
                    current_position
                );
                if let MeasureGestureState::Measuring { start, .. } = self.gesture {
                    let world_position = Vec2::new(current_position.x, current_position.y);
                    self.gesture = MeasureGestureState::Measuring {
                        start,
                        current: world_position,
                    };

                    // Update intersections for preview
                    self.update_intersections(start, world_position);
                    debug!(
                        "📏 MEASURE INPUT CONSUMER: Dragging to {:?}",
                        world_position
                    );
                }
            }
            InputEvent::MouseRelease {
                button, position, ..
            } => {
                debug!(
                    "📏 MEASURE INPUT CONSUMER: Mouse release: {:?} at {:?}",
                    button, position
                );
                if button == &bevy::input::mouse::MouseButton::Left {
                    if let MeasureGestureState::Measuring { start, current } = self.gesture {
                        info!("📏 MEASURE INPUT CONSUMER: Measure gesture completed from {:?} to {:?}", start, current);
                    }

                    // Reset state immediately after measurement
                    self.gesture = MeasureGestureState::Ready;
                    self.intersections.clear();
                }
            }
            _ => {}
        }
    }
}

impl MeasureInputConsumer {
    /// Update intersection points for preview
    fn update_intersections(&mut self, _start: Vec2, _end: Vec2) {
        self.intersections.clear();
        // Real intersection detection will be handled by the render system
        // This is just a placeholder that gets overridden
    }

    /// Get the measuring line with axis locking if shift is pressed
    pub fn get_measuring_line(&self) -> Option<(Vec2, Vec2)> {
        match self.gesture {
            MeasureGestureState::Measuring { start, current } => {
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
            MeasureGestureState::Ready => None,
        }
    }
}

/// System to process input events and route them to appropriate consumers
#[allow(clippy::too_many_arguments)]
pub fn process_input_events(
    mut input_events: EventReader<InputEvent>,
    input_state: Res<InputState>,
    mut selection_consumer: ResMut<SelectionInputConsumer>,
    _pen_consumer: ResMut<PenInputConsumer>,
    mut knife_consumer: ResMut<KnifeInputConsumer>,
    mut shape_consumer: ResMut<ShapeInputConsumer>,
    mut hyper_consumer: ResMut<HyperInputConsumer>,
    mut text_consumer: ResMut<TextInputConsumer>,
    mut camera_consumer: ResMut<CameraInputConsumer>,
    mut measure_consumer: ResMut<MeasureInputConsumer>,
    _pen_tool_state: Option<ResMut<crate::tools::pen::PenToolState>>,
    _fontir_app_state: Option<ResMut<crate::core::state::FontIRAppState>>,
    _app_state_changed: bevy::ecs::event::EventWriter<
        crate::editing::selection::events::AppStateChanged,
    >,
    _active_sort_query: Query<
        (Entity, &crate::editing::sort::Sort, &Transform),
        With<crate::editing::sort::ActiveSort>,
    >,
) {
    let events: Vec<_> = input_events.read().collect();
    if !events.is_empty() {
        println!("🖊️ PEN_DEBUG: Processing {} input events", events.len());
    }

    // Get active sort position for coordinate conversion - disabled since pen tool doesn't use InputConsumer anymore
    // let _active_sort_position = active_sort_query.iter().next()
    //     .map(|(_, _, transform)| transform.translation.truncate())
    //     .unwrap_or(Vec2::ZERO);

    for event in events {
        if matches!(event, InputEvent::MouseClick { .. }) {
            println!(
                "🖊️ PEN_DEBUG: Mouse click event detected, current input mode: {:?}",
                input_state.mode
            );
        }

        // Route events to consumers based on priority
        // High priority: Text input
        if text_consumer.should_handle_input(event, &input_state) {
            text_consumer.handle_input(event, &input_state);
            continue;
        }

        // Mode-specific consumers

        if knife_consumer.should_handle_input(event, &input_state) {
            info!(
                "🔪 INPUT_CONSUMER: Routing event to knife consumer: {:?}",
                event
            );
            knife_consumer.handle_input(event, &input_state);
            continue;
        }

        // Shape tool input consumption
        if shape_consumer.should_handle_input(event, &input_state) {
            shape_consumer.handle_input(event, &input_state);
            continue; // Consume the event - don't let it fall through to selection
        }

        if hyper_consumer.should_handle_input(event, &input_state) {
            hyper_consumer.handle_input(event, &input_state);
            continue;
        }

        if measure_consumer.should_handle_input(event, &input_state) {
            measure_consumer.handle_input(event, &input_state);
            continue;
        }

        // Normal mode consumers
        if selection_consumer.should_handle_input(event, &input_state) {
            selection_consumer.handle_input(event, &input_state);
            continue;
        }

        // Low priority: Camera control
        if camera_consumer.should_handle_input(event, &input_state) {
            camera_consumer.handle_input(event, &input_state);
            continue;
        }

        debug!("[INPUT CONSUMER] No consumer handled event: {:?}", event);
    }
}

/// Plugin for the input consumer system
/// System to process selection events stored by SelectionInputConsumer
pub fn process_selection_events(
    mut commands: Commands,
    mut selection_consumer: ResMut<SelectionInputConsumer>,
    time: Res<Time>,
    mut double_click_state: ResMut<crate::editing::selection::input::mouse::DoubleClickState>,
    selectable_query: Query<
        (
            Entity,
            &GlobalTransform,
            Option<&GlyphPointReference>,
            Option<&PointType>,
        ),
        With<Selectable>,
    >,
    active_sort_state: Res<crate::editing::sort::ActiveSortState>,
    sort_point_entities: Query<&crate::editing::sort::manager::SortPointEntity>,
    mut enhanced_points_query: Query<
        &mut crate::editing::selection::enhanced_point_component::EnhancedPointType,
    >,
    point_refs_query: Query<&crate::editing::selection::components::GlyphPointReference>,
    mut selection_state: ResMut<SelectionState>,
    _selected_query: Query<Entity, With<Selected>>,
    mut visual_update_tracker: ResMut<crate::rendering::glyph_renderer::SortVisualUpdateTracker>,
    mut enhanced_attributes: ResMut<
        crate::editing::selection::entity_management::EnhancedPointAttributes,
    >,
) {
    if selection_consumer.pending_events.is_empty() {
        return;
    }

    // Process all pending events
    let events = std::mem::take(&mut selection_consumer.pending_events);

    for event in events {
        if let InputEvent::MouseClick {
            button,
            position,
            modifiers,
        } = event
        {
            if button == bevy::input::mouse::MouseButton::Left {
                println!(
                    "[SELECTION PROCESSOR] Processing mouse click at {:?}",
                    position
                );

                // Use the existing selection logic from the original mouse.rs
                let active_sort_entity = active_sort_state
                    .active_sort_entity
                    .unwrap_or(Entity::PLACEHOLDER);

                // Check for point selection and double-click
                if let Some(clicked_entity) =
                    crate::editing::selection::input::mouse::find_clicked_point(
                        &position,
                        &selectable_query,
                        active_sort_entity,
                        &sort_point_entities,
                    )
                {
                    // Handle double-click detection for smooth point toggle
                    let now = time.elapsed_secs();
                    let is_double_click =
                        if let Some(last_click) = double_click_state.last_click_time {
                            (now - last_click)
                            < crate::editing::selection::input::mouse::DOUBLE_CLICK_THRESHOLD_SECS
                            && double_click_state.last_clicked_entity == Some(clicked_entity)
                        } else {
                            false
                        };

                    if is_double_click {
                        println!("[SELECTION PROCESSOR] Double-click detected - toggling smooth point for entity {:?}", clicked_entity);

                        // Get point reference information for enhanced attributes
                        let point_ref = if let Ok(point_ref) = point_refs_query.get(clicked_entity)
                        {
                            point_ref
                        } else {
                            println!("[SELECTION PROCESSOR] Could not get point reference for entity {:?}", clicked_entity);
                            continue;
                        };

                        // Create key for enhanced attributes lookup
                        let attr_key = (
                            point_ref.glyph_name.clone(),
                            point_ref.contour_index,
                            point_ref.point_index,
                        );

                        // Handle smooth point toggle
                        match enhanced_points_query.get_mut(clicked_entity) {
                            Ok(mut enhanced_point) => {
                                let current_smooth =
                                    enhanced_point.ufo_point.smooth.unwrap_or(false);
                                let new_smooth = !current_smooth;
                                enhanced_point.ufo_point.smooth = Some(new_smooth);

                                // Also update enhanced attributes for UFO save persistence
                                let ufo_point = enhanced_attributes
                                    .attributes
                                    .entry(attr_key.clone())
                                    .or_insert_with(|| {
                                        crate::core::state::ufo_point::UfoPoint::line_to(0.0, 0.0)
                                    });
                                ufo_point.smooth = Some(new_smooth);

                                println!("[SELECTION PROCESSOR] Toggled smooth point: entity {:?} is now smooth={}",
                                      clicked_entity, new_smooth);

                                // IMPORTANT: Trigger visual update so the point shape changes immediately
                                visual_update_tracker.needs_update = true;
                                println!("[SELECTION PROCESSOR] Triggered visual update for smooth point change");

                                // Also ensure the rendering data is marked for update
                                println!("[SELECTION PROCESSOR] Enhanced point component updated with smooth={}", new_smooth);

                                // IMPORTANT: If point became smooth, make handles collinear
                                if new_smooth {
                                    println!("[SELECTION PROCESSOR] Point became smooth - applying collinear handle constraints");
                                    // TODO(human): Implement collinear handle constraint logic here
                                    // This should find adjacent off-curve points and make them collinear with this point
                                    //
                                    // Steps needed:
                                    // 1. Get the glyph name and contour index from the clicked point
                                    // 2. Find all points in the same contour, sorted by point_index
                                    // 3. Locate the current point in the sequence
                                    // 4. Find adjacent off-curve control points (before and after)
                                    // 5. Calculate the line through the smooth point and one handle
                                    // 6. Reposition the other handle to be collinear
                                    // 7. Update both Transform components (for immediate visual) and FontIR data (for persistence)
                                    //
                                    // Consider using GlyphPointReference to navigate the contour structure
                                    // and both Transform queries for immediate updates and FontIR updates for data persistence
                                }
                            }
                            Err(_) => {
                                // Entity doesn't have EnhancedPointType component, add it with default values
                                println!("[SELECTION PROCESSOR] Adding EnhancedPointType component to entity {:?}", clicked_entity);

                                // Create a default UfoPoint (we'll use Line type as a safe default for on-curve points)
                                let mut ufo_point =
                                    crate::core::state::ufo_point::UfoPoint::line_to(0.0, 0.0);
                                ufo_point.smooth = Some(true); // Set to smooth since we're toggling

                                let enhanced_point = crate::editing::selection::enhanced_point_component::EnhancedPointType::new(ufo_point.clone());

                                commands.entity(clicked_entity).insert(enhanced_point);

                                // Also update enhanced attributes for UFO save persistence
                                enhanced_attributes.attributes.insert(attr_key, ufo_point);

                                println!("[SELECTION PROCESSOR] Added EnhancedPointType and set smooth=true for entity {:?}", clicked_entity);

                                // IMPORTANT: Trigger visual update so the point shape changes immediately
                                visual_update_tracker.needs_update = true;
                                println!("[SELECTION PROCESSOR] Triggered visual update for smooth point change");
                            }
                        }

                        // Reset double-click state
                        double_click_state.last_click_time = None;
                        double_click_state.last_clicked_entity = None;
                    } else {
                        // Update double-click state for next potential double-click
                        double_click_state.last_click_time = Some(now);
                        double_click_state.last_clicked_entity = Some(clicked_entity);

                        // Handle single-click selection
                        if modifiers.ctrl || modifiers.super_key {
                            // Multi-select: toggle selection
                            if selection_state.selected.contains(&clicked_entity) {
                                commands.entity(clicked_entity).remove::<Selected>();
                                selection_state.selected.remove(&clicked_entity);
                                println!("[SELECTION PROCESSOR] Ctrl+click: removed entity {:?} from selection", clicked_entity);
                            } else {
                                commands.entity(clicked_entity).insert(Selected);
                                selection_state.selected.insert(clicked_entity);
                                println!("[SELECTION PROCESSOR] Ctrl+click: added entity {:?} to selection", clicked_entity);
                            }
                        } else {
                            // Single select: clear others and select this one
                            for entity in selection_state.selected.clone() {
                                commands.entity(entity).remove::<Selected>();
                            }
                            selection_state.selected.clear();

                            commands.entity(clicked_entity).insert(Selected);
                            selection_state.selected.insert(clicked_entity);
                            println!("[SELECTION PROCESSOR] Single click: selected entity {:?} exclusively", clicked_entity);
                        }
                    }
                } else {
                    println!("[SELECTION PROCESSOR] No point clicked - clearing selection");
                    // Clear selection if clicking empty space
                    for entity in selection_state.selected.clone() {
                        commands.entity(entity).remove::<Selected>();
                    }
                    selection_state.selected.clear();
                }
            }
        }
    }
}

pub struct InputConsumerPlugin;

impl Plugin for InputConsumerPlugin {
    fn build(&self, app: &mut App) {
        info!("[INPUT CONSUMER] Registering InputConsumerPlugin");

        // Register all input consumers as resources
        app.init_resource::<SelectionInputConsumer>()
            .init_resource::<PenInputConsumer>()
            .init_resource::<KnifeInputConsumer>()
            .init_resource::<ShapeInputConsumer>()
            .init_resource::<HyperInputConsumer>()
            .init_resource::<TextInputConsumer>()
            .init_resource::<CameraInputConsumer>()
            .init_resource::<MeasureInputConsumer>()
            .add_systems(
                Update,
                (process_input_events, process_selection_events).chain(),
            );

        info!("[INPUT CONSUMER] InputConsumerPlugin registration complete");
    }
}
