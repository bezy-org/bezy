//! UFO file I/O operations

use anyhow::Result;
use norad::Font;
use std::path::Path;

/// Load a UFO font file from disk
#[allow(dead_code)]
pub fn load_ufo_from_path(path: impl AsRef<Path>) -> Result<Font> {
    let font = Font::load(path)?;
    Ok(font)
}

