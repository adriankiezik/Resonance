//! Transform system for spatial hierarchy.
//!
//! Provides Transform (local) and GlobalTransform (world space) components,
//! along with parent-child hierarchy support.

pub mod components;
pub mod hierarchy;
pub mod plugin;
pub mod systems;

pub use components::{GlobalTransform, Transform};
pub use hierarchy::{Children, Parent};
pub use plugin::TransformPlugin;
