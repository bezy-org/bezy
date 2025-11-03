//! Font metrics and measurement data
//!
//! This module contains structures for font metrics like ascender,
//! descender, x-height, and other measurement information.

use bevy::prelude::*;
use norad::Font;

/// Font information
#[derive(Clone, Default)]
pub struct FontInfo {
    pub family_name: String,
    pub style_name: String,
    pub units_per_em: f64,
    pub metrics: FontMetrics,
    pub ascender: Option<f64>,
    pub descender: Option<f64>,
    pub x_height: Option<f64>,
    pub cap_height: Option<f64>,
}

/// Font metrics for spacing and positioning
#[derive(Clone, Default)]
pub struct FontMetrics {
    pub units_per_em: f64,
    pub descender: Option<f64>,
    pub x_height: Option<f64>,
    pub cap_height: Option<f64>,
    pub ascender: Option<f64>,
    pub italic_angle: Option<f64>,
    pub line_height: f64,
}

impl FontInfo {
    /// Extract font info from norad Font
    pub fn from_norad_font(font: &Font) -> Self {
        let units_per_em = font
            .font_info
            .units_per_em
            .map(|v| v.to_string().parse().unwrap_or(1024.0))
            .unwrap_or(1024.0);
        let ascender = font.font_info.ascender;
        let descender = font.font_info.descender;
        let x_height = font.font_info.x_height;
        let cap_height = font.font_info.cap_height;
        let _italic_angle = font.font_info.italic_angle;

        let metrics = FontMetrics::from_ufo(font);

        Self {
            family_name: Self::extract_string_field(
                &font.font_info,
                |info| &info.family_name,
                "Untitled",
            ),
            style_name: Self::extract_string_field(
                &font.font_info,
                |info| &info.style_name,
                "Regular",
            ),
            units_per_em,
            metrics,
            ascender,
            descender,
            x_height,
            cap_height,
        }
    }

    /// Helper to extract string fields with defaults
    fn extract_string_field<F>(font_info: &norad::FontInfo, getter: F, default: &str) -> String
    where
        F: Fn(&norad::FontInfo) -> &Option<String>,
    {
        getter(font_info)
            .as_ref()
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    /// Get a display name combining family and style names
    pub fn get_display_name(&self) -> String {
        let parts: Vec<&str> = [&self.family_name, &self.style_name]
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.as_str())
            .collect();

        if parts.is_empty() {
            "Untitled Font".to_string()
        } else {
            parts.join(" ")
        }
    }

    /// Convert back to norad FontInfo
    pub fn to_norad_font_info(&self) -> norad::FontInfo {
        let mut info = norad::FontInfo::default();

        // Set family and style names
        if !self.family_name.is_empty() {
            info.family_name = Some(self.family_name.clone());
        }
        if !self.style_name.is_empty() {
            info.style_name = Some(self.style_name.clone());
        }

        // Set numeric values
        if let Some(units_per_em) =
            norad::fontinfo::NonNegativeIntegerOrFloat::new(self.units_per_em)
        {
            info.units_per_em = Some(units_per_em);
        }
        info.ascender = self.ascender;
        info.descender = self.descender;
        info.x_height = self.x_height;
        info.cap_height = self.cap_height;
        info
    }

    /// Get ascender value with sensible default based on UPM
    pub fn ascender_or_default(&self) -> f32 {
        self.ascender
            .unwrap_or(self.units_per_em * 0.8)
            .as_f32()
    }

    /// Get descender value with sensible default based on UPM
    pub fn descender_or_default(&self) -> f32 {
        self.descender
            .unwrap_or(-(self.units_per_em * 0.2))
            .as_f32()
    }

    /// Get x-height value with sensible default based on UPM
    pub fn x_height_or_default(&self) -> f32 {
        self.x_height
            .unwrap_or(self.units_per_em * 0.5)
            .as_f32()
    }

    /// Get cap-height value with sensible default based on UPM
    pub fn cap_height_or_default(&self) -> f32 {
        self.cap_height
            .unwrap_or(self.units_per_em * 0.7)
            .as_f32()
    }

    /// Calculate the UPM top position for a sort at the given position
    pub fn upm_top(&self, sort_position: Vec2) -> f32 {
        sort_position.y + self.ascender_or_default()
    }

    /// Calculate the UPM bottom position for a sort at the given position
    pub fn upm_bottom(&self, sort_position: Vec2) -> f32 {
        sort_position.y + self.descender_or_default()
    }

    /// Calculate the baseline position for a sort at the given position
    pub fn baseline(&self, sort_position: Vec2) -> f32 {
        sort_position.y
    }

    /// Calculate the x-height position for a sort at the given position
    pub fn x_height_position(&self, sort_position: Vec2) -> f32 {
        sort_position.y + self.x_height_or_default()
    }

    /// Calculate the cap-height position for a sort at the given position
    pub fn cap_height_position(&self, sort_position: Vec2) -> f32 {
        sort_position.y + self.cap_height_or_default()
    }
}

/// Extension trait for converting f64 to f32
trait AsF32 {
    fn as_f32(self) -> f32;
}

impl AsF32 for f64 {
    fn as_f32(self) -> f32 {
        self as f32
    }
}

impl FontMetrics {
    /// Extract metrics from a UFO
    pub fn from_ufo(ufo: &Font) -> Self {
        let font_info = &ufo.font_info;

        let units_per_em = font_info
            .units_per_em
            .map(|v| v.to_string().parse().unwrap_or(1024.0))
            .unwrap_or(1024.0);

        // Load metrics from UFO, using reasonable defaults based on units_per_em if missing
        let ascender = font_info.ascender.unwrap_or(units_per_em * 0.8); // 80% of UPM
        let descender = font_info.descender.unwrap_or(-(units_per_em * 0.2)); // -20% of UPM
        let x_height = font_info.x_height;
        let cap_height = font_info.cap_height;
        let _italic_angle = font_info.italic_angle;

        let line_height = ascender - descender;

        Self {
            units_per_em,
            descender: Some(descender),
            x_height,
            cap_height,
            ascender: Some(ascender),
            italic_angle: None,
            line_height,
        }
    }
}
