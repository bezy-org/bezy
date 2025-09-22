//! A font editor built with Rust, the Bevy game engine, and Linebender crates.
//!
//! The enjoyment of one's tools is an essential ingredient of successful work.
//! — Donald Knuth

use anyhow::Result;
use bezy::core;
use std::sync::Arc;
use std::thread;
use tokio::sync::mpsc;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

/// Set up log redirection to ~/.config/bezy/logs/
fn setup_log_redirection() -> Result<()> {
    use bezy::core::config_file::ConfigFile;

    // Initialize logs directory
    ConfigFile::initialize_logs_directory()?;

    // Get the log file path
    let log_file_path = ConfigFile::current_log_file();

    // Create/open the log file
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)?;

    // Redirect stdout and stderr to the log file
    unsafe {
        libc::dup2(log_file.as_raw_fd(), libc::STDOUT_FILENO);
        libc::dup2(log_file.as_raw_fd(), libc::STDERR_FILENO);
    }

    // Print initial log message to confirm redirection
    println!("=== Bezy started at {} ===", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Logs redirected to: {:?}", log_file_path);

    Ok(())
}

/// Create and run the application with the given CLI arguments.
fn run_app(cli_args: core::cli::CliArgs) -> Result<()> {
    if cli_args.no_tui {
        // Only set up log redirection when TUI is disabled
        if let Err(e) = setup_log_redirection() {
            eprintln!("Failed to setup log redirection: {}", e);
        }
        let mut app = core::app::create_app(cli_args)?;
        app.run();
        Ok(())
    } else {
        // When TUI is enabled, skip log redirection to allow TUI terminal control
        run_app_with_tui(cli_args)
    }
}

/// Run the application with TUI enabled (both GUI and TUI simultaneously)
fn run_app_with_tui(cli_args: core::cli::CliArgs) -> Result<()> {
    // Create communication channels
    let (tui_tx, tui_rx) = mpsc::unbounded_channel();
    let (app_tx, app_rx) = mpsc::unbounded_channel();

    let cli_args_arc = Arc::new(cli_args);

    // Clone for the TUI thread
    let cli_args_tui = cli_args_arc.clone();

    // Spawn TUI in a separate thread - it can handle terminal I/O from background
    let tui_handle = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            if let Err(e) = bezy::tui::run_tui(cli_args_tui, tui_tx, app_rx).await {
                eprintln!("TUI error: {}", e);
            }
        });
    });

    // Run Bevy app in the main thread (needed for proper window initialization on Linux)
    match core::app::create_app_with_tui((*cli_args_arc).clone(), tui_rx, app_tx) {
        Ok(mut app) => {
            app.run();
        },
        Err(e) => {
            eprintln!("Failed to create Bevy app: {}", e);
            return Err(e);
        },
    }

    // Wait for TUI thread to finish
    let _ = tui_handle.join();

    Ok(())
}

fn main() {
    core::platform::init_panic_handling();
    let cli_args = core::platform::get_cli_args();

    // Handle --new-config flag specially
    if cli_args.new_config {
        match core::config_file::ConfigFile::initialize_config_directory() {
            Ok(()) => {
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Failed to initialize config directory: {}", e);
                std::process::exit(1);
            }
        }
    }

    match run_app(cli_args) {
        Ok(()) => {}
        Err(error) => core::platform::handle_error(error),
    }
}
