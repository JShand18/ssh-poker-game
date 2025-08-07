use std::path::Path;
use sqlx::{SqlitePool, Pool, Sqlite};
use anyhow::Result;
use log::info;

pub mod models;
pub mod schema;
pub mod operations;
pub mod error;

pub use models::*;
pub use error::{DatabaseError, DatabaseResult};

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path to the SQLite database file
    pub database_path: String,
    /// Whether to create the database if it doesn't exist
    pub create_if_missing: bool,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_path: "poker_game.db".to_string(),
            create_if_missing: true,
            max_connections: 10,
        }
    }
}

/// Database connection pool and operations
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection with the given configuration
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        info!("Connecting to database: {}", config.database_path);
        
        // Build SQLite DSN and ensure parent directory exists if needed
        let database_url = if config.database_path == ":memory:" {
            "sqlite::memory:".to_string()
        } else {
            let db_path = Path::new(&config.database_path);
            if config.create_if_missing {
                if let Some(parent) = db_path.parent() {
                    if !parent.as_os_str().is_empty() && !parent.exists() {
                        std::fs::create_dir_all(parent)?;
                    }
                }
            }
            info!("Creating new database file: {}", config.database_path);
            format!("sqlite://{}", config.database_path)
        };
        
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&database_url)
            .await?;

        let db = Self { pool };
        
        // Run migrations to create tables
        db.migrate().await?;
        
        Ok(db)
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Run database migrations to create tables
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations");
        
        // Create tables
        schema::create_tables(&self.pool).await?;
        
        info!("Database migrations completed");
        Ok(())
    }

    /// Check database health
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    /// Close the database connection
    pub async fn close(self) {
        self.pool.close().await;
    }

    /// Create a new user
    pub async fn create_user(&self, new_user: NewUser) -> DatabaseResult<User> {
        operations::UserOperations::create(&self.pool, new_user).await
    }

    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> DatabaseResult<Option<User>> {
        operations::UserOperations::find_by_username(&self.pool, username).await
    }

    /// Update user password
    pub async fn update_user_password(&self, username: &str, password_hash: &str) -> DatabaseResult<()> {
        // First get the user
        let user = self.get_user_by_username(username).await?;
        if let Some(user) = user {
            sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE username = ?")
                .bind(password_hash)
                .bind(chrono::Utc::now().to_rfc3339())
                .bind(username)
                .execute(&self.pool)
                .await?;
            Ok(())
        } else {
            Err(DatabaseError::UserNotFound(username.to_string()))
        }
    }

    /// Create a session for a user
    pub async fn create_session(&self, user_id: &str, duration_hours: i64) -> DatabaseResult<models::UserSession> {
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(duration_hours);
        let new_session = models::NewSession {
            user_id: user_id.to_string(),
            expires_at,
        };
        operations::SessionOperations::create(&self.pool, new_session).await
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> DatabaseResult<Option<models::UserSession>> {
        operations::SessionOperations::find_by_id(&self.pool, session_id).await
    }

    /// Update session activity
    pub async fn update_session_activity(&self, session_id: &str) -> DatabaseResult<()> {
        operations::SessionOperations::update_activity(&self.pool, session_id).await
    }

    /// Deactivate session (logout)
    pub async fn deactivate_session(&self, session_id: &str) -> DatabaseResult<()> {
        operations::SessionOperations::deactivate(&self.pool, session_id).await
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> DatabaseResult<usize> {
        operations::SessionOperations::cleanup_expired(&self.pool).await
    }

    /// Create a test/in-memory database for testing
    pub async fn new_in_memory() -> DatabaseResult<Self> {
        let config = DatabaseConfig {
            database_path: ":memory:".to_string(),
            create_if_missing: true,
            max_connections: 5,
        };
        Self::new(config).await.map_err(|e| DatabaseError::OperationFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_creation() {
        let config = DatabaseConfig {
            database_path: ":memory:".to_string(),
            create_if_missing: true,
            max_connections: 5,
        };

        let db = Database::new(config).await.unwrap();
        
        // Test health check
        db.health_check().await.unwrap();
        
        db.close().await;
    }
} 