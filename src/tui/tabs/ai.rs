use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::sync::mpsc;

use crate::tui::communication::TuiMessage;
pub use super::game_of_life::GameOfLifeState;

#[derive(Debug, Clone)]
pub struct AIState {
    pub game: GameOfLifeState,
}

impl AIState {
    pub fn new() -> Self {
        Self {
            game: GameOfLifeState::new(80, 40),
        }
    }
}

/// Handle key events for the AI tab
pub async fn handle_key_event(
    state: &mut AIState,
    key: KeyEvent,
    _app_tx: &mpsc::UnboundedSender<TuiMessage>,
) -> Result<()> {
    match key.code {
        KeyCode::Char(' ') => {
            state.game.toggle_pause();
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            state.game.reset();
        }
        _ => {}
    }
    Ok(())
}

/// Draw the AI tab UI
pub fn draw(f: &mut Frame, state: &mut AIState, area: Rect) {
    // Update the game state
    state.game.update();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(area);

    // Calculate cell size based on available space
    let game_area = chunks[0];
    let available_width = game_area.width.saturating_sub(2) as usize;
    let available_height = game_area.height.saturating_sub(2) as usize;

    // Adjust game size to fit terminal if needed
    state.game.set_size(
        available_width.min(state.game.width),
        available_height.min(state.game.height),
    );

    // Create the grid display
    let mut grid_lines = Vec::new();
    for row in &state.game.grid {
        let mut line = String::new();
        for &cell in row {
            line.push(if cell { '█' } else { ' ' });
        }
        grid_lines.push(Line::from(line));
    }

    let game_widget = Paragraph::new(grid_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(
                "Conway's Game of Life - Generation: {} {}",
                state.game.generation,
                if state.game.paused { "(PAUSED)" } else { "" }
            )),
    );

    f.render_widget(game_widget, chunks[0]);

    // Controls info
    let controls = Paragraph::new(vec![Line::from(
        "Space: Pause/Resume | R: Reset | Game auto-updates 8 times per second",
    )])
    .block(Block::default().borders(Borders::ALL).title("Controls"));

    f.render_widget(controls, chunks[1]);
}