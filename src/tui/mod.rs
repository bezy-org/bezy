pub mod app;
pub mod communication;
pub mod events;
pub mod message_handler;
pub mod tabs;
pub mod ui;

use crate::core::config::CliArgs;
use anyhow::Result;
use communication::{AppMessage, TuiMessage};
use std::sync::Arc;
use std::thread;
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
    use ratatui::{backend::CrosstermBackend, Terminal};
    use std::io;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // NOTE: We cannot redirect stdout/stderr when TUI is active
    // The TUI needs stdout for terminal control
    // Bevy logging must be configured to write directly to files (not via stdout)

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

/// Run the application with TUI enabled (both GUI and TUI simultaneously)
pub fn run_app_with_tui(cli_args: CliArgs) -> Result<()> {
    // Create communication channels
    let (tui_tx, tui_rx) = mpsc::unbounded_channel();
    let (app_tx, app_rx) = mpsc::unbounded_channel();

    let cli_args_arc = Arc::new(cli_args);

    // Clone for the TUI thread
    let cli_args_tui = cli_args_arc.clone();

    // Spawn TUI in a separate thread - it can handle terminal I/O from background
    let tui_handle = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
            eprintln!("Fatal error: Failed to create tokio runtime for TUI: {}", e);
            std::process::exit(1);
        });
        rt.block_on(async {
            if let Err(e) = run_tui(cli_args_tui, tui_tx, app_rx).await {
                eprintln!("TUI error: {}", e);
            }
        });
    });

    // Run Bevy app in the main thread (needed for proper window initialization on Linux)
    // The TUI-specific configuration will disable console logging to prevent terminal corruption
    let app_result = match crate::core::app::create_app_with_tui((*cli_args_arc).clone(), tui_rx, app_tx) {
        Ok(mut app) => {
            app.run();
            Ok(())
        }
        Err(e) => Err(e),
    };

    // Wait for TUI thread to finish cleanup before any error output
    let _ = tui_handle.join();

    // Now safe to output errors after TUI has cleaned up
    if let Err(e) = app_result {
        eprintln!("Failed to create Bevy app: {}", e);
        return Err(e);
    }

    Ok(())
}
