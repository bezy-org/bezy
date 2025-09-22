pub mod app;
pub mod communication;
pub mod events;
pub mod message_handler;
pub mod tabs;
pub mod ui;

use crate::core::cli::CliArgs;
use anyhow::Result;
use communication::{AppMessage, TuiMessage};
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn run_tui(
    _cli_args: Arc<CliArgs>,
    app_tx: mpsc::UnboundedSender<TuiMessage>,
    mut app_rx: mpsc::UnboundedReceiver<AppMessage>,
) -> Result<()> {
    use crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{
        backend::CrosstermBackend,
        Terminal,
    };
    use std::io;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = app::App::new(app_tx.clone());
    let result = app.run(&mut terminal, &mut app_rx).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}