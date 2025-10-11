//! Scene management demonstration.
//!
//! This example shows how to:
//! - Create and save scenes in RON and Bincode formats
//! - Compare performance between RON (dev) and Bincode (production)
//! - Use the SceneManager
//! - Create and use prefabs
//! - Transition between scenes
//! - Convert between formats for build pipelines

use ferrite_app::Engine;
use ferrite_scene::{
    Prefab, PrefabRegistry, PrefabSpawner, Scene, SceneConverter, SceneEntity,
    SceneManager, ScenePlugin, SerializationFormat,
};
use std::collections::HashMap;
use std::time::Instant;

fn main() {
    env_logger::init();

    // Initialize engine and add scene plugin
    let mut engine = Engine::new()
        .add_plugin(ScenePlugin);

    // Demo 0: Format comparison (RON vs Bincode)
    demo_format_comparison();

    // Demo 1: Create and save a scene
    demo_create_and_save_scene(&mut engine);

    // Demo 2: Load a scene
    demo_load_scene(&mut engine);

    // Demo 3: Create and use prefabs
    demo_prefabs(&mut engine);

    // Demo 4: Scene transitions
    demo_scene_transitions(&mut engine);

    log::info!("Scene demo completed successfully!");
}

/// Demo 1: Create and save a scene to RON format
fn demo_create_and_save_scene(engine: &mut Engine) {
    log::info!("\n=== Demo 1: Create and Save Scene ===");

    // Create a new scene
    let mut scene = Scene::new("main_menu");

    // Add some entities to the scene
    let mut button_entity = SceneEntity {
        name: Some("start_button".to_string()),
        components: HashMap::new(),
    };

    // Add some component data (as RON values)
    button_entity.components.insert(
        "Transform".to_string(),
        ron::Value::Map(vec![
            (
                ron::Value::String("position".to_string()),
                ron::Value::Seq(vec![
                    ron::Value::Number(ron::Number::new(0.0)),
                    ron::Value::Number(ron::Number::new(0.0)),
                    ron::Value::Number(ron::Number::new(0.0)),
                ]),
            ),
        ].into_iter().collect()),
    );

    button_entity.components.insert(
        "Button".to_string(),
        ron::Value::Map(vec![
            (
                ron::Value::String("label".to_string()),
                ron::Value::String("Start Game".to_string()),
            ),
        ].into_iter().collect()),
    );

    scene.add_entity(button_entity);

    // Add another entity
    let mut background_entity = SceneEntity {
        name: Some("background".to_string()),
        components: HashMap::new(),
    };

    background_entity.components.insert(
        "Sprite".to_string(),
        ron::Value::Map(vec![
            (
                ron::Value::String("texture".to_string()),
                ron::Value::String("background.png".to_string()),
            ),
        ].into_iter().collect()),
    );

    scene.add_entity(background_entity);

    // Serialize to RON
    match scene.to_ron() {
        Ok(ron_data) => {
            log::info!("Scene serialized to RON:\n{}", ron_data);

            // Register the scene with the manager
            let mut scene_manager = engine.world.resource_mut::<SceneManager>();
            scene_manager.register_scene(scene);
            log::info!("Scene 'main_menu' registered");
        }
        Err(e) => log::error!("Failed to serialize scene: {}", e),
    }
}

/// Demo 2: Load a scene from RON data
fn demo_load_scene(engine: &mut Engine) {
    log::info!("\n=== Demo 2: Load Scene from RON ===");

    // Example RON data for a game level scene
    let ron_data = r#"
(
    name: "level_1",
    entities: [
        (
            name: Some("player"),
            components: {
                "Transform": Map({
                    String("position"): Seq([Number(0.0), Number(0.0), Number(0.0)]),
                    String("rotation"): Seq([Number(0.0), Number(0.0), Number(0.0)]),
                }),
                "Player": Map({
                    String("health"): Number(100.0),
                    String("speed"): Number(5.0),
                }),
            },
        ),
        (
            name: Some("enemy_1"),
            components: {
                "Transform": Map({
                    String("position"): Seq([Number(10.0), Number(0.0), Number(5.0)]),
                }),
                "Enemy": Map({
                    String("health"): Number(50.0),
                    String("damage"): Number(10.0),
                }),
            },
        ),
    ],
)
"#;

    // Load the scene
    let mut scene_manager = engine.world.resource_mut::<SceneManager>();
    match scene_manager.load_scene_from_ron("level_1", ron_data) {
        Ok(_) => {
            log::info!("Scene 'level_1' loaded successfully");

            // Activate the scene
            if let Err(e) = scene_manager.activate_scene("level_1") {
                log::error!("Failed to activate scene: {}", e);
            } else {
                log::info!("Scene 'level_1' is now active");
            }
        }
        Err(e) => log::error!("Failed to load scene: {}", e),
    }
}

