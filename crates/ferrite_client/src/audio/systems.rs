//! Audio playback and spatial audio systems.

use super::backend::{AudioBackend, MemorySource};
use super::components::*;
use bevy_ecs::prelude::*;
use ferrite_assets::{AssetCache, AudioData};
use ferrite_transform::Transform;
use glam::Vec3;
use rodio::Source;

/// System to initialize audio sinks for new audio sources
pub fn initialize_audio_sources(
    mut commands: Commands,
    audio_backend: Res<AudioBackend>,
    query: Query<(Entity, &Transform, Option<&Spatial3dAudio>), Added<AudioSource>>,
) {
    for (entity, transform, spatial) in query.iter() {
        // Create spatial sink if Spatial3dAudio component exists, otherwise regular sink
        let result = if let Some(_) = spatial {
            let pos = [transform.position.x, transform.position.y, transform.position.z];
            log::info!("Creating spatial sink for entity {:?} at position {:?}", entity, pos);
            audio_backend.create_spatial_sink(entity, pos)
        } else {
            log::info!("Creating regular sink for entity {:?}", entity);
            audio_backend.create_sink(entity)
        };

        if let Err(e) = result {
            log::error!("Failed to create audio sink for entity {:?}: {}", entity, e);
            // Remove the audio source if we can't create a sink
            commands.entity(entity).remove::<AudioSource>();
        } else {
            log::info!("Successfully created audio sink for entity {:?}", entity);
        }
    }
}

/// System to play audio sources
pub fn play_audio_sources(
    audio_backend: Res<AudioBackend>,
    asset_cache: Res<AssetCache>,
    mut query: Query<(Entity, &mut AudioSource), Changed<AudioSource>>,
) {
    for (entity, audio_source) in query.iter_mut() {
        // Skip if not playing
        if !audio_source.is_playing() {
            continue;
        }

        // Get audio data from cache
        let Some(audio_handle) = &audio_source.audio_handle else {
            log::warn!("Audio source {:?} has no audio handle", entity);
            continue;
        };

        let Some(audio_data) = asset_cache.get::<AudioData>(audio_handle.id) else {
            log::debug!("Audio data not yet loaded for {:?}", audio_handle.path);
            continue;
        };

        // Check if already playing
        if audio_backend.is_playing(entity) {
            // Update volume if needed
            audio_backend.set_volume(entity, audio_source.volume);
            continue;
        }

        // Create audio source
        let source = MemorySource::new(
            audio_data.samples.clone(),
            audio_data.sample_rate,
            audio_data.channels,
        );

        // Apply pitch
        let source = if audio_source.pitch != 1.0 {
            source.speed(audio_source.pitch)
        } else {
            source.speed(1.0)
        };

        // Apply looping
        let source: Box<dyn Source<Item = f32> + Send> = if audio_source.looping {
            Box::new(source.repeat_infinite())
        } else {
            Box::new(source)
        };

        // Play the audio
        if let Err(e) = audio_backend.play_audio(entity, source, audio_source.volume) {
            log::error!("Failed to play audio for entity {:?}: {}", entity, e);
        } else {
            log::info!(
                "Started playing audio for entity {:?} (volume: {}, pitch: {}, loop: {})",
                entity,
                audio_source.volume,
                audio_source.pitch,
                audio_source.looping
            );
        }
    }
}

/// System to handle paused/stopped audio
pub fn handle_audio_state_changes(
    audio_backend: Res<AudioBackend>,
    query: Query<(Entity, &AudioSource), Changed<AudioSource>>,
) {
    for (entity, audio_source) in query.iter() {
        match audio_source.state {
            PlaybackState::Playing => {
                audio_backend.resume(entity);
            }
            PlaybackState::Paused => {
                audio_backend.pause(entity);
            }
            PlaybackState::Stopped => {
                audio_backend.stop(entity);
            }
        }
    }
}

