//! Example demonstrating ECS usage with components and systems.

use ferrite::prelude::*;

// Custom component
#[derive(Component, Debug)]
struct Player {
    name: String,
    score: u32,
}

#[derive(Component, Debug)]
struct Enemy {
    health: f32,
}

fn main() {
    init_logger();

    log::info!("Starting ECS Demo");

    let mut engine = Engine::new()
        .add_plugin(CorePlugin)
        .add_startup_system(spawn_entities)
        .add_system(Stage::Update, player_system)
        .add_system(Stage::Update, enemy_system);

    // Run startup
    engine.startup();

    // Run a few frames
    for _ in 0..5 {
        engine.update();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn spawn_entities(mut commands: Commands) {
    log::info!("Spawning entities...");

    // Spawn player
    commands.spawn((
        Player {
            name: "Hero".to_string(),
            score: 0,
        },
        Transform::from_position(Vec3::new(0.0, 0.0, 0.0)),
    ));

    // Spawn enemies
    for i in 0..3 {
        commands.spawn((
            Enemy { health: 100.0 },
            Transform::from_position(Vec3::new(i as f32 * 2.0, 0.0, 5.0)),
        ));
    }
}

fn player_system(mut query: Query<(&mut Player, &Transform)>) {
    for (mut player, transform) in query.iter_mut() {
        player.score += 1;
        log::debug!(
            "Player '{}' at {:?}, score: {}",
            player.name,
            transform.position,
            player.score
        );
    }
}

fn enemy_system(mut query: Query<(&mut Enemy, &Transform)>) {
    for (mut enemy, transform) in query.iter_mut() {
        enemy.health -= 1.0;
        log::debug!(
            "Enemy at {:?}, health: {}",
            transform.position,
            enemy.health
        );
    }
}