/// Demo 3: Create and use prefabs
fn demo_prefabs(engine: &mut Engine) {
    log::info!("\n=== Demo 3: Prefabs ===");

    let mut prefab_registry = engine.world.resource_mut::<PrefabRegistry>();

    // Create an enemy prefab
    let enemy_prefab = Prefab::new("basic_enemy")
        .with_component(
            "Transform",
            ron::Value::Map(vec![
                (
                    ron::Value::String("position".to_string()),
                    ron::Value::Seq(vec![
                        ron::Value::Number(ron::Number::new(0.0)),
                        ron::Value::Number(ron::Number::new(0.0)),
                        ron::Value::Number(ron::Number::new(0.0)),
                    ]),
                ),
            ].into_iter().collect()),
        )
        .with_component(
            "Enemy",
            ron::Value::Map(vec![
                (
                    ron::Value::String("health".to_string()),
                    ron::Value::Number(ron::Number::new(50.0)),
                ),
                (
                    ron::Value::String("damage".to_string()),
                    ron::Value::Number(ron::Number::new(10.0)),
                ),
            ].into_iter().collect()),
        );

    // Register the prefab
    prefab_registry.register(enemy_prefab);
    log::info!("Registered prefab: basic_enemy");

    // Create a more complex prefab with children
    let turret_base = Prefab::new("turret_base")
        .with_component(
            "Transform",
            ron::Value::Map(vec![
                (
                    ron::Value::String("position".to_string()),
                    ron::Value::Seq(vec![
                        ron::Value::Number(ron::Number::new(0.0)),
                        ron::Value::Number(ron::Number::new(0.0)),
                        ron::Value::Number(ron::Number::new(0.0)),
                    ]),
                ),
            ].into_iter().collect()),
        );

    let turret_gun = Prefab::new("turret_gun")
        .with_component(
            "Transform",
            ron::Value::Map(vec![
                (
                    ron::Value::String("position".to_string()),
                    ron::Value::Seq(vec![
                        ron::Value::Number(ron::Number::new(0.0)),
                        ron::Value::Number(ron::Number::new(1.0)),
                        ron::Value::Number(ron::Number::new(0.0)),
                    ]),
                ),
            ].into_iter().collect()),
        );

    let turret_prefab = turret_base.with_child(turret_gun);
    prefab_registry.register(turret_prefab);
    log::info!("Registered prefab: turret_base (with child)");

    // Demonstrate prefab retrieval
    if let Some(prefab) = prefab_registry.spawn_prefab("basic_enemy") {
        log::info!("Retrieved prefab 'basic_enemy': {} components", prefab.components.len());
    }

    // List all registered prefabs
    let prefab_names = prefab_registry.prefab_names();
    log::info!("Registered prefabs: {:?}", prefab_names);
}

/// Demo 4: Scene transitions
fn demo_scene_transitions(engine: &mut Engine) {
    log::info!("\n=== Demo 4: Scene Transitions ===");

    // First, register a second scene
    let scene2_ron = r#"
(
    name: "level_2",
    entities: [
        (
            name: Some("player"),
            components: {
                "Transform": Map({
                    String("position"): Seq([Number(0.0), Number(0.0), Number(0.0)]),
                }),
            },
        ),
    ],
)
"#;

    let mut scene_manager = engine.world.resource_mut::<SceneManager>();

    if let Err(e) = scene_manager.load_scene_from_ron("level_2", scene2_ron) {
        log::error!("Failed to load level_2: {}", e);
        return;
    }

    log::info!("Loaded scene: level_2");

    // Check current scene
    if let Some(current) = scene_manager.active_scene() {
        log::info!("Current active scene: {}", current);
    }

    // Transition to level_2
    match scene_manager.transition_to("level_2") {
        Ok(_) => {
            log::info!("Transition initiated to level_2");
            log::info!("Transition state: {:?}", scene_manager.transition_state());

            // Complete the transition
            if let Err(e) = scene_manager.complete_transition() {
                log::error!("Failed to complete transition: {}", e);
            } else {
                log::info!("Transition completed!");
                if let Some(current) = scene_manager.active_scene() {
                    log::info!("New active scene: {}", current);
                }
            }
        }
        Err(e) => log::error!("Failed to initiate transition: {}", e),
    }

    // List all loaded scenes
    let loaded = scene_manager.loaded_scene_names();
    log::info!("All loaded scenes: {:?}", loaded);
}

