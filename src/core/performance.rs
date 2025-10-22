use bevy_ecs::prelude::*;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const SAMPLE_SIZE: usize = 120;
const LOG_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Resource)]
pub struct PerformanceAnalytics {
    frame_times: VecDeque<Duration>,
    last_frame_start: Instant,
    current_frame_start: Instant,
    last_log_time: Instant,
    total_frames: u64,
}

impl Default for PerformanceAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceAnalytics {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            frame_times: VecDeque::with_capacity(SAMPLE_SIZE),
            last_frame_start: now,
            current_frame_start: now,
            last_log_time: now,
            total_frames: 0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.current_frame_start = Instant::now();
    }

    pub fn end_frame(&mut self) {
        let frame_time = self.current_frame_start.elapsed();

        if self.frame_times.len() >= SAMPLE_SIZE {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(frame_time);

        self.last_frame_start = self.current_frame_start;
        self.total_frames += 1;
    }

    pub fn should_log(&self) -> bool {
        self.last_log_time.elapsed() >= LOG_INTERVAL
    }

    pub fn log_analytics(&mut self) {
        if self.frame_times.is_empty() {
            return;
        }

        let avg_frame_time = self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32;
        let min_frame_time = self.frame_times.iter().min().copied().unwrap_or(Duration::ZERO);
        let max_frame_time = self.frame_times.iter().max().copied().unwrap_or(Duration::ZERO);

        let avg_fps = if avg_frame_time.as_secs_f64() > 0.0 {
            1.0 / avg_frame_time.as_secs_f64()
        } else {
            0.0
        };

        let min_fps = if max_frame_time.as_secs_f64() > 0.0 {
            1.0 / max_frame_time.as_secs_f64()
        } else {
            0.0
        };

        let max_fps = if min_frame_time.as_secs_f64() > 0.0 {
            1.0 / min_frame_time.as_secs_f64()
        } else {
            0.0
        };

        log::info!(
            "Performance: FPS: {:.1} (min: {:.1}, max: {:.1}) | Frame Time: {:.2}ms (min: {:.2}ms, max: {:.2}ms) | Total Frames: {}",
            avg_fps,
            min_fps,
            max_fps,
            avg_frame_time.as_secs_f64() * 1000.0,
            min_frame_time.as_secs_f64() * 1000.0,
            max_frame_time.as_secs_f64() * 1000.0,
            self.total_frames
        );

        self.last_log_time = Instant::now();
    }

    pub fn fps(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let avg_frame_time = self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32;
        if avg_frame_time.as_secs_f64() > 0.0 {
            1.0 / avg_frame_time.as_secs_f64()
        } else {
            0.0
        }
    }

    pub fn avg_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::ZERO;
        }
        self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32
    }

    pub fn total_frames(&self) -> u64 {
        self.total_frames
    }

    pub fn frame_times(&self) -> &VecDeque<Duration> {
        &self.frame_times
    }

    pub fn min_frame_time(&self) -> Duration {
        self.frame_times.iter().min().copied().unwrap_or(Duration::ZERO)
    }

    pub fn max_frame_time(&self) -> Duration {
        self.frame_times.iter().max().copied().unwrap_or(Duration::ZERO)
    }

    pub fn min_fps(&self) -> f64 {
        let max_frame_time = self.max_frame_time();
        if max_frame_time.as_secs_f64() > 0.0 {
            1.0 / max_frame_time.as_secs_f64()
        } else {
            0.0
        }
    }

    pub fn max_fps(&self) -> f64 {
        let min_frame_time = self.min_frame_time();
        if min_frame_time.as_secs_f64() > 0.0 {
            1.0 / min_frame_time.as_secs_f64()
        } else {
            0.0
        }
    }
}

pub fn begin_frame_system(mut analytics: ResMut<PerformanceAnalytics>) {
    analytics.begin_frame();
}

pub fn end_frame_system(mut analytics: ResMut<PerformanceAnalytics>) {
    analytics.end_frame();

    if analytics.should_log() {
        analytics.log_analytics();
    }
}

#[derive(Default)]
pub struct PerformancePlugin;

impl crate::app::Plugin for PerformancePlugin {
    fn build(&self, engine: &mut crate::app::Resonance) {
        engine.world.insert_resource(PerformanceAnalytics::new());

        if let Some(schedule) = engine.schedules.get_mut(crate::app::Stage::PreUpdate) {
            schedule.add_systems(begin_frame_system);
        }

        if let Some(schedule) = engine.schedules.get_mut(crate::app::Stage::Last) {
            schedule.add_systems(end_frame_system);
        }
    }

    fn name(&self) -> &str {
        "PerformancePlugin"
    }
}
