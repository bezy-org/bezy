//! Platform-specific functionality and error handling.
//!
//! This module provides platform abstractions for initialization,
//! error handling, and platform-specific configurations.

/// Initialize platform-specific panic handling.
///
/// For WebAssembly builds, this sets up console error reporting
/// so that Rust panics appear in the browser's developer console.
pub fn init_panic_handling() {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
    }
}

/// Handle application errors with platform-appropriate logging.
///
/// - On native platforms: Prints to stderr and exits with code 1
/// - On WebAssembly: Logs to the browser console
pub fn handle_error(error: anyhow::Error) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!();
        eprintln!("Error starting Bezy:");
        eprintln!("{error}");
        eprintln!();
        eprintln!("Try running with --help for usage information.");
        eprintln!("Or visit: https://bezy.org");
        std::process::exit(1);
    }
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::error_1(&format!("Error starting Bezy: {}", error).into());
    }
}

/// Get CLI arguments based on the platform.
///
/// - On native platforms: Parses command line arguments
/// - On WebAssembly: Returns default arguments for web
pub fn get_cli_args() -> crate::core::cli::CliArgs {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use clap::Parser;
        crate::core::cli::CliArgs::parse()
    }
    #[cfg(target_arch = "wasm32")]
    {
        crate::core::cli::CliArgs::default_for_web()
    }
}
