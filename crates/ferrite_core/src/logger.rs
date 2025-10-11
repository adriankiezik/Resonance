//! Logging setup for the engine.
//!
//! Provides a simple interface to initialize logging using env_logger.

use log::LevelFilter;
use std::io::Write;

/// Initialize the logger with default settings.
///
/// Sets up colored output and reasonable default log levels.
pub fn init_logger() {
    init_logger_with_level(LevelFilter::Info);
}

/// Initialize the logger with a specific log level.
pub fn init_logger_with_level(level: LevelFilter) {
    env_logger::Builder::from_default_env()
        .filter_level(level)
        .format(|buf, record| {
            let level_style = match record.level() {
                log::Level::Error => "\x1b[31m", // Red
                log::Level::Warn => "\x1b[33m",  // Yellow
                log::Level::Info => "\x1b[32m",  // Green
                log::Level::Debug => "\x1b[36m", // Cyan
                log::Level::Trace => "\x1b[35m", // Magenta
            };
            writeln!(
                buf,
                "{}[{:5}]\x1b[0m [{}] {}",
                level_style,
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();
}

/// Initialize the logger for testing (no colors, simpler format).
#[cfg(test)]
pub fn init_test_logger() {
    let _ = env_logger::Builder::from_default_env()
        .is_test(true)
        .filter_level(LevelFilter::Trace)
        .try_init();
}
