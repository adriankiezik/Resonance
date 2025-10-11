//! Texture asset loading.
//!
//! Loads image files (PNG, JPG) and provides utilities for GPU texture creation.

use crate::loader::{AssetLoader, LoadError};
use image::DynamicImage;
use std::path::Path;

/// Loaded texture data ready for GPU upload
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGBA8 format
    pub format: TextureFormat,
}

/// Texture format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    Rgba8,
    Rgb8,
    R8,
}

impl TextureData {
    /// Create from an image
    pub fn from_image(image: DynamicImage) -> Self {
        let width = image.width();
        let height = image.height();
        let rgba = image.to_rgba8();
        let data = rgba.into_raw();

        Self {
            width,
            height,
            data,
            format: TextureFormat::Rgba8,
        }
    }

    /// Create a fallback texture (magenta checkerboard)
    pub fn fallback() -> Self {
        let size = 64;
        let mut data = Vec::with_capacity((size * size * 4) as usize);

        for y in 0..size {
            for x in 0..size {
                let checker = ((x / 8) + (y / 8)) % 2 == 0;
                if checker {
                    // Magenta
                    data.push(255);
                    data.push(0);
                    data.push(255);
                    data.push(255);
                } else {
                    // Black
                    data.push(0);
                    data.push(0);
                    data.push(0);
                    data.push(255);
                }
            }
        }

        Self {
            width: size,
            height: size,
            data,
            format: TextureFormat::Rgba8,
        }
    }

    /// Create a solid color texture
    pub fn solid_color(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            width: 1,
            height: 1,
            data: vec![r, g, b, a],
            format: TextureFormat::Rgba8,
        }
    }

    /// Create a white texture
    pub fn white() -> Self {
        Self::solid_color(255, 255, 255, 255)
    }

    /// Create a black texture
    pub fn black() -> Self {
        Self::solid_color(0, 0, 0, 255)
    }
}

/// Image asset loader (loads to DynamicImage)
pub struct ImageLoader;

impl AssetLoader for ImageLoader {
    type Asset = DynamicImage;

    fn load(&self, path: &Path) -> Result<Self::Asset, LoadError> {
        image::open(path).map_err(|e| LoadError::LoadFailed(e.to_string()))
    }

    fn extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "bmp", "gif", "tga", "webp"]
    }
}

/// Texture asset loader (loads to TextureData ready for GPU)
pub struct TextureLoader;

impl AssetLoader for TextureLoader {
    type Asset = TextureData;

    fn load(&self, path: &Path) -> Result<Self::Asset, LoadError> {
        let image = image::open(path).map_err(|e| LoadError::LoadFailed(e.to_string()))?;
        Ok(TextureData::from_image(image))
    }

    fn extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "bmp", "gif", "tga", "webp"]
    }
}

/// Load texture from raw bytes
pub fn load_texture_from_bytes(bytes: &[u8]) -> Result<TextureData, LoadError> {
    let image = image::load_from_memory(bytes)
        .map_err(|e| LoadError::LoadFailed(format!("Failed to decode image: {}", e)))?;
    Ok(TextureData::from_image(image))
}
