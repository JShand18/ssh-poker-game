use thiserror::Error;

/// Database error types
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLx database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    
    #[error("Migration error: {0}")]
    Migration(String),
    
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("Game not found: {0}")]
    GameNotFound(String),
    
    #[error("Duplicate user: {0}")]
    DuplicateUser(String),
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Database operation failed: {0}")]
    OperationFailed(String),
}

/// Result type for database operations
pub type DatabaseResult<T> = Result<T, DatabaseError>; 