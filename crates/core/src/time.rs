
use bevy_ecs::prelude::*;
use std::time::{Duration, Instant};

#[derive(Resource, Clone, Copy)]
pub struct Time {

    startup: Instant,

    last_update: Instant,

    delta: Duration,

    time_scale: f32,

    paused: bool,
}

impl Time {

    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            startup: now,
            last_update: now,
            delta: Duration::ZERO,
            time_scale: 1.0,
            paused: false,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        if self.paused {

            self.last_update = now;
        } else {
            self.delta = now.duration_since(self.last_update);
            self.last_update = now;
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.startup.elapsed()
    }

    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed().as_secs_f32()
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    pub fn delta_seconds(&self) -> f32 {
        if self.paused {
            0.0
        } else {
            self.delta.as_secs_f32() * self.time_scale
        }
    }

    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;

        self.last_update = Instant::now();
    }

    pub fn toggle_pause(&mut self) {
        if self.paused {
            self.resume();
        } else {
            self.pause();
        }
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Resource, Clone, Copy)]
pub struct FixedTime {

    timestep: Duration,

    accumulator: Duration,

    max_accumulator: Duration,
}

impl FixedTime {

    pub fn new(rate: u32) -> Self {
        let timestep = Duration::from_secs_f32(1.0 / rate as f32);
        Self {
            timestep,
            accumulator: Duration::ZERO,
            max_accumulator: timestep * 10, 
        }
    }

    pub fn accumulate(&mut self, delta: Duration) {
        self.accumulator = (self.accumulator + delta).min(self.max_accumulator);
    }

    pub fn should_update(&self) -> bool {
        self.accumulator >= self.timestep
    }

    pub fn consume_step(&mut self) {
        self.accumulator = self.accumulator.saturating_sub(self.timestep);
    }

    pub fn timestep(&self) -> Duration {
        self.timestep
    }

    pub fn timestep_seconds(&self) -> f32 {
        self.timestep.as_secs_f32()
    }

    pub fn alpha(&self) -> f32 {
        self.accumulator.as_secs_f32() / self.timestep.as_secs_f32()
    }
}

impl Default for FixedTime {
    fn default() -> Self {
        Self::new(60) 
    }
}

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameTick(pub u64);

impl GameTick {

    pub fn new() -> Self {
        Self(0)
    }

    pub fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

pub fn time_system(mut time: ResMut<Time>) {
    time.update();
}

pub fn fixed_time_system(time: Res<Time>, mut fixed_time: ResMut<FixedTime>) {
    fixed_time.accumulate(time.delta());
}

pub fn game_tick_system(mut tick: ResMut<GameTick>) {
    tick.increment();
}

pub struct TimePlugin;