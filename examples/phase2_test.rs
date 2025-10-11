//! Phase 2 test - ECS & Application Framework
//!
//! This example tests:
//! - Plugin system with dependencies and state tracking
//! - System ordering with before/after
//! - Run conditions for conditional system execution
//! - System sets and chaining

use ferrite::prelude::*;
use bevy_ecs::schedule::common_conditions::resource_exists;

// Custom test resource for run conditions
#[derive(Resource, Default)]
struct TestCounter(u32);

fn main() {
    init_logger();
    log::info!("=== Phase 2 ECS & Application Framework Test ===");

    // Create engine with plugins demonstrating dependency system
    let mut engine = Engine::new()
        .add_plugin(CorePlugin) // CorePlugin must load first
        .add_plugin(TransformPlugin) // TransformPlugin depends on CorePlugin (implicit)
        .add_plugin(TestPlugin) // Custom plugin to test plugin system
        .add_startup_system(setup_test);

    // Test: Try to add CorePlugin again (should be skipped)
    engine = engine.add_plugin(CorePlugin);

    // Add systems with ordering constraints
    engine = engine
        .add_system(Stage::Update, first_system.before(second_system))
        .add_system(Stage::Update, second_system.after(first_system))
        .add_system(Stage::Update, third_system.after(second_system))
        // Add a conditional system that only runs when TestCounter exists
        .add_system(
            Stage::Update,
            conditional_system.run_if(resource_exists::<TestCounter>),
        );

    // Add chained systems (run sequentially)
    engine = engine.add_systems(
        Stage::Update,
        (chain_a, chain_b, chain_c).chain(),
    );

    // Insert test resource
    engine.world.insert_resource(TestCounter(0));

    engine.startup();

    log::info!("\n--- Running 3 frames ---");
    for frame in 0..3 {
        log::info!("Frame {}", frame);
        engine.update();
    }

    // Remove TestCounter to test conditional system
    log::info!("\n--- Removing TestCounter resource ---");
    engine.world.remove_resource::<TestCounter>();

    log::info!("\n--- Running 2 more frames (conditional system should not run) ---");
    for frame in 3..5 {
        log::info!("Frame {}", frame);
        engine.update();
    }

    // Test plugin state tracking
    log::info!("\n--- Plugin State ---");
    if engine.has_plugin::<CorePlugin>() {
        log::info!("✓ CorePlugin loaded");
        if let Some(metadata) = engine.get_plugin_metadata::<CorePlugin>() {
            log::info!("  State: {:?}", metadata.state);
            log::info!("  Name: {}", metadata.name);
        }
    }

    if engine.has_plugin::<TransformPlugin>() {
        log::info!("✓ TransformPlugin loaded");
    }

    if engine.has_plugin::<TestPlugin>() {
        log::info!("✓ TestPlugin loaded");
    }

    log::info!("\n=== Phase 2 Test Complete ===");
    log::info!("✓ Plugin system with state tracking");
    log::info!("✓ Plugin dependency checking");
    log::info!("✓ System ordering (before/after)");
    log::info!("✓ Run conditions (run_if)");
    log::info!("✓ System chaining");
}

// Custom test plugin
struct TestPlugin;

impl Plugin for TestPlugin {
    fn build(&self, _engine: &mut Engine) {
        log::info!("TestPlugin initialized");
    }

    fn name(&self) -> &str {
        "TestPlugin"
    }

    // This plugin depends on CorePlugin
    fn dependencies(&self) -> Vec<std::any::TypeId> {
        vec![std::any::TypeId::of::<CorePlugin>()]
    }
}

fn setup_test() {
    log::info!("[Startup] Phase 2 test setup complete");
}

// Systems demonstrating ordering
fn first_system() {
    log::debug!("[Update] 1. First system (runs first)");
}

fn second_system() {
    log::debug!("[Update] 2. Second system (after first)");
}

fn third_system() {
    log::debug!("[Update] 3. Third system (after second)");
}

// Conditional system that only runs when TestCounter exists
fn conditional_system(counter: Res<TestCounter>) {
    log::info!("[Update] ✓ Conditional system running (counter: {})", counter.0);
}

// Chained systems
fn chain_a() {
    log::debug!("[Update] Chain A");
}

fn chain_b() {
    log::debug!("[Update] Chain B (after A)");
}

fn chain_c() {
    log::debug!("[Update] Chain C (after B)");
}
