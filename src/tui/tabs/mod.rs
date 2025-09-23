use crate::tui::communication::TuiMessage;
use anyhow::Result;
use crossterm::event::KeyEvent;
use tokio::sync::mpsc;

pub mod ai;
pub mod edit;
pub mod file;
pub mod font_info;
pub mod game_of_life;
pub mod glyph;
pub mod help;
pub mod logs;
pub mod path;
pub mod qa;
pub mod unicode;

#[derive(Debug, Clone, PartialEq)]
pub enum TabType {
    File,
    Edit,
    Unicode, // renamed from Codepoints
    FontInfo,
    QA,
    Glyph,
    Path,
    AI,
    Help,
}

impl TabType {
    pub fn title(&self) -> &'static str {
        match self {
            TabType::File => "File",
            TabType::Edit => "Edit",
            TabType::Unicode => "Unicode",
            TabType::FontInfo => "Font Info",
            TabType::QA => "QA",
            TabType::Glyph => "Glyph",
            TabType::Path => "Path",
            TabType::AI => "AI",
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
    File(file::FileState),
    Edit(edit::EditState),
    Unicode(unicode::GlyphsState),
    FontInfo(font_info::FontInfoState),
    QA(qa::QAState),
    Glyph(glyph::GlyphState),
    Path(path::PathState),
    AI(ai::AIState),
    Help(help::HelpState),
}

impl Tab {
    pub fn new(tab_type: TabType) -> Self {
        let state = match tab_type {
            TabType::File => TabState::File(file::FileState::new()),
            TabType::Edit => TabState::Edit(edit::EditState::new()),
            TabType::Unicode => TabState::Unicode(unicode::GlyphsState::new()),
            TabType::FontInfo => TabState::FontInfo(font_info::FontInfoState::new()),
            TabType::QA => {
                let mut qa_state = qa::QAState::new();
                qa_state.load_demo_data(); // Load demo data for prototype
                TabState::QA(qa_state)
            },
            TabType::Glyph => TabState::Glyph(glyph::GlyphState::new()),
            TabType::Path => TabState::Path(path::PathState::new()),
            TabType::AI => TabState::AI(ai::AIState::new()),
            TabType::Help => TabState::Help(help::HelpState::new()),
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
            TabState::Unicode(state) => {
                unicode::handle_key_event(state, key, app_tx, app).await
            }
            TabState::File(state) => file::handle_key_event(state, key, app_tx).await,
            TabState::Edit(state) => edit::handle_key_event(state, key, app_tx).await,
            TabState::FontInfo(state) => font_info::handle_key_event(state, key, app_tx).await,
            TabState::QA(state) => qa::handle_key_event(state, key, app_tx).await,
            TabState::Glyph(state) => glyph::handle_key_event(state, key, app_tx).await,
            TabState::Path(state) => path::handle_key_event(state, key, app_tx).await,
            TabState::AI(state) => ai::handle_key_event(state, key, app_tx).await,
            TabState::Help(state) => help::handle_key_event(state, key, app_tx).await,
        }
    }
}