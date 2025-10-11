//! Material system for rendering.

use bevy_ecs::prelude::*;
use ferrite_core::math::*;

/// Material component that defines how a mesh should be rendered
#[derive(Component, Debug, Clone)]
pub enum Material {
    /// Solid color material (no texture)
    Color(ColorMaterial),
    /// Textured material with optional color tint
    Textured(TexturedMaterial),
}

impl Material {
    /// Create a solid color material
    pub fn color(color: Vec4) -> Self {
        Self::Color(ColorMaterial { color })
    }

    /// Create a textured material
    pub fn textured() -> Self {
        Self::Textured(TexturedMaterial { tint: Vec4::ONE })
    }

    /// Create a textured material with a color tint
    pub fn textured_with_tint(tint: Vec4) -> Self {
        Self::Textured(TexturedMaterial { tint })
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::Color(ColorMaterial {
            color: Vec4::ONE, // White
        })
    }
}

/// Color material (no texture, just vertex colors or solid color)
#[derive(Debug, Clone, Copy)]
pub struct ColorMaterial {
    /// Base color of the material
    pub color: Vec4,
}

/// Textured material
#[derive(Debug, Clone, Copy)]
pub struct TexturedMaterial {
    /// Color tint applied to the texture
    pub tint: Vec4,
}
