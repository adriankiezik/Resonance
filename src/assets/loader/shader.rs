use crate::assets::cache::CachePolicy;
use crate::assets::loader::{AssetLoader, LoadError};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct ShaderData {
    pub source: String,
    pub shader_type: ShaderType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderType {
    Wgsl,
    Glsl,
}

impl ShaderData {
    pub fn new(source: String, shader_type: ShaderType) -> Self {
        Self {
            source,
            shader_type,
        }
    }
}

pub struct WgslLoader;

impl AssetLoader for WgslLoader {
    type Asset = ShaderData;

    fn load(&self, path: &Path) -> Result<Self::Asset, LoadError> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| LoadError::LoadFailed(format!("Failed to read shader file: {}", e)))?;

        if !source.contains("@vertex") && !source.contains("@fragment") {
            log::warn!("Shader may not be valid WGSL (missing @vertex or @fragment)");
        }

        Ok(ShaderData {
            source,
            shader_type: ShaderType::Wgsl,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["wgsl"]
    }

    fn cache_policy(&self) -> CachePolicy {
        CachePolicy::Strong
    }
}

pub fn load_shader_from_bytes(
    bytes: &[u8],
    shader_type: ShaderType,
) -> Result<ShaderData, LoadError> {
    let source = String::from_utf8(bytes.to_vec())
        .map_err(|e| LoadError::LoadFailed(format!("Invalid UTF-8 in shader: {}", e)))?;

    Ok(ShaderData {
        source,
        shader_type,
    })
}
