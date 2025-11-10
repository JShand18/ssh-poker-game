use anyhow::Result;
use async_trait::async_trait;
use log::{info, debug, warn, error};
use russh::{
    server::{Config, Handler, Server, Session, Auth, Msg},
    Channel, ChannelId, CryptoVec, MethodSet
};
use russh_keys::key;
use russh_keys::key::KeyPair;
use tokio::net::TcpListener;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use data_store::Database;
use uuid::Uuid;
use hybrid_metrics::{PokerMetrics, MonitoringConfig};

pub mod config;
pub mod error;
pub mod auth;
pub mod secure_auth;
pub mod session;
pub mod ssh_handler;


pub use config::{ServerConfig, AuthConfig};
pub use error::{Result as SshResult, SshError};
pub use auth::{AuthHandler, AuthResult};
pub use secure_auth::SecureAuthService;
pub use session::SessionManager;


struct Client {
    session_id: Option<Uuid>,
    username: Option<String>,
    authenticated: bool,
}

impl Client {
    fn new() -> Self {
        Self { 
            session_id: None,
            username: None,
            authenticated: false,
        }
    }

    fn authenticate(&mut self, username: String, session_id: Uuid) {
        self.username = Some(username);
        self.session_id = Some(session_id);
        self.authenticated = true;
    }
}

struct ServerState {
    _database: Database,
    auth_service: Arc<Mutex<SecureAuthService>>,
    session_manager: Arc<SessionManager>,
    clients: Arc<Mutex<HashMap<usize, Client>>>,
    id_counter: Arc<Mutex<usize>>,
}

impl ServerState {
    async fn new(database: Database) -> Self {
        let auth_service = Arc::new(Mutex::new(SecureAuthService::new(database.clone())));
        let session_manager = Arc::new(SessionManager::new());
        
        // Start cleanup task
        SessionManager::start_cleanup_task(session_manager.clone());
        
        Self {
            _database: database,
            auth_service,
            session_manager,
            clients: Arc::new(Mutex::new(HashMap::new())),
            id_counter: Arc::new(Mutex::new(0)),
        }
    }

    async fn _session_count(&self) -> usize {
        self.session_manager.session_count().await
    }

    async fn _table_count(&self) -> usize {
        self.session_manager.table_count().await
    }
}

struct SshServerHandler {
    state: Arc<ServerState>,
    client_id: usize,
    authenticated_user: Option<String>,
    session_id: Option<Uuid>,
    terminal_size: (u16, u16),
    channel_id: Option<ChannelId>,
}

impl SshServerHandler {
    fn new(state: Arc<ServerState>, client_id: usize) -> Self {
        Self { 
            state, 
            client_id,
            authenticated_user: None,
            session_id: None,
            terminal_size: (80, 24), // Default terminal size
            channel_id: None,
        }
    }

