use crate::tui::communication::TuiMessage;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct GlyphsState {
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub search_query: String,
    pub is_searching: bool,
}

impl GlyphsState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            scroll_offset: 0,
            search_query: String::new(),
            is_searching: false,
        }
    }

    pub fn select_next(&mut self, max_items: usize) {
        if max_items > 0 {
            self.selected_index = (self.selected_index + 1).min(max_items - 1);
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn page_down(&mut self, max_items: usize, page_size: usize) {
        if max_items > 0 {
            self.selected_index = (self.selected_index + page_size).min(max_items - 1);
        }
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.selected_index = self.selected_index.saturating_sub(page_size);
    }

    pub fn update_scroll(&mut self, visible_items: usize) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_items {
            self.scroll_offset = self.selected_index - visible_items + 1;
        }
    }
}

pub async fn handle_key_event(
    state: &mut GlyphsState,
    key: KeyEvent,
    app_tx: &mpsc::UnboundedSender<TuiMessage>,
    app: &crate::tui::app::App,
) -> Result<()> {
    if state.is_searching {
        match key.code {
            KeyCode::Esc => {
                state.is_searching = false;
                state.search_query.clear();
            }
            KeyCode::Enter => {
                state.is_searching = false;
            }
            KeyCode::Backspace => {
                state.search_query.pop();
            }
            KeyCode::Char(c) => {
                state.search_query.push(c);
            }
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                state.select_next(app.glyphs.len());
            }
            KeyCode::Up | KeyCode::Char('k') => {
                state.select_previous();
            }
            KeyCode::PageDown => {
                state.page_down(app.glyphs.len(), 10);
            }
            KeyCode::PageUp => {
                state.page_up(10);
            }
            KeyCode::Enter => {
                // Send message to select current glyph
                if let Some(glyph) = app.glyphs.get(state.selected_index) {
                    let _ = app_tx.send(TuiMessage::SelectGlyph(glyph.codepoint.clone()));
                }
            }
            KeyCode::Char('/') => {
                state.is_searching = true;
                state.search_query.clear();
            }
            _ => {}
        }
    }
    Ok(())
}

pub async fn handle_key_event_simple(
    state: &mut GlyphsState,
    key: KeyEvent,
    app_tx: &mpsc::UnboundedSender<TuiMessage>,
    glyphs_len: usize,
    glyphs: &[crate::tui::communication::GlyphInfo],
) -> Result<()> {
    if state.is_searching {
        match key.code {
            KeyCode::Esc => {
                state.is_searching = false;
                state.search_query.clear();
            }
            KeyCode::Enter => {
                state.is_searching = false;
            }
            KeyCode::Backspace => {
                state.search_query.pop();
            }
            KeyCode::Char(c) => {
                state.search_query.push(c);
            }
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                state.select_next(glyphs_len);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                state.select_previous();
            }
            KeyCode::PageDown => {
                state.page_down(glyphs_len, 10);
            }
            KeyCode::PageUp => {
                state.page_up(10);
            }
            KeyCode::Enter => {
                // Send message to select current glyph
                if let Some(glyph) = glyphs.get(state.selected_index) {
                    let _ = app_tx.send(TuiMessage::SelectGlyph(glyph.codepoint.clone()));
                }
            }
            KeyCode::Char('/') => {
                state.is_searching = true;
                state.search_query.clear();
            }
            _ => {}
        }
    }
    Ok(())
}