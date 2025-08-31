//! Sort label management system
//!
//! Manages text labels (glyph names and unicode values) for sorts.
//! The actual sort rendering is handled by the mesh-based system in mesh_glyph_outline.rs

#![allow(clippy::type_complexity)]

use crate::core::state::{AppState, FontIRAppState, SortLayoutMode};
use crate::editing::sort::{ActiveSort, InactiveSort, Sort};
use crate::ui::theme::{MONO_FONT_PATH, SORT_ACTIVE_METRICS_COLOR, SORT_INACTIVE_METRICS_COLOR};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::TextBounds;
use std::collections::HashSet;

/// Component to mark text entities that display glyph names for sorts
#[derive(Component)]
pub struct SortGlyphNameText {
    pub sort_entity: Entity,
}

/// Component to mark text entities that display unicode values for sorts
#[derive(Component)]
pub struct SortUnicodeText {
    pub sort_entity: Entity,
}

/// System to manage text labels (glyph name and unicode) for all sorts
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn manage_sort_labels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    app_state: Option<Res<AppState>>,
    sorts_query: Query<
        (Entity, &Sort, &Transform),
        (Changed<Sort>, Or<(With<ActiveSort>, With<InactiveSort>)>),
    >,
    existing_name_text_query: Query<(Entity, &SortGlyphNameText)>,
    existing_unicode_text_query: Query<(Entity, &SortUnicodeText)>,
    all_sorts_query: Query<Entity, With<Sort>>,
    active_sorts_query: Query<(Entity, &Sort), With<ActiveSort>>,
) {
    // Early return if AppState not available
    let Some(app_state) = app_state else {
        warn!("Sort labels skipped - AppState not available (using FontIR)");
        return;
    };

    // Remove text for sorts that no longer exist
    let existing_sort_entities: HashSet<Entity> = all_sorts_query.iter().collect();

    // Clean up glyph name text entities
    for (text_entity, sort_name_text) in existing_name_text_query.iter() {
        if !existing_sort_entities.contains(&sort_name_text.sort_entity) {
            commands.entity(text_entity).despawn();
        }
    }

    // Clean up unicode text entities
    for (text_entity, sort_unicode_text) in existing_unicode_text_query.iter() {
        if !existing_sort_entities.contains(&sort_unicode_text.sort_entity) {
            commands.entity(text_entity).despawn();
        }
    }

    // Create or update text labels for changed sorts
    for (sort_entity, sort, transform) in sorts_query.iter() {
        // Determine text color based on sort state
        let text_color = if active_sorts_query
            .iter()
            .any(|(entity, _)| entity == sort_entity)
        {
            SORT_ACTIVE_METRICS_COLOR
        } else {
            SORT_INACTIVE_METRICS_COLOR
        };

        let position = transform.translation.truncate();

        // Calculate position for glyph name text
        let name_transform = calculate_glyph_name_transform(position, &app_state, &sort.glyph_name);

        // Check if name text already exists
        let mut _name_text_exists = false;
        for (text_entity, sort_name_text) in existing_name_text_query.iter() {
            if sort_name_text.sort_entity == sort_entity {
                // Update existing text
                commands.entity(text_entity).despawn();
                _name_text_exists = true;
                break;
            }
        }

        // Create glyph name text
        commands.spawn((
            Text2d(sort.glyph_name.clone()),
            TextFont {
                font: asset_server.load(MONO_FONT_PATH),
                font_size: 12.0,
                ..default()
            },
            TextColor(text_color),
            Anchor::Center,
            TextBounds::UNBOUNDED,
            name_transform,
            SortGlyphNameText { sort_entity },
        ));

        // Get unicode value for this glyph
        if let Some(unicode_string) = get_unicode_for_glyph(&sort.glyph_name, &app_state) {
            // Check if unicode text already exists
            let mut _unicode_text_exists = false;
            for (text_entity, sort_unicode_text) in existing_unicode_text_query.iter() {
                if sort_unicode_text.sort_entity == sort_entity {
                    // Update existing text
                    commands.entity(text_entity).despawn();
                    _unicode_text_exists = true;
                    break;
                }
            }

            // Calculate position for unicode text (below glyph name)
            let mut unicode_transform = name_transform;
            unicode_transform.translation.y -= 20.0;

            // Create unicode text
            commands.spawn((
                Text2d(unicode_string),
                TextFont {
                    font: asset_server.load(MONO_FONT_PATH),
                    font_size: 10.0,
                    ..default()
                },
                TextColor(text_color.with_alpha(0.7)),
                Anchor::Center,
                TextBounds::UNBOUNDED,
                unicode_transform,
                SortUnicodeText { sort_entity },
            ));
        }
    }
}

