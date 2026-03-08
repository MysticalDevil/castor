use std::path::PathBuf;
use thiserror::Error;

/// Custom error type for Castor.
#[derive(Debug, Error)]
pub enum CastorError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("WalkDir error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Audit log error: {0}")]
    Audit(String),

    #[error("Execution failed: {0}")]
    Execution(String),

    #[error("Batch not found: {0}")]
    BatchNotFound(String),
}

/// A specialized Result type for Castor operations.
pub type Result<T> = std::result::Result<T, CastorError>;
