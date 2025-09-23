use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use tokio::sync::mpsc;

use crate::tui::communication::TuiMessage;

#[derive(Debug, Clone)]
pub struct HelpState {
    pub scroll_offset: usize,
}

impl HelpState {
    pub fn new() -> Self {
        Self { scroll_offset: 0 }
    }
}

/// Handle key events for the Help tab
pub async fn handle_key_event(
    _state: &mut HelpState,
    _key: KeyEvent,
    _app_tx: &mpsc::UnboundedSender<TuiMessage>,
) -> Result<()> {
    // TODO: Implement help navigation (scrolling, search)
    Ok(())
}

/// Draw the Help tab UI
pub fn draw(f: &mut Frame, _state: &mut HelpState, area: Rect) {
    let help_text = vec![
        Line::from(vec![
            Span::styled("Global Controls:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("  Ctrl+Q         - Quit application"),
        Line::from("  Tab            - Next tab"),
        Line::from("  Shift+Tab      - Previous tab"),
        Line::from("  1-9            - Jump to tab by number"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Unicode Tab:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("  ↑/↓ or j/k     - Navigate codepoint list"),
        Line::from("  Page Up/Down   - Navigate by page"),
        Line::from("  Enter          - Select codepoint in editor"),
        Line::from("  /              - Search codepoints"),
        Line::from("  Esc            - Exit search"),
        Line::from(""),
        Line::from(vec![
            Span::styled("AI Tab:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("  Space          - Pause/Resume Game of Life"),
        Line::from("  R              - Reset with new random state"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("  Use number keys 1-9 to quickly jump between tabs"),
        Line::from("  Tab/Shift+Tab to cycle through tabs"),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}