#![allow(unused_variables)]
#![allow(unused_mut)]

use bevy::prelude::*;

/// Initial setup system that runs on startup.
pub fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // Note: Checkerboard is now dynamically managed by the CheckerboardPlugin
    // and doesn't need manual spawning at startup

    // Camera setup is now handled by CameraPlugin - no need to spawn cameras here
    info!("Basic setup complete - cameras handled by CameraPlugin");
}
