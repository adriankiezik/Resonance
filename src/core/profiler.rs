// Profiler stub - profiling functionality removed
use bevy_ecs::prelude::Resource;

#[derive(Resource, Default)]
pub struct Profiler;

impl Profiler {
    pub fn record_timing(&mut self, _name: &str, _duration: std::time::Duration) {
        // No-op
    }

    pub fn record_timing_owned(&mut self, _name: &str, _duration: std::time::Duration) {
        // No-op
    }
}
