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
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection with the given configuration
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        info!("Connecting to database: {}", config.database_path);
        
        // Create database file if it doesn't exist and create_if_missing is true
        if config.create_if_missing && !Path::new(&config.database_path).exists() {
            info!("Creating new database file: {}", config.database_path);
        }

        let database_url = format!("sqlite:{}", config.database_path);
        
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