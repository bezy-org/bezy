//! UI pane to display information about the current glyph
//!
//! Shows glyph name, Unicode codepoint, advance width, side bearings,
//! and side bearings in the lower left corner of the window.

use crate::embedded_assets::{AssetServerFontExt, EmbeddedFonts};
use crate::core::state::fontir_app_state::FontIRAppState;
use crate::core::state::AppState;
use crate::ui::theme::*;
use crate::ui::themes::CurrentTheme;
use bevy::prelude::*;
use kurbo::{BezPath, PathEl};

/// Resource to store current glyph metrics for display
#[derive(Resource, Default)]
pub struct CurrentGlyphMetrics {
    pub glyph_name: String,
    pub unicode: String,
    pub advance: String,
    pub left_bearing: String,
    pub right_bearing: String,
    pub left_group: String,
    pub right_group: String,
}

/// Component marker for the glyph pane
#[derive(Component, Default)]
pub struct GlyphPane;

// Remove the GlyphPaneContent component since we'll hide the entire pane

/// Component marker for the glyph name text
#[derive(Component)]
pub struct GlyphNameText;

/// Component marker for the glyph unicode text
#[derive(Component)]
pub struct GlyphUnicodeText;

/// Component marker for the glyph advance text
#[derive(Component)]
pub struct GlyphAdvanceText;

/// Component marker for the glyph left bearing text
#[derive(Component)]
pub struct GlyphLeftBearingText;

/// Component marker for the glyph right bearing text
#[derive(Component)]
pub struct GlyphRightBearingText;

/// Component marker for the glyph left kerning group text
#[derive(Component)]
pub struct GlyphLeftGroupText;

/// Component marker for the glyph right kerning group text
#[derive(Component)]
pub struct GlyphRightGroupText;

/// Plugin that adds the glyph pane functionality
pub struct GlyphPanePlugin;

impl Plugin for GlyphPanePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentGlyphMetrics>()
            .add_systems(Startup, setup_glyph_pane)
            .add_systems(
                Update,
                (
                    update_glyph_pane,
                    update_glyph_metrics,
                    toggle_glyph_pane_visibility,
                ),
            );
    }
}

/// System to set up the glyph pane during startup
fn setup_glyph_pane(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    embedded_fonts: Res<EmbeddedFonts>,
    theme: Res<CurrentTheme>,
) {
    spawn_glyph_pane(&mut commands, &asset_server, &embedded_fonts, &theme);
}

/// System to update the glyph pane display
///
/// Currently displays values from the CurrentGlyphMetrics resource.
fn update_glyph_pane(world: &mut World) {
    // Get the current metrics resource
    let metrics = world.resource::<CurrentGlyphMetrics>();

    // Debug log what's in the resource
    static mut LOG_COUNT: u32 = 0;
    unsafe {
        LOG_COUNT += 1;
        if LOG_COUNT % 60 == 0 {
            // Log every second at 60fps
            info!("update_glyph_pane: Resource contains - glyph: '{}', advance: '{}', lsb: '{}', rsb: '{}'", 
                  metrics.glyph_name, metrics.advance, metrics.left_bearing, metrics.right_bearing);
        }
    }

    // Format the values for display
    let glyph_name = if metrics.glyph_name.is_empty() {
        "None".to_string()
    } else {
        metrics.glyph_name.clone()
    };

    let unicode = if metrics.unicode.is_empty() {
        "None".to_string()
    } else {
        metrics.unicode.to_uppercase()
    };

    let advance = if metrics.advance.is_empty() {
        "--".to_string()
    } else {
        metrics.advance.clone()
    };

    let lsb = if metrics.left_bearing.is_empty() {
        "--".to_string()
    } else {
        metrics.left_bearing.clone()
    };

    let rsb = if metrics.right_bearing.is_empty() {
        "--".to_string()
    } else {
        metrics.right_bearing.clone()
    };

    let left_group = if metrics.left_group.is_empty() {
        "None".to_string()
    } else {
        metrics.left_group.clone()
    };

    let right_group = if metrics.right_group.is_empty() {
        "None".to_string()
    } else {
        metrics.right_group.clone()
    };

    // Update the texts in the UI
    let mut name_query = world.query_filtered::<&mut Text, With<GlyphNameText>>();
    let name_count = name_query.iter_mut(world).count();
    info!(
        "update_glyph_pane: Found {} GlyphNameText entities to update",
        name_count
    );
    for mut text in name_query.iter_mut(world) {
        info!(
            "update_glyph_pane: Setting glyph name text to '{}'",
            glyph_name
        );
        *text = Text::new(glyph_name.clone());
    }

    let mut unicode_query = world.query_filtered::<&mut Text, With<GlyphUnicodeText>>();
    for mut text in unicode_query.iter_mut(world) {
        *text = Text::new(unicode.clone());
    }

    let mut advance_query = world.query_filtered::<&mut Text, With<GlyphAdvanceText>>();
    let advance_count = advance_query.iter_mut(world).count();
    info!(
        "update_glyph_pane: Found {} GlyphAdvanceText entities to update",
        advance_count
    );
    for mut text in advance_query.iter_mut(world) {
        info!("update_glyph_pane: Setting advance text to '{}'", advance);
        *text = Text::new(advance.clone());
    }

    let mut lsb_query = world.query_filtered::<&mut Text, With<GlyphLeftBearingText>>();
    for mut text in lsb_query.iter_mut(world) {
        *text = Text::new(lsb.clone());
    }

    let mut rsb_query = world.query_filtered::<&mut Text, With<GlyphRightBearingText>>();
    for mut text in rsb_query.iter_mut(world) {
        *text = Text::new(rsb.clone());
    }

    let mut left_group_query = world.query_filtered::<&mut Text, With<GlyphLeftGroupText>>();
    for mut text in left_group_query.iter_mut(world) {
        *text = Text::new(left_group.clone());
    }

    let mut right_group_query = world.query_filtered::<&mut Text, With<GlyphRightGroupText>>();
    for mut text in right_group_query.iter_mut(world) {
        *text = Text::new(right_group.clone());
    }
}

