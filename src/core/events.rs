/// Core event types for the Resonance engine
///
/// This module provides common message types that can be used throughout the engine.
/// Messages are based on Bevy ECS's buffered message system.
///
/// # Example
/// ```no_run
/// use resonance::prelude::*;
/// use resonance::core::events::WindowResized;
///
/// fn handle_resize(mut events: MessageReader<WindowResized>) {
///     for event in events.read() {
///         println!("Window resized to {}x{}", event.width, event.height);
///     }
/// }
/// ```

use bevy_ecs::prelude::*;

/// Message fired when the window is resized
#[derive(Message, Clone, Copy, Debug)]
pub struct WindowResized {
    pub width: u32,
    pub height: u32,
}

/// Message fired when the window gains or loses focus
#[derive(Message, Clone, Copy, Debug)]
pub struct WindowFocusChanged {
    pub focused: bool,
}

/// Message fired when an asset finishes loading (success or failure)
#[derive(Message, Clone, Debug)]
pub struct AssetLoaded {
    pub path: String,
    pub success: bool,
}

/// Message fired before the engine shuts down
#[derive(Message, Clone, Copy, Debug)]
pub struct EngineShutdown;

/// Plugin that adds core event types to the engine
#[derive(Default)]
pub struct EventsPlugin;

impl crate::app::Plugin for EventsPlugin {
    fn build(&self, engine: &mut crate::app::Resonance) {
        // Initialize message resources
        use bevy_ecs::message::Messages;
        engine.world.init_resource::<Messages<WindowResized>>();
        engine.world.init_resource::<Messages<WindowFocusChanged>>();
        engine.world.init_resource::<Messages<AssetLoaded>>();
        engine.world.init_resource::<Messages<EngineShutdown>>();

        // Add global message update system to clear old messages each frame
        // In bevy_ecs 0.17, message_update_system handles all message types automatically
        use crate::app::Stage;
        if let Some(schedule) = engine.schedules.get_mut(Stage::Last) {
            schedule.add_systems(bevy_ecs::message::message_update_system);
        }
    }

    fn name(&self) -> &'static str {
        "EventsPlugin"
    }
}
