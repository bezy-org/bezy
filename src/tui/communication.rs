use serde::{Deserialize, Serialize};

/// Generate glyph list from FontIR data using Norad for Unicode mappings
pub fn generate_glyph_list(
    fontir_state: &crate::core::state::FontIRAppState,
    _app_state: Option<&crate::core::AppState>, // Keep for API compatibility but use FontIR for Unicode
) -> Vec<GlyphInfo> {
    let mut glyphs = Vec::new();

    // Extract glyph data from FontIR
    for (glyph_name, glyph) in &fontir_state.glyph_cache {
        // Get the first available instance for this glyph
        if let Some((_location, glyph_instance)) = glyph.sources().iter().next() {
            // Get Unicode codepoints directly from UFO via FontIR
            let unicode_codepoints = fontir_state.get_unicode_for_glyph_name(glyph_name);
            let unicode_value = unicode_codepoints.first().copied();

            let glyph_info = GlyphInfo {
                codepoint: glyph_name.clone(),
                name: Some(glyph_name.clone()),
                unicode: unicode_value,
                width: Some(glyph_instance.width as f32),
            };

            glyphs.push(glyph_info);
        }
    }

    // Sort glyphs by Unicode value, then by name
    glyphs.sort_by(|a, b| {
        match (a.unicode, b.unicode) {
            (Some(a_unicode), Some(b_unicode)) => a_unicode.cmp(&b_unicode),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.codepoint.cmp(&b.codepoint),
        }
    });

    glyphs
}

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
    SelectGlyph(u32), // Unicode codepoint instead of glyph name
    RequestGlyphList,
    RequestFontInfo,
    ChangeZoom(f32),
    ForceRedraw, // Force immediate GUI redraw
    QAReportReady(crate::qa::QAReport),
    QAAnalysisFailed(String),
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