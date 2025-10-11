//! Engine runner - the main game loop.

use crate::engine::Engine;
use std::time::{Duration, Instant};

pub mod window_runner;

/// Target frame rate for the engine (60 FPS)
const TARGET_FPS: u32 = 60;
const TARGET_FRAME_TIME: Duration = Duration::from_micros(1_000_000 / TARGET_FPS as u64);

/// Frame timing statistics
#[derive(Debug, Clone, Copy)]
pub struct FrameTiming {
    pub frame_count: u64,
    pub frame_time: Duration,
    pub fps: f32,
}

impl FrameTiming {
    fn new() -> Self {
        Self {
            frame_count: 0,
            frame_time: Duration::ZERO,
            fps: 0.0,
        }
    }
}

/// Run the engine with a simple game loop.
///
/// This is a basic runner that will be replaced by platform-specific
/// runners (e.g., winit event loop for client, simple loop for server).
///
/// For now, this runs a fixed number of iterations for testing purposes.
pub fn run(mut engine: Engine) {
    engine.startup();

    log::info!("Running engine with target {} FPS", TARGET_FPS);

    let mut frame_count = 0;
    const MAX_TEST_FRAMES: u32 = 10;
    let mut timing = FrameTiming::new();

    while engine.is_running() && frame_count < MAX_TEST_FRAMES {
        let frame_start = Instant::now();

        engine.update();

        // Calculate frame time
        let frame_elapsed = frame_start.elapsed();
        timing.frame_time = frame_elapsed;
        timing.frame_count += 1;
        timing.fps = 1.0 / frame_elapsed.as_secs_f32();

        // Frame limiting: sleep if we finished early
        if frame_elapsed < TARGET_FRAME_TIME {
            let sleep_time = TARGET_FRAME_TIME - frame_elapsed;
            std::thread::sleep(sleep_time);
        }

        frame_count += 1;
    }

    log::info!(
        "Engine stopped after {} frames (avg frame time: {:.2}ms)",
        frame_count,
        timing.frame_time.as_secs_f64() * 1000.0
    );
}

/// Run the engine indefinitely (for real applications)
///
/// This will be the main game loop once window/event handling is implemented.
/// For servers, this runs until explicitly stopped.
pub fn run_forever(mut engine: Engine) {
    engine.startup();

    log::info!("Running engine indefinitely with target {} FPS", TARGET_FPS);

    let mut timing = FrameTiming::new();
    let mut last_fps_log = Instant::now();

    while engine.is_running() {
        let frame_start = Instant::now();

        engine.update();

        // Calculate frame time
        let frame_elapsed = frame_start.elapsed();
        timing.frame_time = frame_elapsed;
        timing.frame_count += 1;
        timing.fps = 1.0 / frame_elapsed.as_secs_f32();

        // Log FPS every second
        if last_fps_log.elapsed() >= Duration::from_secs(1) {
            log::debug!(
                "FPS: {:.1} | Frame time: {:.2}ms",
                timing.fps,
                timing.frame_time.as_secs_f64() * 1000.0
            );
            last_fps_log = Instant::now();
        }

        // Frame limiting: sleep if we finished early
        if frame_elapsed < TARGET_FRAME_TIME {
            let sleep_time = TARGET_FRAME_TIME - frame_elapsed;
            std::thread::sleep(sleep_time);
        }
    }

    log::info!(
        "Engine stopped after {} frames",
        timing.frame_count
    );
}
