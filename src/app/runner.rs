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
        let mut time = world.resource_mut::<crate::core::Time>();
        time.update();

        self.run_schedule(
            schedules.get_mut(Stage::PreUpdate).unwrap(),
            world,
            "Stage::PreUpdate",
        );
        self.run_schedule(
            schedules.get_mut(Stage::Update).unwrap(),
            world,
            "Stage::Update",
        );

        let delta = world.resource::<crate::core::Time>().delta();
        let mut fixed_time = world.resource_mut::<crate::core::FixedTime>();
        fixed_time.accumulate(delta);

        while world.resource::<crate::core::FixedTime>().should_update() {
            let mut tick = world.resource_mut::<crate::core::GameTick>();
            tick.increment();

            self.run_schedule(
                schedules.get_mut(Stage::FixedUpdate).unwrap(),
                world,
                "Stage::FixedUpdate",
            );

            world
                .resource_mut::<crate::core::FixedTime>()
                .consume_step();
        }

        self.run_schedule(
            schedules.get_mut(Stage::PostUpdate).unwrap(),
            world,
            "Stage::PostUpdate",
        );

        if self.enable_rendering {
            self.run_schedule(
                schedules.get_mut(Stage::Render).unwrap(),
                world,
                "Stage::Render",
            );
        }

        self.run_schedule(
            schedules.get_mut(Stage::Last).unwrap(),
            world,
            "Stage::Last",
        );
    }
}

pub struct ResonanceRunnerBuilder {
    profiling_enabled: bool,
    _use_fixed_timestep: bool,
    enable_rendering: bool,
}

impl Default for ResonanceRunnerBuilder {
    fn default() -> Self {
        Self {
            profiling_enabled: false,
            _use_fixed_timestep: true,
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
