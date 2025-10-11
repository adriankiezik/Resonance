
pub mod error;
pub mod logger;
pub mod math;
pub mod time;

pub use error::{FerriteError, Result};
pub use logger::init_logger;
pub use math::*;
pub use time::{
    fixed_time_system, game_tick_system, time_system, FixedTime, GameTick, Time, TimePlugin,
};