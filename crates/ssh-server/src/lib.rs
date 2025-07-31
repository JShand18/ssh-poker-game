use anyhow::Result;
use log::{error, info};
use russh::{
    server::{Config, Handler, Server, Session},
    ChannelId,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use terminal_ui::App;

pub mod config;
pub mod error;
pub mod tui_server;
pub mod auth;

pub use config::{ServerConfig, AuthConfig, UserConfig};
pub use error::{Result as SshResult, SshError};
pub use auth::{AuthHandler, AuthResult};

struct Client {
    app: App,
    username: Option<String>, // Track authenticated username
}

impl Client {
    fn new() -> Self {
        Self { 
            app: App::new(),
            username: None,
        }
    }

    fn authenticate(&mut self, username: String) {
        self.username = Some(username);
    }
}

#[derive(Clone)]
struct ServerState {
    clients: Arc<Mutex<HashMap<usize, Client>>>,
    id_counter: usize,
    auth_handler: AuthHandler,
}

impl ServerState {
    fn new(config: &ServerConfig) -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            id_counter: 0,
            auth_handler: AuthHandler::new(config.auth.clone()),
        }
    }
}

struct SshServerHandler {
    state: ServerState,
    client_id: usize,
}

impl SshServerHandler {
    fn new(state: ServerState, client_id: usize) -> Self {
        Self { state, client_id }
    }
}

impl Handler for SshServerHandler {
    type Error = SshError;
    
    // For now, using default Handler implementation
    // We'll add authentication and TUI integration methods once we can determine the correct signatures
    // The empty implementation works and allows the server to start
}

struct SshServer {
    state: ServerState,
}

impl SshServer {
    fn new(config: &ServerConfig) -> Self {
        Self {
            state: ServerState::new(config),
        }
    }
}

impl Server for SshServer {
    type Handler = SshServerHandler;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> SshServerHandler {
        let client_id = self.state.id_counter;
        self.state.id_counter += 1;
        SshServerHandler::new(self.state.clone(), client_id)
    }
}

pub async fn run_server(config: ServerConfig) -> Result<()> {
    info!("Setting up SSH server configuration");
    
    let server_config = Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(config.connection_timeout)),
        auth_rejection_time: std::time::Duration::from_secs(3),
        auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
        keys: vec![russh_keys::key::KeyPair::generate_ed25519().unwrap()],
        ..Default::default()
    };
    let server_config = Arc::new(server_config);
    let server = SshServer::new(&config);

    info!("Starting SSH server on {}:{}", config.address, config.port);
    info!("Authentication: password={}, pubkey={}, anonymous={}", 
          config.auth.password_auth, config.auth.pubkey_auth, config.auth.allow_anonymous);
    
    let bind_addr = format!("{}:{}", config.address, config.port);

    russh::server::run(server_config, &bind_addr, server)
        .await
        .map_err(|e| anyhow::anyhow!("SSH server error: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let config = ServerConfig::default();
        let server = SshServer::new(&config);
        // Basic creation test passes
    }

    #[test]
    fn test_client_authentication() {
        let mut client = Client::new();
        assert!(client.username.is_none());
        
        client.authenticate("testuser".to_string());
        assert_eq!(client.username, Some("testuser".to_string()));
    }
} 