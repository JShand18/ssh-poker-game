use sqlx::SqlitePool;
use crate::error::DatabaseResult;
use log::info;

/// Create all database tables
pub async fn create_tables(pool: &SqlitePool) -> DatabaseResult<()> {
    info!("Creating database tables");
    
    create_users_table(pool).await?;
    create_games_table(pool).await?;
    create_game_participants_table(pool).await?;
    create_player_stats_table(pool).await?;
    create_game_events_table(pool).await?;
    
    info!("All database tables created successfully");
    Ok(())
}

/// Create the users table
async fn create_users_table(pool: &SqlitePool) -> DatabaseResult<()> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE,
            password_hash TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT 1,
            last_login TEXT
        );
        
        CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
        CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
        CREATE INDEX IF NOT EXISTS idx_users_active ON users(is_active);
    "#;
    
    sqlx::query(sql).execute(pool).await?;
    info!("Users table created");
    Ok(())
}

/// Create the games table
async fn create_games_table(pool: &SqlitePool) -> DatabaseResult<()> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS games (
            id TEXT PRIMARY KEY,
            game_type TEXT NOT NULL,
            status INTEGER NOT NULL DEFAULT 0,
            pot_size INTEGER NOT NULL DEFAULT 0,
            small_blind INTEGER NOT NULL,
            big_blind INTEGER NOT NULL,
            max_players INTEGER NOT NULL,
            current_players INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            started_at TEXT,
            ended_at TEXT,
            winner_id TEXT,
            FOREIGN KEY (winner_id) REFERENCES users(id)
        );
        
        CREATE INDEX IF NOT EXISTS idx_games_status ON games(status);
        CREATE INDEX IF NOT EXISTS idx_games_type ON games(game_type);
        CREATE INDEX IF NOT EXISTS idx_games_created ON games(created_at);
    "#;
    
    sqlx::query(sql).execute(pool).await?;
    info!("Games table created");
    Ok(())
}

/// Create the game_participants table
async fn create_game_participants_table(pool: &SqlitePool) -> DatabaseResult<()> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS game_participants (
            id TEXT PRIMARY KEY,
            game_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            seat_position INTEGER NOT NULL,
            starting_chips INTEGER NOT NULL,
            final_chips INTEGER NOT NULL,
            is_winner BOOLEAN NOT NULL DEFAULT 0,
            joined_at TEXT NOT NULL,
            left_at TEXT,
            FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id),
            UNIQUE(game_id, seat_position),
            UNIQUE(game_id, user_id)
        );
        
        CREATE INDEX IF NOT EXISTS idx_participants_game ON game_participants(game_id);
        CREATE INDEX IF NOT EXISTS idx_participants_user ON game_participants(user_id);
        CREATE INDEX IF NOT EXISTS idx_participants_winner ON game_participants(is_winner);
    "#;
    
    sqlx::query(sql).execute(pool).await?;
    info!("Game participants table created");
    Ok(())
}

/// Create the player_stats table
async fn create_player_stats_table(pool: &SqlitePool) -> DatabaseResult<()> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS player_stats (
            user_id TEXT PRIMARY KEY,
            games_played INTEGER NOT NULL DEFAULT 0,
            games_won INTEGER NOT NULL DEFAULT 0,
            total_winnings INTEGER NOT NULL DEFAULT 0,
            total_losses INTEGER NOT NULL DEFAULT 0,
            biggest_win INTEGER NOT NULL DEFAULT 0,
            biggest_loss INTEGER NOT NULL DEFAULT 0,
            average_session_length INTEGER NOT NULL DEFAULT 0,
            last_updated TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        
        CREATE INDEX IF NOT EXISTS idx_stats_games_played ON player_stats(games_played);
        CREATE INDEX IF NOT EXISTS idx_stats_winnings ON player_stats(total_winnings);
    "#;
    
    sqlx::query(sql).execute(pool).await?;
    info!("Player stats table created");
    Ok(())
}

/// Create the game_events table
async fn create_game_events_table(pool: &SqlitePool) -> DatabaseResult<()> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS game_events (
            id TEXT PRIMARY KEY,
            game_id TEXT NOT NULL,
            user_id TEXT,
            event_type TEXT NOT NULL,
            event_data TEXT,
            amount INTEGER,
            created_at TEXT NOT NULL,
            FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id)
        );
        
        CREATE INDEX IF NOT EXISTS idx_events_game ON game_events(game_id);
        CREATE INDEX IF NOT EXISTS idx_events_user ON game_events(user_id);
        CREATE INDEX IF NOT EXISTS idx_events_type ON game_events(event_type);
        CREATE INDEX IF NOT EXISTS idx_events_created ON game_events(created_at);
    "#;
    
    sqlx::query(sql).execute(pool).await?;
    info!("Game events table created");
    Ok(())
}

/// Check if database tables exist and are properly set up
pub async fn verify_schema(pool: &SqlitePool) -> DatabaseResult<bool> {
    let tables = vec!["users", "games", "game_participants", "player_stats", "game_events"];
    
    for table in tables {
        let row = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name=?")
            .bind(table)
            .fetch_optional(pool)
            .await?;
            
        if row.is_none() {
            return Ok(false);
        }
    }
    
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Database, DatabaseConfig};

    #[tokio::test]
    async fn test_schema_creation() {
        let config = DatabaseConfig {
            database_path: ":memory:".to_string(),
            create_if_missing: true,
            max_connections: 5,
        };

        let db = Database::new(config).await.unwrap();
        
        // Verify schema
        let schema_ok = verify_schema(db.pool()).await.unwrap();
        assert!(schema_ok, "Database schema should be properly created");
        
        db.close().await;
    }
} 