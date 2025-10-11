//! Audio playback system using rodio.
//!
//! Provides:
//! - Audio playback with volume and pitch control
//! - 3D spatial audio with distance attenuation
//! - Doppler effect for moving audio sources
//! - Audio looping and one-shot sounds
//! - Multiple simultaneous audio sources

pub mod backend;
pub mod components;
pub mod plugin;
pub mod systems;

pub use backend::AudioBackend;
pub use components::*;
pub use plugin::{AudioPlugin, AudioPluginConfig};
