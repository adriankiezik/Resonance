
use bevy_ecs::prelude::*;
use crate::assets::AssetHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Component, Debug, Clone)]
pub struct AudioSource {
    pub audio_handle: Option<AssetHandle<crate::assets::AudioData>>,
    pub volume: f32,
    pub pitch: f32,
    pub looping: bool,
    pub state: PlaybackState,
    pub play_on_spawn: bool,
}

impl AudioSource {
    pub fn new(audio_handle: AssetHandle<crate::assets::AudioData>) -> Self {
        Self {
            audio_handle: Some(audio_handle),
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            state: PlaybackState::Stopped,
            play_on_spawn: false,
        }
    }

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

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.max(0.1);
        self
    }

    pub fn looping(mut self) -> Self {
        self.looping = true;
        self
    }

    pub fn play_on_spawn(mut self) -> Self {
        self.play_on_spawn = true;
        self.state = PlaybackState::Playing;
        self
    }

    pub fn play(&mut self) {
        self.state = PlaybackState::Playing;
    }

    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
    }

    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
    }

    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct AudioListener {
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

#[derive(Component, Debug, Clone, Copy)]
pub struct Spatial3dAudio {
    pub max_distance: f32,
    pub reference_distance: f32,
    pub rolloff_factor: f32,
    pub doppler_enabled: bool,
    pub doppler_factor: f32,
}

impl Spatial3dAudio {

    pub fn new() -> Self {
        Self {
            max_distance: 100.0,
            reference_distance: 1.0,
            rolloff_factor: 1.0,
            doppler_enabled: false,
            doppler_factor: 1.0,
        }
    }

    pub fn with_max_distance(mut self, distance: f32) -> Self {
        self.max_distance = distance.max(0.1);
        self
    }

    pub fn with_reference_distance(mut self, distance: f32) -> Self {
        self.reference_distance = distance.max(0.1);
        self
    }

    pub fn with_rolloff(mut self, rolloff: f32) -> Self {
        self.rolloff_factor = rolloff.max(0.0);
        self
    }

    pub fn with_doppler(mut self, factor: f32) -> Self {
        self.doppler_enabled = true;
        self.doppler_factor = factor.clamp(0.0, 5.0);
        self
    }

    pub fn calculate_attenuation(&self, distance: f32) -> f32 {
        if distance <= self.reference_distance {
            return 1.0;
        }

        if distance >= self.max_distance {
            return 0.0;
        }

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

#[derive(Component, Debug)]
pub struct AudioOneShot;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct AudioVelocity {
    pub velocity: glam::Vec3,
}

impl AudioVelocity {
    pub fn new(velocity: glam::Vec3) -> Self {
        Self { velocity }
    }
}
