use log::info;
use ssh_server::{run_server, ServerConfig};
use database::{Database, DatabaseConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    info!("🎮 Starting SSH Poker Server");
    
    // Initialize database
    let db_config = DatabaseConfig::default();
    let database = Database::new(db_config).await?;
    info!("✅ Database initialized successfully");
    
    // Test database health
    database.health_check().await?;
    info!("✅ Database health check passed");
    
    // Setup SSH server configuration
    let server_config = ServerConfig::default();
    info!("🔧 Server configuration loaded");
    info!("📍 Binding to: {}:{}", server_config.address, server_config.port);
    info!("🔐 Authentication: password={}, pubkey={}, anonymous={}", 
          server_config.auth.password_auth, 
          server_config.auth.pubkey_auth, 
          server_config.auth.allow_anonymous);
    
    // Start the SSH server
    info!("🚀 Starting SSH server...");
    run_server(server_config).await?;
    
    Ok(())
} 