/// System to update text label positions when sorts move
pub fn update_sort_label_positions(
    sorts_query: Query<(Entity, &Transform), (With<Sort>, Changed<Transform>)>,
    mut name_text_query: Query<(&mut Transform, &SortGlyphNameText), Without<Sort>>,
    mut unicode_text_query: Query<
        (&mut Transform, &SortUnicodeText),
        (Without<Sort>, Without<SortGlyphNameText>),
    >,
    app_state: Option<Res<AppState>>,
) {
    let Some(app_state) = app_state else {
        return;
    };

    for (sort_entity, sort_transform) in sorts_query.iter() {
        let position = sort_transform.translation.truncate();

        // Update glyph name text position
        for (mut text_transform, sort_name_text) in name_text_query.iter_mut() {
            if sort_name_text.sort_entity == sort_entity {
                // Get glyph advance width for proper positioning
                if let Some(selected_glyph) = &app_state.workspace.selected {
                    if let Some(glyph_data) = app_state.workspace.font.glyphs.get(selected_glyph) {
                        let advance_width = glyph_data.advance_width as f32;
                        let descender =
                            app_state.workspace.info.metrics.descender.unwrap_or(-200.0) as f32;

                        // Position text to the right of the glyph, at descender height
                        text_transform.translation.x = position.x + advance_width + 20.0;
                        text_transform.translation.y = position.y + descender;
                        text_transform.translation.z = 10.0; // Above glyph
                    }
                }
            }
        }

        // Update unicode text position
        for (mut text_transform, sort_unicode_text) in unicode_text_query.iter_mut() {
            if sort_unicode_text.sort_entity == sort_entity {
                // Position below glyph name
                for (name_transform, sort_name_text) in name_text_query.iter() {
                    if sort_name_text.sort_entity == sort_entity {
                        text_transform.translation =
                            name_transform.translation + Vec3::new(0.0, -20.0, 0.0);
                        break;
                    }
                }
            }
        }
    }
}

/// System to update text label colors when sort states change
pub fn update_sort_label_colors(
    active_sorts_query: Query<Entity, Added<ActiveSort>>,
    inactive_sorts_query: Query<Entity, Added<InactiveSort>>,
    mut name_text_query: Query<(&mut TextColor, &SortGlyphNameText), Without<SortUnicodeText>>,
    mut unicode_text_query: Query<(&mut TextColor, &SortUnicodeText), Without<SortGlyphNameText>>,
) {
    // Update colors for newly active sorts
    for sort_entity in active_sorts_query.iter() {
        for (mut text_color, sort_name_text) in name_text_query.iter_mut() {
            if sort_name_text.sort_entity == sort_entity {
                text_color.0 = SORT_ACTIVE_METRICS_COLOR;
            }
        }
        for (mut text_color, sort_unicode_text) in unicode_text_query.iter_mut() {
            if sort_unicode_text.sort_entity == sort_entity {
                text_color.0 = SORT_ACTIVE_METRICS_COLOR.with_alpha(0.7);
            }
        }
    }

    // Update colors for newly inactive sorts
    for sort_entity in inactive_sorts_query.iter() {
        for (mut text_color, sort_name_text) in name_text_query.iter_mut() {
            if sort_name_text.sort_entity == sort_entity {
                text_color.0 = SORT_INACTIVE_METRICS_COLOR;
            }
        }
        for (mut text_color, sort_unicode_text) in unicode_text_query.iter_mut() {
            if sort_unicode_text.sort_entity == sort_entity {
                text_color.0 = SORT_INACTIVE_METRICS_COLOR.with_alpha(0.7);
            }
        }
    }
}

/// Calculate transform for glyph name text positioning
fn calculate_glyph_name_transform(
    sort_position: Vec2,
    app_state: &AppState,
    glyph_name: &str,
) -> Transform {
    // Get glyph advance width for proper positioning
    let advance_width = if let Some(glyph_data) = app_state.workspace.font.glyphs.get(glyph_name) {
        glyph_data.advance_width as f32
    } else {
        500.0 // Default if glyph not found
    };

    let descender = app_state.workspace.info.metrics.descender.unwrap_or(-200.0) as f32;

    // Position text to the right of the glyph, at descender height
    Transform::from_xyz(
        sort_position.x + advance_width + 20.0,
        sort_position.y + descender,
        10.0, // Above glyph
    )
}

/// Get unicode value string for a glyph name
fn get_unicode_for_glyph(glyph_name: &str, app_state: &AppState) -> Option<String> {
    // Check if glyph has unicode value
    if let Some(glyph_data) = app_state.workspace.font.glyphs.get(glyph_name) {
        if !glyph_data.unicode_values.is_empty() {
            // Format unicode values
            let unicode_strings: Vec<String> = glyph_data
                .unicode_values
                .iter()
                .map(|&code| format!("U+{:04X}", code as u32))
                .collect();
            return Some(unicode_strings.join(", "));
        }
    }
    None
}
