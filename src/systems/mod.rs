//! Bevy Systems and Plugins
//!
//! This module contains Bevy-specific systems and plugin configurations:
//! - Plugin management and configuration
//! - Command handling for user actions
//! - UI interaction detection and processing
//! - Input consumer system


pub mod commands;
pub mod fontir_lifecycle;
pub mod input_consumer;
pub mod lifecycle;
pub mod plugins;
pub mod sorts;
pub mod startup_layout;
pub mod text_buffer_manager;
pub mod text_shaping;
pub mod ui_interaction;

// Re-export commonly used items
pub use commands::CommandsPlugin;
pub use fontir_lifecycle::{initialize_font_loading, load_font_deferred, DeferredFontLoading};
pub use input_consumer::InputConsumerPlugin;
pub use lifecycle::{exit_on_esc, load_ufo_font};
pub use plugins::{configure_default_plugins, BezySystems};
pub use startup_layout::{center_camera_on_startup_layout, create_startup_layout, migrate_sort_advance_widths};
pub use text_buffer_manager::TextBufferManagerPlugin;
pub use text_shaping::TextShapingPlugin;
pub use ui_interaction::UiInteractionPlugin;
