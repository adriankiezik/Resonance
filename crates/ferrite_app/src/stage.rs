//! Execution stages for organizing systems.
//!
//! Stages allow you to control the order in which systems run.
//! Systems within a stage can run in parallel if they don't conflict.

use bevy_ecs::schedule::ScheduleLabel;

/// Built-in execution stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub enum Stage {
    /// Runs once at startup, before the main loop begins
    Startup,
    /// Runs every frame before Update (e.g., input collection)
    PreUpdate,
    /// Main game logic stage
    Update,
    /// Runs after Update (e.g., transform propagation, physics)
    PostUpdate,
    /// Fixed timestep stage for deterministic simulation
    FixedUpdate,
    /// Rendering stage (client only)
    Render,
    /// Cleanup stage (e.g., removing despawned entities)
    Last,
}

impl Stage {
    /// Get all stages in execution order (excluding FixedUpdate which runs separately)
    pub fn all() -> Vec<Stage> {
        vec![
            Stage::PreUpdate,
            Stage::Update,
            Stage::PostUpdate,
            Stage::Render,
            Stage::Last,
        ]
    }

    /// Get the stage name as a string
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
