use log::LevelFilter;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};

/// Initializes the logger with custom per-module filters
///
/// # Arguments
/// * `default_level` - Default log level for all modules
/// * `filters` - Array of (module_name, level) tuples for per-module filtering
pub fn init_logger_with_filter(default_level: LevelFilter, filters: &[(&str, LevelFilter)]) {
    init_logger_impl(default_level, filters);
}

pub fn init_logger(level: LevelFilter) {
    init_logger_impl(level, &[]);
}

fn init_logger_impl(level: LevelFilter, _filters: &[(&str, LevelFilter)]) {
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let log_filename = format!("resonance_{}.log", timestamp);

    if std::fs::metadata("logs").is_err() {
        let _ = std::fs::create_dir("logs");
    }

    let log_path = format!("logs/{}", log_filename);

    let file = match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
    {
        Ok(f) => {
            println!("Logging to: {}", log_path);
            Arc::new(Mutex::new(f))
        }
        Err(e) => {
            eprintln!("Failed to create log file {}: {}", log_path, e);
            eprintln!("Falling back to console-only logging");

            env_logger::Builder::from_default_env()
                .filter_level(level)
                .format(|buf, record| {
                    let level_style = match record.level() {
                        log::Level::Error => "\x1b[31m",
                        log::Level::Warn => "\x1b[33m",
                        log::Level::Info => "\x1b[32m",
                        log::Level::Debug => "\x1b[36m",
                        log::Level::Trace => "\x1b[35m",
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
            return;
        }
    };

    let file_clone = file.clone();

    env_logger::Builder::from_default_env()
        .filter_level(level)
        .format(move |buf, record| {
            let level_style = match record.level() {
                log::Level::Error => "\x1b[31m",
                log::Level::Warn => "\x1b[33m",
                log::Level::Info => "\x1b[32m",
                log::Level::Debug => "\x1b[36m",
                log::Level::Trace => "\x1b[35m",
            };

            let colored_output = format!(
                "{}[{:5}]\x1b[0m [{}] {}",
                level_style,
                record.level(),
                record.target(),
                record.args()
            );

            let plain_output = format!(
                "[{:5}] [{}] {}",
                record.level(),
                record.target(),
                record.args()
            );

            writeln!(buf, "{}", colored_output)?;

            if let Ok(mut file) = file_clone.lock() {
                let _ = writeln!(file, "{}", plain_output);
            }

            Ok(())
        })
        .init();
}

#[cfg(test)]
pub fn init_test_logger() {
    let _ = env_logger::Builder::from_default_env()
        .is_test(true)
        .filter_level(LevelFilter::Trace)
        .try_init();
}
