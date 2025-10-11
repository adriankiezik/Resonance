//! Font asset loading.
//!
//! Loads TrueType fonts (TTF) using ab_glyph.

use crate::loader::{AssetLoader, LoadError};
use ab_glyph::{Font, FontArc, ScaleFont};
use std::path::Path;

/// Loaded font data
#[derive(Clone)]
pub struct FontData {
    /// The font arc (can be cheaply cloned)
    pub font: FontArc,
    /// Font family name
    pub family_name: String,
}

impl FontData {
    pub fn new(font: FontArc) -> Self {
        // Try to get family name from font
        let family_name = "Unknown".to_string(); // ab_glyph doesn't expose this easily

        Self { font, family_name }
    }

    /// Get font metrics
    pub fn ascent(&self, size: f32) -> f32 {
        self.font.as_scaled(size).ascent()
    }

    pub fn descent(&self, size: f32) -> f32 {
        self.font.as_scaled(size).descent()
    }

    pub fn line_gap(&self, size: f32) -> f32 {
        self.font.as_scaled(size).line_gap()
    }

    /// Get height for a given font size
    pub fn height(&self, size: f32) -> f32 {
        let scaled = self.font.as_scaled(size);
        scaled.ascent() - scaled.descent() + scaled.line_gap()
    }
}

// Note: Fallback font should be provided by the application
// as it requires including font data in the binary

/// TTF font loader
pub struct TtfLoader;

impl AssetLoader for TtfLoader {
    type Asset = FontData;

    fn load(&self, path: &Path) -> Result<Self::Asset, LoadError> {
        let bytes = std::fs::read(path)
            .map_err(|e| LoadError::LoadFailed(format!("Failed to read font file: {}", e)))?;

        let font = FontArc::try_from_vec(bytes)
            .map_err(|e| LoadError::LoadFailed(format!("Failed to parse TTF font: {:?}", e)))?;

        log::info!("Loaded TTF font from {:?}", path);

        Ok(FontData::new(font))
    }

    fn extensions(&self) -> &[&str] {
        &["ttf", "otf"]
    }
}

/// Load font from raw bytes
pub fn load_font_from_bytes(bytes: &[u8]) -> Result<FontData, LoadError> {
    let font = FontArc::try_from_vec(bytes.to_vec())
        .map_err(|e| LoadError::LoadFailed(format!("Failed to parse font: {:?}", e)))?;

    Ok(FontData::new(font))
}
