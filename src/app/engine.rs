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
use std::{any::TypeId, collections::HashMap, time::Duration};

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
    /// Target frame time for headless mode (used to calculate sleep duration)
    target_frametime: Duration,
}

impl Resonance {
    /// Creates a new engine instance in client mode (with rendering)
    pub fn new() -> Self {
        Self::new_with_mode(ResonanceMode::Client)
    }

    /// Creates a builder for configuring the engine before initialization
    ///
    /// # Example
    /// ```no_run
    /// use resonance::prelude::*;
    ///
    /// Resonance::builder()
    ///     .with_mode(ResonanceMode::Server)
    ///     .with_tickrate(20) // 20 FPS for server
    ///     .build()
    ///     .add_plugin(DefaultPlugins)
    ///     .run();
    /// ```
    pub fn builder() -> ResonanceBuilder {
        ResonanceBuilder::default()
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
            target_frametime: Duration::from_millis(16), // Default 62.5 FPS
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

    pub fn with_graphics_settings(mut self, settings: crate::renderer::GraphicsSettings) -> Self {
        self.world.insert_resource(settings);
        self
    }

    pub fn with_resource<R: bevy_ecs::prelude::Resource>(mut self, resource: R) -> Self {
        self.world.insert_resource(resource);
        self
    }

    /// Sets the target tickrate for headless (server) mode
    ///
    /// # Arguments
    /// * `fps` - Target frames per second (e.g., 20 for slower servers, 128 for competitive games)
    ///
    /// # Example
    /// ```no_run
    /// Resonance::new_with_mode(ResonanceMode::Server)
    ///     .with_tickrate(20)
    ///     .run();
    /// ```
    pub fn with_tickrate(mut self, fps: u32) -> Self {
        self.target_frametime = Duration::from_secs_f32(1.0 / fps.max(1) as f32);
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
                    plugin_short_name,
                    dep_short_name
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

    /// Spawns an empty entity
    pub fn spawn_entity(&mut self) -> bevy_ecs::world::EntityWorldMut<'_> {
        self.world.spawn_empty()
    }

    /// Convenience method to spawn a camera entity with common settings
    ///
    /// # Arguments
    /// * `position` - Camera position in world space
    /// * `target` - Point the camera should look at
    ///
    /// # Returns
    /// Entity ID of the spawned camera
    ///
    /// # Example
    /// ```no_run
    /// let camera = engine.spawn_camera(
    ///     Vec3::new(0.0, 10.0, 10.0),
    ///     Vec3::ZERO
    /// );
    /// ```
    pub fn spawn_camera(&mut self, position: glam::Vec3, target: glam::Vec3) -> Entity {
        use crate::transform::{Transform, GlobalTransform};
        use crate::renderer::Camera;

        let mut transform = Transform::from_position(position);
        transform.look_at(target, glam::Vec3::Y);

        self.world.spawn((
            transform,
            GlobalTransform::default(),
            Camera::new(70.0_f32.to_radians(), 16.0 / 9.0, 0.1, 1000.0),
        )).id()
    }

    /// Convenience method to spawn a mesh entity
    ///
    /// # Arguments
    /// * `mesh` - Mesh asset handle
    /// * `position` - Position in world space
    ///
    /// # Returns
    /// Entity ID of the spawned mesh
    ///
    /// # Example
    /// ```no_run
    /// let mesh_handle = assets.load(MeshLoader::new(), "models/cube.obj");
    /// let entity = engine.spawn_mesh(mesh_handle, Vec3::new(0.0, 0.0, 0.0));
    /// ```
    pub fn spawn_mesh(
        &mut self,
        mesh: crate::assets::AssetHandle<Vec<crate::assets::MeshData>>,
        position: glam::Vec3
    ) -> Entity {
        use crate::transform::{Transform, GlobalTransform};
        use crate::renderer::{Mesh, Aabb};

        self.world.spawn((
            Mesh::new(mesh),
            Transform::from_position(position),
            GlobalTransform::default(),
            Aabb::new(glam::Vec3::ZERO, glam::Vec3::ZERO),
        )).id()
    }

    /// Convenience method to spawn a directional light (sun-like)
    ///
    /// # Arguments
    /// * `direction` - Direction the light points
    /// * `color` - Light color (RGB)
    /// * `intensity` - Light intensity
    pub fn spawn_directional_light(
        &mut self,
        direction: glam::Vec3,
        color: glam::Vec3,
        intensity: f32,
    ) -> Entity {
        use crate::renderer::DirectionalLight;

        self.world.spawn(DirectionalLight {
            direction: direction.normalize(),
            color,
            intensity,
            cast_shadows: true,
        }).id()
    }

    pub fn run(mut self) {
        if self.has_plugin::<crate::window::WindowPlugin>() {
            return crate::window::runner::run(self);
        }

        self.startup();

        while self.is_running() {
            let frame_start = std::time::Instant::now();

            self.update();

            // Sleep to maintain target framerate in headless mode
            let elapsed = frame_start.elapsed();
            if elapsed < self.target_frametime {
                std::thread::sleep(self.target_frametime - elapsed);
            }
        }
    }
}

impl Default for Resonance {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for configuring the Resonance engine before initialization
///
/// Provides a fluent API for setting up engine configuration, ensuring
/// settings are applied in the correct order.
///
/// # Example
/// ```no_run
/// use resonance::prelude::*;
///
/// let engine = Resonance::builder()
///     .with_mode(ResonanceMode::Server)
///     .with_log_level(log::LevelFilter::Info)
///     .with_tickrate(20)
///     .build();
/// ```
pub struct ResonanceBuilder {
    mode: ResonanceMode,
    log_level: Option<log::LevelFilter>,
    graphics_settings: Option<crate::renderer::GraphicsSettings>,
    tickrate: Option<u32>,
}

impl Default for ResonanceBuilder {
    fn default() -> Self {
        Self {
            mode: ResonanceMode::Client,
            log_level: None,
            graphics_settings: None,
            tickrate: None,
        }
    }
}

impl ResonanceBuilder {
    /// Sets the engine mode (Client with rendering, or Server headless)
    pub fn with_mode(mut self, mode: ResonanceMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the logging level for the engine
    pub fn with_log_level(mut self, level: log::LevelFilter) -> Self {
        self.log_level = Some(level);
        self
    }

    /// Sets graphics settings (MSAA, VSync, etc.)
    pub fn with_graphics_settings(mut self, settings: crate::renderer::GraphicsSettings) -> Self {
        self.graphics_settings = Some(settings);
        self
    }

    /// Sets the target tickrate for headless mode (frames per second)
    pub fn with_tickrate(mut self, fps: u32) -> Self {
        self.tickrate = Some(fps);
        self
    }

    /// Builds the engine with the configured settings
    pub fn build(self) -> Resonance {
        // Initialize logger first
        if let Some(level) = self.log_level {
            crate::core::init_logger(level);
        }

        let mut engine = Resonance::new_with_mode(self.mode);

        if let Some(settings) = self.graphics_settings {
            engine.world.insert_resource(settings);
        }

        if let Some(tickrate) = self.tickrate {
            engine.target_frametime = Duration::from_secs_f32(1.0 / tickrate.max(1) as f32);
        }

        engine
    }
}
