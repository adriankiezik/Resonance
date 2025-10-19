
use bevy_ecs::schedule::ScheduleLabel;

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

    pub fn all() -> Vec<Stage> {
        vec![
            Stage::PreUpdate,
            Stage::Update,
            Stage::PostUpdate,
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
