//! Window-based runner using winit event loop.
//!
//! This runner integrates the engine with winit's event loop,
//! handling window events and rendering.

use crate::engine::Engine;
use std::time::{Duration, Instant};

/// Window application state for winit event loop
pub struct WindowApp {
    pub engine: Option<Engine>,
    last_update: Option<Instant>,
    target_frame_time: Duration,
}

impl WindowApp {
    pub fn new(engine: Engine) -> Self {
        Self {
            engine: Some(engine),
            last_update: None,
            target_frame_time: Duration::from_micros(1_000_000 / 60), // 60 FPS
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();

        // Check if we should update based on frame rate limit
        if let Some(last) = self.last_update {
            let elapsed = now.duration_since(last);
            if elapsed < self.target_frame_time {
                return;
            }
        }

        // Update the engine
        if let Some(ref mut engine) = self.engine {
            engine.update();
        }

        self.last_update = Some(now);
    }
}
