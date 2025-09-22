//! Select tool resource definitions
//!
//! This module provides the SelectModeActive resource that's used by the selection systems
//! to determine when selection behavior should be active. The actual tool behavior is
//! handled by the config-based toolbar system.

use bevy::prelude::*;

/// Resource to track if select mode is active
#[derive(Resource, Default, PartialEq, Eq)]
pub struct SelectModeActive(pub bool);

/// Just an alias for clarity
pub type SelectMode = SelectModeActive;