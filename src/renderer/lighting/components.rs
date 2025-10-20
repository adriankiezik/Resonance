use bevy_ecs::prelude::Component;
use glam::Vec3;

#[derive(Component, Clone, Debug)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub cast_shadows: bool,
}

impl DirectionalLight {
    pub fn new(direction: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            direction: direction.normalize(),
            color,
            intensity,
            cast_shadows: true,
        }
    }

    pub fn sun() -> Self {
        Self::new(
            Vec3::new(0.5, -1.0, 0.3),
            Vec3::new(1.0, 0.98, 0.95),
            1.0,
        )
    }
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self::sun()
    }
}

#[derive(Component, Clone, Debug)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub radius: f32,
    pub cast_shadows: bool,
}

impl PointLight {
    pub fn new(position: Vec3, color: Vec3, intensity: f32, radius: f32) -> Self {
        Self {
            position,
            color,
            intensity,
            radius,
            cast_shadows: false,
        }
    }

    pub fn attenuation(&self, distance: f32) -> f32 {
        let ratio = distance / self.radius;
        let attenuation = 1.0 - ratio.powi(4);
        attenuation.max(0.0) / (1.0 + distance * distance)
    }
}

impl Default for PointLight {
    fn default() -> Self {
        Self::new(Vec3::ZERO, Vec3::ONE, 1.0, 10.0)
    }
}

#[derive(Component, Clone, Debug)]
pub struct AmbientLight {
    pub color: Vec3,
    pub intensity: f32,
}

impl AmbientLight {
    pub fn new(color: Vec3, intensity: f32) -> Self {
        Self { color, intensity }
    }
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self::new(Vec3::new(0.4, 0.5, 0.6), 0.3)
    }
}
