//! Error handling conventions for Resonance Engine.
//!
//! # Error Handling Guidelines
//!
//! Resonance uses different error handling strategies depending on the context:
//!
//! ## 1. Plugin Initialization (`plugin.rs` build methods)
//!
//! **Use**: Generally infallible - plugins should not fail to load
//!
//! **Pattern**: Log errors and continue with degraded functionality
//!
//! ```rust,ignore
//! fn build(&self, engine: &mut Resonance) {
//!     if let Err(e) = some_fallible_operation() {
//!         log::error!("Failed to initialize feature: {}", e);
//!         // Continue without this feature
//!     }
//! }
//! ```
//!
//! ## 2. Asset Loaders (`assets/loader/*.rs`)
//!
//! **Use**: `Result<T, LoadError>` for asset-specific failures
//!
//! **Pattern**: Return detailed errors, caller decides how to handle
//!
//! ```rust,ignore
//! fn load(&self, path: &Path) -> Result<TextureData, LoadError> {
//!     std::fs::read(path)
//!         .map_err(|e| LoadError::IoError(e.to_string()))?;
//!     // ...
//! }
//! ```
//!
//! ## 3. Rendering Code (`renderer/**/*.rs`)
//!
//! **Use**: `anyhow::Result<T>` for flexibility with wgpu errors
//!
//! **Pattern**: Use `?` operator for error propagation, log at top level
//!
//! ```rust,ignore
//! fn render(&mut self) -> anyhow::Result<()> {
//!     let output = self.surface.get_current_texture()?;
//!     // ... rendering code
//!     Ok(())
//! }
//! ```
//!
//! ## 4. ECS Systems (`systems/**/*.rs`)
//!
//! **Use**: Generally infallible - log errors instead of propagating
//!
//! **Pattern**: Systems should not panic or return errors
//!
//! ```rust,ignore
//! fn my_system(query: Query<&Transform>) {
//!     for transform in query.iter() {
//!         if let Err(e) = do_something(transform) {
//!             log::warn!("Failed to process entity: {}", e);
//!             // Continue processing other entities
//!         }
//!     }
//! }
//! ```
//!
//! ## 5. User-Facing API (`app/engine.rs`, public interfaces)
//!
//! **Use**: `Result<T, ResonanceError>` for errors users should handle
//!
//! **Pattern**: Provide actionable error messages
//!
//! ```rust,ignore
//! pub fn initialize(&mut self) -> Result<()> {
//!     self.renderer.init()
//!         .map_err(|e| ResonanceError::Rendering(format!("GPU init failed: {}", e)))?;
//!     Ok(())
//! }
//! ```
//!
//! ## Error Type Selection Guide
//!
//! | Context | Error Type | When to Use |
//! |---------|-----------|-------------|
//! | Asset loading | `LoadError` | File I/O, parsing errors |
//! | Rendering | `anyhow::Result` | wgpu/GPU errors |
//! | Public API | `ResonanceError` | User-facing errors |
//! | ECS Systems | None (log only) | Runtime game logic |
//! | Plugin init | None (log only) | Optional features |

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ResonanceError>;

#[derive(Debug, Error)]
pub enum ResonanceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Asset loading error: {0}")]
    AssetLoad(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Physics error: {0}")]
    Physics(String),
    #[error("Rendering error: {0}")]
    Rendering(String),
    #[error("Audio error: {0}")]
    Audio(String),
    #[error("Scene error: {0}")]
    Scene(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("{0}")]
    Custom(String),
}

impl ResonanceError {
    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }

    pub fn asset_load(msg: impl Into<String>) -> Self {
        Self::AssetLoad(msg.into())
    }

    pub fn scene(msg: impl Into<String>) -> Self {
        Self::Scene(msg.into())
    }

    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn invalid_operation(msg: impl Into<String>) -> Self {
        Self::InvalidOperation(msg.into())
    }
}

impl From<anyhow::Error> for ResonanceError {
    fn from(err: anyhow::Error) -> Self {
        ResonanceError::Custom(err.to_string())
    }
}
