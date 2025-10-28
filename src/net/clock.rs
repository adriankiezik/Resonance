/// Network time synchronization
///
/// Provides a clock that synchronizes local time with server time,
/// smoothing out network jitter.

use std::time::{Duration, Instant};

/// Synchronized network clock
pub struct NetworkClock {
    /// Local time when clock was created
    start_instant: Instant,
    /// Offset to add to local time to get server time
    server_offset: Duration,
    /// Accumulated offset adjustments for smoothing
    offset_accumulator: Duration,
    /// Smoothing factor (0.0 = no smoothing, 1.0 = instant)
    smoothing_factor: f64,
}

impl NetworkClock {
    pub fn new() -> Self {
        Self {
            start_instant: Instant::now(),
            server_offset: Duration::ZERO,
            offset_accumulator: Duration::ZERO,
            smoothing_factor: 0.1,
        }
    }

    /// Update clock with server time sample
    pub fn update_server_time(&mut self, server_time: f64) {
        let local_time = self.local_time();
        let measured_offset = server_time - local_time;

        // Smooth the offset to avoid sudden jumps
        let target_offset = Duration::from_secs_f64(measured_offset);
        self.offset_accumulator = Duration::from_secs_f64(
            self.offset_accumulator.as_secs_f64() * (1.0 - self.smoothing_factor) +
            target_offset.as_secs_f64() * self.smoothing_factor
        );

        self.server_offset = self.offset_accumulator;
    }

    /// Get current local time (seconds since start)
    pub fn local_time(&self) -> f64 {
        self.start_instant.elapsed().as_secs_f64()
    }

    /// Get current server time (local time + offset)
    pub fn server_time(&self) -> f64 {
        self.local_time() + self.server_offset.as_secs_f64()
    }

    /// Get round-trip time estimate
    pub fn ping(&self) -> Duration {
        // RTT estimation would require tracking ping/pong timestamps
        // Simplified for now
        Duration::from_millis(50)
    }
}

impl Default for NetworkClock {
    fn default() -> Self {
        Self::new()
    }
}
