//! Asset management system.
//!
//! Provides:
//! - Asset loading from disk
//! - Asset caching
//! - Hot reloading (TODO)
//! - Asset handles for safe references

pub mod cache;
pub mod handle;
pub mod loader;
pub mod plugin;

pub use cache::AssetCache;
pub use handle::{AssetHandle, AssetId};
pub use loader::{AssetLoader, LoadError};
pub use plugin::AssetsPlugin;
