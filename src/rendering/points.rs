//! Point rendering system
//!
//! This module handles the mesh-based rendering of points (both on-curve and off-curve)
//! replacing the previous gizmo-based point rendering.

#![allow(clippy::too_many_arguments)]

use crate::editing::selection::components::{PointType, Selected};
use crate::editing::sort::manager::SortPointEntity;
use crate::editing::sort::ActiveSort;
use crate::rendering::zoom_aware_scaling::CameraResponsiveScale;
use crate::ui::theme::*;
use crate::ui::themes::CurrentTheme;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::render::view::Visibility;
use bevy::sprite::{ColorMaterial, MeshMaterial2d};

/// Component to mark entities as point visual meshes
#[derive(Component)]
pub struct PointMesh {
    pub point_entity: Entity,
    pub is_outer: bool, // true for outer shape, false for inner dot
}

/// System to render points using meshes instead of gizmos
#[allow(clippy::type_complexity)]
pub fn render_points_with_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    _point_entities: Query<
        (Entity, &GlobalTransform, &PointType),
        (With<SortPointEntity>, Without<Selected>),
    >,
    all_point_entities: Query<
        (Entity, &GlobalTransform, &PointType, Option<&Selected>),
        With<SortPointEntity>,
    >,
    active_sorts: Query<Entity, With<ActiveSort>>,
    existing_point_meshes: Query<Entity, With<PointMesh>>,
    theme: Res<CurrentTheme>,
    camera_scale: Res<CameraResponsiveScale>,
) {
    let _all_point_count = all_point_entities.iter().count();
    let _existing_mesh_count = existing_point_meshes.iter().count();
    let active_sort_count = active_sorts.iter().count();

    // Early return if no active sorts
    if active_sort_count == 0 {
        // Clean up existing point meshes when no active sorts
        for entity in existing_point_meshes.iter() {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.despawn();
            }
        }
        return;
    }

    // Clear existing point meshes
    for entity in existing_point_meshes.iter() {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }

    // Render all points using meshes
    for (point_entity, transform, point_type, selected) in all_point_entities.iter() {
        let position = transform.translation().truncate();

        // Determine colors for two-layer system
        // Swap primary and secondary to make secondary the outline/center (darker) and primary the middle (lighter)
        let (outline_color, middle_color) = if selected.is_some() {
            (
                theme.theme().selected_secondary_color(), // darker color for outline/center
                theme.theme().selected_primary_color(),   // lighter color for middle ring
            )
        } else if point_type.is_on_curve {
            (
                theme.theme().on_curve_secondary_color(), // darker color for outline/center
                theme.theme().on_curve_primary_color(),   // lighter color for middle ring
            )
        } else {
            (
                theme.theme().off_curve_secondary_color(), // darker color for outline/center
                theme.theme().off_curve_primary_color(),   // lighter color for middle ring
            )
        };

        // Create the three-layer point shape
        if point_type.is_on_curve && theme.theme().use_square_for_on_curve() {
            // On-curve points: square with three layers
            let base_size = theme.theme().on_curve_point_radius()
                * theme.theme().on_curve_square_adjustment()
                * 2.0;

            // Layer 1: Base shape (full width) - outline color (darker)
            commands.spawn((
                PointMesh {
                    point_entity,
                    is_outer: true,
                },
                Sprite {
                    color: outline_color,
                    custom_size: Some(Vec2::splat(base_size)),
                    ..default()
                },
                Transform::from_translation(position.extend(10.0)), // Above outlines
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));

            // Layer 2: Slightly smaller shape - middle color (lighter)
            let secondary_size = base_size * 0.7;
            commands.spawn((
                PointMesh {
                    point_entity,
                    is_outer: false,
                },
                Sprite {
                    color: middle_color,
                    custom_size: Some(Vec2::splat(secondary_size)),
                    ..default()
                },
                Transform::from_translation(position.extend(11.0)), // Above base
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));

            // Layer 3: Small center shape - outline color (darker, only for non-selected points)
            if selected.is_none() {
                let center_size = base_size * theme.theme().on_curve_inner_circle_ratio();
                commands.spawn((
                    PointMesh {
                        point_entity,
                        is_outer: false,
                    },
                    Sprite {
                        color: outline_color,
                        custom_size: Some(Vec2::splat(center_size)),
                        ..default()
                    },
                    Transform::from_translation(position.extend(12.0)), // Above secondary
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ));
            }
        } else {
            // Off-curve points and circular on-curve points: circle with three layers
            let base_radius = if point_type.is_on_curve {
                theme.theme().on_curve_point_radius()
            } else {
                theme.theme().off_curve_point_radius()
            };

            // Layer 1: Base circle (full size) - outline color (darker)
            commands.spawn((
                PointMesh {
                    point_entity,
                    is_outer: true,
                },
                Mesh2d(meshes.add(Circle::new(base_radius))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(outline_color))),
                Transform::from_translation(position.extend(10.0)), // Above outlines
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));

            // Layer 2: Slightly smaller circle - middle color (lighter)
            let secondary_radius = base_radius * 0.7;
            commands.spawn((
                PointMesh {
                    point_entity,
                    is_outer: false,
                },
                Mesh2d(meshes.add(Circle::new(secondary_radius))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(middle_color))),
                Transform::from_translation(position.extend(11.0)), // Above base
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));

            // Layer 3: Small center circle - outline color (darker, only for non-selected points)
            if selected.is_none() {
                let center_radius = base_radius
                    * if point_type.is_on_curve {
                        theme.theme().on_curve_inner_circle_ratio()
                    } else {
                        theme.theme().off_curve_inner_circle_ratio()
                    };
                commands.spawn((
                    PointMesh {
                        point_entity,
                        is_outer: false,
                    },
                    Mesh2d(meshes.add(Circle::new(center_radius))),
                    MeshMaterial2d(materials.add(ColorMaterial::from_color(outline_color))),
                    Transform::from_translation(position.extend(12.0)), // Above secondary
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ));
            }
        }

        // Add crosshairs for selected points using outline color
        if selected.is_some() {
            let line_size = if point_type.is_on_curve {
                theme.theme().on_curve_point_radius()
            } else {
                theme.theme().off_curve_point_radius()
            };

            // Use camera-responsive line width (1.0 base, same as outlines and handles)
            let line_width = camera_scale.adjusted_line_width();

            // Make crosshair lines slightly shorter to fit within point bounds
            let crosshair_length = line_size * 1.6;

            // Horizontal line - outline color (darker)
            commands.spawn((
                PointMesh {
                    point_entity,
                    is_outer: false,
                },
                Sprite {
                    color: outline_color,
                    custom_size: Some(Vec2::new(crosshair_length, line_width)),
                    ..default()
                },
                Transform::from_translation(position.extend(13.0)), // Above everything
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));

            // Vertical line - outline color (darker)
            commands.spawn((
                PointMesh {
                    point_entity,
                    is_outer: false,
                },
                Sprite {
                    color: outline_color,
                    custom_size: Some(Vec2::new(line_width, crosshair_length)),
                    ..default()
                },
                Transform::from_translation(position.extend(13.0)), // Above everything
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        }
    }
}

/// Plugin for mesh-based point rendering
pub struct PointRenderingPlugin;

impl Plugin for PointRenderingPlugin {
    fn build(&self, app: &mut App) {
        // CRITICAL: Must use PostEditingRenderingSet, not individual .after() dependencies!
        // This ensures points render AFTER point entities are spawned.
        // Using .after() alone caused race conditions and 1-2 second rendering lag.
        app.add_systems(
            Update,
            render_points_with_meshes.in_set(crate::rendering::PostEditingRenderingSet),
        );
    }
}
