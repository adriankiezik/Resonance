//! Application and engine core module.
//!
//! This module contains the main engine struct, plugin system, and execution stages.
//!
//! # System Ordering Guide
//!
//! Resonance uses Bevy ECS for system scheduling. Some systems have critical ordering
//! requirements that must be maintained for correct behavior.
//!
//! ## Critical System Dependencies
//!
//! ### 1. Transform Propagation Before Frustum Culling
//!
//! **Location**: `renderer/plugin.rs:52-53`
//!
//! **Rule**: `prepare_indirect_draw_data` must run AFTER `propagate_transforms`
//!
//! **Why**: The camera's Transform is updated by user systems (e.g., FlyCam) in the Update
//! stage. In PostUpdate, we need Transform → GlobalTransform synchronization to complete
//! before we read the camera's GlobalTransform to compute the view frustum for culling.
//!
//! **Without this ordering**: Systems might execute in parallel, causing `prepare_indirect`
//! to read a stale GlobalTransform and compute frustum from the previous frame's camera
//! position/rotation. This causes one-frame lag and flickering at frustum edges.
//!
//! **Implementation**:
//! ```rust,ignore
//! schedule.add_systems((
//!     crate::renderer::systems::prepare_indirect_draw_data
//!         .after(crate::transform::systems::propagate_transforms),
//!     // other systems...
//! ));
//! ```
//!
//! ### 2. Mesh Upload Before Rendering
//!
//! **Location**: `renderer/plugin.rs:28`
//!
//! **Rule**: `upload_meshes` runs in PreUpdate, before Render stage
//!
//! **Why**: GPU buffers must exist before draw calls reference them.
//!
//! ### 3. Input Handling in PreUpdate
//!
//! **Location**: Input plugin
//!
//! **Rule**: Input systems run in PreUpdate stage
//!
//! **Why**: User input must be available for game logic in Update stage.
//!
//! ## Adding Systems with Dependencies
//!
//! When adding new systems, use Bevy's `.before()` and `.after()` system ordering:
//!
//! ```rust
//! use resonance::prelude::*;
//!
//! Resonance::new()
//!     .add_plugin(DefaultPlugins)
//!     .add_system(Stage::Update, my_system.after(other_system))
//!     .run();
//! ```

pub mod default_plugins;
pub mod engine;
pub mod plugin;
pub mod runner;
pub mod stage;

pub use default_plugins::DefaultPlugins;
pub use engine::{Resonance, ResonanceMode};
pub use plugin::{CorePlugin, Plugin, PluginMetadata, PluginState};
pub use stage::Stage;
