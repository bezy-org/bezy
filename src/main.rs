//! A font editor built with Rust, the Bevy game engine, and Linebender crates.
//!
//! The enjoyment of one's tools is an essential ingredient of successful work.
//! — Donald Knuth
//!
//! Many have tried to replace FontForge—all have failed. I might fail, in fact
//! history says I probably will. Yet, the current state of affairs is so bad I
//! feel I must try. — Fredrick Brennan

use bezy::core;

fn main() {
    core::platform::init_panic_handling();
    let cli_args = core::platform::get_cli_args();
    match core::run_app(cli_args) {
        Ok(()) => {}
        Err(error) => core::platform::handle_error(error),
    }
}
