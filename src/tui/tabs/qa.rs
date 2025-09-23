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
pub struct QAState {
    // TODO: Add quality assurance state fields
}

impl QAState {
    pub fn new() -> Self {
        Self {}
    }
}

/// Handle key events for the QA tab
pub async fn handle_key_event(
    _state: &mut QAState,
    _key: KeyEvent,
    _app_tx: &mpsc::UnboundedSender<TuiMessage>,
) -> Result<()> {
    // TODO: Implement QA-specific key handling
    Ok(())
}

/// Draw the QA tab UI
pub fn draw(f: &mut Frame, _state: &mut QAState, area: Rect) {
    let paragraph = Paragraph::new("Quality Assurance tools coming soon...")
        .block(Block::default().borders(Borders::ALL).title("QA"))
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}