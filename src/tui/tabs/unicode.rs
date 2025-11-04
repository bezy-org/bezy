use crate::tui::communication::{GlyphInfo, TuiMessage};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct GlyphsState {
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub search_query: String,
    pub is_searching: bool,
}

impl Default for GlyphsState {
    fn default() -> Self {
        Self::new()
    }
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
                // Send message to select current glyph using Unicode codepoint
                if let Some(glyph) = app.glyphs.get(state.selected_index) {
                    if let Some(unicode) = glyph.unicode {
                        let _ = app_tx.send(TuiMessage::SelectGlyph(unicode));
                    }
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
                // Send message to select current glyph using Unicode codepoint
                if let Some(glyph) = glyphs.get(state.selected_index) {
                    if let Some(unicode) = glyph.unicode {
                        let _ = app_tx.send(TuiMessage::SelectGlyph(unicode));
                    }
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

/// Draw the Unicode tab UI
pub fn draw(f: &mut Frame, glyphs: &[GlyphInfo], state: &mut GlyphsState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(area);

    // Filter glyphs based on search query
    let filtered_glyphs: Vec<(usize, &GlyphInfo)> =
        if state.is_searching && !state.search_query.is_empty() {
            glyphs
                .iter()
                .enumerate()
                .filter(|(_, g)| {
                    let name = g.name.as_deref().unwrap_or(&g.codepoint);
                    let unicode_str = g
                        .unicode
                        .map(|u| format!("U+{:04X}", u))
                        .unwrap_or_default();
                    name.to_lowercase()
                        .contains(&state.search_query.to_lowercase())
                        || unicode_str
                            .to_lowercase()
                            .contains(&state.search_query.to_lowercase())
                })
                .collect()
        } else {
            glyphs.iter().enumerate().collect()
        };

    // Create list items
    let items: Vec<ListItem> = filtered_glyphs
        .iter()
        .map(|(_, glyph)| {
            let name = glyph.name.as_deref().unwrap_or(&glyph.codepoint);
            let unicode = glyph
                .unicode
                .map(|u| format!("U+{:04X}", u))
                .unwrap_or_else(|| "U+0000".to_string());
            let line = format!("{} {}", unicode, name);
            ListItem::new(Line::from(line))
        })
        .collect();

    // Update scroll based on selection
    let visible_height = chunks[0].height.saturating_sub(2) as usize;
    state.update_scroll(visible_height);

    // Calculate the selected index in the filtered list
    let filtered_selected = if state.is_searching {
        // Find the index in the filtered list
        filtered_glyphs
            .iter()
            .position(|(orig_idx, _)| *orig_idx == state.selected_index)
            .unwrap_or(0)
    } else {
        state.selected_index
    };

    // Create the list widget
    let _list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled("Unicode", Style::default().fg(Color::Green))),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    // Render the list with scroll offset
    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(filtered_selected.saturating_sub(state.scroll_offset)));

    // Only show the visible portion
    let visible_items: Vec<ListItem> = filtered_glyphs
        .iter()
        .skip(state.scroll_offset)
        .take(visible_height)
        .map(|(_, glyph)| {
            let name = glyph.name.as_deref().unwrap_or(&glyph.codepoint);
            let unicode = glyph
                .unicode
                .map(|u| format!("U+{:04X}", u))
                .unwrap_or_else(|| "U+0000".to_string());
            let line = format!("{} {}", unicode, name);
            ListItem::new(Line::from(line))
        })
        .collect();

    let visible_list = List::new(visible_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled("Unicode", Style::default().fg(Color::Green))),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(visible_list, chunks[0], &mut list_state);

    // Controls/status area
    let controls_text = if state.is_searching {
        format!(
            "Search: {} | Press Esc to cancel, Enter to confirm",
            state.search_query
        )
    } else {
        let selected_glyph = filtered_glyphs
            .get(filtered_selected)
            .and_then(|(_, g)| g.name.as_deref())
            .unwrap_or("None");
        format!(
            "Selected: {} | Use ↑↓ or j/k to navigate, Enter to select, / to search",
            selected_glyph
        )
    };

    let controls = Paragraph::new(controls_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled("Controls", Style::default().fg(Color::Green))),
    );

    f.render_widget(controls, chunks[1]);
}
