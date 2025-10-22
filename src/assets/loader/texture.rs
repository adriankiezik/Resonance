use crate::assets::loader::{AssetLoader, LoadError};
use crate::core::math::*;
use image::DynamicImage;
use std::path::Path;

#[derive(Debug)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub format: TextureFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    Rgba8,
    Rgb8,
    R8,
}

impl TextureFormat {
    pub fn channels(&self) -> u32 {
        match self {
            TextureFormat::Rgba8 => 4,
            TextureFormat::Rgb8 => 3,
            TextureFormat::R8 => 1,
        }
    }
}

impl TextureData {
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

    pub fn solid_color(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            width: 1,
            height: 1,
            data: vec![r, g, b, a],
            format: TextureFormat::Rgba8,
        }
    }

    pub fn white() -> Self {
        Self::solid_color(255, 255, 255, 255)
    }

    pub fn black() -> Self {
        Self::solid_color(0, 0, 0, 255)
    }

    pub fn sample(&self, uv: Vec2) -> Option<Vec3> {
        let u = uv.x.fract();
        let v = 1.0 - uv.y.fract();

        let x = (u * self.width as f32) as u32;
        let y = (v * self.height as f32) as u32;

        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);

        let idx = ((y * self.width + x) * self.format.channels()) as usize;

        if idx + 2 < self.data.len() {
            let r = self.data[idx] as f32 / 255.0;
            let g = self.data[idx + 1] as f32 / 255.0;
            let b = self.data[idx + 2] as f32 / 255.0;
            Some(Vec3::new(r, g, b))
        } else {
            None
        }
    }
}

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

pub fn load_texture_from_bytes(bytes: &[u8]) -> Result<TextureData, LoadError> {
    let image = image::load_from_memory(bytes)
        .map_err(|e| LoadError::LoadFailed(format!("Failed to decode image: {}", e)))?;
    Ok(TextureData::from_image(image))
}
