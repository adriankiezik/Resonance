//! Audio plugin for the Ferrite engine.

use super::backend::AudioBackend;
use super::systems::*;
use ferrite_app::{Engine, Plugin, Stage};

/// Configuration for the audio plugin
pub struct AudioPluginConfig {
    /// Enable 3D spatial audio
    pub enable_spatial_audio: bool,
    /// Enable Doppler effect
    pub enable_doppler: bool,
}

impl Default for AudioPluginConfig {
    fn default() -> Self {
        Self {
            enable_spatial_audio: true,
            enable_doppler: true,
        }
    }
}

/// Plugin for audio playback and spatial audio
pub struct AudioPlugin {
    config: AudioPluginConfig,
}

impl AudioPlugin {
    /// Create with default configuration
    pub fn new() -> Self {
        Self {
            config: AudioPluginConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: AudioPluginConfig) -> Self {
        Self { config }
    }

    /// Disable spatial audio
    pub fn without_spatial_audio(mut self) -> Self {
        self.config.enable_spatial_audio = false;
        self
    }

    /// Disable Doppler effect
    pub fn without_doppler(mut self) -> Self {
        self.config.enable_doppler = false;
        self
    }
}

impl Default for AudioPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for AudioPlugin {
    fn build(&self, engine: &mut Engine) {
        // Initialize audio backend
        match AudioBackend::new() {
            Ok(backend) => {
                engine.world.insert_resource(backend);
                log::info!("Audio backend initialized");
            }
            Err(e) => {
                log::error!("Failed to initialize audio backend: {}", e);
                return;
            }
        }

        // Add core audio systems
        if let Some(schedule) = engine.schedules.get_mut(Stage::PreUpdate) {
            schedule.add_systems((
                handle_play_on_spawn,
                initialize_audio_sources,
            ));
        }

        if let Some(schedule) = engine.schedules.get_mut(Stage::Update) {
            schedule.add_systems((
                play_audio_sources,
                handle_audio_state_changes,
            ));
        }

        // Add spatial audio systems if enabled
        if self.config.enable_spatial_audio {
            if let Some(schedule) = engine.schedules.get_mut(Stage::Update) {
                schedule.add_systems(update_spatial_audio);
            }
            log::info!("3D spatial audio enabled");
        }

        // Add Doppler effect if enabled
        if self.config.enable_doppler && self.config.enable_spatial_audio {
            if let Some(schedule) = engine.schedules.get_mut(Stage::Update) {
                schedule.add_systems(apply_doppler_effect);
            }
            log::info!("Doppler effect enabled");
        }

        // Add cleanup systems
        if let Some(schedule) = engine.schedules.get_mut(Stage::PostUpdate) {
            schedule.add_systems((
                cleanup_one_shot_audio,
                cleanup_audio_backend,
            ));
        }

        log::info!("AudioPlugin initialized");
    }
}
