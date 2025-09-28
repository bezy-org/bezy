use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::sync::mpsc;

use crate::tui::communication::TuiMessage;

#[derive(Debug, Clone)]
pub struct GlyphState {
    // TODO: Add glyph-specific state fields
    pub current_glyph: Option<String>,
}

impl GlyphState {
    pub fn new() -> Self {
        Self {
            current_glyph: None,
        }
    }
}

/// Handle key events for the Glyph tab
pub async fn handle_key_event(
    _state: &mut GlyphState,
    _key: KeyEvent,
    _app_tx: &mpsc::UnboundedSender<TuiMessage>,
) -> Result<()> {
    // TODO: Implement glyph-specific key handling
    // - Navigate between glyphs
    // - Edit glyph properties
    // - View glyph metrics
    Ok(())
}

/// Draw the Glyph tab UI
pub fn draw(f: &mut Frame, _state: &mut GlyphState, area: Rect) {
    let paragraph = Paragraph::new("Glyph editing tools coming soon...")
        .block(Block::default().borders(Borders::ALL).title("Glyph"))
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}
