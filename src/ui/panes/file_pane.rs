#![allow(unreachable_code, dead_code)]
//! File Pane Module
//!
//! This module implements a floating panel in the upper left corner that displays
//! information about the currently loaded font files and allows switching between
//! UFO masters in a designspace.


use crate::systems::sorts::sort_entities::BufferSortEntities;
use crate::ui::theme::*;
use crate::ui::theme_system::UiBorderRadius;
use crate::ui::themes::CurrentTheme;
use crate::utils::embedded_assets::{AssetServerFontExt, EmbeddedFonts};
use bevy::prelude::*;
use bevy::ui::Display;
use bevy::window::Window;
use chrono::{DateTime, Local};
use norad::designspace::DesignSpaceDocument;
use std::path::PathBuf;
use std::time::SystemTime;

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
            crate::ui::theme_system::WidgetBorderRadius,
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
                        TextColor(theme.active_color()),
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
                        TextColor(theme.active_color()),
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
                        TextColor(theme.active_color()),
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
                        TextColor(theme.active_color()),
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
    mut file_info: ResMut<FileInfo>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    // TODO: Re-enable after FontIR removal - update file info from FontIR state
    // FontIR removed - file info update logic needs to be reimplemented
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
                if let Ok(mut entity_commands) = commands.get_entity(child) {
                    entity_commands.despawn();
                }
            }
        }
    }

    // TODO: Re-enable after FontIR removal - check if single UFO or designspace
    // FontIR removed - always show master buttons for now
    let should_show_masters = true;

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
                BorderRadius::all(Val::Px(MASTER_BUTTON_SIZE / 2.0)), // Perfect circle
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

// Type aliases for complex row queries
type DesignspaceRowQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Node,
    (
        With<DesignspaceRowContainer>,
        Without<SavedRowContainer>,
        Without<ExportedRowContainer>,
    ),
>;

type SavedRowQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Node,
    (
        With<SavedRowContainer>,
        Without<ExportedRowContainer>,
        Without<DesignspaceRowContainer>,
    ),
>;

type ExportedRowQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Node,
    (
        With<ExportedRowContainer>,
        Without<SavedRowContainer>,
        Without<DesignspaceRowContainer>,
    ),
>;

/// Updates the displayed file information
#[allow(clippy::too_many_arguments)]
fn update_file_display(
    file_info: Res<FileInfo>,
    mut designspace_query: DesignspaceTextQuery,
    mut ufo_query: CurrentUFOTextQuery,
    mut saved_query: LastSavedTextQuery,
    mut exported_query: LastExportedTextQuery,
    mut designspace_row_query: DesignspaceRowQuery,
    mut saved_row_query: SavedRowQuery,
    mut exported_row_query: ExportedRowQuery,
) {
    // Check if this is a single UFO or designspace
    // TODO: Re-enable after FontIR removal - check if single UFO
    let is_single_ufo = false; // Default to designspace for now

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
    buffer_entities: Res<BufferSortEntities>,
    sort_query: Query<Entity, With<crate::editing::sort::Sort>>,
) {
    for event in switch_events.read() {
        debug!(
            "🔄 Processing master switch event: switching to master {} at path: {}",
            event.master_index,
            event.master_path.display()
        );

        // FontIR removed - master switching temporarily disabled
        debug!("Master switch temporarily disabled due to FontIR removal");
        let _ = event;
    }
}

/// Shows/hides the file pane (always visible for now, but could be toggled later)
fn toggle_file_pane_visibility(mut file_pane: Query<&mut Visibility, With<FilePane>>) {
    // For now, always keep the file pane visible
    for mut visibility in file_pane.iter_mut() {
        *visibility = Visibility::Visible;
    }
}
