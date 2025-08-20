//! A font editor built with Rust, the Bevy game engine, and Linebender crates.
//!
//! The enjoyment of one's tools is an essential ingredient of successful work.
//! — Donald Knuth

use anyhow::Result;
use bezy::core;
use clap::Parser;
use std::process;

/// Create and run the application with the given CLI arguments.
fn run_app(cli_args: core::cli::CliArgs) -> Result<()> {
    let mut app = core::app::create_app(cli_args)?;
    app.run();
    Ok(())
}

/// Better error reporting for WebAssembly builds.
fn init_panic_handling() {
    // Only compile this code when building for WebAssembly.
    #[cfg(target_arch = "wasm32")]
    {
        // Make Rust panics show up in the browser's developer console.
        console_error_panic_hook::set_once();
    }
}

/// Handle application errors with platform-appropriate logging.
fn handle_error(error: anyhow::Error) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!();
        eprintln!("Error starting Bezy:");
        eprintln!("{error}");
        eprintln!();
        eprintln!("Try running with --help for usage information.");
        eprintln!("Or visit: https://bezy.org");
        process::exit(1);
    }
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::error_1(
            &format!("Error starting Bezy: {}", error).into(),
        );
    }
}

fn main() {
    init_panic_handling();
    let cli_args = {
        #[cfg(not(target_arch = "wasm32"))]
        {
            core::cli::CliArgs::parse()
        }
        #[cfg(target_arch = "wasm32")]
        {
            core::cli::CliArgs::default_for_web()
        }
    };

    // Create and run the application
    match run_app(cli_args) {
        Ok(()) => {}
        Err(error) => handle_error(error),
    }
}
