//! Application framework for the Ferrite game engine.
//!
//! This crate provides the core application structure:
//! - Engine builder and runner
//! - Plugin system for modular features
//! - Execution stages for organizing systems
//! - Main game loop

pub mod engine;
pub mod plugin;
pub mod runner;
pub mod stage;

pub use engine::{Engine, EngineMode};
pub use plugin::{Plugin, PluginMetadata, PluginState};
pub use runner::run;
pub use stage::Stage;
