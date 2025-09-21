//! File Pane Module
//!
//! This module implements a floating panel in the upper left corner that displays
//! information about the currently loaded font files and allows switching between
//! UFO masters in a designspace.

use crate::core::state::fontir_app_state::FontIRAppState;
use crate::embedded_assets::{AssetServerFontExt, EmbeddedFonts};
use crate::systems::sorts::sort_entities::BufferSortEntities;
use crate::ui::theme::*;
use crate::ui::themes::{CurrentTheme, UiBorderRadius};
use bevy::prelude::*;
use bevy::ui::Display;
use bevy::window::{PrimaryWindow, Window, WindowMode};
use chrono::{DateTime, Local};
use norad::designspace::DesignSpaceDocument;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// DESIGN CONSTANTS
// ============================================================================

/// Size of each master button (same as coordinate pane quadrant buttons)
const MASTER_BUTTON_SIZE: f32 = 24.0;

/// Gap between master buttons
const MASTER_BUTTON_GAP: f32 = 8.0;

/// Spacing between label and value
const LABEL_VALUE_SPACING: f32 = 8.0;

/// Spacing between file info rows
const ROW_SPACING: f32 = WIDGET_ROW_LEADING;

/// Extra spacing after master selector (between circles and text)
const MASTER_SELECTOR_MARGIN: f32 = 16.0;

/// File pane internal padding
const FILE_PANE_PADDING: f32 = 16.0;

/// File pane border width
const FILE_PANE_BORDER: f32 = 2.0;

// ============================================================================
// COMPONENTS & RESOURCES
// ============================================================================

/// Resource that tracks file information and save state
#[derive(Resource, Default)]
pub struct FileInfo {
    pub designspace_path: String,
    pub current_ufo: String,
    pub last_saved: Option<SystemTime>,
    pub last_exported: Option<SystemTime>,
    pub masters: Vec<UFOMaster>,
    pub current_master_index: usize,
}

/// Information about a UFO master (a single UFO file on disk)
#[derive(Clone, Debug)]
pub struct UFOMaster {
    pub name: String,
    pub style_name: String,
    pub filename: String,
    pub file_path: PathBuf,
}

/// Component marker for the file pane
#[derive(Component, Default)]
pub struct FilePane;

/// Component markers for file info text elements
#[derive(Component, Default)]
pub struct DesignspacePathText;

#[derive(Component, Default)]
pub struct CurrentUFOText;

#[derive(Component, Default)]
pub struct LastSavedText;

#[derive(Component, Default)]
pub struct LastExportedText;

/// Component marker for the saved row container
#[derive(Component, Default)]
pub struct SavedRowContainer;

/// Component marker for the exported row container
#[derive(Component, Default)]
pub struct ExportedRowContainer;

/// Component marker for the designspace row container
#[derive(Component, Default)]
pub struct DesignspaceRowContainer;

/// Component for master selection buttons
#[derive(Component)]
pub struct MasterButton {
    pub master_index: usize,
}

/// Component marker for the master button container
#[derive(Component)]
pub struct MasterButtonContainer;

/// Event for switching UFO masters
#[derive(Event)]
pub struct SwitchMasterEvent {
    pub master_index: usize,
    pub master_path: PathBuf,
}

// ============================================================================
// PLUGIN
// ============================================================================

pub struct FilePanePlugin;

impl Plugin for FilePanePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FileInfo>()
            .add_event::<SwitchMasterEvent>()
            .add_systems(Startup, spawn_file_pane)
            .add_systems(
                Update,
                (
                    update_file_info,
                    update_file_display,
                    update_master_buttons,
                    handle_master_buttons,
                    handle_switch_master_events,
                    toggle_file_pane_visibility,
                ),
            );
    }
}

// ============================================================================
// UI CREATION
// ============================================================================

