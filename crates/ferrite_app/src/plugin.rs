//! Plugin system for modular engine features.
//!
//! Plugins allow you to package related functionality (components, systems, resources)
//! into reusable modules that can be easily added to the engine.

use crate::engine::Engine;
use std::any::TypeId;

/// Trait for engine plugins.
///
/// Plugins can register components, systems, and resources with the engine.
///
/// # Example
/// ```ignore
/// pub struct MyGamePlugin;
///
/// impl Plugin for MyGamePlugin {
///     fn build(&self, engine: &mut Engine) {
///         engine
///             .add_system(Stage::Update, my_system)
///             .add_startup_system(setup);
///     }
/// }
/// ```
pub trait Plugin: Send + Sync + 'static {
    /// Build the plugin by registering its functionality with the engine
    fn build(&self, engine: &mut Engine);

    /// Optional plugin name for debugging
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Get the unique type ID of this plugin
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    /// Optional dependencies - plugins that must be loaded before this one
    fn dependencies(&self) -> Vec<TypeId> {
        Vec::new()
    }

    /// Whether this plugin should run in client mode
    fn is_client_plugin(&self) -> bool {
        true
    }

    /// Whether this plugin should run in server mode
    fn is_server_plugin(&self) -> bool {
        true
    }
}

/// Plugin state tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// Plugin is ready to be built
    Ready,
    /// Plugin is currently building
    Building,
    /// Plugin has finished building
    Built,
    /// Plugin build failed
    Failed,
}

/// Plugin metadata for tracking and dependency resolution
pub struct PluginMetadata {
    pub type_id: TypeId,
    pub name: String,
    pub state: PluginState,
    pub dependencies: Vec<TypeId>,
}

/// Plugin for core functionality (time, logging)
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, engine: &mut Engine) {
        use ferrite_core::{FixedTime, GameTick, Time};

        // Add time resources
        engine.world.insert_resource(Time::new());
        engine.world.insert_resource(FixedTime::default());
        engine.world.insert_resource(GameTick::new());

        log::info!("CorePlugin initialized");
    }
}
