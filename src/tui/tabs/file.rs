use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::sync::mpsc;

use crate::tui::communication::TuiMessage;

#[derive(Debug, Clone)]
pub struct FileState {
    pub selected_index: usize,
}

impl FileState {
    pub fn new() -> Self {
        Self { selected_index: 0 }
    }
}

/// Handle key events for the File tab
pub async fn handle_key_event(
    _state: &mut FileState,
    key: KeyEvent,
    _app_tx: &mpsc::UnboundedSender<TuiMessage>,
) -> Result<()> {
    match key {
        KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            // TODO: Implement save functionality
        }
        KeyEvent {
            code: KeyCode::Char('S'),
            modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ..
        } => {
            // TODO: Implement save as functionality
        }
        KeyEvent {
            code: KeyCode::Char('o'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            // TODO: Implement open file functionality
        }
        KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            // TODO: Implement export functionality
        }
        _ => {}
    }
    Ok(())
}

/// Draw the File tab UI
pub fn draw(f: &mut Frame, _state: &mut FileState, area: Rect) {
    let file_menu = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  File Operations",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  Ctrl+S         - Save current font"),
        Line::from("  Ctrl+Shift+S   - Save As..."),
        Line::from("  Ctrl+O         - Open font file"),
        Line::from("  Ctrl+E         - Export as TTF"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Recent Files",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  No recent files"),
    ];

    let paragraph =
        Paragraph::new(file_menu).block(Block::default().borders(Borders::ALL).title("File"));

    f.render_widget(paragraph, area);
}
