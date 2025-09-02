//! Point rendering system
//!
//! This module handles the mesh-based rendering of points (both on-curve and off-curve)
//! replacing the previous gizmo-based point rendering.

#![allow(clippy::too_many_arguments)]

use crate::editing::selection::components::{PointType, Selected};
use crate::editing::sort::ActiveSort;
use crate::rendering::zoom_aware_scaling::CameraResponsiveScale;
use crate::systems::sort_manager::SortPointEntity;
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
            commands.entity(entity).despawn();
        }
        return;
    }

    // Clear existing point meshes
    for entity in existing_point_meshes.iter() {
        commands.entity(entity).despawn();
    }

    // Render all points using meshes
    for (point_entity, transform, point_type, selected) in all_point_entities.iter() {
        let position = transform.translation().truncate();

        // Determine colors for two-layer system
        let (primary_color, secondary_color) = if selected.is_some() {
            (
                theme.theme().selected_primary_color(),
                theme.theme().selected_secondary_color(),
            )
        } else if point_type.is_on_curve {
            (
                theme.theme().on_curve_primary_color(),
                theme.theme().on_curve_secondary_color(),
            )
        } else {
            (
                theme.theme().off_curve_primary_color(),
                theme.theme().off_curve_secondary_color(),
            )
        };

        // Create the three-layer point shape
        if point_type.is_on_curve && USE_SQUARE_FOR_ON_CURVE {
            // On-curve points: square with three layers
            let base_size = ON_CURVE_POINT_RADIUS * ON_CURVE_SQUARE_ADJUSTMENT * 2.0;

            // Layer 1: Base shape (full width) - primary color
            commands.spawn((
                PointMesh {
                    point_entity,
                    is_outer: true,
                },
                Sprite {
                    color: primary_color,
                    custom_size: Some(Vec2::splat(base_size)),
                    ..default()
                },
                Transform::from_translation(position.extend(10.0)), // Above outlines
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));

            // Layer 2: Slightly smaller shape - secondary color
            let secondary_size = base_size * 0.7;
            commands.spawn((
                PointMesh {
                    point_entity,
                    is_outer: false,
                },
                Sprite {
                    color: secondary_color,
                    custom_size: Some(Vec2::splat(secondary_size)),
                    ..default()
                },
                Transform::from_translation(position.extend(11.0)), // Above base
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));

            // Layer 3: Small center shape - primary color (only for non-selected points)
            if selected.is_none() {
                let center_size = base_size * ON_CURVE_INNER_CIRCLE_RATIO;
                commands.spawn((
                    PointMesh {
                        point_entity,
                        is_outer: false,
                    },
                    Sprite {
                        color: primary_color,
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
                ON_CURVE_POINT_RADIUS
            } else {
                OFF_CURVE_POINT_RADIUS
            };

            // Layer 1: Base circle (full size) - primary color
            commands.spawn((
                PointMesh {
                    point_entity,
                    is_outer: true,
                },
                Mesh2d(meshes.add(Circle::new(base_radius))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(primary_color))),
                Transform::from_translation(position.extend(10.0)), // Above outlines
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));

            // Layer 2: Slightly smaller circle - secondary color
            let secondary_radius = base_radius * 0.7;
            commands.spawn((
                PointMesh {
                    point_entity,
                    is_outer: false,
                },
                Mesh2d(meshes.add(Circle::new(secondary_radius))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(secondary_color))),
                Transform::from_translation(position.extend(11.0)), // Above base
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));

            // Layer 3: Small center circle - primary color (only for non-selected points)
            if selected.is_none() {
                let center_radius = base_radius
                    * if point_type.is_on_curve {
                        ON_CURVE_INNER_CIRCLE_RATIO
                    } else {
                        OFF_CURVE_INNER_CIRCLE_RATIO
                    };
                commands.spawn((
                    PointMesh {
                        point_entity,
                        is_outer: false,
                    },
                    Mesh2d(meshes.add(Circle::new(center_radius))),
                    MeshMaterial2d(materials.add(ColorMaterial::from_color(primary_color))),
                    Transform::from_translation(position.extend(12.0)), // Above secondary
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ));
            }
        }

        // Add crosshairs for selected points using primary color only
        if selected.is_some() {
            let line_size = if point_type.is_on_curve {
                ON_CURVE_POINT_RADIUS
            } else {
                OFF_CURVE_POINT_RADIUS
            };

            // Use camera-responsive line width (1.0 base, same as outlines and handles)
            let line_width = camera_scale.adjusted_line_width();

            // Make crosshair lines slightly shorter to fit within point bounds
            let crosshair_length = line_size * 1.6;

            // Horizontal line - primary color only
            commands.spawn((
                PointMesh {
                    point_entity,
                    is_outer: false,
                },
                Sprite {
                    color: primary_color,
                    custom_size: Some(Vec2::new(crosshair_length, line_width)),
                    ..default()
                },
                Transform::from_translation(position.extend(13.0)), // Above everything
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));

            // Vertical line - primary color only
            commands.spawn((
                PointMesh {
                    point_entity,
                    is_outer: false,
                },
                Sprite {
                    color: primary_color,
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
        app.add_systems(
            Update,
            render_points_with_meshes
                .after(crate::systems::text_editor_sorts::spawn_active_sort_points_optimized)
                .after(crate::editing::selection::nudge::handle_nudge_input),
        );
    }
}
