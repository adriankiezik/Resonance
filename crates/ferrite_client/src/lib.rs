//! Client-only functionality for the Ferrite engine.
//!
//! Provides:
//! - Window management (winit)
//! - Rendering (wgpu)
//! - Input handling (keyboard, mouse, gamepad)
//! - Audio playback (rodio)

pub mod audio;
pub mod input;
pub mod plugin;
pub mod renderer;
pub mod window;

pub use plugin::ClientPlugin;
pub use window::{run_with_window, Window, WindowConfig, WindowMode};

// Re-export commonly used types for examples
pub use winit::keyboard::KeyCode;
