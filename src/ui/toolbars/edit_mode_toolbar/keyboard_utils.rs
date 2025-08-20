//! Keyboard utilities for the edit mode toolbar
//!
//! This module provides shared utilities for handling keyboard shortcuts
//! across all toolbar tools, particularly for disabling single-character
//! hotkeys when text input is active.

use bevy::prelude::*;

/// Check if single-character hotkeys should be disabled
///
/// Returns true when the user is in a text input mode where single-character
/// keyboard shortcuts should be disabled to allow normal typing.
///
/// This function accepts both Res and ResMut types by using AsRef
pub fn should_disable_single_char_hotkeys<T1, T2>(
    text_mode_active: Option<&T1>,
    current_text_placement_mode: Option<&T2>,
) -> bool 
where
    T1: AsRef<super::text::TextModeActive>,
    T2: AsRef<super::text::CurrentTextPlacementMode>,
{
    // Check if we're in text insert mode
    if let (Some(text_active), Some(placement_mode)) = 
        (text_mode_active, current_text_placement_mode) {
        // Disable single-char hotkeys when in text mode with Insert placement
        let text_active = text_active.as_ref();
        let placement_mode = placement_mode.as_ref();
        text_active.0 && placement_mode.0 == super::text::TextPlacementMode::Insert
    } else {
        false
    }
}