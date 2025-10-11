//! Time management and tick system for deterministic simulation.
//!
//! This module provides time tracking for both variable timestep (rendering)
//! and fixed timestep (game logic/physics) updates, crucial for multiplayer games.

use bevy_ecs::prelude::*;
use std::time::{Duration, Instant};

/// Resource tracking frame time and delta time.
///
/// Updated every frame to track elapsed time and delta between frames.
#[derive(Resource, Clone, Copy)]
pub struct Time {
    /// Time since the engine started
    startup: Instant,
    /// Time the last frame started
    last_update: Instant,
    /// Duration since the last frame
    delta: Duration,
    /// Time scale multiplier (1.0 = normal speed, 0.5 = half speed, etc.)
    time_scale: f32,
    /// Whether time is paused
    paused: bool,
}

impl Time {
    /// Create a new Time resource
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

    /// Update the time with the current instant
    pub fn update(&mut self) {
        let now = Instant::now();
        if self.paused {
            // When paused, still track real time but don't update delta
            self.last_update = now;
        } else {
            self.delta = now.duration_since(self.last_update);
            self.last_update = now;
        }
    }

    /// Get the time since startup
    pub fn elapsed(&self) -> Duration {
        self.startup.elapsed()
    }

    /// Get the time since startup in seconds
    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed().as_secs_f32()
    }

    /// Get the delta time (time since last frame)
    pub fn delta(&self) -> Duration {
        self.delta
    }

    /// Get the delta time in seconds
    pub fn delta_seconds(&self) -> f32 {
        if self.paused {
            0.0
        } else {
            self.delta.as_secs_f32() * self.time_scale
        }
    }

    /// Get the time scale
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Set the time scale (affects delta_seconds)
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }

    /// Check if time is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Pause time (delta will be zero)
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume time
    pub fn resume(&mut self) {
        self.paused = false;
        // Reset last_update to prevent large delta on resume
        self.last_update = Instant::now();
    }

    /// Toggle pause state
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

/// Resource for fixed timestep simulation (physics, game logic).
///
/// Essential for deterministic multiplayer - ensures consistent simulation
/// regardless of framerate.
#[derive(Resource, Clone, Copy)]
pub struct FixedTime {
    /// Fixed timestep duration (e.g., 1/60 second)
    timestep: Duration,
    /// Accumulated time that hasn't been simulated yet
    accumulator: Duration,
    /// Maximum accumulator to prevent spiral of death
    max_accumulator: Duration,
}

impl FixedTime {
    /// Create a new fixed timestep with the given rate (Hz)
    pub fn new(rate: u32) -> Self {
        let timestep = Duration::from_secs_f32(1.0 / rate as f32);
        Self {
            timestep,
            accumulator: Duration::ZERO,
            max_accumulator: timestep * 10, // Allow up to 10 steps behind
        }
    }

    /// Accumulate frame time
    pub fn accumulate(&mut self, delta: Duration) {
        self.accumulator = (self.accumulator + delta).min(self.max_accumulator);
    }

    /// Check if we should run a fixed update
    pub fn should_update(&self) -> bool {
        self.accumulator >= self.timestep
    }

    /// Consume one timestep from the accumulator
    pub fn consume_step(&mut self) {
        self.accumulator = self.accumulator.saturating_sub(self.timestep);
    }

    /// Get the fixed timestep duration
    pub fn timestep(&self) -> Duration {
        self.timestep
    }

    /// Get the fixed timestep in seconds
    pub fn timestep_seconds(&self) -> f32 {
        self.timestep.as_secs_f32()
    }

    /// Get interpolation alpha for smooth rendering between fixed steps
    pub fn alpha(&self) -> f32 {
        self.accumulator.as_secs_f32() / self.timestep.as_secs_f32()
    }
}

impl Default for FixedTime {
    fn default() -> Self {
        Self::new(60) // 60 Hz by default
    }
}

/// Resource tracking game ticks for deterministic simulation.
///
/// Used in multiplayer to ensure all clients simulate the same tick.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameTick(pub u64);

impl GameTick {
    /// Create a new game tick starting at 0
    pub fn new() -> Self {
        Self(0)
    }

    /// Increment the tick
    pub fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }

    /// Get the current tick number
    pub fn get(&self) -> u64 {
        self.0
    }
}

/// System to update the Time resource every frame
pub fn time_system(mut time: ResMut<Time>) {
    time.update();
}

/// System to update fixed time accumulator
pub fn fixed_time_system(time: Res<Time>, mut fixed_time: ResMut<FixedTime>) {
    fixed_time.accumulate(time.delta());
}

/// System to increment the game tick during fixed updates
pub fn game_tick_system(mut tick: ResMut<GameTick>) {
    tick.increment();
}

/// Plugin to add time management to the engine
pub struct TimePlugin;

// TODO: This will be implemented once we have the plugin system in ferrite_app
// For now, this is a marker trait. Users will manually add time resources and systems.
