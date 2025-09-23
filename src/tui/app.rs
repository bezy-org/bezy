use crate::tui::{
    communication::{AppMessage, TuiMessage, FontInfo, GlyphInfo},
    events::{handle_events, InputEvent},
    tabs::{unicode, Tab, TabState, TabType},
    ui,
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{backend::Backend, Terminal};
use tokio::sync::mpsc;

pub struct App {
    pub tabs: Vec<Tab>,
    pub current_tab: usize,
    pub app_tx: mpsc::UnboundedSender<TuiMessage>,
    pub font_info: Option<FontInfo>,
    pub glyphs: Vec<GlyphInfo>,
    pub current_glyph: Option<String>,
    pub logs: Vec<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new(app_tx: mpsc::UnboundedSender<TuiMessage>) -> Self {
        let tabs = vec![
            Tab::new(TabType::File),      // 1
            Tab::new(TabType::Edit),      // 2
            Tab::new(TabType::Unicode),   // 3 (renamed from Codepoints)
            Tab::new(TabType::FontInfo),  // 4
            Tab::new(TabType::QA),        // 5
            Tab::new(TabType::Glyph),     // 6
            Tab::new(TabType::Path),      // 7
            Tab::new(TabType::AI),        // 8
            Tab::new(TabType::Help),      // 9
        ];

        Self {
            tabs,
            current_tab: 0, // Start on File tab
            app_tx,
            font_info: None,
            glyphs: Vec::new(),
            current_glyph: None,
            logs: Vec::new(),
            should_quit: false,
        }
    }

    pub async fn run<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        app_rx: &mut mpsc::UnboundedReceiver<AppMessage>,
    ) -> Result<()> {
        let (input_tx, mut input_rx) = mpsc::unbounded_channel();

        // Spawn event handler
        tokio::spawn(handle_events(input_tx));

        // Request initial data
        let _ = self.app_tx.send(TuiMessage::RequestFontInfo);
        let _ = self.app_tx.send(TuiMessage::RequestGlyphList);

        loop {
            terminal.draw(|f| ui::draw(f, self))?;

            tokio::select! {
                // Handle input events
                Some(input_event) = input_rx.recv() => {
                    match input_event {
                        InputEvent::Key(key) => {
                            self.handle_key_event(key).await?;
                        }
                        InputEvent::Resize(_, _) => {
                            // Terminal will automatically handle resize
                        }
                        InputEvent::Quit => {
                            self.should_quit = true;
                        }
                    }
                }

                // Handle app messages
                Some(app_message) = app_rx.recv() => {
                    self.handle_app_message(app_message).await?;
                }

                // Add timeout to continuously redraw for Game of Life
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                    // This ensures the UI redraws regularly for animations like Game of Life
                }
            }

            if self.should_quit {
                let _ = self.app_tx.send(TuiMessage::Quit);
                break;
            }
        }

        Ok(())
    }

    async fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            (KeyCode::Tab, _) => {
                self.next_tab();
            }
            (KeyCode::BackTab, _) => {
                self.previous_tab();
            }
            (KeyCode::Char(c), _) if c.is_ascii_digit() => {
                if let Some(digit) = c.to_digit(10) {
                    let tab_index = (digit as usize).saturating_sub(1);
                    if tab_index < self.tabs.len() {
                        self.current_tab = tab_index;
                    }
                }
            }
            _ => {
                // Forward key to current tab with context
                let current_tab_idx = self.current_tab;
                let app_tx = self.app_tx.clone();
                let glyphs_len = self.glyphs.len();

                if let Some(tab) = self.tabs.get_mut(current_tab_idx) {
                    match &mut tab.state {
                        TabState::Unicode(state) => {
                            unicode::handle_key_event_simple(state, key, &app_tx, glyphs_len, &self.glyphs).await?;
                        }
                        TabState::AI(state) => {
                            // Handle Game of Life controls
                            match key.code {
                                crossterm::event::KeyCode::Char(' ') => {
                                    state.game.toggle_pause();
                                }
                                crossterm::event::KeyCode::Char('r') | crossterm::event::KeyCode::Char('R') => {
                                    state.game.reset();
                                }
                                _ => {}
                            }
                        }
                        TabState::QA(state) => {
                            crate::tui::tabs::qa::handle_key_event(state, key, &app_tx).await?;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_app_message(&mut self, message: AppMessage) -> Result<()> {
        match message {
            AppMessage::FontInfo(info) => {
                self.font_info = Some(info.clone());
                // Update the FontInfo tab state
                for tab in &mut self.tabs {
                    if let TabState::FontInfo(ref mut state) = tab.state {
                        state.update(info.clone());
                    }
                }
            }
            AppMessage::GlyphList(glyphs) => {
                self.glyphs = glyphs;
            }
            AppMessage::CurrentGlyph(glyph) => {
                self.current_glyph = Some(glyph);
            }
            AppMessage::LogLine(line) => {
                self.logs.push(line);
                // Keep only last 1000 log lines
                if self.logs.len() > 1000 {
                    self.logs.drain(0..self.logs.len() - 1000);
                }
            }
            AppMessage::FontLoaded(path) => {
                self.logs.push(format!("Font loaded: {}", path));
            }
            AppMessage::Error(error) => {
                self.logs.push(format!("Error: {}", error));
            }
        }
        Ok(())
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % self.tabs.len();
    }

    pub fn previous_tab(&mut self) {
        if self.current_tab > 0 {
            self.current_tab -= 1;
        } else {
            self.current_tab = self.tabs.len() - 1;
        }
    }

    pub fn get_current_tab(&self) -> &Tab {
        &self.tabs[self.current_tab]
    }

    pub fn get_current_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.current_tab]
    }

}