/// Demo 0: Format comparison and performance benchmarks
fn demo_format_comparison() {
    log::info!("\n=== Demo 0: Format Comparison (RON vs Bincode) ===");

    // Create a test scene with multiple entities
    let mut scene = Scene::new("benchmark_scene");

    // Add 100 entities to get meaningful size/performance data
    for i in 0..100 {
        let mut entity = SceneEntity {
            name: Some(format!("entity_{}", i)),
            components: HashMap::new(),
        };

        // Add Transform component
        entity.components.insert(
            "Transform".to_string(),
            ron::Value::Map(vec![
                (
                    ron::Value::String("position".to_string()),
                    ron::Value::Seq(vec![
                        ron::Value::Number(ron::Number::new(i as f64 * 10.0)),
                        ron::Value::Number(ron::Number::new(i as f64 * 5.0)),
                        ron::Value::Number(ron::Number::new(0.0)),
                    ]),
                ),
                (
                    ron::Value::String("rotation".to_string()),
                    ron::Value::Seq(vec![
                        ron::Value::Number(ron::Number::new(0.0)),
                        ron::Value::Number(ron::Number::new(0.0)),
                        ron::Value::Number(ron::Number::new(0.0)),
                    ]),
                ),
            ].into_iter().collect()),
        );

        // Add Enemy component
        entity.components.insert(
            "Enemy".to_string(),
            ron::Value::Map(vec![
                (
                    ron::Value::String("health".to_string()),
                    ron::Value::Number(ron::Number::new(100.0)),
                ),
                (
                    ron::Value::String("damage".to_string()),
                    ron::Value::Number(ron::Number::new(10.0 + (i as f64 % 5.0))),
                ),
            ].into_iter().collect()),
        );

        scene.add_entity(entity);
    }

    log::info!("Created test scene with {} entities", scene.entities.len());

    // === RON Serialization ===
    log::info!("\n--- RON Format (Human-readable, Development) ---");

    let start = Instant::now();
    let ron_data = scene.to_ron().expect("Failed to serialize to RON");
    let ron_serialize_time = start.elapsed();
    let ron_bytes = ron_data.as_bytes();

    log::info!("RON Serialization: {:?}", ron_serialize_time);
    log::info!("RON Size: {} bytes ({:.2} KB)", ron_bytes.len(), ron_bytes.len() as f32 / 1024.0);

    let start = Instant::now();
    let _scene_from_ron = Scene::from_ron(&ron_data).expect("Failed to deserialize from RON");
    let ron_deserialize_time = start.elapsed();
    log::info!("RON Deserialization: {:?}", ron_deserialize_time);

    // === Bincode Serialization ===
    log::info!("\n--- Bincode Format (Binary, Production) ---");

    let start = Instant::now();
    let bincode_data = scene.to_bincode().expect("Failed to serialize to Bincode");
    let bincode_serialize_time = start.elapsed();

    log::info!("Bincode Serialization: {:?}", bincode_serialize_time);
    log::info!("Bincode Size: {} bytes ({:.2} KB)", bincode_data.len(), bincode_data.len() as f32 / 1024.0);

    let start = Instant::now();
    let bincode_deserialize_result = Scene::from_bincode(&bincode_data);
    let bincode_deserialize_time = start.elapsed();

    let bincode_success = bincode_deserialize_result.is_ok();
    match bincode_deserialize_result {
        Ok(_) => {
            log::info!("Bincode Deserialization: {:?}", bincode_deserialize_time);
        }
        Err(e) => {
            // Note: ron::Value contains dynamic types that bincode's serde layer doesn't fully support
            // In production, you'd use typed component data instead of ron::Value for full bincode support
            log::warn!("Bincode deserialization has limitations with ron::Value: {}", e);
            log::info!("Note: For full Bincode support, use typed component data instead of ron::Value");
            log::info!("Bincode Deserialization attempt: {:?}", bincode_deserialize_time);
        }
    }

    // === Performance Comparison ===
    log::info!("\n--- Performance Comparison ---");

    let savings = SceneConverter::calculate_savings(ron_bytes, &bincode_data);
    log::info!("{}", savings.format());

    let serialize_speedup = ron_serialize_time.as_secs_f64() / bincode_serialize_time.as_secs_f64();

    log::info!("Serialization: Bincode is {:.2}x faster than RON", serialize_speedup);

    // Only show deserialization speedup if bincode deserialization succeeded
    if bincode_success {
        let deserialize_speedup = ron_deserialize_time.as_secs_f64() / bincode_deserialize_time.as_secs_f64();
        log::info!("Deserialization: Bincode is {:.2}x faster than RON", deserialize_speedup);
    }

    // === Format Conversion ===
    log::info!("\n--- Format Conversion ---");

    let start = Instant::now();
    let (baked_data, compression_ratio) = SceneConverter::bake_scene(ron_bytes)
        .expect("Failed to bake scene");
    let conversion_time = start.elapsed();

    log::info!("Conversion time: {:?}", conversion_time);
    log::info!("Compression ratio: {:.2}x", compression_ratio);
    log::info!("Baked size: {} bytes", baked_data.len());

    // === Recommendations ===
    log::info!("\n--- Recommendations ---");
    log::info!("✓ Use RON (.ron) for development - easy to read and edit");
    log::info!("✓ Use Bincode (.scene) for production - {:.1}x smaller, {:.1}x faster serialization",
        compression_ratio, serialize_speedup);
    log::info!("✓ Add scene baking to your build pipeline for optimal performance");
    log::info!("");
    log::info!("Note: Current implementation uses ron::Value for component storage,");
    log::info!("which has limited bincode support. For production use with Bincode,");
    log::info!("consider using typed component data (custom enums/structs) instead.");
}
