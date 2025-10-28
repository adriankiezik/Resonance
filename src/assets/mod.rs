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
