//! Tests for the Engine and plugin system

use ferrite_app::{Engine, EngineMode, Plugin, PluginState, Stage};
use std::any::TypeId;

// Test plugin
struct TestPlugin {
    name: String,
}

impl TestPlugin {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Plugin for TestPlugin {
    fn build(&self, _engine: &mut Engine) {
        // Plugin does nothing for these tests
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// Plugin that depends on TestPlugin
struct DependentPlugin;

impl Plugin for DependentPlugin {
    fn build(&self, _engine: &mut Engine) {}

    fn name(&self) -> &str {
        "DependentPlugin"
    }

    fn dependencies(&self) -> Vec<TypeId> {
        vec![TypeId::of::<TestPlugin>()]
    }
}

// Client-only plugin
struct ClientOnlyPlugin;

impl Plugin for ClientOnlyPlugin {
    fn build(&self, _engine: &mut Engine) {}

    fn is_client_plugin(&self) -> bool {
        true
    }

    fn is_server_plugin(&self) -> bool {
        false
    }
}

// Server-only plugin
struct ServerOnlyPlugin;

impl Plugin for ServerOnlyPlugin {
    fn build(&self, _engine: &mut Engine) {}

    fn is_client_plugin(&self) -> bool {
        false
    }

    fn is_server_plugin(&self) -> bool {
        true
    }
}

#[test]
fn test_engine_creation() {
    let engine = Engine::new();

    assert_eq!(engine.mode, EngineMode::Client);
    assert!(!engine.is_running());
    assert!(engine.is_client());
    assert!(!engine.is_server());
}

#[test]
fn test_engine_with_mode() {
    let engine = Engine::with_mode(EngineMode::Server);

    assert_eq!(engine.mode, EngineMode::Server);
    assert!(!engine.is_client());
    assert!(engine.is_server());
}

#[test]
fn test_engine_set_mode() {
    let engine = Engine::new().set_mode(EngineMode::Server);

    assert_eq!(engine.mode, EngineMode::Server);
}

#[test]
fn test_plugin_loading() {
    let engine = Engine::new().add_plugin(TestPlugin::new("Test1"));

    assert!(engine.has_plugin::<TestPlugin>());

    if let Some(metadata) = engine.get_plugin_metadata::<TestPlugin>() {
        assert_eq!(metadata.state, PluginState::Built);
        assert_eq!(metadata.name, "Test1");
    } else {
        panic!("Plugin metadata not found");
    }
}

#[test]
fn test_duplicate_plugin_detection() {
    let engine = Engine::new()
        .add_plugin(TestPlugin::new("Test1"))
        .add_plugin(TestPlugin::new("Test1")); // Duplicate

    // Should only be loaded once
    assert!(engine.has_plugin::<TestPlugin>());

    // Metadata should still show Built state
    if let Some(metadata) = engine.get_plugin_metadata::<TestPlugin>() {
        assert_eq!(metadata.state, PluginState::Built);
    }
}

#[test]
fn test_plugin_dependencies() {
    // Load dependent plugin WITHOUT dependency - should fail
    let engine = Engine::new().add_plugin(DependentPlugin);

    // Plugin should be in Failed state
    if let Some(metadata) = engine.get_plugin_metadata::<DependentPlugin>() {
        assert_eq!(metadata.state, PluginState::Failed);
    } else {
        panic!("Plugin metadata should exist even for failed plugins");
    }
}

#[test]
fn test_plugin_dependencies_success() {
    // Load dependency first, then dependent plugin
    let engine = Engine::new()
        .add_plugin(TestPlugin::new("Test"))
        .add_plugin(DependentPlugin);

    // Both plugins should be Built
    assert!(engine.has_plugin::<TestPlugin>());
    assert!(engine.has_plugin::<DependentPlugin>());

    if let Some(metadata) = engine.get_plugin_metadata::<DependentPlugin>() {
        assert_eq!(metadata.state, PluginState::Built);
        assert_eq!(metadata.dependencies.len(), 1);
    }
}

#[test]
fn test_client_only_plugin_in_client_mode() {
    let engine = Engine::with_mode(EngineMode::Client).add_plugin(ClientOnlyPlugin);

    // Should load in client mode
    assert!(engine.has_plugin::<ClientOnlyPlugin>());
}

#[test]
fn test_client_only_plugin_in_server_mode() {
    let engine = Engine::with_mode(EngineMode::Server).add_plugin(ClientOnlyPlugin);

    // Should NOT load in server mode
    assert!(!engine.has_plugin::<ClientOnlyPlugin>());
}

#[test]
fn test_server_only_plugin_in_server_mode() {
    let engine = Engine::with_mode(EngineMode::Server).add_plugin(ServerOnlyPlugin);

    // Should load in server mode
    assert!(engine.has_plugin::<ServerOnlyPlugin>());
}

#[test]
fn test_server_only_plugin_in_client_mode() {
    let engine = Engine::with_mode(EngineMode::Client).add_plugin(ServerOnlyPlugin);

    // Should NOT load in client mode
    assert!(!engine.has_plugin::<ServerOnlyPlugin>());
}

#[test]
fn test_engine_startup_and_stop() {
    use ferrite_core::{FixedTime, GameTick, Time};

    let mut engine = Engine::new();

    // Manually insert resources
    engine.world.insert_resource(Time::new());
    engine.world.insert_resource(FixedTime::default());
    engine.world.insert_resource(GameTick::new());

    assert!(!engine.is_running());

    engine.startup();
    assert!(engine.is_running());

    engine.stop();
    assert!(!engine.is_running());
}

#[test]
fn test_system_registration() {
    use bevy_ecs::prelude::*;

    // Test system
    fn test_system() {}

    let engine = Engine::new().add_system(Stage::Update, test_system);

    // Just verify it compiles and doesn't crash
    assert_eq!(engine.mode, EngineMode::Client);
}

#[test]
fn test_startup_system_registration() {
    use bevy_ecs::prelude::*;

    fn startup_system() {}

    let engine = Engine::new().add_startup_system(startup_system);

    // Verify it compiles
    assert_eq!(engine.mode, EngineMode::Client);
}

#[test]
fn test_multiple_systems_registration() {
    use bevy_ecs::prelude::*;

    fn system1() {}
    fn system2() {}
    fn system3() {}

    let engine = Engine::new().add_systems(Stage::Update, (system1, system2, system3));

    // Verify it compiles
    assert_eq!(engine.mode, EngineMode::Client);
}

#[test]
fn test_stage_enum() {
    use ferrite_app::Stage;

    // Verify all stages exist
    let _stages = [
        Stage::Startup,
        Stage::PreUpdate,
        Stage::Update,
        Stage::PostUpdate,
        Stage::FixedUpdate,
        Stage::Render,
        Stage::Last,
    ];

    // Stages should be Copy and Clone
    let stage = Stage::Update;
    let _stage_copy = stage;
    let _stage_clone = stage.clone();
}

#[test]
fn test_engine_builder_pattern() {
    use bevy_ecs::prelude::*;

    fn test_system() {}

    // Test fluent builder API
    let engine = Engine::new()
        .set_mode(EngineMode::Client)
        .add_plugin(TestPlugin::new("Builder"))
        .add_system(Stage::Update, test_system)
        .add_startup_system(test_system);

    assert!(engine.has_plugin::<TestPlugin>());
    assert_eq!(engine.mode, EngineMode::Client);
}
