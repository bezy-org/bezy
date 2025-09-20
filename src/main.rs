//! A font editor built with Rust, the Bevy game engine, and Linebender crates.
//!
//! The enjoyment of one's tools is an essential ingredient of successful work.
//! — Donald Knuth

use anyhow::Result;
use bezy::core;

/// Create and run the application with the given CLI arguments.
fn run_app(cli_args: core::cli::CliArgs) -> Result<()> {
    let mut app = core::app::create_app(cli_args)?;
    app.run();
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
