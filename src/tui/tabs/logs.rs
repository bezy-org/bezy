use crate::tui::communication::TuiMessage;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct LogsState {
    pub scroll_offset: usize,
    pub auto_scroll: bool,
}

impl Default for LogsState {
    fn default() -> Self {
        Self::new()
    }
}

impl LogsState {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            auto_scroll: true,
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            self.auto_scroll = false;
        }
    }

    pub fn scroll_down(&mut self, max_lines: usize, visible_lines: usize) {
        if self.scroll_offset + visible_lines < max_lines {
            self.scroll_offset += 1;
        } else {
            self.auto_scroll = true;
        }
    }

    pub fn update_auto_scroll(&mut self, max_lines: usize, visible_lines: usize) {
        if self.auto_scroll && max_lines > visible_lines {
            self.scroll_offset = max_lines - visible_lines;
        }
    }
}

pub async fn handle_key_event(
    state: &mut LogsState,
    key: KeyEvent,
    _app_tx: &mpsc::UnboundedSender<TuiMessage>,
    app: &crate::tui::app::App,
) -> Result<()> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.scroll_up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.scroll_down(app.logs.len(), 20);
        }
        KeyCode::Home => {
            state.scroll_offset = 0;
            state.auto_scroll = false;
        }
        KeyCode::End => {
            state.auto_scroll = true;
        }
        _ => {}
    }
    Ok(())
}

pub async fn handle_key_event_simple(
    state: &mut LogsState,
    key: KeyEvent,
    _app_tx: &mpsc::UnboundedSender<TuiMessage>,
    logs_len: usize,
) -> Result<()> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.scroll_up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.scroll_down(logs_len, 20);
        }
        KeyCode::Home => {
            state.scroll_offset = 0;
            state.auto_scroll = false;
        }
        KeyCode::End => {
            state.auto_scroll = true;
        }
        _ => {}
    }
    Ok(())
}
