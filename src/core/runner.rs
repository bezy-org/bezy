//! Application runner logic
//!
//! Handles the different ways to run the Bezy application

use crate::core::config::{CliArgs, ConfigFile};
// use crate::logging;  // Not currently used
use anyhow::Result;

/// Create and run the application with the given CLI arguments.
/// Handles special CLI flags and delegates to appropriate runners.
pub fn run_app(cli_args: CliArgs) -> Result<()> {
    // Handle --new-config flag specially
    if cli_args.new_config {
        match ConfigFile::initialize_config_directory() {
            Ok(()) => {
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Failed to initialize config directory: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Run the main application
    if cli_args.no_tui {
        // Only redirect logs when NOT using TUI (for debugging)
        // When TUI is enabled, redirection happens AFTER TUI captures terminal
        let mut app = crate::core::app::create_app(cli_args)?;
        app.run();
        Ok(())
    } else {
        #[cfg(feature = "tui")]
        {
            crate::tui::run_app_with_tui(cli_args)
        }
        #[cfg(not(feature = "tui"))]
        {
            eprintln!("TUI feature not compiled. Use --no-tui flag to run without TUI.");
            std::process::exit(1);
        }
    }
}
