use thiserror::Error;

#[derive(Error, Debug)]
pub enum TuiError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Terminal error: {0}")]
    Terminal(String),
    
    #[error("Render error: {0}")]
    Render(String),
    
    #[error("Game error: {0}")]
    Game(String),
    
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, TuiError>; 