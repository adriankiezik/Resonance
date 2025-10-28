// Re-export essential Bevy ECS types for user convenience
pub use bevy_ecs::prelude::{
    Commands, Component, Entity, In, IntoSystem, Local, Query, Res, ResMut, Resource, With,
    Without, World,
};

// Engine core (CorePlugin is internal only, not exposed)
pub use crate::app::{DefaultPlugins, Plugin, Resonance, ResonanceMode, Stage};

// Assets
pub use crate::assets::{AssetCache, AssetHandle, AssetId, AssetsPlugin};

// Audio
pub use crate::audio::{AudioListener, AudioPlugin, AudioSource, Spatial3dAudio};

// Core utilities
pub use crate::core::{
    FixedTime, GameTick, PerformanceAnalytics, PerformancePlugin, ResonanceError, Result, Time,
    TimePlugin,
};

// Input
pub use crate::input::{Input, InputPlugin, KeyCode};

// Renderer (including commonly used graphics settings)
pub use crate::renderer::{Camera, GraphicsSettings, Mesh, MsaaSampleCount, RenderPlugin, Renderer};

// Transforms
pub use crate::transform::{Children, GlobalTransform, Parent, Transform, TransformPlugin};

// Window
pub use crate::window::{Window, WindowConfig, WindowMode, WindowPlugin};

// Math - re-export commonly used glam types
pub use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
