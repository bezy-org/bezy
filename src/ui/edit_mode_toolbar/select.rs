//! Select tool resource definitions
//!
//! This module provides the SelectModeActive resource that's used by the selection systems
//! to determine when selection behavior should be active. The actual tool behavior is
//! handled by the config-based toolbar system.


// Use SelectModeActive from tools::select and re-export it
pub use crate::tools::select::SelectModeActive;

/// Just an alias for clarity
pub type SelectMode = SelectModeActive;
