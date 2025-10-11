//! Asset management system.
//!
//! Provides:
//! - Asset loading from disk (sync and async)
//! - Asset caching with type safety
//! - Hot reloading with file system watching
//! - Asset handles for safe references
//! - Support for multiple asset types:
//!   - Textures (PNG, JPG, etc.)
//!   - Meshes (OBJ, GLTF)
//!   - Audio (WAV, MP3, OGG, FLAC)
//!   - Shaders (WGSL)
//!   - Fonts (TTF, OTF)

pub mod async_loader;
pub mod audio_loader;
pub mod cache;
pub mod font_loader;
pub mod handle;
pub mod hot_reload;
pub mod loader;
pub mod mesh_loader;
pub mod plugin;
pub mod shader_loader;
pub mod texture_loader;

pub use async_loader::{AsyncAssetLoader, LoadProgress, LoadState};
pub use audio_loader::{AudioData, AudioLoader};
pub use cache::AssetCache;
pub use font_loader::{FontData, TtfLoader};
pub use handle::{AssetHandle, AssetId};
pub use hot_reload::{HotReloadEvent, HotReloadEventKind, HotReloadWatcher};
pub use loader::{AssetLoader, LoadError};
pub use mesh_loader::{GltfLoader, MeshData, ObjLoader};
pub use plugin::AssetsPlugin;
pub use shader_loader::{ShaderData, ShaderType, WgslLoader};
pub use texture_loader::{TextureData, TextureFormat, TextureLoader};
