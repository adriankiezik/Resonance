
use thiserror::Error;

pub type Result<T> = std::result::Result<T, FerriteError>;

#[derive(Debug, Error)]
pub enum FerriteError {

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

impl FerriteError {

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

impl From<anyhow::Error> for FerriteError {
    fn from(err: anyhow::Error) -> Self {
        FerriteError::Custom(err.to_string())
    }
}