/// Spawns the file pane in the upper-left corner
pub fn spawn_file_pane(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    _embedded_fonts: Res<EmbeddedFonts>,
    theme: Res<CurrentTheme>,
) {
    // Position to visually align with toolbar content, accounting for our border and padding
    // Toolbar content is at: TOOLBAR_CONTAINER_MARGIN = 16px from edge
    // Our content will be at: position + border + padding = position + 2px + 16px
    // To match toolbar: position + 2px + 16px = 16px, so position = -2px
    // But we want some visible margin, so let's use a small positive offset
    let position = UiRect {
        right: Val::Px(TOOLBAR_CONTAINER_MARGIN + 4.0), // Slight adjustment to better align
        top: Val::Px(TOOLBAR_CONTAINER_MARGIN + 4.0),
        left: Val::Auto,
        bottom: Val::Auto,
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: position.left,
                right: position.right,
                top: position.top,
                bottom: position.bottom,
                padding: UiRect::all(Val::Px(FILE_PANE_PADDING)),
                margin: UiRect::all(Val::Px(0.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(WIDGET_ROW_LEADING),
                border: UiRect::all(Val::Px(FILE_PANE_BORDER)),
                width: Val::Auto, // Auto-size to content
                height: Val::Auto,
                min_width: Val::Auto,
                min_height: Val::Auto,
                max_width: Val::Auto, // No max width constraint
                max_height: Val::Percent(50.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            BackgroundColor(theme.theme().widget_background_color()),
            BorderColor(theme.theme().widget_border_color()),
            BorderRadius::all(Val::Px(theme.theme().widget_border_radius())),
            crate::ui::themes::WidgetBorderRadius,
            FilePane,
            Name::new("FilePane"),
        ))
        .with_children(|parent| {
            // ============ UFO MASTER SELECTOR ============
            parent
                .spawn(Node {
                    position_type: PositionType::Relative,
                    width: Val::Auto,  // Auto-size to content
                    height: Val::Auto, // Auto-size to content
                    margin: UiRect::bottom(Val::Px(MASTER_SELECTOR_MARGIN)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(MASTER_BUTTON_GAP),
                    ..default()
                })
                .insert(MasterButtonContainer)
                .with_children(|_container| {
                    // Master buttons will be created dynamically by update_master_buttons system
                });

            // ============ FILE INFO ROWS ============

            // Designspace path row (first text row, below master selector)
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(ROW_SPACING)),
                        display: Display::Flex, // Will be hidden for single UFOs
                        ..default()
                    },
                    DesignspaceRowContainer,
                ))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(LABEL_VALUE_SPACING)),
                            ..default()
                        },
                        Text::new("DS:"),
                        TextFont {
                            font: _asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &_embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_primary()),
                    ));
                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: _asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &_embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_secondary()),
                        DesignspacePathText,
                    ));
                });

            // Current UFO row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(ROW_SPACING)),
                    ..default()
                })
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(LABEL_VALUE_SPACING)),
                            ..default()
                        },
                        Text::new("UFO:"),
                        TextFont {
                            font: _asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &_embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_primary()),
                    ));
                    // Value
                    row.spawn((
                        Text::new("Loading..."),
                        TextFont {
                            font: _asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &_embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_secondary()),
                        CurrentUFOText,
                    ));
                });

            // Last saved row (initially hidden)
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(ROW_SPACING)),
                        display: Display::None, // Initially hidden
                        ..default()
                    },
                    SavedRowContainer,
                ))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(LABEL_VALUE_SPACING)),
                            ..default()
                        },
                        Text::new("Saved:"),
                        TextFont {
                            font: _asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &_embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_primary()),
                    ));
                    // Value
                    row.spawn((
                        Text::new(""),
                        TextFont {
                            font: _asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &_embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_secondary()),
                        LastSavedText,
                    ));
                });

            // Last exported row (initially hidden)
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        display: Display::None, // Initially hidden
                        ..default()
                    },
                    ExportedRowContainer,
                ))
                .with_children(|row| {
                    // Label
                    row.spawn((
                        Node {
                            margin: UiRect::right(Val::Px(LABEL_VALUE_SPACING)),
                            ..default()
                        },
                        Text::new("Exported:"),
                        TextFont {
                            font: _asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &_embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_primary()),
                    ));
                    // Value
                    row.spawn((
                        Text::new(""),
                        TextFont {
                            font: _asset_server.load_font_with_fallback(
                                theme.theme().mono_font_path(),
                                &_embedded_fonts,
                            ),
                            font_size: WIDGET_TEXT_FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme.get_ui_text_secondary()),
                        LastExportedText,
                    ));
                });
        });
}

// ============================================================================
// SYSTEMS
// ============================================================================

