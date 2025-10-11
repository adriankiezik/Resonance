
pub mod prelude;

pub use resonance_app as app;
pub use resonance_assets as assets;
pub use resonance_core as core;
pub use resonance_transform as transform;

#[cfg(feature = "client")]
pub use resonance_input as input;

#[cfg(feature = "client")]
pub use resonance_renderer as renderer;

#[cfg(feature = "client")]
pub use resonance_window as window;

#[cfg(feature = "audio")]
pub use resonance_audio as audio;

pub use bevy_ecs;
pub use glam;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");