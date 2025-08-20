//! Application settings and configuration
//!
//! This module contains all configurable settings for the Bezy font editor.
//! For visual/UI settings, see ui/theme.rs

use crate::ui::themes::ThemeVariant;
use bevy::prelude::*;

/// Default UFO file to load if none is specified
pub const DEFAULT_UFO_PATH: &str = "assets/fonts/bezy-grotesk.designspace";
pub const WINDOW_TITLE: &str = "Bezy";
pub const DEFAULT_WINDOW_SIZE: (f32, f32) = (1280.0, 768.0);

/// Configuration for grid snapping behavior
#[derive(Debug, Clone, Copy)]
pub struct GridSettings {
    pub enabled: bool,
    /// Size of the grid in font units
    pub unit_size: f32,
    /// Multiplier for sort placement grid (coarser than point grid)
    pub sort_multiplier: f32,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            unit_size: 2.0,
            sort_multiplier: 8.0,
        }
    }
}

/// Configuration for keyboard nudging behavior
#[derive(Debug, Clone, Copy)]
pub struct NudgeSettings {
    /// Default nudge amount (arrow keys)
    pub default: f32,
    /// Nudge amount with Shift modifier
    pub shift: f32,
    /// Nudge amount with Cmd/Ctrl modifier
    pub cmd: f32,
}

impl Default for NudgeSettings {
    fn default() -> Self {
        Self {
            default: 2.0,
            shift: 8.0,
            cmd: 32.0,
        }
    }
}

/// Main settings resource containing all configuration
///
/// This is a Bevy resource that can be accessed from any system.
/// Settings are loaded at startup and can be modified at runtime.
#[derive(Resource, Debug, Clone, Default)]
pub struct BezySettings {
    pub grid: GridSettings,
    pub nudge: NudgeSettings,
    pub theme: ThemeVariant,
}

impl BezySettings {
    pub fn apply_grid_snap(&self, position: Vec2) -> Vec2 {
        if self.grid.enabled {
            Vec2::new(
                (position.x / self.grid.unit_size).round()
                    * self.grid.unit_size,
                (position.y / self.grid.unit_size).round()
                    * self.grid.unit_size,
            )
        } else {
            position
        }
    }

    pub fn set_theme(&mut self, theme: ThemeVariant) {
        self.theme = theme;
    }

    pub fn get_theme(&self) -> ThemeVariant {
        self.theme.clone()
    }
}