/// Updates file information from FontIR state
fn update_file_info(
    fontir_state: Option<Res<FontIRAppState>>,
    mut file_info: ResMut<FileInfo>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    if let Some(state) = fontir_state {
        // Check if we should show full path (fullscreen or large window)
        let show_full_path = windows
            .single()
            .ok()
            .map(|window| {
                let is_fullscreen = window.mode != bevy::window::WindowMode::Windowed;
                let is_wide = window.width() > 1200.0;

                // Debug logging
                if file_info.is_changed() {
                    debug!(
                        "Window mode: {:?}, Width: {}, Show full path: {}",
                        window.mode,
                        window.width(),
                        is_fullscreen || is_wide
                    );
                }

                // Show full path if:
                // 1. Window is in fullscreen mode, OR
                // 2. Window width is greater than 1200 pixels (large enough to show full path)
                is_fullscreen || is_wide
            })
            .unwrap_or(false);

        // Update designspace path (only for actual designspaces, not single UFOs)
        if !state.is_single_ufo() {
            if let Some(path_str) = state.source_path.to_str() {
                if show_full_path {
                    // In fullscreen: show full path with ~/ notation
                    let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
                    let full_path = if path_str.starts_with(&home_dir) {
                        format!("~{}", &path_str[home_dir.len()..])
                    } else {
                        path_str.to_string()
                    };
                    file_info.designspace_path = full_path;
                } else {
                    // In windowed mode: show just the filename
                    let filename = state
                        .source_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown");
                    file_info.designspace_path = filename.to_string();
                }
            }
        }

        // Load UFO masters from designspace file
        if file_info.masters.is_empty() {
            if let Ok(masters) = load_masters_from_designspace(&state.source_path) {
                file_info.masters = masters;
            } else {
                // Fallback for single UFO files or when designspace loading fails
                file_info.masters = vec![UFOMaster {
                    name: "Regular".to_string(),
                    style_name: "Regular".to_string(),
                    filename: state
                        .source_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    file_path: state.source_path.clone(),
                }];
            }
        }

        // Update current UFO
        if state.is_single_ufo() {
            // For single UFOs, show the path with same logic as designspace would have
            if let Some(path_str) = state.source_path.to_str() {
                if show_full_path {
                    // In fullscreen: show full path with ~/ notation
                    let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
                    let full_path = if path_str.starts_with(&home_dir) {
                        format!("~{}", &path_str[home_dir.len()..])
                    } else {
                        path_str.to_string()
                    };
                    file_info.current_ufo = full_path;
                } else {
                    // In windowed mode: show just the filename
                    let filename = state
                        .source_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown");
                    file_info.current_ufo = filename.to_string();
                }
            }
        } else {
            // For designspaces, show the current master's filename
            if let Some(current_master) = file_info.masters.get(file_info.current_master_index) {
                file_info.current_ufo = current_master.filename.clone();
            }
        }
    }
}

/// Load UFO masters from a designspace file
fn load_masters_from_designspace(
    source_path: &PathBuf,
) -> Result<Vec<UFOMaster>, Box<dyn std::error::Error>> {
    // Check if it's a designspace file
    if source_path.extension().and_then(|s| s.to_str()) != Some("designspace") {
        return Err("Not a designspace file".into());
    }

    // Load and parse the designspace
    let designspace = DesignSpaceDocument::load(source_path)?;
    let mut masters = Vec::new();

    let designspace_dir = source_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    for source in &designspace.sources {
        let ufo_path = designspace_dir.join(&source.filename);

        masters.push(UFOMaster {
            name: source
                .name
                .clone()
                .unwrap_or_else(|| source.filename.clone()),
            style_name: source
                .stylename
                .clone()
                .unwrap_or_else(|| "Regular".to_string()),
            filename: source.filename.clone(),
            file_path: ufo_path,
        });
    }

    Ok(masters)
}

