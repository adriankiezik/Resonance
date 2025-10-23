pub mod error;
pub mod logger;
pub mod math;
pub mod memory_stats;
pub mod performance;
pub mod profiler;
pub mod time;

pub mod egui_plugin;
pub mod general_info_ui;
pub mod performance_ui;
pub mod profiler_ui;

pub use error::{ResonanceError, Result};
pub use logger::init_logger;
pub use math::*;
pub use memory_stats::{AssetMemoryStats, GpuMemoryStats, MemoryTracker, format_bytes};
pub use performance::{PerformanceAnalytics, PerformancePlugin};
pub use profiler::{ProfileScope, Profiler, ProfilerPlugin, TimingEntry};
pub use time::{
    FixedTime, GameTick, Time, TimePlugin, fixed_time_system, game_tick_system, time_system,
};

pub use egui_plugin::{EditorUiRenderFn, EguiContext, EguiPlugin, render_ui};
pub use general_info_ui::render_general_info_panel;
pub use performance_ui::render_performance_panel;
pub use profiler_ui::{ProfilerUiState, render_profiler_panel};
