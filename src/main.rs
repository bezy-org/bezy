//! A font editor built with Rust, the Bevy game engine, and Linebender crates.
//!
//! The enjoyment of one's tools is an essential ingredient of successful work.
//! — Donald Knuth

use anyhow::Result;
use bezy::core;
use std::sync::Arc;
use std::thread;
use tokio::sync::mpsc;

/// Create and run the application with the given CLI arguments.
fn run_app(cli_args: core::cli::CliArgs) -> Result<()> {
    if cli_args.no_tui {
        let mut app = core::app::create_app(cli_args)?;
        app.run();
        Ok(())
    } else {
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

    // Spawn TUI in a separate thread
    let tui_handle = thread::spawn(move || {
        // Create a new tokio runtime for the TUI thread
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            if let Err(e) = bezy::tui::run_tui(cli_args_tui, tui_tx, app_rx).await {
                eprintln!("TUI error: {}", e);
            }
        });
    });

    // Create and run the Bevy app in the main thread
    let mut app = core::app::create_app_with_tui((*cli_args_arc).clone(), tui_rx, app_tx)?;
    app.run();

    // Wait for TUI thread to finish (happens when app exits)
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
