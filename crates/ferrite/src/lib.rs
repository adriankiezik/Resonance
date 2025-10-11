//! # Ferrite Game Engine
//!
//! A multiplayer-ready game engine built in Rust with a focus on:
//! - **Performance**: Built on bevy_ecs for high-performance ECS
//! - **Modularity**: Plugin-based architecture for extensibility
//! - **Networking**: First-class multiplayer support with client-server architecture
//! - **Cross-platform**: Runs on Windows, macOS, and Linux
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use ferrite::prelude::*;
//!
//! fn main() {
//!     let mut engine = Engine::new()
//!         .add_plugin(CorePlugin)
//!         .add_plugin(TransformPlugin)
//!         .add_system(Stage::Update, my_game_system);
//!
//!     // For now, manually call startup and update
//!     // The .run() method will be implemented later
//!     engine.startup();
//!     engine.update();
//! }
//!
//! fn my_game_system() {
//!     // Your game logic here
//! }
//! ```
//!
//! ## Features
//!
//! - `client`: Client-only features (rendering, audio, input)
//! - `server`: Server-only features (authority, validation)
//! - `full`: Both client and server features
//!
//! ## Architecture
//!
//! The engine is organized into several crates:
//!
//! - **ferrite_core**: Core utilities (time, math, logging)
//! - **ferrite_app**: Application framework (engine, plugins, stages)
//! - **ferrite_transform**: Transform system and spatial hierarchy
//! - **ferrite_physics**: Basic physics simulation
//! - **ferrite_network**: Multiplayer networking
//! - **ferrite_client**: Client rendering and input
//! - **ferrite_server**: Server authority and validation
//! - **ferrite_assets**: Asset loading and management
//! - **ferrite_scene**: Scene management and entity prefabs

pub mod prelude;

// Re-export core crates
pub use ferrite_app as app;
pub use ferrite_assets as assets;
pub use ferrite_core as core;
pub use ferrite_network as network;
pub use ferrite_physics as physics;
pub use ferrite_scene as scene;
pub use ferrite_transform as transform;

// Re-export client and server based on features
#[cfg(feature = "client")]
pub use ferrite_client as client;

#[cfg(feature = "server")]
pub use ferrite_server as server;

// Re-export commonly used types at the root level
pub use bevy_ecs;
pub use glam;

// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
