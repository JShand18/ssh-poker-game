use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// User model for player accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
}

/// New user data for creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
}

/// Game session model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub game_type: String, // "texas_holdem", etc.
    pub status: GameStatus,
    pub pot_size: i64, // in cents to avoid floating point issues
    pub small_blind: i64,
    pub big_blind: i64,
    pub max_players: i32,
    pub current_players: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub winner_id: Option<String>,
}

/// Game status enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i32)]
pub enum GameStatus {
    Waiting = 0,
    InProgress = 1,
    Finished = 2,
    Cancelled = 3,
}

/// Game participant model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameParticipant {
    pub id: String,
    pub game_id: String,
    pub user_id: String,
    pub seat_position: i32,
    pub starting_chips: i64,
    pub final_chips: i64,
    pub is_winner: bool,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
}

/// Player statistics model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub user_id: String,
    pub games_played: i32,
    pub games_won: i32,
    pub total_winnings: i64, // in cents
    pub total_losses: i64,   // in cents
    pub biggest_win: i64,
    pub biggest_loss: i64,
    pub average_session_length: i32, // in minutes
    pub last_updated: DateTime<Utc>,
}

/// Game event for detailed history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub id: String,
    pub game_id: String,
    pub user_id: Option<String>,
    pub event_type: String, // "bet", "fold", "call", "raise", "deal", etc.
    pub event_data: Option<String>, // JSON data for event details
    pub amount: Option<i64>, // For bet/raise events
    pub created_at: DateTime<Utc>,
}

impl User {
    /// Create a new user with generated ID and timestamps
    pub fn new(username: String, email: Option<String>, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            email,
            password_hash,
            created_at: now,
            updated_at: now,
            is_active: true,
            last_login: None,
        }
    }
}

impl Game {
    /// Create a new game with generated ID and timestamps
    pub fn new(
        game_type: String,
        small_blind: i64,
        big_blind: i64,
        max_players: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            game_type,
            status: GameStatus::Waiting,
            pot_size: 0,
            small_blind,
            big_blind,
            max_players,
            current_players: 0,
            created_at: Utc::now(),
            started_at: None,
            ended_at: None,
            winner_id: None,
        }
    }
}

impl GameParticipant {
    /// Create a new game participant
    pub fn new(
        game_id: String,
        user_id: String,
        seat_position: i32,
        starting_chips: i64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            game_id,
            user_id,
            seat_position,
            starting_chips,
            final_chips: starting_chips,
            is_winner: false,
            joined_at: Utc::now(),
            left_at: None,
        }
    }
}

impl GameEvent {
    /// Create a new game event
    pub fn new(
        game_id: String,
        user_id: Option<String>,
        event_type: String,
        event_data: Option<String>,
        amount: Option<i64>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            game_id,
            user_id,
            event_type,
            event_data,
            amount,
            created_at: Utc::now(),
        }
    }
} 