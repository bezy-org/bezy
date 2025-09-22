use crate::tui::communication::TuiMessage;
use anyhow::Result;
use crossterm::event::KeyEvent;
use tokio::sync::mpsc;

pub mod glyphs;
pub mod font_info;
pub mod logs;
pub mod help;

#[derive(Debug, Clone, PartialEq)]
pub enum TabType {
    Glyphs,
    FontInfo,
    Logs,
    Help,
}

impl TabType {
    pub fn title(&self) -> &'static str {
        match self {
            TabType::Glyphs => "Glyphs",
            TabType::FontInfo => "Font Info",
            TabType::Logs => "Logs",
            TabType::Help => "Help",
        }
    }
}

pub struct Tab {
    pub tab_type: TabType,
    pub state: TabState,
}

#[derive(Debug, Clone)]
pub enum TabState {
    Glyphs(glyphs::GlyphsState),
    FontInfo,
    Logs(logs::LogsState),
    Help,
}

impl Tab {
    pub fn new(tab_type: TabType) -> Self {
        let state = match tab_type {
            TabType::Glyphs => TabState::Glyphs(glyphs::GlyphsState::new()),
            TabType::FontInfo => TabState::FontInfo,
            TabType::Logs => TabState::Logs(logs::LogsState::new()),
            TabType::Help => TabState::Help,
        };

        Self { tab_type, state }
    }

    pub async fn handle_key_event(
        &mut self,
        key: KeyEvent,
        app_tx: &mpsc::UnboundedSender<TuiMessage>,
        app: &crate::tui::app::App,
    ) -> Result<()> {
        match &mut self.state {
            TabState::Glyphs(state) => {
                glyphs::handle_key_event(state, key, app_tx, app).await
            }
            TabState::Logs(state) => {
                logs::handle_key_event(state, key, app_tx, app).await
            }
            _ => Ok(()),
        }
    }
}