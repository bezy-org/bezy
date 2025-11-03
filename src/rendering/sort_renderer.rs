//! Sort label management system
//!
//! Manages text labels (glyph names and unicode values) for sorts.
//! The actual sort rendering is handled by the mesh-based system in mesh_glyph_outline.rs

#![allow(clippy::type_complexity)]

use crate::core::state::AppState;
use crate::editing::sort::{ActiveSort, InactiveSort, Sort, SortSystemSet};
use crate::ui::themes::CurrentTheme;
use crate::utils::embedded_assets::{AssetServerFontExt, EmbeddedFonts};
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
    embedded_fonts: Res<EmbeddedFonts>,
    theme: Res<CurrentTheme>,
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
        return;
    };

    let changed_count = sorts_query.iter().count();
    if changed_count > 0 {
        info!("🏷️ manage_sort_labels running: {} changed sorts", changed_count);
    }

    // Remove text for sorts that no longer exist
    let existing_sort_entities: HashSet<Entity> = all_sorts_query.iter().collect();

    // Clean up glyph name text entities
    for (text_entity, sort_name_text) in existing_name_text_query.iter() {
        if !existing_sort_entities.contains(&sort_name_text.sort_entity) {
            if let Ok(mut entity_commands) = commands.get_entity(text_entity) {
                entity_commands.despawn();
            }
        }
    }

    // Clean up unicode text entities
    for (text_entity, sort_unicode_text) in existing_unicode_text_query.iter() {
        if !existing_sort_entities.contains(&sort_unicode_text.sort_entity) {
            if let Ok(mut entity_commands) = commands.get_entity(text_entity) {
                entity_commands.despawn();
            }
        }
    }

    // Create or update text labels for changed sorts
    for (sort_entity, sort, transform) in sorts_query.iter() {
        // Determine text color based on sort state
        let text_color = if active_sorts_query
            .iter()
            .any(|(entity, _)| entity == sort_entity)
        {
            theme.theme().sort_active_metrics_color()
        } else {
            theme.theme().sort_inactive_metrics_color()
        };

        let position = transform.translation.truncate();

        // Calculate position for glyph name text
        let name_transform = calculate_sort_info_transform(position, &app_state, &sort.glyph_name);

        // Check if name text already exists
        let mut _name_text_exists = false;
        for (text_entity, sort_name_text) in existing_name_text_query.iter() {
            if sort_name_text.sort_entity == sort_entity {
                // Update existing text
                if let Ok(mut entity_commands) = commands.get_entity(text_entity) {
                    entity_commands.despawn();
                }
                _name_text_exists = true;
                break;
            }
        }

        // Create glyph name text
        commands.spawn((
            Text2d(sort.glyph_name.clone()),
            TextFont {
                font: asset_server
                    .load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                font_size: 48.0,
                ..default()
            },
            TextColor(text_color),
            Anchor::TopLeft,
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
                    if let Ok(mut entity_commands) = commands.get_entity(text_entity) {
                        entity_commands.despawn();
                    }
                    _unicode_text_exists = true;
                    break;
                }
            }

            // Calculate position for unicode text (below glyph name)
            let mut unicode_transform = name_transform;
            unicode_transform.translation.y -= 60.0;

            // Create unicode text
            commands.spawn((
                Text2d(unicode_string),
                TextFont {
                    font: asset_server
                        .load_font_with_fallback(theme.theme().mono_font_path(), &embedded_fonts),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(text_color.with_alpha(0.7)),
                Anchor::TopLeft,
                TextBounds::UNBOUNDED,
                unicode_transform,
                SortUnicodeText { sort_entity },
            ));
        }
    }
}

/// System to update text label positions when sorts move
pub fn update_sort_label_positions(
    sorts_query: Query<(Entity, &Transform, &Sort), Changed<Transform>>,
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

    for (sort_entity, sort_transform, sort) in sorts_query.iter() {
        let position = sort_transform.translation.truncate();

        // Margins for positioning (reduced to half)
        let left_margin = 10.0;

        // Position at the top-left of the sort, just below the top UPM line
        let upm = app_state.workspace.info.units_per_em as f32;
        let top_upm_y = position.y + upm;
        let label_top_y = top_upm_y - 2.5; // Small offset just below top UPM line

        // Update glyph name text position
        for (mut text_transform, sort_name_text) in name_text_query.iter_mut() {
            if sort_name_text.sort_entity == sort_entity {
                text_transform.translation.x = position.x + left_margin;
                text_transform.translation.y = label_top_y;
                text_transform.translation.z = 10.0; // Above glyph
            }
        }

        // Update unicode text position
        for (mut text_transform, sort_unicode_text) in unicode_text_query.iter_mut() {
            if sort_unicode_text.sort_entity == sort_entity {
                // Position below glyph name
                for (name_transform, sort_name_text) in name_text_query.iter() {
                    if sort_name_text.sort_entity == sort_entity {
                        text_transform.translation =
                            name_transform.translation + Vec3::new(0.0, -60.0, 0.0);
                        break;
                    }
                }
            }
        }
    }
}

/// System to update text label colors when sort states change
pub fn update_sort_label_colors(
    theme: Res<CurrentTheme>,
    active_sorts_query: Query<Entity, Added<ActiveSort>>,
    inactive_sorts_query: Query<Entity, Added<InactiveSort>>,
    mut name_text_query: Query<(&mut TextColor, &SortGlyphNameText), Without<SortUnicodeText>>,
    mut unicode_text_query: Query<(&mut TextColor, &SortUnicodeText), Without<SortGlyphNameText>>,
) {
    // Update colors for newly active sorts
    for sort_entity in active_sorts_query.iter() {
        for (mut text_color, sort_name_text) in name_text_query.iter_mut() {
            if sort_name_text.sort_entity == sort_entity {
                text_color.0 = theme.theme().sort_active_metrics_color();
            }
        }
        for (mut text_color, sort_unicode_text) in unicode_text_query.iter_mut() {
            if sort_unicode_text.sort_entity == sort_entity {
                text_color.0 = theme.theme().sort_active_metrics_color();
            }
        }
    }

    // Update colors for newly inactive sorts
    for sort_entity in inactive_sorts_query.iter() {
        for (mut text_color, sort_name_text) in name_text_query.iter_mut() {
            if sort_name_text.sort_entity == sort_entity {
                text_color.0 = theme.theme().sort_inactive_metrics_color();
            }
        }
        for (mut text_color, sort_unicode_text) in unicode_text_query.iter_mut() {
            if sort_unicode_text.sort_entity == sort_entity {
                text_color.0 = theme.theme().sort_inactive_metrics_color();
            }
        }
    }
}

/// Calculate transform for sort info text positioning
fn calculate_sort_info_transform(
    sort_position: Vec2,
    app_state: &AppState,
    glyph_name: &str,
) -> Transform {
    // Margins for positioning (reduced to half)
    let left_margin = 10.0;

    // Position at the top-left of the sort, just below the top UPM line
    let upm = app_state.workspace.info.units_per_em as f32;
    let top_upm_y = sort_position.y + upm;
    let label_top_y = top_upm_y - 2.5; // Small offset just below top UPM line

    // Position text with offset
    Transform::from_xyz(
        sort_position.x + left_margin,
        label_top_y,
        200.0, // Above glyph
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

/// Plugin for sort label rendering systems
pub struct SortLabelRenderingPlugin;

impl Plugin for SortLabelRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                manage_sort_labels,
                update_sort_label_positions,
                update_sort_label_colors,
            )
                .chain()
                .in_set(SortSystemSet::Rendering),
        );
    }
}
