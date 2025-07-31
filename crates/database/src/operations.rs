use sqlx::{SqlitePool, Row};
use chrono::Utc;
use crate::models::{User, NewUser, Game, PlayerStats, GameStatus};
use crate::error::{DatabaseError, DatabaseResult};
use log::info;

/// User operations
pub struct UserOperations;

impl UserOperations {
    /// Create a new user
    pub async fn create(pool: &SqlitePool, new_user: NewUser) -> DatabaseResult<User> {
        let user = User::new(new_user.username, new_user.email, new_user.password_hash);
        
        let result = sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, created_at, updated_at, is_active, last_login)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.created_at.to_rfc3339())
        .bind(user.updated_at.to_rfc3339())
        .bind(user.is_active)
        .bind(user.last_login.map(|dt| dt.to_rfc3339()))
        .execute(pool)
        .await;

        match result {
            Ok(_) => {
                info!("Created user: {}", user.username);
                Ok(user)
            }
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(DatabaseError::DuplicateUser(user.username))
            }
            Err(e) => Err(DatabaseError::Sqlx(e)),
        }
    }

    /// Find user by username
    pub async fn find_by_username(pool: &SqlitePool, username: &str) -> DatabaseResult<Option<User>> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, updated_at, is_active, last_login FROM users WHERE username = ? AND is_active = 1"
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let user = User {
                    id: row.get("id"),
                    username: row.get("username"),
                    email: row.get("email"),
                    password_hash: row.get("password_hash"),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                        .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
                        .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?
                        .with_timezone(&chrono::Utc),
                    is_active: row.get("is_active"),
                    last_login: row.get::<Option<String>, _>("last_login")
                        .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&chrono::Utc)))
                        .transpose()
                        .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?,
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    /// Find user by ID
    pub async fn find_by_id(pool: &SqlitePool, user_id: &str) -> DatabaseResult<Option<User>> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, updated_at, is_active, last_login FROM users WHERE id = ? AND is_active = 1"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let user = User {
                    id: row.get("id"),
                    username: row.get("username"),
                    email: row.get("email"),
                    password_hash: row.get("password_hash"),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                        .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
                        .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?
                        .with_timezone(&chrono::Utc),
                    is_active: row.get("is_active"),
                    last_login: row.get::<Option<String>, _>("last_login")
                        .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&chrono::Utc)))
                        .transpose()
                        .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?,
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    /// Update user's last login time
    pub async fn update_last_login(pool: &SqlitePool, user_id: &str) -> DatabaseResult<()> {
        let now = Utc::now();
        sqlx::query("UPDATE users SET last_login = ?, updated_at = ? WHERE id = ?")
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// List all active users
    pub async fn list_active(pool: &SqlitePool) -> DatabaseResult<Vec<User>> {
        let rows = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, updated_at, is_active, last_login FROM users WHERE is_active = 1 ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await?;

        let mut users = Vec::new();
        for row in rows {
            let user = User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
                    .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?
                    .with_timezone(&chrono::Utc),
                is_active: row.get("is_active"),
                last_login: row.get::<Option<String>, _>("last_login")
                    .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&chrono::Utc)))
                    .transpose()
                    .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?,
            };
            users.push(user);
        }

        Ok(users)
    }
}

/// Game operations
pub struct GameOperations;

