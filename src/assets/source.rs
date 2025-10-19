use crate::assets::loader::LoadError;
use crate::assets::pak::{PakArchive, PakError};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum AssetSourceConfig {
    Auto,
    FileSystem(PathBuf),
    PakFile(PathBuf),
}

impl Default for AssetSourceConfig {
    fn default() -> Self {
        Self::Auto
    }
}

impl AssetSourceConfig {
    pub fn resolve(self) -> Result<AssetSource, LoadError> {
        match self {
            AssetSourceConfig::Auto => {
                #[cfg(debug_assertions)]
                {
                    let path = PathBuf::from("assets");
                    if path.exists() {
                        Ok(AssetSource::FileSystem { root: path })
                    } else {
                        std::fs::create_dir_all(&path).map_err(|e| {
                            LoadError::LoadFailed(format!(
                                "Failed to create assets directory: {}",
                                e
                            ))
                        })?;
                        Ok(AssetSource::FileSystem { root: path })
                    }
                }

                #[cfg(not(debug_assertions))]
                {
                    let path = PathBuf::from("game_assets.pak");
                    if path.exists() {
                        let pak = PakArchive::open(&path).map_err(|e| {
                            LoadError::LoadFailed(format!("Failed to load PAK: {}", e))
                        })?;
                        Ok(AssetSource::PakArchive { pak: Arc::new(pak) })
                    } else {
                        Err(LoadError::NotFound(format!(
                            "PAK file not found: {}. Run 'cargo run --bin asset-packer' to create it.",
                            path.display()
                        )))
                    }
                }
            }
            AssetSourceConfig::FileSystem(root) => {
                if !root.exists() {
                    log::warn!(
                        "Assets directory not found at {}, creating it",
                        root.display()
                    );
                    std::fs::create_dir_all(&root).map_err(|e| {
                        LoadError::LoadFailed(format!("Failed to create assets directory: {}", e))
                    })?;
                }
                Ok(AssetSource::FileSystem { root })
            }
            AssetSourceConfig::PakFile(path) => {
                let pak = PakArchive::open(&path)
                    .map_err(|e| LoadError::LoadFailed(format!("Failed to load PAK: {}", e)))?;
                Ok(AssetSource::PakArchive { pak: Arc::new(pak) })
            }
        }
    }
}

pub enum AssetSource {
    FileSystem { root: PathBuf },
    PakArchive { pak: Arc<PakArchive> },
}

impl AssetSource {
    pub async fn load_bytes(&self, path: &str) -> Result<Vec<u8>, LoadError> {
        match self {
            AssetSource::FileSystem { root } => {
                let full_path = root.join(path);

                tokio::fs::read(&full_path).await.map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        LoadError::NotFound(format!(
                            "Asset not found: {} (full path: {})",
                            path,
                            full_path.display()
                        ))
                    } else {
                        LoadError::LoadFailed(format!("Failed to read {}: {}", path, e))
                    }
                })
            }
            AssetSource::PakArchive { pak } => pak.get(path).map_err(|e| match e {
                PakError::AssetNotFound(_) => {
                    LoadError::NotFound(format!("Asset not found in PAK: {}", path))
                }
                _ => LoadError::LoadFailed(format!("Failed to read from PAK: {}", e)),
            }),
        }
    }

    pub fn exists(&self, path: &str) -> bool {
        match self {
            AssetSource::FileSystem { root } => root.join(path).exists(),
            AssetSource::PakArchive { pak } => pak.exists(path),
        }
    }

    pub fn get_filesystem_path(&self, path: &str) -> Option<PathBuf> {
        match self {
            AssetSource::FileSystem { root } => Some(root.join(path)),
            AssetSource::PakArchive { .. } => None,
        }
    }

    pub fn supports_hot_reload(&self) -> bool {
        matches!(self, AssetSource::FileSystem { .. })
    }

    pub fn list_assets(&self) -> Vec<String> {
        match self {
            AssetSource::FileSystem { root } => {
                let mut assets = Vec::new();
                match walkdir::WalkDir::new(root)
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
                {
                    Ok(entries) => {
                        for entry in entries {
                            if entry.file_type().is_file() {
                                if let Ok(relative) = entry.path().strip_prefix(root) {
                                    assets.push(relative.to_string_lossy().replace('\\', "/"));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to walk assets directory {}: {}", root.display(), e);
                    }
                }
                assets
            }
            AssetSource::PakArchive { pak } => pak.list(),
        }
    }
}
