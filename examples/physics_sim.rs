//! Example showing basic physics simulation.

use ferrite::prelude::*;

fn main() {
    init_logger();

    log::info!("Starting Physics Simulation");

    let mut engine = Engine::new()
        .add_plugin(CorePlugin)
        .add_plugin(TransformPlugin)
        .add_plugin(PhysicsPlugin::default())
        .add_startup_system(spawn_physics_objects)
        .add_system(Stage::Update, log_positions);

    engine.startup();

    // Run simulation
    for i in 0..30 {
        engine.update();
        if i % 10 == 0 {
            log::info!("=== Frame {} ===", i);
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn spawn_physics_objects(mut commands: Commands) {
    log::info!("Spawning physics objects...");

    // Spawn a falling object
    commands.spawn((
        Transform::from_position(Vec3::new(0.0, 10.0, 0.0)),
        GlobalTransform::default(),
        RigidBody::Dynamic,
        Velocity::default(),
        Acceleration::default(),
        Mass::default(),
        // ApplyGravity, // TODO: Uncomment when gravity system is active
    ));
}

fn log_positions(query: Query<(Entity, &Transform, &Velocity)>) {
    for (entity, transform, velocity) in query.iter() {
        log::debug!(
            "Entity {:?} - Pos: {:?}, Vel: {:?}",
            entity,
            transform.position,
            velocity.linear
        );
    }
}
