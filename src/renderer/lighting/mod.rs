pub mod components;

pub use components::{AmbientLight, DirectionalLight, PointLight};

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct DirectionalLightUniform {
    pub direction: [f32; 3],
    pub intensity: f32,
    pub color: [f32; 3],
    pub _padding: f32,
}

impl DirectionalLightUniform {
    pub fn from_light(light: &DirectionalLight) -> Self {
        Self {
            direction: light.direction.normalize().to_array(),
            intensity: light.intensity,
            color: light.color.to_array(),
            _padding: 0.0,
        }
    }
}

impl Default for DirectionalLightUniform {
    fn default() -> Self {
        Self {
            direction: [0.0, -1.0, 0.0],
            intensity: 1.0,
            color: [1.0, 1.0, 1.0],
            _padding: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PointLightUniform {
    pub position: [f32; 3],
    pub intensity: f32,
    pub color: [f32; 3],
    pub radius: f32,
}

impl PointLightUniform {
    pub fn from_light(light: &PointLight) -> Self {
        Self {
            position: light.position.to_array(),
            intensity: light.intensity,
            color: light.color.to_array(),
            radius: light.radius,
        }
    }
}

impl Default for PointLightUniform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            intensity: 0.0,
            color: [0.0, 0.0, 0.0],
            radius: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct AmbientLightUniform {
    pub color: [f32; 3],
    pub intensity: f32,
}

impl AmbientLightUniform {
    pub fn from_light(light: &AmbientLight) -> Self {
        Self {
            color: light.color.to_array(),
            intensity: light.intensity,
        }
    }
}

impl Default for AmbientLightUniform {
    fn default() -> Self {
        Self {
            color: [0.3, 0.3, 0.3],
            intensity: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LightingUniform {
    pub directional: DirectionalLightUniform,
    pub ambient: AmbientLightUniform,
    pub point_light_count: u32,
    pub ao_mode: u32,
    pub ao_debug: u32,
    pub _padding1: f32,
    pub _padding2: [f32; 3],
    pub _padding3: f32,
    pub _padding4: [f32; 3],
    pub _padding5: f32,
}

impl Default for LightingUniform {
    fn default() -> Self {
        Self {
            directional: DirectionalLightUniform::default(),
            ambient: AmbientLightUniform::default(),
            point_light_count: 0,
            ao_mode: 0,
            ao_debug: 0,
            _padding1: 0.0,
            _padding2: [0.0; 3],
            _padding3: 0.0,
            _padding4: [0.0; 3],
            _padding5: 0.0,
        }
    }
}