    async fn handle_terminal_input(&mut self, data: &[u8], session: &mut Session) -> Result<(), SshError> {
        if let Some(session_id) = self.session_id {
            // Update session activity
            self.state.session_manager.update_session_activity(&session_id).await;
            
            // Parse input and handle game actions
            if let Ok(input_str) = String::from_utf8(data.to_vec()) {
                for ch in input_str.chars() {
                    match ch {
                        'n' | 'N' => {
                            // Create a new table and join it
                            let table_id = self.state.session_manager
                                .create_table("New Table".to_string(), 6, 10, 20).await;
                            if let Some(session_id) = self.session_id {
                                if let Err(e) = self.state.session_manager.join_table(&session_id, &table_id, 1000).await {
                                    warn!("Failed to join newly created table {}: {}", table_id, e);
                                }
                            }
                        }
                        '1'..='9' => {
                            let idx = (ch as u8 - b'1') as usize;
                            let tables = self.state.session_manager.list_tables().await;
                            if let Some((table_id, _name, players, max_players)) = tables.get(idx).cloned() {
                                if players < max_players {
                                    if let Some(session_id) = self.session_id {
                                        if let Err(e) = self.state.session_manager.join_table(&session_id, &table_id, 1000).await {
                                            warn!("Failed to join table {}: {}", table_id, e);
                                        }
                                    }
                                }
                            }
                        }
                        'f' | 'F' => {
                            // Fold action
                            let action = poker_engine::Action::Fold;
                            if let Err(_e) = self.state.session_manager.process_game_action(&session_id, action).await {
                                error!("Failed to process fold action: {}", _e);
                            }
                        }
                        'c' | 'C' => {
                            // Call/Check action
                            let action = poker_engine::Action::Call;
                            if let Err(_e) = self.state.session_manager.process_game_action(&session_id, action).await {
                                // Try check if call fails
                                let action = poker_engine::Action::Check;
                                if let Err(_e) = self.state.session_manager.process_game_action(&session_id, action).await {
                                    error!("Failed to process call/check action: {}", _e);
                                }
                            }
                        }
                        'r' | 'R' => {
                            // Raise action (simplified - would need amount input in real implementation)
                            let action = poker_engine::Action::Raise(50); // Default raise amount
                            if let Err(_e) = self.state.session_manager.process_game_action(&session_id, action).await {
                                error!("Failed to process raise action: {}", _e);
                            }
                        }
                        'a' | 'A' => {
                            // All-in action
                            let action = poker_engine::Action::AllIn;
                            if let Err(_e) = self.state.session_manager.process_game_action(&session_id, action).await {
                                error!("Failed to process all-in action: {}", _e);
                            }
                        }
                        'q' | 'Q' => {
                            // Quit/disconnect
                            info!("User {} requested disconnect", self.authenticated_user.as_ref().unwrap_or(&"unknown".to_string()));
                            session.disconnect(russh::Disconnect::ByApplication, "User requested disconnect", "");
                            return Ok(());
                        }
                        _ => {
                            // Handle other keys or ignore
                            debug!("Unhandled input character: {}", ch);
                        }
                    }
                }
            }
            
            // Update and render the terminal UI
            self.update_terminal_display(session).await?;
        }
        
        Ok(())
    }

    async fn update_terminal_display(&self, session: &mut Session) -> Result<(), SshError> {
        if let (Some(session_id), Some(channel_id)) = (self.session_id, self.channel_id) {
            // Get current table state
            if let Some(table_id) = self.state.session_manager.get_player_table(&session_id).await {
                if let Some(game_state) = self.state.session_manager.get_table_state(&table_id).await {
                    // Render the game state to terminal
                    let display = self.render_game_state(&game_state);
                    
                    // Send to terminal  
                    let _ = session.data(channel_id, CryptoVec::from_slice(display.as_bytes()));
                }
            } else {
                // Show lobby/table selection
                let lobby_display = self.render_lobby().await;
                let _ = session.data(channel_id, CryptoVec::from_slice(lobby_display.as_bytes()));
            }
        }
        
        Ok(())
    }

    fn render_game_state(&self, game_state: &poker_engine::GameState) -> String {
        // This will be implemented with the rich ASCII UI in the next step
        format!("Game State: {} players, Phase: {:?}\n", 
                game_state.active_player_count(), 
                game_state.current_phase)
    }

    async fn render_lobby(&self) -> String {
        let tables = self.state.session_manager.list_tables().await;
        let mut display = String::new();
        display.push_str("\n=== POKER LOBBY ===\n\n");
        
        if tables.is_empty() {
            display.push_str("No active tables. Press 'n' to create a new table.\n");
        } else {
            display.push_str("Available Tables:\n");
            for (i, (_, name, players, max_players)) in tables.iter().enumerate() {
                display.push_str(&format!("{}. {} ({}/{} players)\n", i + 1, name, players, max_players));
            }
            display.push_str("\nPress number to join table, 'n' for new table\n");
        }
        
        display.push_str("\nCommands: (q)uit\n");
        display
    }
}

#[async_trait]
impl Handler for SshServerHandler {
    type Error = SshError;

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        debug!("Opening session channel for client {}", self.client_id);
        
        // Store the channel ID for later use
        self.channel_id = Some(channel.id());
        
