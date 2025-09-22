use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphInfo {
    pub codepoint: String,
    pub name: Option<String>,
    pub unicode: Option<u32>,
    pub width: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    pub family_name: Option<String>,
    pub style_name: Option<String>,
    pub version: Option<String>,
    pub ascender: Option<f32>,
    pub descender: Option<f32>,
    pub cap_height: Option<f32>,
    pub x_height: Option<f32>,
    pub units_per_em: Option<f32>,
}

#[derive(Debug, Clone)]
pub enum TuiMessage {
    SelectGlyph(String),
    RequestGlyphList,
    RequestFontInfo,
    ChangeZoom(f32),
    Quit,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    CurrentGlyph(String),
    GlyphList(Vec<GlyphInfo>),
    FontInfo(FontInfo),
    FontLoaded(String),
    LogLine(String),
    Error(String),
}