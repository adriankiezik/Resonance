
use log::LevelFilter;
use std::io::Write;

pub fn init_logger(level: LevelFilter) {
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
}

#[cfg(test)]
pub fn init_test_logger() {
    let _ = env_logger::Builder::from_default_env()
        .is_test(true)
        .filter_level(LevelFilter::Trace)
        .try_init();
}
