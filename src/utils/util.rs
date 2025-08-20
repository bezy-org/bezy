//! Shared helpers.

use bevy::prelude::Vec2 as BevyVec2;
use std::convert::TryFrom;

/// Unwrap an optional, printing a message and returning if it is missing.
///
/// This should generate less code than unwrap? Honestly it's a total
/// experiment.
#[macro_export]
macro_rules! bail {
    ($opt:expr $(,)?) => {
        match $opt {
            Some(val) => val,
            None => {
                eprintln!("[{}:{}] bailed", file!(), line!());
                return
            }
        }
    };
     ($opt:expr, $($arg:tt)+) => {
        match $opt {
            Some(val) => val,
            None => {
                eprintln!("[{}:{}] bailed: ", file!(), line!());
                eprintln!($($arg)+);
                return
            }
        }
    };
}

/// Compute scale between two sizes, returning a Vec2
#[allow(dead_code)]
pub fn compute_scale(pre: (f32, f32), post: (f32, f32)) -> BevyVec2 {
    let ensure_finite = |f: f32| if f.is_finite() { f } else { 1.0 };
    let x = ensure_finite(post.0 / pre.0);
    let y = ensure_finite(post.1 / pre.1);
    BevyVec2::new(x, y)
}

/// Creates a new blank font with some placeholder glyphs.
#[allow(dead_code)]
pub fn create_blank_font() -> norad::Font {
    let mut font = norad::Font::new();
    font.font_info = norad::FontInfo {
        family_name: Some("Untitled".into()),
        style_name: Some("Regular".into()),
        units_per_em: Some(TryFrom::try_from(1000.0f64).unwrap()),
        descender: Some(-200.0),
        ascender: Some(800.0),
        cap_height: Some(700.0),
        x_height: Some(500.0),
        ..Default::default()
    };

    let layer = font.default_layer_mut();
    ('a'..='z')
        .chain('A'..='Z')
        .map(|chr| norad::Glyph::new(&chr.to_string()))
        .for_each(|glyph| layer.insert_glyph(glyph));
    font
}