/// Updates master buttons based on loaded masters
#[allow(clippy::too_many_arguments)]
fn update_master_buttons(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    _embedded_fonts: Res<EmbeddedFonts>,
    theme: Res<CurrentTheme>,
    file_info: Res<FileInfo>,
    fontir_state: Option<Res<FontIRAppState>>,
    mut container_query: Query<(Entity, &mut Node), With<MasterButtonContainer>>,
    existing_buttons: Query<Entity, With<MasterButton>>,
    children_query: Query<&Children>,
) {
    if !file_info.is_changed() {
        return;
    }

    // Find the master button container
    let Ok((container_entity, mut container_node)) = container_query.single_mut() else {
        return;
    };

    // Clear existing buttons
    if let Ok(children) = children_query.get(container_entity) {
        for child in children.iter() {
            if existing_buttons.contains(child) {
                commands.entity(child).despawn();
            }
        }
    }

    // Only show master buttons for designspace sources (not single UFOs)
    let should_show_masters = fontir_state
        .as_ref()
        .map(|state| !state.is_single_ufo())
        .unwrap_or(true); // Default to showing if no fontir state

    if !should_show_masters {
        // Hide the container for single UFOs
        container_node.display = Display::None;
        return;
    } else {
        // Show the container for designspaces
        container_node.display = Display::Flex;
    }

    // Create new buttons based on loaded masters
    for (i, _master) in file_info.masters.iter().enumerate() {
        let is_selected = i == file_info.current_master_index;

        let button_entity = commands
            .spawn((
                Button,
                Node {
                    width: Val::Px(MASTER_BUTTON_SIZE),
                    height: Val::Px(MASTER_BUTTON_SIZE),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
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
                MasterButton { master_index: i },
            ))
            .id();

        commands.entity(container_entity).add_child(button_entity);
    }
}

// Type aliases for complex query types
type DesignspaceTextQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<DesignspacePathText>,
        Without<CurrentUFOText>,
        Without<LastSavedText>,
        Without<LastExportedText>,
    ),
>;
type CurrentUFOTextQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<CurrentUFOText>,
        Without<DesignspacePathText>,
        Without<LastSavedText>,
        Without<LastExportedText>,
    ),
>;
type LastSavedTextQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<LastSavedText>,
        Without<DesignspacePathText>,
        Without<CurrentUFOText>,
        Without<LastExportedText>,
    ),
>;
type LastExportedTextQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<LastExportedText>,
        Without<DesignspacePathText>,
        Without<CurrentUFOText>,
        Without<LastSavedText>,
    ),
>;

/// Updates the displayed file information
fn update_file_display(
    file_info: Res<FileInfo>,
    fontir_state: Option<Res<FontIRAppState>>,
    mut designspace_query: DesignspaceTextQuery,
    mut ufo_query: CurrentUFOTextQuery,
    mut saved_query: LastSavedTextQuery,
    mut exported_query: LastExportedTextQuery,
    mut designspace_row_query: Query<
        &mut Node,
        (
            With<DesignspaceRowContainer>,
            Without<SavedRowContainer>,
            Without<ExportedRowContainer>,
        ),
    >,
    mut saved_row_query: Query<
        &mut Node,
        (
            With<SavedRowContainer>,
            Without<ExportedRowContainer>,
            Without<DesignspaceRowContainer>,
        ),
    >,
    mut exported_row_query: Query<
        &mut Node,
        (
            With<ExportedRowContainer>,
            Without<SavedRowContainer>,
            Without<DesignspaceRowContainer>,
        ),
    >,
) {
    // Check if this is a single UFO or designspace
    let is_single_ufo = fontir_state
        .as_ref()
        .map(|state| state.is_single_ufo())
        .unwrap_or(false);

    // Show/hide designspace row based on source type
    if let Ok(mut node) = designspace_row_query.single_mut() {
        node.display = if is_single_ufo {
            Display::None // Hide DS row for single UFOs
        } else {
            Display::Flex // Show DS row for designspaces
        };
    }

    // Update designspace path (only relevant for designspaces)
    if let Ok(mut text) = designspace_query.single_mut() {
        *text = Text::new(file_info.designspace_path.clone());
    }

    // Update current UFO
    if let Ok(mut text) = ufo_query.single_mut() {
        *text = Text::new(file_info.current_ufo.clone());
    }

    // Update last saved time and visibility
    if let Ok(mut text) = saved_query.single_mut() {
        if let Some(save_time) = file_info.last_saved {
            // Convert SystemTime to DateTime<Local> for human-readable formatting
            let datetime: DateTime<Local> = save_time.into();
            *text = Text::new(datetime.format("%Y-%m-%d %H:%M:%S").to_string());

            // Show the saved row
            if let Ok(mut node) = saved_row_query.single_mut() {
                node.display = Display::Flex;
            }
        }
    }

    // Update last exported time and visibility
    if let Ok(mut text) = exported_query.single_mut() {
        if let Some(export_time) = file_info.last_exported {
            // Convert SystemTime to DateTime<Local> for human-readable formatting
            let datetime: DateTime<Local> = export_time.into();
            *text = Text::new(datetime.format("%Y-%m-%d %H:%M:%S").to_string());

            // Show the exported row
            if let Ok(mut node) = exported_row_query.single_mut() {
                node.display = Display::Flex;
            }
        }
    }
}