impl GameOperations {
    /// Create a new game
    pub async fn create(
        pool: &SqlitePool,
        game_type: String,
        small_blind: i64,
        big_blind: i64,
        max_players: i32,
    ) -> DatabaseResult<Game> {
        let game = Game::new(game_type, small_blind, big_blind, max_players);
        
        sqlx::query(
            r#"
            INSERT INTO games (id, game_type, status, pot_size, small_blind, big_blind, 
                             max_players, current_players, created_at, started_at, ended_at, winner_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&game.id)
        .bind(&game.game_type)
        .bind(game.status as i32)  // Now Copy, no need to clone
        .bind(game.pot_size)
        .bind(game.small_blind)
        .bind(game.big_blind)
        .bind(game.max_players)
        .bind(game.current_players)
        .bind(game.created_at.to_rfc3339())
        .bind(game.started_at.map(|dt| dt.to_rfc3339()))
        .bind(game.ended_at.map(|dt| dt.to_rfc3339()))
        .bind(&game.winner_id)
        .execute(pool)
        .await?;

        info!("Created game: {}", game.id);
        Ok(game)
    }

    /// Find game by ID
    pub async fn find_by_id(pool: &SqlitePool, game_id: &str) -> DatabaseResult<Option<Game>> {
        let row = sqlx::query(
            "SELECT id, game_type, status, pot_size, small_blind, big_blind, max_players, current_players, created_at, started_at, ended_at, winner_id FROM games WHERE id = ?"
        )
        .bind(game_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let status_int: i32 = row.get("status");
                let status = match status_int {
                    0 => GameStatus::Waiting,
                    1 => GameStatus::InProgress,
                    2 => GameStatus::Finished,
                    3 => GameStatus::Cancelled,
                    _ => return Err(DatabaseError::OperationFailed("Invalid game status".to_string())),
                };

                let game = Game {
                    id: row.get("id"),
                    game_type: row.get("game_type"),
                    status,
                    pot_size: row.get("pot_size"),
                    small_blind: row.get("small_blind"),
                    big_blind: row.get("big_blind"),
                    max_players: row.get("max_players"),
                    current_players: row.get("current_players"),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                        .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?
                        .with_timezone(&chrono::Utc),
                    started_at: row.get::<Option<String>, _>("started_at")
                        .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&chrono::Utc)))
                        .transpose()
                        .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?,
                    ended_at: row.get::<Option<String>, _>("ended_at")
                        .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&chrono::Utc)))
                        .transpose()
                        .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?,
                    winner_id: row.get("winner_id"),
                };
                Ok(Some(game))
            }
            None => Ok(None),
        }
    }

    /// List waiting games
    pub async fn list_waiting(pool: &SqlitePool) -> DatabaseResult<Vec<Game>> {
        let rows = sqlx::query(
            "SELECT id, game_type, status, pot_size, small_blind, big_blind, max_players, current_players, created_at, started_at, ended_at, winner_id FROM games WHERE status = ? ORDER BY created_at ASC"
        )
        .bind(GameStatus::Waiting as i32)
        .fetch_all(pool)
        .await?;

        let mut games = Vec::new();
        for row in rows {
            let game = Game {
                id: row.get("id"),
                game_type: row.get("game_type"),
                status: GameStatus::Waiting, // We know it's waiting from the query
                pot_size: row.get("pot_size"),
                small_blind: row.get("small_blind"),
                big_blind: row.get("big_blind"),
                max_players: row.get("max_players"),
                current_players: row.get("current_players"),
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?
                    .with_timezone(&chrono::Utc),
                started_at: row.get::<Option<String>, _>("started_at")
                    .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&chrono::Utc)))
                    .transpose()
                    .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?,
                ended_at: row.get::<Option<String>, _>("ended_at")
                    .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&chrono::Utc)))
                    .transpose()
                    .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?,
                winner_id: row.get("winner_id"),
            };
            games.push(game);
        }

        Ok(games)
    }

    /// Update game status
    pub async fn update_status(
        pool: &SqlitePool,
        game_id: &str,
        status: GameStatus,
    ) -> DatabaseResult<()> {
        sqlx::query("UPDATE games SET status = ? WHERE id = ?")
            .bind(status as i32)
            .bind(game_id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

/// Statistics operations
pub struct StatsOperations;

impl StatsOperations {
    /// Get or create player stats
    pub async fn get_or_create(pool: &SqlitePool, user_id: &str) -> DatabaseResult<PlayerStats> {
        // Try to get existing stats
        if let Some(stats) = Self::get_by_user_id(pool, user_id).await? {
            return Ok(stats);
        }

        // Create new stats
        let stats = PlayerStats {
            user_id: user_id.to_string(),
            games_played: 0,
            games_won: 0,
            total_winnings: 0,
            total_losses: 0,
            biggest_win: 0,
            biggest_loss: 0,
            average_session_length: 0,
            last_updated: Utc::now(),
        };

        sqlx::query(
            r#"
            INSERT INTO player_stats (user_id, games_played, games_won, total_winnings, 
                                    total_losses, biggest_win, biggest_loss, 
                                    average_session_length, last_updated)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&stats.user_id)
        .bind(stats.games_played)
        .bind(stats.games_won)
        .bind(stats.total_winnings)
        .bind(stats.total_losses)
        .bind(stats.biggest_win)
        .bind(stats.biggest_loss)
        .bind(stats.average_session_length)
        .bind(stats.last_updated.to_rfc3339())
        .execute(pool)
        .await?;

        Ok(stats)
    }

    /// Get player stats by user ID
    pub async fn get_by_user_id(pool: &SqlitePool, user_id: &str) -> DatabaseResult<Option<PlayerStats>> {
        let row = sqlx::query(
            "SELECT user_id, games_played, games_won, total_winnings, total_losses, biggest_win, biggest_loss, average_session_length, last_updated FROM player_stats WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let stats = PlayerStats {
                    user_id: row.get("user_id"),
                    games_played: row.get("games_played"),
                    games_won: row.get("games_won"),
                    total_winnings: row.get("total_winnings"),
                    total_losses: row.get("total_losses"),
                    biggest_win: row.get("biggest_win"),
                    biggest_loss: row.get("biggest_loss"),
                    average_session_length: row.get("average_session_length"),
                    last_updated: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("last_updated"))
                        .map_err(|e| DatabaseError::OperationFailed(format!("Date parse error: {}", e)))?
                        .with_timezone(&chrono::Utc),
                };
                Ok(Some(stats))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Database, DatabaseConfig};

    async fn setup_test_db() -> Database {
        let config = DatabaseConfig {
            database_path: ":memory:".to_string(),  // Use in-memory database for tests
            create_if_missing: true,
            max_connections: 5,
        };

        Database::new(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_user_operations() {
        let db = setup_test_db().await;
        
        // Test user creation
        let new_user = NewUser {
            username: "testuser".to_string(),
            email: Some("test@example.com".to_string()),
            password_hash: "hashedpassword".to_string(),
        };

        let user = UserOperations::create(db.pool(), new_user).await.unwrap();
        assert_eq!(user.username, "testuser");
        assert!(user.is_active);

        // Test finding user by username
        let found_user = UserOperations::find_by_username(db.pool(), "testuser")
            .await
            .unwrap();
        assert!(found_user.is_some());
        assert_eq!(found_user.unwrap().id, user.id);

        // Test finding user by ID
        let found_user = UserOperations::find_by_id(db.pool(), &user.id)
            .await
            .unwrap();
        assert!(found_user.is_some());

        // Test updating last login
        UserOperations::update_last_login(db.pool(), &user.id)
            .await
            .unwrap();

        db.close().await;
    }

    #[tokio::test]
    async fn test_game_operations() {
        let db = setup_test_db().await;
        
        // Test game creation
        let game = GameOperations::create(
            db.pool(),
            "texas_holdem".to_string(),
            100,  // small blind
            200,  // big blind
            6,    // max players
        ).await.unwrap();

        assert_eq!(game.game_type, "texas_holdem");
        assert_eq!(game.small_blind, 100);
        assert_eq!(game.big_blind, 200);
        assert_eq!(game.max_players, 6);

        // Test finding game by ID
        let found_game = GameOperations::find_by_id(db.pool(), &game.id)
            .await
            .unwrap();
        assert!(found_game.is_some());

        // Test listing waiting games
        let waiting_games = GameOperations::list_waiting(db.pool())
            .await
            .unwrap();
        assert_eq!(waiting_games.len(), 1);

        db.close().await;
    }

    #[tokio::test]
    async fn test_stats_operations() {
        let db = setup_test_db().await;
        
        // Create a user first
        let new_user = NewUser {
            username: "statsuser".to_string(),
            email: None,
            password_hash: "hashedpassword".to_string(),
        };
        let user = UserOperations::create(db.pool(), new_user).await.unwrap();

        // Test getting or creating stats
        let stats = StatsOperations::get_or_create(db.pool(), &user.id)
            .await
            .unwrap();
        assert_eq!(stats.user_id, user.id);
        assert_eq!(stats.games_played, 0);

        // Test getting existing stats
        let existing_stats = StatsOperations::get_by_user_id(db.pool(), &user.id)
            .await
            .unwrap();
        assert!(existing_stats.is_some());

        db.close().await;
    }
} 