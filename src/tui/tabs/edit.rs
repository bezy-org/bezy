use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::sync::mpsc;

use crate::tui::communication::TuiMessage;

#[derive(Debug, Clone)]
pub struct EditState {
    pub selected_index: usize,
}

impl Default for EditState {
    fn default() -> Self {
        Self::new()
    }
}

impl EditState {
    pub fn new() -> Self {
        Self { selected_index: 0 }
    }
}

/// Handle key events for the Edit tab
pub async fn handle_key_event(
    _state: &mut EditState,
    key: KeyEvent,
    _app_tx: &mpsc::UnboundedSender<TuiMessage>,
) -> Result<()> {
    match key {
        KeyEvent {
            code: KeyCode::Char('z'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            // TODO: Implement undo functionality
        }
        KeyEvent {
            code: KeyCode::Char('Z'),
            modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ..
        } => {
            // TODO: Implement redo functionality
        }
        KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            // TODO: Implement cut functionality
        }
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            // TODO: Implement copy functionality
        }
        KeyEvent {
            code: KeyCode::Char('v'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            // TODO: Implement paste functionality
        }
        _ => {}
    }
    Ok(())
}

/// Draw the Edit tab UI
pub fn draw(f: &mut Frame, _state: &mut EditState, area: Rect) {
    let edit_menu = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Edit Operations",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  Ctrl+Z         - Undo"),
        Line::from("  Ctrl+Shift+Z   - Redo"),
        Line::from("  Ctrl+X         - Cut"),
        Line::from("  Ctrl+C         - Copy"),
        Line::from("  Ctrl+V         - Paste"),
        Line::from("  Ctrl+A         - Select All"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Transform",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  Ctrl+T         - Transform selection"),
        Line::from("  Ctrl+R         - Rotate"),
        Line::from("  Ctrl+M         - Mirror/Flip"),
    ];

    let paragraph = Paragraph::new(edit_menu).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled("Edit", Style::default().fg(Color::Green))),
    );

    f.render_widget(paragraph, area);
}
