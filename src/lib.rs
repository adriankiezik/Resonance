pub mod app;
pub mod assets;
pub mod core;
pub mod transform;

#[cfg(feature = "input")]
pub mod input;

#[cfg(feature = "renderer")]
pub mod renderer;

#[cfg(feature = "window")]
pub mod window;

#[cfg(feature = "audio")]
pub mod audio;

pub mod build_utils;

pub mod prelude;

pub use bevy_ecs;
pub use glam;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use prelude::*;
