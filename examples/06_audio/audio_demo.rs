//! Audio system demonstration.
//!
//! Demonstrates:
//! - Basic audio playback
//! - Volume and pitch control
//! - Audio looping
//! - 3D spatial audio with distance attenuation
//! - Multiple simultaneous audio sources

use ferrite::prelude::*;
use ferrite_assets::{AssetCache, AssetHandle, AssetLoader, AudioData, AudioLoader};
use ferrite_client::audio::{AudioListener, AudioPlugin, AudioSource, Spatial3dAudio};
use ferrite_core::time::Time;
use glam::Vec3;

fn main() {
    // Initialize logging
    init_logger();

    println!("=== Ferrite Audio System Demo ===\n");
    println!("This demo demonstrates:");
    println!("- 3D spatial audio with stereo panning");
    println!("- Audio source moving left and right (X-axis)");
    println!("- You should hear the sound pan between left and right ears\n");

    // Create engine with audio system
    let mut engine = Engine::new()
        .add_plugin(CorePlugin)
        .add_plugin(TransformPlugin)
        .add_plugin(AssetsPlugin::new())
        .add_plugin(AudioPlugin::new())
        .add_startup_system(setup_audio_demo)
        .add_system(Stage::Update, demo_audio_system)
        .add_system(Stage::Update, move_audio_source);

    // Run startup
    engine.startup();

    // Run for a longer time to demonstrate looping audio
    println!("Playing audio (looping enabled)...\n");
    println!("Press Ctrl+C to stop\n");

    let mut frame = 0;
    loop {
        engine.update();
        std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS

        if frame % 60 == 0 {
            let seconds = frame / 60;
            println!("Playing... {}s", seconds);
        }

        frame += 1;
    }
}

/// Setup audio demo scene
fn setup_audio_demo(
    mut commands: Commands,
    asset_cache: Res<AssetCache>,
) {
    println!("Setting up audio demo scene...\n");

    // Create audio listener (like a camera for audio)
    commands.spawn((
        Transform::from_position(Vec3::ZERO),
        AudioListener::new(),
    ));
    println!("✓ Created audio listener at origin");

    // Load and play the MP3 file
    let audio_loader = AudioLoader;
    match audio_loader.load(std::path::Path::new("assets/audio/background_music.mp3")) {
        Ok(audio_data) => {
            println!("✓ Loaded background music: {}Hz, {} channels",
                audio_data.sample_rate, audio_data.channels);

            // Cache the audio
            let music_handle = AssetHandle::<AudioData>::from_path("assets/audio/background_music.mp3");
            asset_cache.insert(music_handle.id, audio_data);

            // Play the MP3 with 3D spatial audio - starts at center, will move left/right
            commands.spawn((
                Transform::from_position(Vec3::ZERO),
                AudioSource::new(music_handle)
                    .with_volume(1.0)
                    .looping()
                    .play_on_spawn(),
                Spatial3dAudio::new()
                    .with_max_distance(50.0)
                    .with_rolloff(0.5),
            ));
            println!("✓ Playing spatial audio - will move left and right");
        }
        Err(e) => {
            println!("⚠ Could not load background music: {}", e);
            println!("  Make sure assets/audio/background_music.mp3 exists");

            // Fallback: Create a simple test tone
            let test_audio = AudioData::test_tone(440.0, 2.0);
            let handle = AssetHandle::<AudioData>::from_path("procedural_tone");
            asset_cache.insert(handle.id, test_audio);

            commands.spawn((
                Transform::from_position(Vec3::ZERO),
                AudioSource::new(handle)
                    .with_volume(0.2)
                    .play_on_spawn(),
            ));
            println!("✓ Created 440Hz test tone (2 seconds) as fallback");
        }
    }

    println!("\nAudio demo scene setup complete!");
}

/// Update audio demo (could be used for interactive demonstrations)
fn demo_audio_system(
    query: Query<(Entity, &AudioSource, Option<&Spatial3dAudio>)>,
    mut ran_once: Local<bool>,
) {
    if *ran_once {
        return;
    }
    *ran_once = true;

    println!("\n--- Active Audio Sources ---");
    for (entity, audio_source, spatial) in query.iter() {
        println!("Entity {:?}:", entity);
        println!("  State: {:?}", audio_source.state);
        println!("  Volume: {}", audio_source.volume);
        println!("  Pitch: {}", audio_source.pitch);
        println!("  Looping: {}", audio_source.looping);

        if let Some(spatial) = spatial {
            println!("  Spatial Audio:");
            println!("    Max Distance: {}", spatial.max_distance);
            println!("    Rolloff: {}", spatial.rolloff_factor);
            println!("    Doppler: {}", spatial.doppler_enabled);
        }
        println!();
    }
}

/// Move audio source left and right (like the official rodio spatial example)
fn move_audio_source(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<AudioSource>>,
) {
    let elapsed = time.elapsed_seconds();

    // Move left and right along X-axis (most noticeable for stereo panning)
    // Period: 10 seconds for full cycle (left → right → left)
    let cycle_time = 10.0;
    let max_distance = 5.0; // Maximum distance left/right

    for mut transform in query.iter_mut() {
        // Triangle wave: -max to +max and back
        let t = (elapsed % cycle_time) / cycle_time; // 0 to 1
        let x = if t < 0.5 {
            // First half: move from -max to +max
            -max_distance + (t * 2.0 * max_distance * 2.0)
        } else {
            // Second half: move from +max to -max
            max_distance - ((t - 0.5) * 2.0 * max_distance * 2.0)
        };

        transform.position.x = x;
        transform.position.y = 0.0;
        transform.position.z = 0.0; // Keep at same depth as listener
    }
}
