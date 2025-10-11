//! Unified error handling for the Ferrite engine.
//!
//! This module provides common error types used across the engine to ensure
//! consistent and idiomatic error handling patterns.

use thiserror::Error;

/// Common result type using FerriteError
pub type Result<T> = std::result::Result<T, FerriteError>;

/// Top-level error type for the Ferrite engine
#[derive(Debug, Error)]
pub enum FerriteError {
    /// IO errors (file operations, network, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Asset loading errors
    #[error("Asset loading error: {0}")]
    AssetLoad(String),

    /// Network errors
    #[error("Network error: {0}")]
    Network(String),

    /// Physics errors
    #[error("Physics error: {0}")]
    Physics(String),

    /// Rendering errors
    #[error("Rendering error: {0}")]
    Rendering(String),

    /// Audio errors
    #[error("Audio error: {0}")]
    Audio(String),

    /// Scene errors
    #[error("Scene error: {0}")]
    Scene(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Invalid operation or state
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Generic error with custom message
    #[error("{0}")]
    Custom(String),
}

impl FerriteError {
    /// Create a custom error with a message
    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }

    /// Create an asset loading error
    pub fn asset_load(msg: impl Into<String>) -> Self {
        Self::AssetLoad(msg.into())
    }

    /// Create a scene error
    pub fn scene(msg: impl Into<String>) -> Self {
        Self::Scene(msg.into())
    }

    /// Create a serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Create a not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create an invalid operation error
    pub fn invalid_operation(msg: impl Into<String>) -> Self {
        Self::InvalidOperation(msg.into())
    }
}

// Convert from anyhow::Error
impl From<anyhow::Error> for FerriteError {
    fn from(err: anyhow::Error) -> Self {
        FerriteError::Custom(err.to_string())
    }
}