/// System to toggle the visibility of the entire glyph pane based on active sort
fn toggle_glyph_pane_visibility(
    text_editor_state: Res<crate::core::state::text_editor::TextEditorState>,
    mut glyph_pane_query: Query<&mut Visibility, With<GlyphPane>>,
) {
    let visibility = if text_editor_state.get_active_sort().is_some() {
        // If there is an active sort, show the pane
        Visibility::Visible
    } else {
        // If no active sort, hide the entire pane
        Visibility::Hidden
    };

    for mut vis in glyph_pane_query.iter_mut() {
        *vis = visibility;
    }
}

/// Spawns the glyph pane in the lower left corner
pub fn spawn_glyph_pane(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    embedded_fonts: &Res<EmbeddedFonts>,
    theme: &Res<CurrentTheme>,
) {
    // Create the position properties for the glyph pane (bottom left)
    let position_props = UiRect {
        left: Val::Px(theme.theme().widget_margin()),
        bottom: Val::Px(theme.theme().widget_margin()),
        top: Val::Auto,   // Explicitly set top to Auto to prevent stretching
        right: Val::Auto, // Explicitly set right to Auto for correct sizing
    };

    commands
        .spawn(create_widget_style(
            asset_server,
            theme,
            PositionType::Absolute,
            position_props,
            GlyphPane,
            "GlyphPane",
        ))
        .with_children(|parent| {
            // Glyph name row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    width: Val::Auto,
                    height: Val::Auto,
                    margin: UiRect::bottom(Val::Px(WIDGET_ROW_LEADING)),
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(4.0)),
                            width: Val::Auto,
                            ..default()
                        },
                        Text::new("Glyph:"),
                        TextFont {
                            font: asset_server.load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));

                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: asset_server.load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.5, 1.0)),
                        GlyphNameText,
                    ));
                });

            // Unicode value row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    width: Val::Auto,
                    height: Val::Auto,
                    margin: UiRect::bottom(Val::Px(WIDGET_ROW_LEADING)),
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(4.0)),
                            width: Val::Auto,
                            ..default()
                        },
                        Text::new("Unicode:"),
                        TextFont {
                            font: asset_server.load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));

                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: asset_server.load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.5, 1.0)),
                        GlyphUnicodeText,
                    ));
                });

            // Advance width row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(WIDGET_ROW_LEADING)),
                    width: Val::Auto,
                    height: Val::Auto,
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(4.0)),
                            width: Val::Auto,
                            ..default()
                        },
                        Text::new("Advance:"),
                        TextFont {
                            font: asset_server.load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));

                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: asset_server.load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.5, 1.0)),
                        GlyphAdvanceText,
                    ));
                });

            // Left side bearing row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(WIDGET_ROW_LEADING)),
                    width: Val::Auto,
                    height: Val::Auto,
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(4.0)),
                            width: Val::Auto,
                            ..default()
                        },
                        Text::new("LSB:"),
                        TextFont {
                            font: asset_server.load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));

                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: asset_server.load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.5, 1.0)),
                        GlyphLeftBearingText,
                    ));
                });

            // Right side bearing row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(WIDGET_ROW_LEADING)),
                    width: Val::Auto,
                    height: Val::Auto,
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(4.0)),
                            width: Val::Auto,
                            ..default()
                        },
                        Text::new("RSB:"),
                        TextFont {
                            font: asset_server.load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));

                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: asset_server.load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.5, 1.0)),
                        GlyphRightBearingText,
                    ));
                });

            // Left kerning group row
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(WIDGET_ROW_LEADING)),
                    width: Val::Auto,
                    height: Val::Auto,
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(4.0)),
                            width: Val::Auto,
                            ..default()
                        },
                        Text::new("Left Group:"),
                        TextFont {
                            font: asset_server.load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));

                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: asset_server.load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.5, 1.0)),
                        GlyphLeftGroupText,
                    ));
                });

            // Right kerning group row (no bottom margin on last row)
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    width: Val::Auto,
                    height: Val::Auto,
                    ..default()
                },))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(4.0)),
                            width: Val::Auto,
                            ..default()
                        },
                        Text::new("Right Group:"),
                        TextFont {
                            font: asset_server.load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    ));

                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: asset_server.load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(Color::srgba(0.0, 1.0, 0.5, 1.0)),
                        GlyphRightGroupText,
                    ));
                });
        });
}

