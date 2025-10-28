pub mod error;
pub mod egui_plugin;
pub mod logger;
pub mod math;
pub mod memory_stats;
pub mod performance;
pub mod profiler;
pub mod time;

pub use egui_plugin::EguiContext;
pub use error::{ResonanceError, Result};
pub use logger::init_logger;
pub use math::*;
pub use memory_stats::{AssetMemoryStats, GpuMemoryStats, MemoryTracker, format_bytes};
pub use performance::{PerformanceAnalytics, PerformancePlugin};
pub use profiler::Profiler;
pub use time::{
    FixedTime, GameTick, Time, TimePlugin, fixed_time_system, game_tick_system, time_system,
};
