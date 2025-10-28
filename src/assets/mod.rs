//! Asset management system for Resonance Engine.
//!
//! This module provides asynchronous asset loading with caching and state tracking.
//!
//! # Asset Loading Patterns
//!
//! ## Pattern 1: Async Loading (Non-blocking, Recommended)
//!
//! Load assets asynchronously with a default/placeholder asset shown immediately:
//!
//! ```rust
//! use resonance::prelude::*;
//! use resonance::assets::TextureLoader;
//!
//! fn load_textures(assets: Res<Assets>) {
//!     // Returns immediately with default texture
//!     // Real texture loads in background
//!     let handle = assets.load(TextureLoader, "textures/player.png");
//!
//!     // You can use the handle right away - it contains the default asset
//!     // The asset will be swapped when loading completes
//! }
//! ```
//!
//! ## Pattern 2: Check Load State
//!
//! Poll asset loading state to determine when assets are ready:
//!
//! ```rust
//! use resonance::prelude::*;
//! use resonance::assets::TextureData;
//!
//! fn check_loaded(assets: Res<Assets>, handle: &AssetHandle<TextureData>) {
//!     if assets.is_loaded::<TextureData>(handle.id) {
//!         // Asset is fully loaded and ready to use
//!     } else if assets.is_loading::<TextureData>(handle.id) {
//!         // Still loading...
//!     }
//! }
//! ```
//!
//! ## Pattern 3: Batch Loading with Progress
//!
//! Load multiple assets and track overall progress:
//!
//! ```rust
//! use resonance::prelude::*;
//! use resonance::assets::MeshLoader;
//!
//! fn load_level(assets: Res<Assets>) {
//!     let paths = vec!["models/tree.obj", "models/rock.obj", "models/grass.obj"];
//!     let handles = assets.load_batch(MeshLoader, paths);
//!
//!     // Check progress
//!     let (loaded, total) = assets.loading_progress(&handles);
//!     println!("Loaded {} of {} assets", loaded, total);
//!
//!     // Check if all ready
//!     if assets.all_loaded(&handles) {
//!         println!("All assets loaded!");
//!     }
//!
//!     // Check for failures
//!     if assets.any_failed(&handles) {
//!         println!("Some assets failed to load");
//!     }
//! }
//! ```
//!
//! # Available Loaders
//!
//! - `TextureLoader` - PNG, JPEG images
//! - `MeshLoader` (ObjLoader, GltfLoader) - 3D models
//! - `AudioLoader` - Audio files (via symphonia)
//! - `TtfLoader` - TrueType fonts
//! - `WgslLoader` - WGSL shaders

pub mod assets;
pub mod cache;
pub mod handle;
pub mod loader;
pub mod pak;
pub mod plugin;
pub mod source;

pub use assets::{Assets, LoadState};
pub use cache::{AssetCache, CachePolicy};
pub use handle::{AssetHandle, AssetId};
pub use loader::{
    AssetLoader, LoadError,
    audio::{AudioData, AudioLoader},
    font::{FontData, TtfLoader},
    mesh::{GltfLoader, MeshData, ObjLoader},
    shader::{ShaderData, ShaderType, WgslLoader},
    texture::{TextureData, TextureFormat, TextureLoader},
};
pub use pak::{PakArchive, PakBuilder, PakEntry, PakError};
pub use plugin::AssetsPlugin;
pub use source::{AssetSource, AssetSourceConfig};
