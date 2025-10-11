//! Tests for the time management system

use ferrite_core::time::{FixedTime, GameTick, Time};
use std::time::Duration;

#[test]
fn test_time_initialization() {
    let time = Time::new();

    assert_eq!(time.delta_seconds(), 0.0);
    assert_eq!(time.time_scale(), 1.0);
    assert!(!time.is_paused());
}

#[test]
fn test_time_update() {
    let mut time = Time::new();

    // Sleep a bit to ensure time passes
    std::thread::sleep(Duration::from_millis(10));
    time.update();

    // Delta should be non-zero after update
    assert!(time.delta_seconds() > 0.0);
    assert!(time.elapsed_seconds() > 0.0);
}

#[test]
fn test_time_pause_resume() {
    let mut time = Time::new();

    // Initially not paused
    assert!(!time.is_paused());

    // Update to get some delta
    std::thread::sleep(Duration::from_millis(10));
    time.update();
    let delta_before_pause = time.delta_seconds();
    assert!(delta_before_pause > 0.0);

    // Pause time
    time.pause();
    assert!(time.is_paused());

    // Update while paused - delta should be zero
    std::thread::sleep(Duration::from_millis(10));
    time.update();
    assert_eq!(time.delta_seconds(), 0.0);

    // Resume time
    time.resume();
    assert!(!time.is_paused());

    // After resume, delta should be non-zero again
    std::thread::sleep(Duration::from_millis(10));
    time.update();
    assert!(time.delta_seconds() > 0.0);
}

#[test]
fn test_time_toggle_pause() {
    let mut time = Time::new();

    assert!(!time.is_paused());

    time.toggle_pause();
    assert!(time.is_paused());

    time.toggle_pause();
    assert!(!time.is_paused());
}

#[test]
fn test_time_scale() {
    let mut time = Time::new();

    // Default scale is 1.0
    assert_eq!(time.time_scale(), 1.0);

    // Set to half speed
    time.set_time_scale(0.5);
    assert_eq!(time.time_scale(), 0.5);

    std::thread::sleep(Duration::from_millis(10));
    time.update();

    let scaled_delta = time.delta_seconds();
    let raw_delta = time.delta().as_secs_f32();

    // Scaled delta should be approximately half of raw delta
    assert!((scaled_delta - raw_delta * 0.5).abs() < 0.001);

    // Negative scale should be clamped to 0
    time.set_time_scale(-1.0);
    assert_eq!(time.time_scale(), 0.0);
}

#[test]
fn test_fixed_time_default() {
    let fixed_time = FixedTime::default();

    // Default is 60 Hz
    assert_eq!(fixed_time.timestep_seconds(), 1.0 / 60.0);
    assert!(!fixed_time.should_update());
}

#[test]
fn test_fixed_time_accumulation() {
    let mut fixed_time = FixedTime::new(60); // 60 Hz = 16.67ms per step

    // Accumulate less than one timestep
    fixed_time.accumulate(Duration::from_millis(10));
    assert!(!fixed_time.should_update());

    // Accumulate more - should cross threshold
    fixed_time.accumulate(Duration::from_millis(10));
    assert!(fixed_time.should_update());

    // Consume the step
    fixed_time.consume_step();
    assert!(!fixed_time.should_update());
}

#[test]
fn test_fixed_time_multiple_steps() {
    let mut fixed_time = FixedTime::new(60);
    let timestep = Duration::from_secs_f32(1.0 / 60.0);

    // Accumulate 3 timesteps worth
    fixed_time.accumulate(timestep * 3);

    // Should be able to consume 3 steps
    assert!(fixed_time.should_update());
    fixed_time.consume_step();

    assert!(fixed_time.should_update());
    fixed_time.consume_step();

    assert!(fixed_time.should_update());
    fixed_time.consume_step();

    assert!(!fixed_time.should_update());
}

#[test]
fn test_fixed_time_max_accumulator() {
    let mut fixed_time = FixedTime::new(60);
    let timestep = Duration::from_secs_f32(1.0 / 60.0);

    // Try to accumulate way too much (spiral of death scenario)
    // Max accumulator is 10 * timestep
    fixed_time.accumulate(timestep * 100);

    // Should cap at max_accumulator (10 steps)
    let mut steps = 0;
    while fixed_time.should_update() {
        fixed_time.consume_step();
        steps += 1;
    }

    assert_eq!(steps, 10);
}

#[test]
fn test_fixed_time_alpha() {
    let mut fixed_time = FixedTime::new(60);
    let timestep = Duration::from_secs_f32(1.0 / 60.0);

    // Alpha should be 0.0 with no accumulation
    assert_eq!(fixed_time.alpha(), 0.0);

    // Accumulate half a timestep
    fixed_time.accumulate(timestep / 2);

    // Alpha should be approximately 0.5
    let alpha = fixed_time.alpha();
    assert!((alpha - 0.5).abs() < 0.01);

    // Accumulate to full timestep
    fixed_time.accumulate(timestep / 2);

    // Alpha should be approximately 1.0
    let alpha = fixed_time.alpha();
    assert!((alpha - 1.0).abs() < 0.01);
}

#[test]
fn test_game_tick_initialization() {
    let tick = GameTick::new();
    assert_eq!(tick.get(), 0);
}

#[test]
fn test_game_tick_increment() {
    let mut tick = GameTick::new();

    assert_eq!(tick.get(), 0);

    tick.increment();
    assert_eq!(tick.get(), 1);

    tick.increment();
    assert_eq!(tick.get(), 2);

    // Test many increments
    for i in 3..1000 {
        tick.increment();
        assert_eq!(tick.get(), i);
    }
}

#[test]
fn test_game_tick_wrapping() {
    let mut tick = GameTick(u64::MAX - 1);

    tick.increment();
    assert_eq!(tick.get(), u64::MAX);

    // Should wrap to 0
    tick.increment();
    assert_eq!(tick.get(), 0);
}

#[test]
fn test_game_tick_default() {
    let tick = GameTick::default();
    assert_eq!(tick.get(), 0);
}

#[test]
fn test_game_tick_clone() {
    let tick1 = GameTick(42);
    let tick2 = tick1;

    assert_eq!(tick1.get(), 42);
    assert_eq!(tick2.get(), 42);
}