/// Handles master button interactions
fn handle_master_buttons(
    interaction_query: Query<(&Interaction, &MasterButton), Changed<Interaction>>,
    mut file_info: ResMut<FileInfo>,
    mut all_buttons: Query<(&MasterButton, &mut BackgroundColor, &mut BorderColor)>,
    mut switch_events: EventWriter<SwitchMasterEvent>,
    theme: Res<CurrentTheme>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Only switch if we have a valid master at that index
            if button.master_index < file_info.masters.len() {
                file_info.current_master_index = button.master_index;

                // Update all button states
                for (other_button, mut bg, mut border) in all_buttons.iter_mut() {
                    let is_selected = other_button.master_index == button.master_index;
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

                if let Some(current_master) = file_info.masters.get(button.master_index) {
                    debug!(
                        "Switching to UFO master: {} ({})",
                        current_master.style_name, current_master.filename
                    );

                    // Send event to handle the actual master switching
                    switch_events.write(SwitchMasterEvent {
                        master_index: button.master_index,
                        master_path: current_master.file_path.clone(),
                    });
                }
            }
        }
    }
}

/// Handle master switching events
fn handle_switch_master_events(
    mut switch_events: EventReader<SwitchMasterEvent>,
    mut commands: Commands,
    mut fontir_state: Option<ResMut<FontIRAppState>>,
    buffer_entities: Res<BufferSortEntities>,
    sort_query: Query<Entity, With<crate::editing::sort::Sort>>,
) {
    for event in switch_events.read() {
        debug!(
            "🔄 Processing master switch event: switching to master {} at path: {}",
            event.master_index,
            event.master_path.display()
        );

        // Step 1: Try to load the new FontIR state from the master UFO file
        match FontIRAppState::from_path(event.master_path.clone()) {
            Ok(new_fontir_state) => {
                debug!(
                    "✅ Successfully loaded new FontIR state from: {}",
                    event.master_path.display()
                );

                // Step 2: Replace the existing FontIR state
                if let Some(ref mut current_fontir) = fontir_state {
                    **current_fontir = new_fontir_state;
                    debug!("✅ Replaced FontIR state with new master");
                } else {
                    // Insert new FontIR state if none exists
                    commands.insert_resource(new_fontir_state);
                    debug!("✅ Inserted new FontIR state");
                }

                // Step 3: Clear all existing buffer sort entities to force refresh
                // This will trigger the spawn_missing_sort_entities system to recreate them
                // with the new glyph data and advance widths from the new master
                for (&_buffer_index, &entity) in buffer_entities.entities.iter() {
                    if sort_query.get(entity).is_ok() {
                        commands.entity(entity).despawn();
                        debug!(
                            "🗑️ Despawned sort entity {:?} to force refresh with new master",
                            entity
                        );
                    }
                }

                // The buffer entities will be cleared by despawn_missing_buffer_sort_entities system
                // and then recreated by spawn_missing_sort_entities system with the new FontIR data

                debug!(
                    "🔄 Master switch complete - sorts will be recreated with new advance widths"
                );
            }
            Err(e) => {
                error!(
                    "❌ Failed to load FontIR state from {}: {}",
                    event.master_path.display(),
                    e
                );
            }
        }
    }
}

/// Shows/hides the file pane (always visible for now, but could be toggled later)
fn toggle_file_pane_visibility(mut file_pane: Query<&mut Visibility, With<FilePane>>) {
    // For now, always keep the file pane visible
    for mut visibility in file_pane.iter_mut() {
        *visibility = Visibility::Visible;
    }
}
