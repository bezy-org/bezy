//! Coordinate Pane Module
//!
//! This module implements a floating panel that displays coordinates
//! and dimensions of selected elements.

#![allow(unused_mut)]
#![allow(unused_variables)]

use crate::editing::selection::components::Selected;
use crate::utils::embedded_assets::{AssetServerFontExt, EmbeddedFonts};
use crate::geometry::quadrant::Quadrant;
use crate::ui::theme::*;
use crate::ui::themes::{CurrentTheme, UiBorderRadius};
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy::ui::UiRect;

// ============================================================================
// DESIGN CONSTANTS - Easy to tweak for designers
// ============================================================================

/// Size of the quadrant selector widget
const QUADRANT_SELECTOR_SIZE: f32 = 96.0;

/// Size of each quadrant button
const QUADRANT_BUTTON_SIZE: f32 = 24.0;

/// Width of grid lines
const GRID_LINE_WIDTH: f32 = 2.0;

/// Button center positions - grid lines will align with these
/// Using absolute positioning to ensure perfect alignment
const BUTTON_CENTER_POSITIONS: [f32; 3] = [
    QUADRANT_SELECTOR_SIZE * 0.5 / 3.0, // 16.0 for 96px container
    QUADRANT_SELECTOR_SIZE * 0.5,       // 48.0 (center)
    QUADRANT_SELECTOR_SIZE * 2.5 / 3.0, // 80.0 for 96px container
];

/// Spacing between coordinate label and value
const LABEL_VALUE_SPACING: f32 = 8.0;

/// Spacing between coordinate rows (use theme constant)
const ROW_SPACING: f32 = WIDGET_ROW_LEADING;

/// Extra spacing before quadrant selector
const QUADRANT_SELECTOR_MARGIN: f32 = 16.0;

// ============================================================================
// COMPONENTS & RESOURCES
// ============================================================================

/// Resource that tracks the current state of coordinate selection
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct CoordinateSelection {
    /// Number of elements currently selected
    pub count: usize,
    /// Currently active quadrant for reference point
    pub quadrant: Quadrant,
    /// Bounding rectangle of all selected elements
    pub frame: Rect,
}

/// Component marker for the coordinate pane
#[derive(Component, Default)]
pub struct CoordinatePane;

/// Component markers for coordinate value text
#[derive(Component, Default)]
pub struct XValue;

#[derive(Component, Default)]
pub struct YValue;

#[derive(Component, Default)]
pub struct WidthValue;

#[derive(Component, Default)]
pub struct HeightValue;

/// Component for quadrant buttons
#[derive(Component)]
pub struct QuadrantButton(pub Quadrant);

impl Default for QuadrantButton {
    fn default() -> Self {
        Self(Quadrant::Center)
    }
}

// ============================================================================
// PLUGIN
// ============================================================================

pub struct CoordinatePanePlugin;

impl Plugin for CoordinatePanePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CoordinateSelection>()
            .register_type::<Quadrant>()
            .init_resource::<CoordinateSelection>()
            .add_systems(Startup, spawn_coordinate_pane)
            .add_systems(
                Update,
                (
                    update_coordinate_selection,
                    update_coordinate_display,
                    handle_quadrant_buttons,
                    toggle_pane_visibility,
                ),
            );
    }
}

// ============================================================================
// UI CREATION - Clean builder pattern approach
// ============================================================================

