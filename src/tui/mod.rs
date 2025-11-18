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
use std::sync::atomic::{AtomicBool, Ordering};

pub async fn run_tui(
    _cli_args: Arc<CliArgs>,
    app_tx: mpsc::UnboundedSender<TuiMessage>,
    mut app_rx: mpsc::UnboundedReceiver<AppMessage>,
    tui_ready: Arc<AtomicBool>,
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

    tui_ready.store(true, Ordering::Release);

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
    use std::sync::atomic::{AtomicBool, Ordering};

    #[cfg(target_os = "macos")]
    {
        std::env::set_var("OS_ACTIVITY_MODE", "disable");
    }

    if let Err(e) = crate::logging::setup_file_logging_for_tui() {
        eprintln!("Warning: Failed to set up file logging: {}", e);
    }

    let (tui_tx, tui_rx) = mpsc::unbounded_channel();
    let (app_tx, app_rx) = mpsc::unbounded_channel();

    let cli_args_arc = Arc::new(cli_args);

    let cli_args_tui = cli_args_arc.clone();
    let tui_ready = Arc::new(AtomicBool::new(false));
    let tui_ready_clone = tui_ready.clone();

    let tui_handle = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
            use tracing::error;
            error!("Fatal error: Failed to create tokio runtime for TUI: {}", e);
            std::process::exit(1);
        });
        rt.block_on(async {
            if let Err(e) = run_tui(cli_args_tui, tui_tx, app_rx, tui_ready_clone).await {
                use tracing::error;
                error!("TUI error: {}", e);
            }
        });
    });

    // Wait for TUI to enter alternate screen mode before redirecting stderr
    // This prevents system library output from corrupting the TUI display
    while !tui_ready.load(Ordering::Acquire) {
        thread::sleep(std::time::Duration::from_millis(10));
    }

    // Now safe to redirect stderr - TUI is using alternate screen buffer
    if let Err(e) = crate::logging::redirect_stderr_to_log() {
        use tracing::warn;
        warn!("Failed to redirect stderr to log file: {}", e);
    }

    let app_result = match crate::core::app::create_app_with_tui((*cli_args_arc).clone(), tui_rx, app_tx) {
        Ok(mut app) => {
            app.run();
            Ok(())
        }
        Err(e) => Err(e),
    };

    let _ = tui_handle.join();

    if let Err(e) = app_result {
        eprintln!("Failed to create Bevy app: {}", e);
        return Err(e);
    }

    Ok(())
}
