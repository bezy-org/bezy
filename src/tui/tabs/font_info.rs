use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::sync::mpsc;

use crate::tui::communication::{FontInfo, TuiMessage};

#[derive(Debug, Clone)]
pub struct FontInfoState {
    pub font_info: Option<FontInfo>,
}

impl Default for FontInfoState {
    fn default() -> Self {
        Self::new()
    }
}

impl FontInfoState {
    pub fn new() -> Self {
        Self { font_info: None }
    }

    pub fn update(&mut self, info: FontInfo) {
        self.font_info = Some(info);
    }
}

/// Handle key events for the FontInfo tab
pub async fn handle_key_event(
    _state: &mut FontInfoState,
    _key: KeyEvent,
    _app_tx: &mpsc::UnboundedSender<TuiMessage>,
) -> Result<()> {
    // Font info is read-only, no key handling needed
    Ok(())
}

/// Draw the FontInfo tab UI
pub fn draw(f: &mut Frame, state: &FontInfoState, area: Rect) {
    let info_text = if let Some(ref info) = state.font_info {
        vec![
            Line::from(vec![Span::styled(
                "Font Metadata",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(format!(
                "  Family Name:    {}",
                info.family_name.as_deref().unwrap_or("Unknown")
            )),
            Line::from(format!(
                "  Style Name:     {}",
                info.style_name.as_deref().unwrap_or("Unknown")
            )),
            Line::from(format!(
                "  Version:        {}",
                info.version.as_deref().unwrap_or("Unknown")
            )),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Metrics",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(format!(
                "  Units Per Em:   {}",
                info.units_per_em
                    .map_or("N/A".to_string(), |v| v.to_string())
            )),
            Line::from(format!(
                "  Ascender:       {}",
                info.ascender.map_or("N/A".to_string(), |v| v.to_string())
            )),
            Line::from(format!(
                "  Descender:      {}",
                info.descender.map_or("N/A".to_string(), |v| v.to_string())
            )),
            Line::from(format!(
                "  Cap Height:     {}",
                info.cap_height.map_or("N/A".to_string(), |v| v.to_string())
            )),
            Line::from(format!(
                "  X-Height:       {}",
                info.x_height.map_or("N/A".to_string(), |v| v.to_string())
            )),
        ]
    } else {
        vec![Line::from("No font loaded")]
    };

    let paragraph =
        Paragraph::new(info_text).block(Block::default().borders(Borders::ALL).title("Font Info"));

    f.render_widget(paragraph, area);
}
