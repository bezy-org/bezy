#[cfg(feature = "tui")]
use crate::tui::communication::{AppMessage, FontInfo, GlyphInfo, TuiMessage};
use bevy::prelude::*;
#[cfg(feature = "tui")]
use tokio::sync::mpsc;

#[cfg(feature = "tui")]
#[derive(Resource)]
pub struct TuiCommunication {
    pub tui_rx: mpsc::UnboundedReceiver<TuiMessage>,
    pub app_tx: mpsc::UnboundedSender<AppMessage>,
}

#[cfg(feature = "tui")]
impl TuiCommunication {
    pub fn new(
        tui_rx: mpsc::UnboundedReceiver<TuiMessage>,
        app_tx: mpsc::UnboundedSender<AppMessage>,
    ) -> Self {
        Self { tui_rx, app_tx }
    }

    pub fn try_recv(&mut self) -> Option<TuiMessage> {
        self.tui_rx.try_recv().ok()
    }

    pub fn send(&self, message: AppMessage) -> Result<(), mpsc::error::SendError<AppMessage>> {
        self.app_tx.send(message)
    }

    pub fn send_glyph_list(&self, glyphs: Vec<GlyphInfo>) {
        let _ = self.send(AppMessage::GlyphList(glyphs));
    }

    pub fn send_font_info(&self, info: FontInfo) {
        let _ = self.send(AppMessage::FontInfo(info));
    }

    pub fn send_current_glyph(&self, glyph: String) {
        let _ = self.send(AppMessage::CurrentGlyph(glyph));
    }

    pub fn send_log(&self, message: String) {
        let _ = self.send(AppMessage::LogLine(message));
    }
}
