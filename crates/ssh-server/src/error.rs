use thiserror::Error;

/// SSH Server errors
#[derive(Debug, thiserror::Error)]
pub enum SshError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Russh error: {0}")]
    Russh(#[from] russh::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Result type for SSH operations
pub type Result<T> = std::result::Result<T, SshError>; 