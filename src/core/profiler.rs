use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

const LOG_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct TimingEntry {
    pub total_time: Duration,
    pub call_count: u64,
    pub avg_time: Duration,
    pub max_time: Duration,
    pub min_time: Duration,
}

impl TimingEntry {
    fn new() -> Self {
        Self {
            total_time: Duration::ZERO,
            call_count: 0,
            avg_time: Duration::ZERO,
            max_time: Duration::ZERO,
            min_time: Duration::MAX,
        }
    }

    fn record(&mut self, duration: Duration) {
        self.total_time += duration;
        self.call_count += 1;
        self.avg_time = self.total_time / self.call_count as u32;
        self.max_time = self.max_time.max(duration);
        self.min_time = self.min_time.min(duration);
    }
}

#[derive(Resource)]
pub struct Profiler {
    accumulated_timings: HashMap<String, TimingEntry>,
    last_log_time: Instant,
    current_scope: Option<(String, Instant)>,
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            accumulated_timings: HashMap::new(),
            last_log_time: Instant::now(),
            current_scope: None,
        }
    }

    pub fn begin_scope(&mut self, name: String) {
        self.current_scope = Some((name, Instant::now()));
    }

    pub fn end_scope(&mut self) {
        if let Some((name, start)) = self.current_scope.take() {
            let duration = start.elapsed();
            self.record_timing(name, duration);
        }
    }

    pub fn record_timing(&mut self, name: String, duration: Duration) {
        let entry = self.accumulated_timings
            .entry(name)
            .or_insert_with(TimingEntry::new);
        entry.record(duration);
    }

    pub fn should_log(&self) -> bool {
        self.last_log_time.elapsed() >= LOG_INTERVAL
    }

    pub fn log_profiles(&mut self) {
        if self.accumulated_timings.is_empty() {
            return;
        }

        let mut timings: Vec<_> = self.accumulated_timings.iter().collect();
        timings.sort_by(|a, b| b.1.avg_time.cmp(&a.1.avg_time));

        log::info!("=== Performance Profile (5s window, sorted by avg time) ===");

        for (name, timing) in timings.iter() {
            let avg_us = timing.avg_time.as_micros();
            let max_us = timing.max_time.as_micros();
            let min_us = timing.min_time.as_micros();
            let total_ms = timing.total_time.as_secs_f64() * 1000.0;
            let percent = (timing.total_time.as_secs_f64() / 5.0) * 100.0;

            log::info!(
                "  {:40} | Avg: {:7.0}µs | Min: {:7.0}µs | Max: {:7.0}µs | Total: {:7.2}ms ({:5.2}%) | Calls: {}",
                truncate_name(name, 40),
                avg_us,
                min_us,
                max_us,
                total_ms,
                percent,
                timing.call_count
            );
        }

        log::info!("================================================================");

        self.accumulated_timings.clear();
        self.last_log_time = Instant::now();
    }

    pub fn timings(&self) -> &HashMap<String, TimingEntry> {
        &self.accumulated_timings
    }

    pub fn time_since_last_log(&self) -> Duration {
        self.last_log_time.elapsed()
    }
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        format!("{:<width$}", name, width = max_len)
    } else {
        let prefix_len = max_len - 3;
        format!("{}...", &name[..prefix_len])
    }
}

pub struct ProfileScope {
    name: String,
    start: Instant,
}

impl ProfileScope {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }

    pub fn end(self, profiler: &mut Profiler) {
        let duration = self.start.elapsed();
        profiler.record_timing(self.name.clone(), duration);
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
    }
}

#[macro_export]
macro_rules! profile_scope {
    ($profiler:expr, $name:expr) => {
        let _profile_guard = $crate::core::profiler::ProfileScopeGuard::new($name);
        let _profiler_ref = &$profiler;
    };
}

pub struct ProfileScopeGuard {
    name: String,
    start: Instant,
}

impl ProfileScopeGuard {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }
}

impl Drop for ProfileScopeGuard {
    fn drop(&mut self) {
    }
}

pub fn profile_fn<F, R>(profiler: &mut Profiler, name: impl Into<String>, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    profiler.record_timing(name.into(), duration);
    result
}

pub fn end_profiling_system(mut profiler: ResMut<Profiler>) {
    if profiler.should_log() {
        profiler.log_profiles();
    }
}

#[derive(Default)]
pub struct ProfilerPlugin;

impl crate::app::Plugin for ProfilerPlugin {
    fn build(&self, engine: &mut crate::app::Resonance) {
        engine.world.insert_resource(Profiler::new());

        if let Some(schedule) = engine.schedules.get_mut(crate::app::Stage::Last) {
            schedule.add_systems(end_profiling_system);
        }
    }

    fn name(&self) -> &str {
        "ProfilerPlugin"
    }
}