        // Accept the channel
        Ok(true)
    }

    async fn auth_password(
        &mut self,
        user: &str,
        password: &str,
    ) -> Result<Auth, Self::Error> {
        info!("Password authentication attempt for user: {}", user);
        
        let mut auth_service = self.state.auth_service.lock().await;
        match auth_service.authenticate_password(user, password).await {
            Ok(true) => {
                info!("Password authentication successful for user: {}", user);
                
                // Get user from database to create session
                if let Ok(Some(db_user)) = auth_service.get_user(user).await {
                    let session_id = self.state.session_manager.create_session(db_user).await;
                    self.authenticated_user = Some(user.to_string());
                    self.session_id = Some(session_id);
                    
                    // Update client state
                    let mut clients = self.state.clients.lock().await;
                    if let Some(client) = clients.get_mut(&self.client_id) {
                        client.authenticate(user.to_string(), session_id);
                    }
                    
                    Ok(Auth::Accept)
                } else {
                    warn!("User {} authenticated but not found in database", user);
                    Ok(Auth::Reject { proceed_with_methods: None })
                }
            }
            Ok(false) => {
                warn!("Password authentication failed for user: {}", user);
                Ok(Auth::Reject { proceed_with_methods: None })
            }
            Err(e) => {
                error!("Authentication error for user {}: {}", user, e);
                Ok(Auth::Reject { proceed_with_methods: None })
            }
        }
    }

    async fn auth_publickey(
        &mut self,
        user: &str,
        public_key: &key::PublicKey,
    ) -> Result<Auth, Self::Error> {
        info!("Public key authentication attempt for user: {}", user);
        
        let mut auth_service = self.state.auth_service.lock().await;
        match auth_service.authenticate_publickey(user, public_key).await {
            Ok(true) => {
                info!("Public key authentication successful for user: {}", user);
                
                if let Ok(Some(db_user)) = auth_service.get_user(user).await {
                    let session_id = self.state.session_manager.create_session(db_user).await;
                    self.authenticated_user = Some(user.to_string());
                    self.session_id = Some(session_id);
                    
                    // Update client state
                    let mut clients = self.state.clients.lock().await;
                    if let Some(client) = clients.get_mut(&self.client_id) {
                        client.authenticate(user.to_string(), session_id);
                    }
                    
                    Ok(Auth::Accept)
                } else {
                    warn!("User {} authenticated but not found in database", user);
                    Ok(Auth::Reject { proceed_with_methods: None })
                }
            }
            Ok(false) => {
                warn!("Public key authentication failed for user: {}", user);
                Ok(Auth::Reject { proceed_with_methods: None })
            }
            Err(e) => {
                error!("Public key authentication error for user {}: {}", user, e);
                Ok(Auth::Reject { proceed_with_methods: None })
            }
        }
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        debug!("Received data on channel {}: {} bytes", channel, data.len());
        
        if self.authenticated_user.is_some() {
            self.handle_terminal_input(data, session).await?;
        } else {
            warn!("Received data from unauthenticated client {}", self.client_id);
        }
        
        Ok(())
    }


    async fn pty_request(
        &mut self,
        _channel: ChannelId,
        _term: &str,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(russh::Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        debug!("PTY request - size: {}x{}", col_width, row_height);
        self.terminal_size = (col_width as u16, row_height as u16);
        if self.authenticated_user.is_some() {
            self.update_terminal_display(session).await?;
        }
        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        debug!("Shell requested on channel {}", channel);
        self.channel_id = Some(channel);
        self.update_terminal_display(session).await?;
        Ok(())
    }

    async fn window_change_request(
        &mut self,
        _channel: ChannelId,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        debug!("Window size changed to {}x{}", col_width, row_height);
        self.terminal_size = (col_width as u16, row_height as u16);
        if self.authenticated_user.is_some() {
            self.update_terminal_display(session).await?;
        }
        Ok(())
    }

    async fn channel_eof(
        &mut self,
        _channel: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

}

struct SshServer {
    state: Arc<ServerState>,
}

impl SshServer {
    async fn new(database: Database) -> Self {
        let state = Arc::new(ServerState::new(database).await);
        Self { state }
    }
}

impl Server for SshServer {
    type Handler = SshServerHandler;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> SshServerHandler {
        let client_id = {
            let mut counter = futures::executor::block_on(self.state.id_counter.lock());
            let id = *counter;
            *counter += 1;
            id
        };
        
        // Add client to state
        let client = Client::new();
        futures::executor::block_on(async {
            let mut clients = self.state.clients.lock().await;
            clients.insert(client_id, client);
        });
        
        SshServerHandler::new(self.state.clone(), client_id)
    }
}

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
    let mut ssh_config = russh::server::Config::default();
    ssh_config.inactivity_timeout = Some(std::time::Duration::from_secs(3600));
    ssh_config.auth_rejection_time = std::time::Duration::from_secs(3);
    ssh_config.methods = MethodSet::NONE;
    ssh_config.keys.push(KeyPair::generate_ed25519().unwrap());
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
        let db = database.clone();
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
                let handler = SshSessionHandler::new(db, auth_svc, sess_mgr, client_id);
            
            // Run SSH session
            if let Err(e) = russh::server::run_stream(config, stream, handler).await {
                warn!("SSH connection from {} ended: {}", peer, e);
            }
        });
    }
}

