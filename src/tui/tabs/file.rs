use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::sync::mpsc;

use crate::tui::communication::TuiMessage;

#[derive(Debug, Clone)]
pub struct FileState {
    pub selected_index: usize,
    pub file_actions: Vec<crate::tui::communication::FileAction>,
}

impl Default for FileState {
    fn default() -> Self {
        Self::new()
    }
}

impl FileState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            file_actions: Vec::new(),
        }
    }

    pub fn add_file_action(&mut self, action: crate::tui::communication::FileAction) {
        self.file_actions.push(action);
        if self.file_actions.len() > 10 {
            self.file_actions.remove(0);
        }
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
pub fn draw(f: &mut Frame, state: &mut FileState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Min(0)])
        .split(area);

    let mut action_lines = vec![Line::from("")];

    if state.file_actions.is_empty() {
        action_lines.push(Line::from(vec![Span::styled(
            "  No file actions yet",
            Style::default().fg(Color::DarkGray),
        )]));
        action_lines.push(Line::from(""));
        action_lines.push(Line::from(vec![Span::styled(
            "  (Save to see actions here)",
            Style::default().fg(Color::DarkGray),
        )]));
    } else {
        for action in state.file_actions.iter().rev().take(8) {
            action_lines.push(Line::from(vec![Span::styled(
                format!("  {}", action.action),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]));

            action_lines.push(Line::from(vec![Span::styled(
                format!("    {}", action.timestamp),
                Style::default().fg(Color::Yellow),
            )]));

            if let Some(path) = &action.path {
                action_lines.push(Line::from(vec![Span::styled(
                    format!("    {}", path),
                    Style::default().fg(Color::Gray),
                )]));
            }

            action_lines.push(Line::from(""));
        }
    }

    let action_log = Paragraph::new(action_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "File Actions",
                Style::default().fg(Color::Green),
            )),
    );

    f.render_widget(action_log, chunks[0]);

    let file_menu = vec![
        Line::from(""),
        Line::from("  Ctrl+S         - Save current font"),
        Line::from(""),
    ];

    let paragraph = Paragraph::new(file_menu).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "File Operations",
                Style::default().fg(Color::Green),
            )),
    );

    f.render_widget(paragraph, chunks[1]);
}
