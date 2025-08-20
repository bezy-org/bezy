//!    Error handling
//!
//! This module provides error handling using anyhow.
//! As an application (not a library), we prioritize ease of use over
//! complex error type hierarchies for now, that might change in the future.

#[allow(unused_imports)]
pub use anyhow::{anyhow, bail, ensure, Error};
use anyhow::{Context, Result};

/// Result type alias for convenience throughout the application
pub type BezyResult<T> = Result<T>;

/// Helper functions for creating common error contexts
pub trait BezyContext<T> {
    /// Add file operation context to an error
    fn with_file_context<P: AsRef<std::path::Path>>(
        self,
        operation: &str,
        path: P,
    ) -> BezyResult<T>;

    /// Add glyph operation context to an error
    #[allow(dead_code)]
    fn with_glyph_context(
        self,
        operation: &str,
        glyph_name: &str,
    ) -> BezyResult<T>;

    /// Add point operation context to an error
    #[allow(dead_code)]
    fn with_point_context(
        self,
        operation: &str,
        glyph_name: &str,
        contour_idx: usize,
        point_idx: usize,
    ) -> BezyResult<T>;
}

impl<T, E> BezyContext<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_file_context<P: AsRef<std::path::Path>>(
        self,
        operation: &str,
        path: P,
    ) -> BezyResult<T> {
        self.with_context(|| {
            format!("Failed to {} file: {}", operation, path.as_ref().display())
        })
    }

    fn with_glyph_context(
        self,
        operation: &str,
        glyph_name: &str,
    ) -> BezyResult<T> {
        self.with_context(|| {
            format!(
                "Failed to {operation} glyph '{glyph_name}': Operation failed"
            )
        })
    }

    fn with_point_context(
        self,
        operation: &str,
        glyph_name: &str,
        contour_idx: usize,
        point_idx: usize,
    ) -> BezyResult<T> {
        self.with_context(|| {
            format!(
                "Failed to {operation} point {contour_idx}.{point_idx} in glyph '{glyph_name}': Operation failed"
            )
        })
    }
}

/// Helper macros for common error patterns
#[macro_export]
macro_rules! glyph_not_found {
    ($name:expr, $available:expr) => {
        anyhow::anyhow!(
            "Glyph '{}' not found in font (available glyphs: {})",
            $name,
            $available
        )
    };
}

#[macro_export]
macro_rules! point_out_of_bounds {
    ($glyph:expr, $contour:expr, $point:expr, $max:expr) => {
        anyhow::anyhow!(
            "Point index {} out of bounds in contour {} of glyph '{}' (max: {})",
            $point,
            $contour,
            $glyph,
            $max
        )
    };
}

#[macro_export]
macro_rules! contour_out_of_bounds {
    ($glyph:expr, $contour:expr, $max:expr) => {
        anyhow::anyhow!(
            "Contour index {} out of bounds in glyph '{}' (max: {})",
            $contour,
            $glyph,
            $max
        )
    };
}

/// Validation helpers that return anyhow errors
#[allow(dead_code)]
pub fn validate_finite_coords(x: f64, y: f64) -> BezyResult<()> {
    ensure!(x.is_finite(), "X coordinate must be finite, got: {}", x);
    ensure!(y.is_finite(), "Y coordinate must be finite, got: {}", y);
    Ok(())
}

pub fn validate_ufo_path<P: AsRef<std::path::Path>>(path: P) -> BezyResult<()> {
    let path = path.as_ref();

    ensure!(path.exists(), "UFO path does not exist: {}", path.display());
    ensure!(
        path.is_dir(),
        "UFO path must be a directory: {}",
        path.display()
    );

    let metainfo = path.join("metainfo.plist");
    ensure!(
        metainfo.exists(),
        "Invalid UFO: missing metainfo.plist in {}",
        path.display()
    );

    Ok(())
}