/// Legacy run_server function (using old terminal-ui)
pub async fn run_server(database: Database, bind_address: &str, port: u16) -> Result<()> {
    info!("Configuring server for address: {}:{}", bind_address, port);

    // Start Prometheus metrics server in background (free tier)
    let metrics = std::sync::Arc::new(
        PokerMetrics::new(MonitoringConfig { enable_datadog: false, ..Default::default() })?
    );
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        if let Err(e) = hybrid_metrics::start_metrics_server(metrics_clone, 9090).await {
            warn!("metrics server failed: {}", e);
        }
    });

    // Build russh server configuration
    let mut ssh_config = Config::default();
    ssh_config.inactivity_timeout = Some(std::time::Duration::from_secs(3600));
    ssh_config.auth_rejection_time = std::time::Duration::from_secs(3);
    ssh_config.auth_rejection_time_initial = Some(std::time::Duration::from_secs(0));
    ssh_config.methods = MethodSet::PASSWORD; // Enable password auth for MVP
    ssh_config.keys.push(KeyPair::generate_ed25519().unwrap());
    let ssh_config = std::sync::Arc::new(ssh_config);

    // Bind and run russh server (accept loop)
    let addr = format!("{}:{}", bind_address, port);
    info!("Starting SSH server on {}", addr);
    let listener = TcpListener::bind(&addr).await?;
    loop {
        let (stream, peer) = listener.accept().await?;
        let config = ssh_config.clone();
        let db = database.clone();
        tokio::spawn(async move {
            let mut server_factory = SshServer::new(db).await;
            let handler = server_factory.new_client(Some(peer));
            if let Err(e) = russh::server::run_stream(config, stream, handler).await {
                warn!("SSH connection from {} ended with error: {}", peer, e);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_store::Database;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_server_creation() {
        let db = Database::new_in_memory().await.expect("Failed to create test database");
        let server = SshServer::new(db).await;
        // Basic creation test passes
        assert!(!server.state.session_manager.session_count().await > 0);
    }

    #[tokio::test]
    async fn test_client_authentication() {
        let mut client = Client::new();
        assert!(client.username.is_none());
        assert!(!client.authenticated);
        
        let session_id = Uuid::new_v4();
        client.authenticate("testuser".to_string(), session_id);
        assert_eq!(client.username, Some("testuser".to_string()));
        assert!(client.authenticated);
        assert_eq!(client.session_id, Some(session_id));
    }

    #[tokio::test]
    async fn test_server_state_initialization() {
        let db = Database::new_in_memory().await.expect("Failed to create test database");
        let state = ServerState::new(db).await;
        
        assert_eq!(state._session_count().await, 0);
        assert_eq!(state._table_count().await, 0);
    }
}