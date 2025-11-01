use super::stage::Stage;
use bevy_ecs::{
    schedule::{Schedule, Schedules},
    world::World,
};
use std::time::Instant;

pub struct ResonanceRunner {
    profiling_enabled: bool,
    enable_rendering: bool,
}

impl ResonanceRunner {
    pub fn builder() -> ResonanceRunnerBuilder {
        ResonanceRunnerBuilder::default()
    }

    pub fn run_schedule(&self, schedule: &mut Schedule, world: &mut World, stage_name: &'static str) {
        if self.profiling_enabled {
            let start = Instant::now();
            schedule.run(world);
            if let Some(mut profiler) = world.get_resource_mut::<crate::core::Profiler>() {
                profiler.record_timing(stage_name, start.elapsed());
            }
        } else {
            schedule.run(world);
        }
    }

    pub fn run(&self, world: &mut World, schedules: &mut Schedules) {
        // Update time at frame start
        let mut time = world.resource_mut::<crate::core::Time>();
        time.update();

        // Check if paused - skip all systems except time update
        let is_paused = time.is_paused();
        drop(time);

        if is_paused {
            return; // Skip all systems when paused
        }

        // Run pre-update and update stages
        for stage in [Stage::PreUpdate, Stage::Update] {
            self.run_schedule(schedules.get_mut(stage).unwrap(), world, stage.name());
        }

        // Fixed timestep loop for physics/deterministic updates
        let delta = world.resource::<crate::core::Time>().delta();
        let mut fixed_time = world.resource_mut::<crate::core::FixedTime>();
        fixed_time.accumulate(delta);

        while world.resource::<crate::core::FixedTime>().should_update() {
            world.resource_mut::<crate::core::GameTick>().increment();

            self.run_schedule(
                schedules.get_mut(Stage::FixedUpdate).unwrap(),
                world,
                "Stage::FixedUpdate",
            );

            world.resource_mut::<crate::core::FixedTime>().consume_step();
        }

        // Run post-update and cleanup stages
        let post_stages = if self.enable_rendering {
            &[Stage::PostUpdate, Stage::Render, Stage::Last][..]
        } else {
            &[Stage::PostUpdate, Stage::Last][..]
        };

        for &stage in post_stages {
            self.run_schedule(schedules.get_mut(stage).unwrap(), world, stage.name());
        }
    }
}

pub struct ResonanceRunnerBuilder {
    profiling_enabled: bool,
    enable_rendering: bool,
}

impl Default for ResonanceRunnerBuilder {
    fn default() -> Self {
        Self {
            profiling_enabled: false,
            enable_rendering: true,
        }
    }
}

impl ResonanceRunnerBuilder {
    pub fn with_profiling(mut self, enabled: bool) -> Self {
        self.profiling_enabled = enabled;
        self
    }

    pub fn with_rendering(mut self, enabled: bool) -> Self {
        self.enable_rendering = enabled;
        self
    }

    pub fn build(self) -> ResonanceRunner {
        ResonanceRunner {
            profiling_enabled: self.profiling_enabled,
            enable_rendering: self.enable_rendering,
        }
    }
}
