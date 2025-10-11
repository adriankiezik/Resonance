//! Audio components for entities.

use bevy_ecs::prelude::*;
use ferrite_assets::AssetHandle;

/// Audio playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// Audio is currently playing
    Playing,
    /// Audio is paused
    Paused,
    /// Audio is stopped
    Stopped,
}

/// Audio source component
#[derive(Component, Debug, Clone)]
pub struct AudioSource {
    /// Handle to the audio asset
    pub audio_handle: Option<AssetHandle<ferrite_assets::AudioData>>,
    /// Volume (0.0 to 1.0)
    pub volume: f32,
    /// Pitch multiplier (1.0 = normal pitch)
    pub pitch: f32,
    /// Whether the audio should loop
    pub looping: bool,
    /// Playback state
    pub state: PlaybackState,
    /// Play audio on spawn
    pub play_on_spawn: bool,
}

impl AudioSource {
    /// Create a new audio source from an asset handle
    pub fn new(audio_handle: AssetHandle<ferrite_assets::AudioData>) -> Self {
        Self {
            audio_handle: Some(audio_handle),
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            state: PlaybackState::Stopped,
            play_on_spawn: false,
        }
    }

    /// Create from path (will need to be loaded separately)
    pub fn from_path(path: impl Into<String>) -> Self {
        Self {
            audio_handle: Some(AssetHandle::from_path(path)),
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            state: PlaybackState::Stopped,
            play_on_spawn: false,
        }
    }

    /// Set volume
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set pitch
    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.max(0.1);
        self
    }

    /// Enable looping
    pub fn looping(mut self) -> Self {
        self.looping = true;
        self
    }

    /// Play on spawn
    pub fn play_on_spawn(mut self) -> Self {
        self.play_on_spawn = true;
        self.state = PlaybackState::Playing;
        self
    }

    /// Play the audio
    pub fn play(&mut self) {
        self.state = PlaybackState::Playing;
    }

    /// Pause the audio
    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
    }

    /// Stop the audio
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
    }

    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }
}

/// Audio listener component (typically attached to camera)
#[derive(Component, Debug, Clone, Copy)]
pub struct AudioListener {
    /// Global volume multiplier for all audio heard by this listener
    pub global_volume: f32,
}

impl AudioListener {
    pub fn new() -> Self {
        Self {
            global_volume: 1.0,
        }
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.global_volume = volume.clamp(0.0, 1.0);
        self
    }
}

impl Default for AudioListener {
    fn default() -> Self {
        Self::new()
    }
}

/// Spatial audio component for 3D positioned audio
#[derive(Component, Debug, Clone, Copy)]
pub struct Spatial3dAudio {
    /// Maximum distance at which audio can be heard
    pub max_distance: f32,
    /// Reference distance for attenuation calculation
    pub reference_distance: f32,
    /// Attenuation rolloff factor (higher = faster falloff)
    pub rolloff_factor: f32,
    /// Enable Doppler effect
    pub doppler_enabled: bool,
    /// Doppler factor (0.0 = disabled, 1.0 = realistic)
    pub doppler_factor: f32,
}

impl Spatial3dAudio {
    /// Create with default settings
    pub fn new() -> Self {
        Self {
            max_distance: 100.0,
            reference_distance: 1.0,
            rolloff_factor: 1.0,
            doppler_enabled: false,
            doppler_factor: 1.0,
        }
    }

    /// Set maximum hearing distance
    pub fn with_max_distance(mut self, distance: f32) -> Self {
        self.max_distance = distance.max(0.1);
        self
    }

    /// Set reference distance
    pub fn with_reference_distance(mut self, distance: f32) -> Self {
        self.reference_distance = distance.max(0.1);
        self
    }

    /// Set rolloff factor
    pub fn with_rolloff(mut self, rolloff: f32) -> Self {
        self.rolloff_factor = rolloff.max(0.0);
        self
    }

    /// Enable Doppler effect
    pub fn with_doppler(mut self, factor: f32) -> Self {
        self.doppler_enabled = true;
        self.doppler_factor = factor.clamp(0.0, 5.0);
        self
    }

    /// Calculate distance attenuation
    pub fn calculate_attenuation(&self, distance: f32) -> f32 {
        if distance <= self.reference_distance {
            return 1.0;
        }

        if distance >= self.max_distance {
            return 0.0;
        }

        // Inverse distance attenuation
        let attenuation = self.reference_distance
            / (self.reference_distance
                + self.rolloff_factor * (distance - self.reference_distance));

        attenuation.clamp(0.0, 1.0)
    }
}

impl Default for Spatial3dAudio {
    fn default() -> Self {
        Self::new()
    }
}

/// Marker component for one-shot audio that should be removed when finished
#[derive(Component, Debug)]
pub struct AudioOneShot;

/// Velocity component for Doppler effect calculation
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct AudioVelocity {
    pub velocity: glam::Vec3,
}

impl AudioVelocity {
    pub fn new(velocity: glam::Vec3) -> Self {
        Self { velocity }
    }
}