/// Updates the glyph metrics for the current glyph
pub fn update_glyph_metrics(
    app_state: Option<Res<AppState>>,
    fontir_app_state: Option<Res<FontIRAppState>>,
    text_editor_state: Res<crate::core::state::text_editor::TextEditorState>,
    mut metrics: ResMut<CurrentGlyphMetrics>,
) {
    // Debug: Always log when this system runs
    let active_sort = text_editor_state.get_active_sort();
    info!(
        "Glyph pane: update_glyph_metrics running, active_sort = {:?}",
        active_sort
            .as_ref()
            .map(|(idx, sort)| (idx, sort.kind.glyph_name()))
    );

    // TEMPORARY DEBUG: Force metrics extraction for 'a' to test FontIR
    let debug_test = true;

    // Get information from the active sort instead of glyph navigation
    if let Some((_buffer_index, sort_entry)) = active_sort {
        let glyph_name = sort_entry.kind.glyph_name().to_string();

        // Found an active sort, get its details
        metrics.glyph_name = glyph_name.clone();

        // Try FontIR first, then fall back to AppState
        if let Some(fontir_state) = fontir_app_state.as_ref() {
            // Extract metrics from FontIR

            // TODO: Get Unicode from FontIR glyph data when available
            // For now, try to get it from glyph name
            metrics.unicode = if glyph_name == "a" {
                "0061".to_string() // Unicode for 'a'
            } else if glyph_name.len() == 1
                && glyph_name.chars().next().unwrap().is_ascii_lowercase()
            {
                format!("{:04X}", glyph_name.chars().next().unwrap() as u32)
            } else {
                String::new()
            };

            // Get advance width from FontIR
            let advance_width = fontir_state.get_glyph_advance_width(&glyph_name);
            info!(
                "Glyph pane: advance_width for '{}' = {}",
                glyph_name, advance_width
            );
            metrics.advance = format!("{}", advance_width as i32);

            // Calculate sidebearings using FontIR paths
            if let Some(paths) = fontir_state.get_glyph_paths(&glyph_name) {
                info!(
                    "Glyph pane: Found {} paths for glyph '{}'",
                    paths.len(),
                    glyph_name
                );
                let outline_bounds = calculate_fontir_bounds(&paths);

                if let Some((min_x, max_x)) = outline_bounds {
                    // Left side bearing is the distance from origin to the leftmost point
                    let lsb = min_x;

                    // Right side bearing is the distance from the rightmost point to the advance width
                    let rsb = advance_width - max_x;

                    info!("Glyph pane: bounds for '{}': min_x={}, max_x={}, advance={}, lsb={}, rsb={}", 
                          glyph_name, min_x, max_x, advance_width, lsb, rsb);

                    metrics.left_bearing = format!("{}", lsb as i32);
                    metrics.right_bearing = format!("{}", rsb as i32);
                } else {
                    info!(
                        "Glyph pane: Could not calculate bounds for '{}'",
                        glyph_name
                    );
                    // If we couldn't calculate bounds, use placeholder values
                    metrics.left_bearing = "0".to_string();
                    metrics.right_bearing = "0".to_string();
                }
            } else {
                info!("Glyph pane: No paths found for glyph '{}'", glyph_name);
                // No paths found
                metrics.left_bearing = "0".to_string();
                metrics.right_bearing = "0".to_string();
            }

            // Get kerning groups from FontIR if available
            if let Some(fontir_state) = fontir_app_state.as_ref() {
                let (left_group, right_group) = fontir_state.get_glyph_kerning_groups(&glyph_name);
                metrics.left_group = left_group.unwrap_or_else(String::new);
                metrics.right_group = right_group.unwrap_or_else(String::new);
            } else {
                metrics.left_group = String::new();
                metrics.right_group = String::new();
            }
        } else if let Some(state) = app_state.as_ref() {
            // Fallback to AppState (UFO data)
            info!(
                "Glyph pane: FALLBACK BRANCH - Using AppState for glyph '{}'",
                glyph_name
            );
            if let Some(glyph_data) = state.workspace.font.get_glyph(&glyph_name) {
                // Find the first Unicode codepoint for this glyph
                if let Some(first_codepoint) = glyph_data.unicode_values.first() {
                    metrics.unicode = format!("{:04X}", *first_codepoint as u32);
                } else {
                    metrics.unicode = String::new();
                }

                // Get advance width
                metrics.advance = format!("{}", glyph_data.advance_width as i32);

                // Calculate sidebearings based on glyph outline bounds
                let outline_bounds = if let Some(outline) = &glyph_data.outline {
                    if !outline.contours.is_empty() {
                        // Calculate bounds from outline contours
                        let mut min_x = f64::MAX;
                        let mut max_x = f64::MIN;

                        // Iterate through all contours and their points
                        let mut has_points = false;
                        for contour in &outline.contours {
                            for point in &contour.points {
                                has_points = true;
                                if point.x < min_x {
                                    min_x = point.x;
                                }
                                if point.x > max_x {
                                    max_x = point.x;
                                }
                            }
                        }

                        // If we found valid bounds
                        if has_points && min_x != f64::MAX && max_x != f64::MIN {
                            Some((min_x, max_x))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Calculate LSB and RSB if we have both outline bounds and advance width
                if let Some((min_x, max_x)) = outline_bounds {
                    // Left side bearing is the distance from origin to the leftmost point
                    let lsb = min_x;

                    // Right side bearing is the distance from the rightmost point to the advance width
                    let rsb = glyph_data.advance_width - max_x;

                    metrics.left_bearing = format!("{}", lsb as i32);
                    metrics.right_bearing = format!("{}", rsb as i32);
                } else {
                    // If we couldn't calculate bounds, use placeholder values
                    metrics.left_bearing = "0".to_string();
                    metrics.right_bearing = "0".to_string();
                }

                // Get kerning groups from FontIR
                if let Some(fontir_state) = fontir_app_state.as_ref() {
                    info!(
                        "Glyph pane: Using FontIR to get groups for '{}'",
                        glyph_name
                    );
                    let (left_group, right_group) =
                        fontir_state.get_glyph_kerning_groups(&glyph_name);
                    metrics.left_group = left_group.unwrap_or_else(String::new);
                    metrics.right_group = right_group.unwrap_or_else(String::new);
                    info!(
                        "Glyph pane: Set groups - left: '{}', right: '{}'",
                        metrics.left_group, metrics.right_group
                    );
                } else {
                    info!("Glyph pane: No FontIR state available for groups");
                    metrics.left_group = String::new();
                    metrics.right_group = String::new();
                }
            } else {
                // No glyph data found in AppState
                metrics.advance = "-".to_string();
                metrics.left_bearing = "-".to_string();
                metrics.right_bearing = "-".to_string();
                metrics.left_group = String::new();
                metrics.right_group = String::new();
            }
        } else {
            // Neither FontIR nor AppState available - show placeholders
            info!("Glyph pane: NO STATE BRANCH - Neither FontIR nor AppState available for glyph '{}'", glyph_name);
            metrics.unicode = String::new();
            metrics.advance = "-".to_string();
            metrics.left_bearing = "-".to_string();
            metrics.right_bearing = "-".to_string();
            metrics.left_group = String::new();
            metrics.right_group = String::new();
        }
    } else if debug_test {
        // DEBUG TEST: Force extract metrics for 'a' to verify FontIR is working
        info!("DEBUG TEST: Forcing metrics extraction for glyph 'a'");
        let glyph_name = "a".to_string();
        metrics.glyph_name = glyph_name.clone();

        // Try FontIR first
        if let Some(fontir_state) = fontir_app_state.as_ref() {
            info!("DEBUG TEST: Using FontIR for glyph 'a'");

            // Get Unicode
            metrics.unicode = "0061".to_string();

            // Get advance width from FontIR
            let advance_width = fontir_state.get_glyph_advance_width(&glyph_name);
            info!(
                "DEBUG TEST: advance_width for '{}' = {}",
                glyph_name, advance_width
            );
            metrics.advance = format!("{}", advance_width as i32);

            // Calculate sidebearings using FontIR paths
            if let Some(paths) = fontir_state.get_glyph_paths(&glyph_name) {
                info!(
                    "DEBUG TEST: Found {} paths for glyph '{}'",
                    paths.len(),
                    glyph_name
                );
                let outline_bounds = calculate_fontir_bounds(&paths);

                if let Some((min_x, max_x)) = outline_bounds {
                    let lsb = min_x;
                    let rsb = advance_width - max_x;

                    info!("DEBUG TEST: bounds for '{}': min_x={}, max_x={}, advance={}, lsb={}, rsb={}", 
                          glyph_name, min_x, max_x, advance_width, lsb, rsb);

                    metrics.left_bearing = format!("{}", lsb as i32);
                    metrics.right_bearing = format!("{}", rsb as i32);
                } else {
                    info!("DEBUG TEST: Could not calculate bounds");
                    metrics.left_bearing = "0".to_string();
                    metrics.right_bearing = "0".to_string();
                }
            } else {
                info!("DEBUG TEST: No paths found");
                metrics.left_bearing = "0".to_string();
                metrics.right_bearing = "0".to_string();
            }

            metrics.left_group = String::new();
            metrics.right_group = String::new();
        } else {
            info!("DEBUG TEST: FontIR not available");
            metrics.unicode = String::new();
            metrics.advance = "-".to_string();
            metrics.left_bearing = "-".to_string();
            metrics.right_bearing = "-".to_string();
            metrics.left_group = String::new();
            metrics.right_group = String::new();
        }
    } else {
        // No active sort found, clear the metrics
        metrics.glyph_name = String::new();
        metrics.unicode = String::new();
        metrics.advance = "-".to_string();
        metrics.left_bearing = "-".to_string();
        metrics.right_bearing = "-".to_string();
        metrics.left_group = String::new();
        metrics.right_group = String::new();
    }
}

/// Calculate the bounding box of FontIR BezPaths
fn calculate_fontir_bounds(paths: &[BezPath]) -> Option<(f32, f32)> {
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut has_points = false;

    for path in paths {
        for element in path.elements() {
            match element {
                PathEl::MoveTo(pt) => {
                    has_points = true;
                    min_x = min_x.min(pt.x);
                    max_x = max_x.max(pt.x);
                }
                PathEl::LineTo(pt) => {
                    has_points = true;
                    min_x = min_x.min(pt.x);
                    max_x = max_x.max(pt.x);
                }
                PathEl::QuadTo(c1, pt) => {
                    has_points = true;
                    min_x = min_x.min(c1.x).min(pt.x);
                    max_x = max_x.max(c1.x).max(pt.x);
                }
                PathEl::CurveTo(c1, c2, pt) => {
                    has_points = true;
                    min_x = min_x.min(c1.x).min(c2.x).min(pt.x);
                    max_x = max_x.max(c1.x).max(c2.x).max(pt.x);
                }
                PathEl::ClosePath => {
                    // No points to process
                }
            }
        }
    }

    if has_points && min_x != f64::MAX && max_x != f64::MIN {
        Some((min_x as f32, max_x as f32))
    } else {
        None
    }
}