/// System to update 3D spatial audio
pub fn update_spatial_audio(
    audio_backend: Res<AudioBackend>,
    listener_query: Query<&Transform, With<AudioListener>>,
    mut audio_query: Query<(Entity, &Transform, &mut AudioSource, &Spatial3dAudio)>,
) {
    // Get listener position (use first listener found)
    let listener_pos = listener_query
        .iter()
        .next()
        .map(|t| t.position)
        .unwrap_or(Vec3::ZERO);

    for (entity, transform, _audio_source, _spatial) in audio_query.iter_mut() {
        // Update emitter position for spatial audio panning
        // Position is relative to listener
        let emitter_pos = [
            transform.position.x - listener_pos.x,
            transform.position.y - listener_pos.y,
            transform.position.z - listener_pos.z,
        ];
        audio_backend.set_emitter_position(entity, emitter_pos);

        // Note: We don't manually adjust volume for spatial audio
        // Rodio's SpatialSink handles volume attenuation automatically based on distance

        log::trace!(
            "Updated spatial audio position for {:?}: emitter at {:?}, listener at {:?}",
            entity,
            emitter_pos,
            listener_pos
        );
    }
}

/// System to calculate and apply Doppler effect
pub fn apply_doppler_effect(
    listener_query: Query<(&Transform, Option<&AudioVelocity>), With<AudioListener>>,
    mut audio_query: Query<
        (
            &Transform,
            &mut AudioSource,
            &Spatial3dAudio,
            Option<&AudioVelocity>,
        ),
        Without<AudioListener>,
    >,
) {
    // Get listener data
    let Some((listener_transform, listener_velocity)) = listener_query.iter().next() else {
        return;
    };

    let listener_pos = listener_transform.position;
    let listener_vel = listener_velocity
        .map(|v| v.velocity)
        .unwrap_or(Vec3::ZERO);

    const SPEED_OF_SOUND: f32 = 343.0; // m/s in air at 20°C

    for (transform, audio_source, spatial, source_velocity) in audio_query.iter_mut() {
        if !spatial.doppler_enabled || spatial.doppler_factor == 0.0 {
            continue;
        }

        let source_pos = transform.position;
        let source_vel = source_velocity.map(|v| v.velocity).unwrap_or(Vec3::ZERO);

        // Calculate direction from source to listener
        let direction = (listener_pos - source_pos).normalize_or_zero();

        // Calculate relative velocity along the direction
        let source_vel_along = source_vel.dot(direction);
        let listener_vel_along = listener_vel.dot(direction);

        // Doppler shift formula: f' = f * (v + vl) / (v + vs)
        // where v = speed of sound, vl = listener velocity, vs = source velocity
        let doppler_factor = (SPEED_OF_SOUND + listener_vel_along)
            / (SPEED_OF_SOUND + source_vel_along);

        // Apply Doppler effect to pitch (clamped to reasonable range)
        let doppler_pitch = doppler_factor.clamp(0.5, 2.0);
        let final_pitch = audio_source.pitch * doppler_pitch * spatial.doppler_factor;

        // Note: rodio doesn't support dynamic pitch changes on running sinks,
        // so this would require recreating the source. For now, we just log it.
        if (final_pitch - audio_source.pitch).abs() > 0.01 {
            log::trace!(
                "Doppler pitch shift: {:.3} -> {:.3}",
                audio_source.pitch,
                final_pitch
            );
        }
    }
}

/// System to remove finished one-shot audio
pub fn cleanup_one_shot_audio(
    mut commands: Commands,
    audio_backend: Res<AudioBackend>,
    query: Query<Entity, (With<AudioOneShot>, With<AudioSource>)>,
) {
    for entity in query.iter() {
        // Check if audio is still playing
        if !audio_backend.is_playing(entity) {
            log::debug!("Removing finished one-shot audio entity {:?}", entity);
            commands.entity(entity).despawn();
        }
    }
}

/// System to clean up audio backend
pub fn cleanup_audio_backend(audio_backend: Res<AudioBackend>) {
    audio_backend.cleanup_finished();
}

/// System to handle audio sources that want to play on spawn
pub fn handle_play_on_spawn(
    mut query: Query<&mut AudioSource, Added<AudioSource>>,
) {
    for mut audio_source in query.iter_mut() {
        if audio_source.play_on_spawn {
            audio_source.play();
        }
    }
}
