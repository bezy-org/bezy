//! Bevy Systems and Plugins
//!
//! This module contains Bevy-specific systems and plugin configurations:
//! - Plugin management and configuration
//! - Command handling for user actions
//! - UI interaction detection and processing
//! - Input consumer system

#![allow(unused_imports)]

pub mod arabic_shaping;
pub mod commands;
pub mod fontir_lifecycle;
pub mod harfbuzz_shaping;
pub mod input_consumer;
pub mod lifecycle;
pub mod plugins;
pub mod sort_manager;
pub mod startup_layout;
pub mod text_buffer_manager;
pub mod sorts;
pub mod text_shaping;
pub mod ui_interaction;

// Re-export commonly used items
pub use arabic_shaping::ArabicShapingPlugin;
pub use commands::CommandsPlugin;
pub use fontir_lifecycle::load_fontir_font;
pub use harfbuzz_shaping::HarfBuzzShapingPlugin;
pub use input_consumer::InputConsumerPlugin;
pub use lifecycle::{exit_on_esc, load_ufo_font};
pub use plugins::{configure_default_plugins, BezySystems};
pub use startup_layout::{center_camera_on_startup_layout, create_startup_layout};
pub use text_buffer_manager::TextBufferManagerPlugin;
pub use text_shaping::TextShapingPlugin;
pub use ui_interaction::UiInteractionPlugin;