/// Spawns the coordinate pane in the bottom-right corner
pub fn spawn_coordinate_pane(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    embedded_fonts: Res<EmbeddedFonts>,
    theme: Res<CurrentTheme>,
) {
    let position = UiRect {
        right: Val::Px(theme.theme().widget_margin()),
        bottom: Val::Px(theme.theme().widget_margin()),
        top: Val::Auto,
        left: Val::Auto,
    };

    commands
        .spawn(create_widget_style(
            &asset_server,
            &theme,
            PositionType::Absolute,
            position,
            CoordinatePane,
            "CoordinatePane",
        ))
        .with_children(|parent| {
            // ============ COORDINATE ROWS ============

            // X coordinate row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(ROW_SPACING)),
                    ..default()
                })
                .with_children(|row| {
                    // X label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(LABEL_VALUE_SPACING)),
                            ..default()
                        },
                        Text::new("X:"),
                        TextFont {
                            font: asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_primary()),
                    ));
                    // X value
                    row.spawn((
                        Text::new("0"),
                        TextFont {
                            font: asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_secondary()),
                        XValue,
                    ));
                });

            // Y coordinate row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(ROW_SPACING)),
                    ..default()
                })
                .with_children(|row| {
                    // Y label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(LABEL_VALUE_SPACING)),
                            ..default()
                        },
                        Text::new("Y:"),
                        TextFont {
                            font: asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_primary()),
                    ));
                    // Y value
                    row.spawn((
                        Text::new("0"),
                        TextFont {
                            font: asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_secondary()),
                        YValue,
                    ));
                });

            // Width coordinate row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(ROW_SPACING)),
                    ..default()
                })
                .with_children(|row| {
                    // Width label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(LABEL_VALUE_SPACING)),
                            ..default()
                        },
                        Text::new("W:"),
                        TextFont {
                            font: asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_primary()),
                    ));
                    // Width value
                    row.spawn((
                        Text::new("0"),
                        TextFont {
                            font: asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_secondary()),
                        WidthValue,
                    ));
                });

            // Height coordinate row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(QUADRANT_SELECTOR_MARGIN)),
                    ..default()
                })
                .with_children(|row| {
                    // Height label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(LABEL_VALUE_SPACING)),
                            ..default()
                        },
                        Text::new("H:"),
                        TextFont {
                            font: asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_primary()),
                    ));
                    // Height value
                    row.spawn((
                        Text::new("0"),
                        TextFont {
                            font: asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_secondary()),
                        HeightValue,
                    ));
                });

            // ============ QUADRANT SELECTOR ============

            parent
                .spawn(Node {
                    position_type: PositionType::Relative,
                    width: Val::Px(QUADRANT_SELECTOR_SIZE),
                    height: Val::Px(QUADRANT_SELECTOR_SIZE),
                    ..default()
                })
                .with_children(|container| {
                    // Grid lines background
                    container
                        .spawn(Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        })
                        .with_children(|lines| {
                            // Horizontal grid lines - aligned with button centers
                            for &y_pos in BUTTON_CENTER_POSITIONS.iter() {
                                lines.spawn((
                                    Node {
                                        position_type: PositionType::Absolute,
                                        width: Val::Px(
                                            BUTTON_CENTER_POSITIONS[2] - BUTTON_CENTER_POSITIONS[0],
                                        ),
                                        height: Val::Px(GRID_LINE_WIDTH),
                                        top: Val::Px(y_pos - GRID_LINE_WIDTH / 2.0),
                                        left: Val::Px(BUTTON_CENTER_POSITIONS[0]),
                                        ..default()
                                    },
                                    BackgroundColor(theme.theme().button_regular_outline()),
                                ));
                            }
                            // Vertical grid lines - aligned with button centers
                            for &x_pos in BUTTON_CENTER_POSITIONS.iter() {
                                lines.spawn((
                                    Node {
                                        position_type: PositionType::Absolute,
                                        width: Val::Px(GRID_LINE_WIDTH),
                                        height: Val::Px(
                                            BUTTON_CENTER_POSITIONS[2] - BUTTON_CENTER_POSITIONS[0],
                                        ),
                                        left: Val::Px(x_pos - GRID_LINE_WIDTH / 2.0),
                                        top: Val::Px(BUTTON_CENTER_POSITIONS[0]),
                                        ..default()
                                    },
                                    BackgroundColor(theme.theme().button_regular_outline()),
                                ));
                            }
                        });

                    // Quadrant buttons using absolute positioning to align with grid lines
                    let quadrants_with_positions = [
                        // Row 0 (Top)
                        (Quadrant::TopLeft, 0, 0),
                        (Quadrant::Top, 1, 0),
                        (Quadrant::TopRight, 2, 0),
                        // Row 1 (Middle)
                        (Quadrant::Left, 0, 1),
                        (Quadrant::Center, 1, 1),
                        (Quadrant::Right, 2, 1),
                        // Row 2 (Bottom)
                        (Quadrant::BottomLeft, 0, 2),
                        (Quadrant::Bottom, 1, 2),
                        (Quadrant::BottomRight, 2, 2),
                    ];

                    for (quadrant, col, row) in quadrants_with_positions.iter() {
                        let x_center = BUTTON_CENTER_POSITIONS[*col];
                        let y_center = BUTTON_CENTER_POSITIONS[*row];
                        let is_selected = *quadrant == Quadrant::Center;

                        container.spawn((
                            Button,
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Px(QUADRANT_BUTTON_SIZE),
                                height: Val::Px(QUADRANT_BUTTON_SIZE),
                                // Center the button on the grid intersection
                                left: Val::Px(x_center - QUADRANT_BUTTON_SIZE / 2.0),
                                top: Val::Px(y_center - QUADRANT_BUTTON_SIZE / 2.0),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(if is_selected {
                                theme.theme().button_pressed()
                            } else {
                                theme.theme().button_regular()
                            }),
                            BorderColor(if is_selected {
                                theme.theme().button_pressed_outline()
                            } else {
                                theme.theme().button_regular_outline()
                            }),
                            BorderRadius::all(Val::Px(theme.theme().ui_border_radius())),
                            UiBorderRadius,
                            QuadrantButton(*quadrant),
                        ));
                    }
                });
        });
}

// ============================================================================
// SYSTEMS
// ============================================================================

