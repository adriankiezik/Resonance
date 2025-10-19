use super::{
    plugin::{Plugin, PluginMetadata, PluginState},
    stage::Stage,
};
use bevy_ecs::{
    prelude::*,
    schedule::{IntoScheduleConfigs, Schedule},
    system::ScheduleSystem,
};
use std::{any::TypeId, collections::HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineMode {
    Client,
    Server,
}

pub struct Engine {
    pub world: World,
    pub schedules: Schedules,
    pub mode: EngineMode,
    pub running: bool,
    plugins: HashMap<TypeId, PluginMetadata>,
}

impl Engine {
    pub fn new() -> Self {
        Self::with_mode(EngineMode::Client)
    }

    pub fn with_mode(mode: EngineMode) -> Self {
        let world = World::new();
        let mut schedules = Schedules::new();

        schedules.insert(Schedule::new(Stage::Startup));
        schedules.insert(Schedule::new(Stage::PreUpdate));
        schedules.insert(Schedule::new(Stage::Update));
        schedules.insert(Schedule::new(Stage::PostUpdate));
        schedules.insert(Schedule::new(Stage::FixedUpdate));
        schedules.insert(Schedule::new(Stage::Render));
        schedules.insert(Schedule::new(Stage::Last));

        Self {
            world,
            schedules,
            mode,
            running: false,
            plugins: HashMap::new(),
        }
    }

    pub fn set_mode(mut self, mode: EngineMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn add_plugin<P: Plugin>(mut self, plugin: P) -> Self {
        let type_id = plugin.type_id();
        let name = plugin.name().to_string();

        if self.plugins.contains_key(&type_id) {
            log::warn!("Plugin {} already loaded, skipping", name);
            return self;
        }

        let should_load = match self.mode {
            EngineMode::Client => plugin.is_client_plugin(),
            EngineMode::Server => plugin.is_server_plugin(),
        };

        if !should_load {
            log::debug!(
                "Plugin {} skipped (not compatible with {:?} mode)",
                name,
                self.mode
            );
            return self;
        }

        let dependencies = plugin.dependencies();
        for dep_id in &dependencies {
            if !self.plugins.contains_key(dep_id) {
                log::error!(
                    "Plugin {} missing dependency (TypeId: {:?})",
                    name,
                    dep_id
                );
                self.plugins.insert(
                    type_id,
                    PluginMetadata {
                        type_id,
                        name,
                        state: PluginState::Failed,
                        dependencies,
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
                dependencies: dependencies.clone(),
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

    pub fn add_startup_system<M>(
        mut self,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> Self {
        if let Some(schedule) = self.schedules.get_mut(Stage::Startup) {
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
        self.mode == EngineMode::Client
    }

    pub fn is_server(&self) -> bool {
        self.mode == EngineMode::Server
    }

    pub fn run_schedule(&mut self, stage: Stage) {
        if let Some(schedule) = self.schedules.get_mut(stage) {
            schedule.run(&mut self.world);
        }
    }

    pub fn update(&mut self) {
        {
            let mut time = self.world.resource_mut::<crate::core::Time>();
            time.update();
        }

        self.run_schedule(Stage::PreUpdate);
        self.run_schedule(Stage::Update);

        {
            let delta = self.world.resource::<crate::core::Time>().delta();
            let mut fixed_time = self.world.resource_mut::<crate::core::FixedTime>();
            fixed_time.accumulate(delta);
        }

        while self
            .world
            .resource::<crate::core::FixedTime>()
            .should_update()
        {
            {
                let mut tick = self.world.resource_mut::<crate::core::GameTick>();
                tick.increment();
            }

            self.run_schedule(Stage::FixedUpdate);

            self.world
                .resource_mut::<crate::core::FixedTime>()
                .consume_step();
        }

        self.run_schedule(Stage::PostUpdate);

        if self.is_client() {
            self.run_schedule(Stage::Render);
        }

        self.run_schedule(Stage::Last);
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
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
