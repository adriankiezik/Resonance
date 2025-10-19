pub mod error;
pub mod logger;
pub mod math;
pub mod time;

pub use error::{ResonanceError, Result};
pub use logger::init_logger;
pub use math::*;
pub use time::{
    FixedTime, GameTick, Time, TimePlugin, fixed_time_system, game_tick_system, time_system,
};