/// Updates coordinate selection based on selected entities
fn update_coordinate_selection(
    mut coordinate_selection: ResMut<CoordinateSelection>,
    selected_query: Query<
        (
            &GlobalTransform,
            Option<&crate::editing::sort::manager::SortPointEntity>,
        ),
        With<Selected>,
    >,
    sort_transforms: Query<&GlobalTransform, With<crate::editing::sort::Sort>>,
) {
    let selected_count = selected_query.iter().count();
    coordinate_selection.count = selected_count;

    if selected_count == 0 {
        coordinate_selection.frame = Rect::default();
        return;
    }

    // Find the baseline (sort position) for coordinate calculation
    let mut sort_baseline = Vec2::ZERO;
    for (_, sort_point) in selected_query.iter() {
        if let Some(sort_point) = sort_point {
            if let Ok(sort_transform) = sort_transforms.get(sort_point.sort_entity) {
                sort_baseline = sort_transform.translation().truncate();
                break;
            }
        }
    }

    // Calculate bounding box relative to baseline
    let mut min = Vec2::new(f32::INFINITY, f32::INFINITY);
    let mut max = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

    for (transform, _) in selected_query.iter() {
        let world_pos = transform.translation().truncate();
        let relative_pos = world_pos - sort_baseline;
        min = min.min(relative_pos);
        max = max.max(relative_pos);
    }

    coordinate_selection.frame = Rect::from_corners(min, max);
}

/// Updates the displayed coordinate values
#[allow(clippy::type_complexity)]
fn update_coordinate_display(
    coordinate_selection: Res<CoordinateSelection>,
    mut queries: ParamSet<(
        Query<&mut Text, With<XValue>>,
        Query<&mut Text, With<YValue>>,
        Query<&mut Text, With<WidthValue>>,
        Query<&mut Text, With<HeightValue>>,
    )>,
) {
    if coordinate_selection.count == 0 {
        return;
    }

    let reference_point =
        get_quadrant_point(&coordinate_selection.frame, coordinate_selection.quadrant);

    // Update X
    if let Ok(mut text) = queries.p0().single_mut() {
        *text = Text::new(format!("{}", reference_point.x as i32));
    }

    // Update Y
    if let Ok(mut text) = queries.p1().single_mut() {
        *text = Text::new(format!("{}", reference_point.y as i32));
    }

    // Update Width
    if let Ok(mut text) = queries.p2().single_mut() {
        *text = Text::new(format!("{}", coordinate_selection.frame.width() as i32));
    }

    // Update Height
    if let Ok(mut text) = queries.p3().single_mut() {
        *text = Text::new(format!("{}", coordinate_selection.frame.height() as i32));
    }
}

/// Handles quadrant button interactions
fn handle_quadrant_buttons(
    mut interaction_query: Query<(&Interaction, &QuadrantButton), Changed<Interaction>>,
    mut coordinate_selection: ResMut<CoordinateSelection>,
    mut all_buttons: Query<(&QuadrantButton, &mut BackgroundColor, &mut BorderColor)>,
    theme: Res<CurrentTheme>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            coordinate_selection.quadrant = button.0;

            // Update all button states
            for (other_button, mut bg, mut border) in all_buttons.iter_mut() {
                let is_selected = other_button.0 == button.0;
                *bg = BackgroundColor(if is_selected {
                    theme.theme().button_pressed()
                } else {
                    theme.theme().button_regular()
                });
                *border = BorderColor(if is_selected {
                    theme.theme().button_pressed_outline()
                } else {
                    theme.theme().button_regular_outline()
                });
            }
        }
    }
}

/// Shows/hides the coordinate pane based on selection
fn toggle_pane_visibility(
    coordinate_selection: Res<CoordinateSelection>,
    mut coordinate_pane: Query<&mut Visibility, With<CoordinatePane>>,
) {
    let should_show = coordinate_selection.count > 0;

    for mut visibility in coordinate_pane.iter_mut() {
        *visibility = if should_show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Gets the reference point for a quadrant
fn get_quadrant_point(frame: &Rect, quadrant: Quadrant) -> Vec2 {
    let min = frame.min;
    let max = frame.max;
    let center = frame.center();

    match quadrant {
        Quadrant::TopLeft => Vec2::new(min.x, max.y),
        Quadrant::Top => Vec2::new(center.x, max.y),
        Quadrant::TopRight => Vec2::new(max.x, max.y),
        Quadrant::Left => Vec2::new(min.x, center.y),
        Quadrant::Center => center,
        Quadrant::Right => Vec2::new(max.x, center.y),
        Quadrant::BottomLeft => Vec2::new(min.x, min.y),
        Quadrant::Bottom => Vec2::new(center.x, min.y),
        Quadrant::BottomRight => Vec2::new(max.x, min.y),
    }
}
