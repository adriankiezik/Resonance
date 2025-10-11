//! Main engine struct and builder.

use crate::{
    plugin::{Plugin, PluginMetadata, PluginState},
    stage::Stage,
};
use bevy_ecs::{
    prelude::*,
    schedule::{IntoScheduleConfigs, Schedule},
    system::ScheduleSystem,
};
use std::{any::TypeId, collections::HashMap};

/// Engine execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineMode {
    /// Full client with rendering, audio, input
    Client,
    /// Headless server (no rendering, no audio)
    Server,
}

/// Main engine struct that holds the ECS world and schedules.
///
/// Built using the builder pattern, the engine manages the game loop
/// and provides a plugin system for extensibility.
pub struct Engine {
    /// The ECS world containing all entities, components, and resources
    pub world: World,
    /// Schedules for different stages
    pub schedules: Schedules,
    /// Engine mode (client or server)
    pub mode: EngineMode,
    /// Whether the engine is running
    pub running: bool,
    /// Plugin registry for tracking loaded plugins
    plugins: HashMap<TypeId, PluginMetadata>,
}

impl Engine {
    /// Create a new engine in client mode
    pub fn new() -> Self {
        Self::with_mode(EngineMode::Client)
    }

    /// Create a new engine with a specific mode
    pub fn with_mode(mode: EngineMode) -> Self {
        let world = World::new();
        let mut schedules = Schedules::new();

        // Initialize all schedules
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

    /// Set the engine mode
    pub fn set_mode(mut self, mode: EngineMode) -> Self {
        self.mode = mode;
        self
    }

    /// Add a plugin to the engine
    pub fn add_plugin<P: Plugin>(mut self, plugin: P) -> Self {
        let type_id = plugin.type_id();
        let name = plugin.name().to_string();

        // Check if plugin is already loaded
        if self.plugins.contains_key(&type_id) {
            log::warn!("Plugin {} already loaded, skipping", name);
            return self;
        }

        // Check if plugin should run in current mode
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

        // Check dependencies
        let dependencies = plugin.dependencies();
        for dep_id in &dependencies {
            if !self.plugins.contains_key(dep_id) {
                log::error!(
                    "Plugin {} missing dependency (TypeId: {:?})",
                    name,
                    dep_id
                );
                // Add metadata with failed state
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

        log::debug!("Adding plugin: {}", name);

        // Update plugin state to building
        self.plugins.insert(
            type_id,
            PluginMetadata {
                type_id,
                name: name.clone(),
                state: PluginState::Building,
                dependencies: dependencies.clone(),
            },
        );

        // Build the plugin
        plugin.build(&mut self);

        // Update plugin state to built
        if let Some(metadata) = self.plugins.get_mut(&type_id) {
            metadata.state = PluginState::Built;
        }

        log::debug!("Plugin {} built successfully", name);
        self
    }

    /// Check if a plugin is loaded
    pub fn has_plugin<P: Plugin>(&self) -> bool {
        let type_id = TypeId::of::<P>();
        self.plugins
            .get(&type_id)
            .map(|m| m.state == PluginState::Built)
            .unwrap_or(false)
    }

    /// Get plugin metadata
    pub fn get_plugin_metadata<P: Plugin>(&self) -> Option<&PluginMetadata> {
        let type_id = TypeId::of::<P>();
        self.plugins.get(&type_id)
    }

    /// Add a system to a specific stage
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

    /// Add a startup system (runs once before the main loop)
    pub fn add_startup_system<M>(
        mut self,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> Self {
        if let Some(schedule) = self.schedules.get_mut(Stage::Startup) {
            schedule.add_systems(system);
        }
        self
    }

    /// Add multiple systems to a stage
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

    /// Get the engine mode
    pub fn is_client(&self) -> bool {
        self.mode == EngineMode::Client
    }

    /// Check if running as server
    pub fn is_server(&self) -> bool {
        self.mode == EngineMode::Server
    }

    /// Run a schedule
    pub fn run_schedule(&mut self, stage: Stage) {
        if let Some(schedule) = self.schedules.get_mut(stage) {
            log::trace!("Running schedule: {:?}", stage);
            schedule.run(&mut self.world);
        }
    }

    /// Update the engine (called every frame)
    pub fn update(&mut self) {
        // Update time manually (outside of ECS systems)
        {
            let mut time = self.world.resource_mut::<ferrite_core::Time>();
            time.update();
        }

        // Run frame stages
        self.run_schedule(Stage::PreUpdate);
        self.run_schedule(Stage::Update);

        // Handle fixed update (may run multiple times per frame)
        {
            let delta = self.world.resource::<ferrite_core::Time>().delta();
            let mut fixed_time = self.world.resource_mut::<ferrite_core::FixedTime>();
            fixed_time.accumulate(delta);
        }

        while self
            .world
            .resource::<ferrite_core::FixedTime>()
            .should_update()
        {
            // Increment game tick
            {
                let mut tick = self.world.resource_mut::<ferrite_core::GameTick>();
                tick.increment();
            }

            // Run fixed update systems
            self.run_schedule(Stage::FixedUpdate);

            // Consume one timestep
            self.world
                .resource_mut::<ferrite_core::FixedTime>()
                .consume_step();
        }

        self.run_schedule(Stage::PostUpdate);

        // Only run render stage in client mode
        if self.is_client() {
            self.run_schedule(Stage::Render);
        }

        self.run_schedule(Stage::Last);
    }

    /// Start the engine (this will be called by the runner)
    pub fn startup(&mut self) {
        log::info!("Starting Ferrite Engine in {:?} mode", self.mode);
        self.running = true;
        log::debug!("Running startup schedule...");
        self.run_schedule(Stage::Startup);
        log::debug!("Startup complete");
    }

    /// Stop the engine
    pub fn stop(&mut self) {
        log::info!("Stopping engine");
        self.running = false;
    }

    /// Check if the engine is running
    pub fn is_running(&self) -> bool {
        self.running
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
