//! Sort management module
//!
//! Contains all sort-related editing functionality including components,
//! management systems, and plugin registration.

pub mod components;
pub mod manager;
pub mod plugin;

// Re-export commonly used types
pub use components::*;
pub use manager::*;
pub use plugin::*;
