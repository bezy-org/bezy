//! Application logging functionality
//!
//! Handles log file management and output redirection

use std::fs;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

/// Get the path to the bezy config directory
fn config_dir() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));
    config_dir.join("bezy")
}

/// Get the path to the logs directory
pub fn logs_dir() -> PathBuf {
    config_dir().join("logs")
}

/// Get the path to the current log file
pub fn current_log_file() -> PathBuf {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d");
    logs_dir().join(format!("bezy-{}.log", timestamp))
}

/// Initialize the logs directory
pub fn initialize_logs_directory() -> anyhow::Result<()> {
    let logs_dir = logs_dir();
    fs::create_dir_all(&logs_dir)?;
    Ok(())
}

/// Set up log redirection to ~/.config/bezy/logs/
/// Used when running without TUI to capture logs to file
pub fn setup_log_redirection() -> anyhow::Result<()> {
    // Check if config directory exists - if not, we'll fail gracefully
    let config_dir = config_dir();
    if !config_dir.exists() {
        // Config directory doesn't exist, so don't try to create logs
        return Err(anyhow::anyhow!("Config directory doesn't exist"));
    }

    // Initialize logs directory
    initialize_logs_directory()?;

    // Get the log file path
    let log_file_path = current_log_file();

    // Create/open the log file - use truncate instead of append for single log file
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_file_path)?;

    // Redirect stdout and stderr to the log file
    unsafe {
        libc::dup2(log_file.as_raw_fd(), libc::STDOUT_FILENO);
        libc::dup2(log_file.as_raw_fd(), libc::STDERR_FILENO);
    }

    // Print initial log message to confirm redirection
    println!(
        "=== Bezy started at {} ===",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("Logs redirected to: {:?}", log_file_path);

    Ok(())
}

pub fn setup_file_logging_for_tui() -> anyhow::Result<()> {
    use tracing_appender::rolling;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    initialize_logs_directory()?;

    let logs_dir = logs_dir();
    let file_appender = rolling::daily(logs_dir, "bezy.log");

    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(true);

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            #[cfg(debug_assertions)]
            {
                EnvFilter::new("bezy=debug,bevy_render=warn,bevy_winit=warn,wgpu=warn,winit=warn,bevy_ecs::error::handler=error")
            }
            #[cfg(not(debug_assertions))]
            {
                EnvFilter::new("bezy=info,bevy=warn,wgpu=error,winit=error,bevy_ecs::error::handler=error")
            }
        });

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .init();

    Ok(())
}
