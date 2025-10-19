use super::backend::AudioBackend;
use super::systems::*;
use crate::app::{Engine, Plugin, Stage};

pub struct AudioPluginConfig {
    pub enable_spatial_audio: bool,
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

pub struct AudioPlugin {
    config: AudioPluginConfig,
}

impl AudioPlugin {
    pub fn new() -> Self {
        Self {
            config: AudioPluginConfig::default(),
        }
    }

    pub fn with_config(config: AudioPluginConfig) -> Self {
        Self { config }
    }

    pub fn without_spatial_audio(mut self) -> Self {
        self.config.enable_spatial_audio = false;
        self
    }

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
        match AudioBackend::new() {
            Ok(backend) => {
                engine.world.insert_resource(backend);
            }
            Err(e) => {
                log::error!("Failed to initialize audio backend: {}", e);
                return;
            }
        }

        if let Some(schedule) = engine.schedules.get_mut(Stage::PreUpdate) {
            schedule.add_systems((handle_play_on_spawn, initialize_audio_sources));
        }

        if let Some(schedule) = engine.schedules.get_mut(Stage::Update) {
            schedule.add_systems((play_audio_sources, handle_audio_state_changes));
        }

        if self.config.enable_spatial_audio {
            if let Some(schedule) = engine.schedules.get_mut(Stage::Update) {
                schedule.add_systems(update_spatial_audio);
            }
        }

        if self.config.enable_doppler && self.config.enable_spatial_audio {
            if let Some(schedule) = engine.schedules.get_mut(Stage::Update) {
                schedule.add_systems(apply_doppler_effect);
            }
        }
        if let Some(schedule) = engine.schedules.get_mut(Stage::PostUpdate) {
            schedule.add_systems((cleanup_one_shot_audio, cleanup_audio_backend));
        }
    }
}
