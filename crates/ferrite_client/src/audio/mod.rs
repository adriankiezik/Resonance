//! Audio playback using rodio.

use bevy_ecs::prelude::*;

/// Audio engine resource
#[derive(Resource)]
pub struct AudioEngine {
    // TODO: Store rodio output stream and handle
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {}
    }
}

/// Audio source component
#[derive(Component, Debug, Clone)]
pub struct AudioSource {
    pub file_path: String,
    pub volume: f32,
    pub looping: bool,
}

impl AudioSource {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            volume: 1.0,
            looping: false,
        }
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn looping(mut self) -> Self {
        self.looping = true;
        self
    }
}

// TODO: Implement audio playback with rodio
// TODO: Add 3D spatial audio
// TODO: Implement audio mixing and effects
