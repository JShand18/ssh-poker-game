use anyhow::Result;
use log::{info, warn};
use russh_keys::key::KeyPair;
use tokio::net::TcpListener;
use std::sync::Arc;
use tokio::sync::Mutex;
use data_store::Database;
use hybrid_metrics::{PokerMetrics, MonitoringConfig};
use russh::MethodSet;

pub mod error;
pub mod secure_auth;
pub mod session;
pub mod ssh_handler;
pub mod ssh_tui_bridge;

pub use error::{Result as SshResult, SshError};
pub use secure_auth::SecureAuthService;
pub use session::SessionManager;

/// Run SSH poker server with TUI integration
pub async fn run_poker_server(database: Database, bind_address: &str, port: u16) -> Result<()> {
    use ssh_handler::SshSessionHandler;

    info!("Starting SSH server with Casino TUI on {}:{}", bind_address, port);

    // Start metrics server
    let metrics = std::sync::Arc::new(
        PokerMetrics::new(MonitoringConfig { enable_datadog: false, ..Default::default() })?
    );
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        if let Err(e) = hybrid_metrics::start_metrics_server(metrics_clone, 9090).await {
            warn!("Metrics server failed: {}", e);
        }
    });

    // Build SSH configuration
    let ssh_config = russh::server::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
        auth_rejection_time: std::time::Duration::from_secs(3),
        methods: MethodSet::NONE,
        keys: vec![KeyPair::generate_ed25519().unwrap()],
        ..Default::default()
    };
    let ssh_config = std::sync::Arc::new(ssh_config);

    // Create shared services
    let auth_service = Arc::new(Mutex::new(SecureAuthService::new(database.clone())));
    let session_manager = Arc::new(SessionManager::new());
    SessionManager::start_cleanup_task(session_manager.clone());

    // Counter for client IDs
    let client_counter = Arc::new(Mutex::new(0usize));

    // Start SSH server
    let addr = format!("{}:{}", bind_address, port);
    let listener = TcpListener::bind(&addr).await?;
    info!("🎰 Casino SSH Poker server listening on {}", addr);

    loop {
        let (stream, peer) = listener.accept().await?;
        let config = ssh_config.clone();
        let auth_svc = auth_service.clone();
        let sess_mgr = session_manager.clone();
        let counter = client_counter.clone();

        tokio::spawn(async move {
            // Get client ID
            let client_id = {
                let mut c = counter.lock().await;
                let id = *c;
                *c += 1;
                id
            };

            info!("New SSH connection from {} (client {})", peer, client_id);

            // Create handler with TUI
            let handler = SshSessionHandler::new(auth_svc, sess_mgr, client_id);

            // Run SSH session
            if let Err(e) = russh::server::run_stream(config, stream, handler).await {
                warn!("SSH connection from {} ended: {}", peer, e);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_store::Database;

    #[tokio::test]
    async fn test_database_creation() {
        let db = Database::new_in_memory().await.expect("Failed to create test database");
        // Basic creation test
        assert!(db.pool().acquire().await.is_ok());
    }

    #[tokio::test]
    async fn test_session_manager_initialization() {
        let session_manager = SessionManager::new();
        assert_eq!(session_manager.session_count().await, 0);
        assert_eq!(session_manager.table_count().await, 0);
    }
}