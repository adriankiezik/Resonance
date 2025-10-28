use bevy_ecs::schedule::ScheduleLabel;

/// Game loop execution stages run in a specific order each frame.
///
/// # Execution Order
///
/// 1. **Startup** - Runs once at engine initialization (see engine.rs:197-199)
///    - Used for: Scene setup, entity spawning, resource initialization
///    - Runs: Once before the main loop begins
///
/// 2. **PreUpdate** - First stage of each frame (see runner.rs:35-38)
///    - Used for: Input handling, resource preparation, early updates
///    - Runs: Every frame, before Update
///
/// 3. **Update** - Main game logic stage (see runner.rs:39-42)
///    - Used for: Game logic, AI, player control, state updates
///    - Runs: Every frame, variable timestep
///
/// 4. **FixedUpdate** - Fixed timestep physics/simulation (see runner.rs:49-61)
///    - Used for: Physics, deterministic simulation, networking
///    - Runs: 0 or more times per frame based on accumulator
///    - Note: May run multiple times if frame took long, or not at all
///
/// 5. **PostUpdate** - After main logic (see runner.rs:64-68)
///    - Used for: Transform propagation, cleanup, preparation for rendering
///    - Runs: Every frame, after Update and all FixedUpdate iterations
///    - IMPORTANT: Transform hierarchy updates happen here
///
/// 6. **Render** - Rendering stage (see runner.rs:70-76)
///    - Used for: GPU commands, draw calls, render graph execution
///    - Runs: Every frame in client mode only (skipped on server)
///
/// 7. **Last** - Final cleanup stage (see runner.rs:78-82)
///    - Used for: Final cleanup, state transitions, frame-end tasks
///    - Runs: Every frame, after all other stages
///
/// # Example
///
/// ```rust
/// use resonance::prelude::*;
///
/// Resonance::new()
///     .add_plugin(DefaultPlugins)
///     .add_system(Stage::Startup, setup_scene)
///     .add_system(Stage::Update, game_logic)
///     .add_system(Stage::FixedUpdate, physics_step)
///     .run();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub enum Stage {
    Startup,
    PreUpdate,
    Update,
    PostUpdate,
    FixedUpdate,
    Render,
    Last,
}

impl Stage {
    pub fn all() -> [Stage; 7] {
        [
            Stage::Startup,
            Stage::PreUpdate,
            Stage::Update,
            Stage::PostUpdate,
            Stage::FixedUpdate,
            Stage::Render,
            Stage::Last,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Stage::Startup => "Startup",
            Stage::PreUpdate => "PreUpdate",
            Stage::Update => "Update",
            Stage::PostUpdate => "PostUpdate",
            Stage::FixedUpdate => "FixedUpdate",
            Stage::Render => "Render",
            Stage::Last => "Last",
        }
    }
}
