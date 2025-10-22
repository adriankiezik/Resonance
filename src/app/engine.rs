use super::{
    plugin::{Plugin, PluginMetadata, PluginState},
    runner::ResonanceRunner,
    stage::Stage,
};
use bevy_ecs::{
    prelude::*,
    schedule::{IntoScheduleConfigs, Schedule},
    system::ScheduleSystem,
};
use std::{any::TypeId, collections::HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResonanceMode {
    Client,
    Server,
}

pub struct Resonance {
    pub world: World,
    pub schedules: Schedules,
    pub mode: ResonanceMode,
    pub running: bool,
    plugins: HashMap<TypeId, PluginMetadata>,
    runner: ResonanceRunner,
}

impl Resonance {
    pub fn new() -> Self {
        Self::new_with_mode(ResonanceMode::Client)
    }

    pub fn new_with_mode(mode: ResonanceMode) -> Self {
        let world = World::new();
        let mut schedules = Schedules::new();

        for stage in Stage::all() {
            schedules.insert(Schedule::new(stage));
        }

        let has_profiler = world.contains_resource::<crate::core::Profiler>();
        let is_client = mode == ResonanceMode::Client;

        let runner = ResonanceRunner::builder()
            .with_profiling(has_profiler)
            .with_rendering(is_client)
            .build();

        Self {
            world,
            schedules,
            mode,
            running: false,
            plugins: HashMap::new(),
            runner,
        }
    }

    pub fn with_log_level(self, level: log::LevelFilter) -> Self {
        crate::core::init_logger(level);
        self
    }

    pub fn set_mode(mut self, mode: ResonanceMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn add_plugin<P: Plugin>(mut self, plugin: P) -> Self {
        let type_id = plugin.type_id();
        let name = plugin.name().to_string();

        if self.plugins.contains_key(&type_id) {
            log::warn!("Plugin '{}' already loaded, skipping", name);
            return self;
        }

        let should_load = match self.mode {
            ResonanceMode::Client => plugin.is_client_plugin(),
            ResonanceMode::Server => plugin.is_server_plugin(),
        };

        if !should_load {
            log::debug!(
                "Plugin '{}' skipped (not compatible with {:?} mode)",
                name,
                self.mode
            );
            return self;
        }

        let dependencies = plugin.dependencies();

        for (dep_id, dep_name) in &dependencies {
            if !self.plugins.contains_key(dep_id) {
                let dep_short_name = dep_name.split("::").last().unwrap_or(dep_name);
                let plugin_short_name = name.split("::").last().unwrap_or(&name);

                log::error!(
                    "Plugin '{}' is missing required dependency '{}'",
                    name,
                    dep_name
                );
                log::error!(
                    "  → Add .add_plugin({}::default()) before .add_plugin({}::default())",
                    dep_short_name,
                    plugin_short_name
                );

                self.plugins.insert(
                    type_id,
                    PluginMetadata {
                        type_id,
                        name,
                        state: PluginState::Failed,
                        dependencies: dependencies.iter().map(|(id, _)| *id).collect(),
                    },
                );
                return self;
            }
        }

        self.plugins.insert(
            type_id,
            PluginMetadata {
                type_id,
                name: name.clone(),
                state: PluginState::Building,
                dependencies: dependencies.iter().map(|(id, _)| *id).collect(),
            },
        );

        plugin.build(&mut self);

        if let Some(metadata) = self.plugins.get_mut(&type_id) {
            metadata.state = PluginState::Built;
        }

        self
    }

    pub fn has_plugin<P: Plugin>(&self) -> bool {
        let type_id = TypeId::of::<P>();
        self.plugins
            .get(&type_id)
            .map(|m| m.state == PluginState::Built)
            .unwrap_or(false)
    }

    pub fn get_plugin_metadata<P: Plugin>(&self) -> Option<&PluginMetadata> {
        let type_id = TypeId::of::<P>();
        self.plugins.get(&type_id)
    }

    pub fn add_system<M>(
        mut self,
        stage: Stage,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> Self {
        if let Some(schedule) = self.schedules.get_mut(stage) {
            schedule.add_systems(system);
        }
        self
    }

    pub fn add_systems<M>(
        mut self,
        stage: Stage,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> Self {
        if let Some(schedule) = self.schedules.get_mut(stage) {
            schedule.add_systems(systems);
        }
        self
    }

    pub fn is_client(&self) -> bool {
        self.mode == ResonanceMode::Client
    }

    pub fn is_server(&self) -> bool {
        self.mode == ResonanceMode::Server
    }

    pub fn run_schedule(&mut self, stage: Stage) {
        if let Some(schedule) = self.schedules.get_mut(stage) {
            self.runner
                .run_schedule(schedule, &mut self.world, stage.name());
        }
    }

    pub fn update(&mut self) {
        self.runner.run(&mut self.world, &mut self.schedules);
    }

    pub fn startup(&mut self) {
        self.running = true;
        self.run_schedule(Stage::Startup);
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn run(mut self) {
        if self.has_plugin::<crate::window::WindowPlugin>() {
            return crate::window::runner::run(self);
        }

        self.startup();

        while self.is_running() {
            self.update();

            // TBD: Allow configurable tick-rate in headless
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }
}

impl Default for Resonance {
    fn default() -> Self {
        Self::new()
    }
}
