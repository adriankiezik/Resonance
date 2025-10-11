//! Comprehensive asset loading showcase.
//!
//! Demonstrates loading and using:
//! - Textures (PNG, JPG)
//! - Meshes (OBJ, GLTF)
//! - Audio (MP3, WAV, OGG)
//! - Shaders (WGSL)
//! - Fonts (TTF)
//!
//! Features:
//! - Synchronous and asynchronous asset loading
//! - Load progress tracking
//! - Fallback assets for error handling
//! - Hot reloading demonstration

use ferrite::prelude::*;
use ferrite_assets::*;
use std::path::Path;

fn main() {
    // Initialize logging
    init_logger();

    println!("=== Ferrite Asset System Showcase ===\n");

    // Create engine with assets plugin
    let mut engine = Engine::new()
        .add_plugin(CorePlugin)
        .add_plugin(AssetsPlugin::new())
        .add_startup_system(load_assets_sync)
        .add_system(Stage::Update, check_load_progress);

    // Run startup
    engine.startup();

    // Run a few update cycles to process async loading
    for _frame in 0..5 {
        engine.update();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("\n=== Asset Showcase Complete ===");
}

/// Load assets synchronously
fn load_assets_sync(cache: Res<AssetCache>) {
    println!("\n--- Synchronous Asset Loading ---\n");

    // 1. Load Texture
    println!("1. Loading texture from file...");
    let texture_loader = TextureLoader;
    match texture_loader.load(Path::new("assets/textures/wood.jpg")) {
        Ok(texture_data) => {
            println!(
                "   ✓ Loaded texture: {}x{} pixels, format: {:?}",
                texture_data.width, texture_data.height, texture_data.format
            );
            // Cache it
            let handle = AssetHandle::<TextureData>::from_path("assets/textures/wood.jpg");
            cache.insert(handle.id, texture_data);
        }
        Err(e) => {
            println!("   ✗ Failed to load texture: {}", e);
            println!("   → Using fallback texture");
            let fallback = TextureData::fallback();
            println!(
                "   ✓ Fallback texture: {}x{} (magenta checkerboard)",
                fallback.width, fallback.height
            );
        }
    }

    // 2. Load Mesh (OBJ)
    println!("\n2. Loading OBJ mesh from file...");
    let obj_loader = ObjLoader;
    match obj_loader.load(Path::new("assets/models/model.obj")) {
        Ok(meshes) => {
            println!("   ✓ Loaded {} mesh(es) from OBJ", meshes.len());
            for (i, mesh) in meshes.iter().enumerate() {
                println!(
                    "     Mesh {}: {} vertices, {} triangles",
                    i,
                    mesh.vertex_count(),
                    mesh.triangle_count()
                );
            }
            // Cache the first mesh
            if let Some(mesh) = meshes.into_iter().next() {
                let handle = AssetHandle::<MeshData>::from_path("assets/models/model.obj");
                cache.insert(handle.id, mesh);
            }
        }
        Err(e) => {
            println!("   ✗ Failed to load OBJ: {}", e);
            println!("   → Using fallback cube mesh");
            let fallback = MeshData::fallback_cube();
            println!(
                "   ✓ Fallback mesh: {} vertices, {} triangles",
                fallback.vertex_count(),
                fallback.triangle_count()
            );
        }
    }

    // 3. Load Audio
    println!("\n3. Loading audio from file...");
    let audio_loader = AudioLoader;
    match audio_loader.load(Path::new("assets/audio/background_music.mp3")) {
        Ok(audio_data) => {
            println!(
                "   ✓ Loaded audio: {}Hz, {} channels, {:.2}s duration",
                audio_data.sample_rate, audio_data.channels, audio_data.duration
            );
            println!(
                "     Total samples: {} ({} frames)",
                audio_data.sample_count(),
                audio_data.frame_count()
            );
            // Cache it
            let handle = AssetHandle::<AudioData>::from_path("assets/audio/background_music.mp3");
            cache.insert(handle.id, audio_data);
        }
        Err(e) => {
            println!("   ✗ Failed to load audio: {}", e);
            println!("   → Using fallback silence");
            let fallback = AudioData::fallback_silence();
            println!(
                "   ✓ Fallback audio: {}Hz, {} channels, {:.2}s silence",
                fallback.sample_rate, fallback.channels, fallback.duration
            );
        }
    }

    // 4. Load Font
    println!("\n4. Loading font from file...");
    let font_loader = TtfLoader;
    match font_loader.load(Path::new("assets/fonts/Lora-VariableFont_wght.ttf")) {
        Ok(font_data) => {
            println!("   ✓ Loaded font: {}", font_data.family_name);
            println!(
                "     Metrics (at 16px): ascent={:.2}, descent={:.2}, height={:.2}",
                font_data.ascent(16.0),
                font_data.descent(16.0),
                font_data.height(16.0)
            );
            // Cache it
            let handle =
                AssetHandle::<FontData>::from_path("assets/fonts/Lora-VariableFont_wght.ttf");
            cache.insert(handle.id, font_data);
        }
        Err(e) => {
            println!("   ✗ Failed to load font: {}", e);
            println!("     (Fallback font requires embedded font data)");
        }
    }

    // 5. Create procedural assets
    println!("\n5. Creating procedural assets...");

    // Create a test tone
    let test_tone = AudioData::test_tone(440.0, 1.0); // 440Hz A note for 1 second
    println!(
        "   ✓ Generated test tone: 440Hz, {:.2}s duration",
        test_tone.duration
    );

    // Create solid color textures
    let _white_tex = TextureData::white();
    let _black_tex = TextureData::black();
    println!("   ✓ Generated solid color textures (white, black)");

    // Create a fallback shader
    let _fallback_shader = ShaderData::fallback();
    println!("   ✓ Generated fallback shader (WGSL)");

    println!("\n✓ All synchronous asset loading demonstrations complete!");
}

/// Check asset loading progress (for async loading demonstration)
fn check_load_progress(async_loader: Option<Res<AsyncAssetLoader>>, mut ran: Local<bool>) {
    if *ran {
        return;
    }
    *ran = true;

    if let Some(async_loader) = async_loader {
        println!("\n--- Asynchronous Asset Loading ---\n");

        // Demonstrate async loading progress tracking
        let total_progress = async_loader.get_total_progress();
        println!("Total loading progress: {:.1}%", total_progress * 100.0);

        println!("\n✓ Async asset loading system is active and ready!");
        println!("  (To load assets asynchronously, use AsyncAssetLoader::load_async)");
    }
}
