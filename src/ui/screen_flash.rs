//! Screen flash feedback system for visual confirmation of file actions
//!
//! Provides a brief full-screen flash of color when actions like save/export complete

use bevy::prelude::*;
use crate::ui::theme::CurrentTheme;

#[derive(Resource)]
pub struct ScreenFlash {
    active: bool,
    elapsed: f32,
    duration: f32,
}

impl Default for ScreenFlash {
    fn default() -> Self {
        Self {
            active: false,
            elapsed: 0.0,
            duration: 0.3,
        }
    }
}

impl ScreenFlash {
    pub fn trigger(&mut self) {
        self.active = true;
        self.elapsed = 0.0;
    }

    pub fn opacity(&self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let progress = self.elapsed / self.duration;
        if progress < 0.5 {
            progress * 2.0 * 0.15
        } else {
            (1.0 - progress) * 2.0 * 0.15
        }
    }

    pub fn update(&mut self, delta: f32) {
        if self.active {
            self.elapsed += delta;
            if self.elapsed >= self.duration {
                self.active = false;
                self.elapsed = 0.0;
            }
        }
    }
}

#[derive(Component)]
struct ScreenFlashOverlay;

pub struct ScreenFlashPlugin;

impl Plugin for ScreenFlashPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenFlash>()
            .add_systems(Startup, setup_flash_overlay)
            .add_systems(
                Update,
                (update_flash_state, render_flash_overlay).chain(),
            );
    }
}

fn setup_flash_overlay(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
        ScreenFlashOverlay,
        GlobalZIndex(10000),
    ));
}

fn update_flash_state(time: Res<Time>, mut flash: ResMut<ScreenFlash>) {
    flash.update(time.delta_secs());
}

fn render_flash_overlay(
    flash: Res<ScreenFlash>,
    theme: Res<CurrentTheme>,
    mut query: Query<&mut BackgroundColor, With<ScreenFlashOverlay>>,
) {
    let opacity = flash.opacity();

    for mut bg_color in query.iter_mut() {
        if opacity > 0.0 {
            let mut color = theme.action_color();
            color = color.with_alpha(opacity);
            *bg_color = BackgroundColor(color);
        } else {
            *bg_color = BackgroundColor(Color::NONE);
        }
    }
}
