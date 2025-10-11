
use crate::engine::Engine;
use std::time::{Duration, Instant};

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
            target_frame_time: Duration::from_micros(1_000_000 / 60), 
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();

        if let Some(last) = self.last_update {
            let elapsed = now.duration_since(last);
            if elapsed < self.target_frame_time {
                return;
            }
        }

        if let Some(ref mut engine) = self.engine {
            engine.update();
        }

        self.last_update = Some(now);
    }
}