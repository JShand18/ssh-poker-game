use thiserror::Error;

#[derive(Error, Debug)]
pub enum PokerError {
    #[error("Insufficient chips: needed {needed}, available {available}")]
    InsufficientChips { needed: u64, available: u64 },
    
    #[error("Invalid action: {0}")]
    InvalidAction(String),
    
    #[error("Invalid game state: {0}")]
    InvalidGameState(String),
    
    #[error("Player not found: {0}")]
    PlayerNotFound(usize),
    
    #[error("Invalid bet amount: {0}")]
    InvalidBetAmount(String),
    
    #[error("Hand evaluation failed: {0}")]
    HandEvaluationError(String),
}

pub type Result<T> = std::result::Result<T, PokerError>; 