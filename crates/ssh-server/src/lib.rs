use anyhow::Result;
use log::{error, info, debug, warn};
use russh::{
    server::{Config, Handler, Server, Session, Auth, Msg},
    Channel, ChannelId, CryptoVec, MethodSet
};
use russh_keys::key;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use terminal_ui::App;
use database::Database;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use futures::SinkExt;

pub mod config;
pub mod error;
pub mod tui_server;
pub mod auth;
pub mod secure_auth;
pub mod session;

pub use config::{ServerConfig, AuthConfig, UserConfig};
pub use error::{Result as SshResult, SshError};
pub use auth::{AuthHandler, AuthResult};
pub use secure_auth::SecureAuthService;
pub use session::{PlayerSession, SessionManager};

struct Client {
    app: App,
    session_id: Option<Uuid>,
    username: Option<String>,
    channel_sender: Option<mpsc::Sender<Vec<u8>>>,
    authenticated: bool,
}

impl Client {
    fn new() -> Self {
        Self { 
            app: App::new(),
            session_id: None,
            username: None,
            channel_sender: None,
            authenticated: false,
        }
    }

    fn authenticate(&mut self, username: String, session_id: Uuid) {
        self.username = Some(username);
        self.session_id = Some(session_id);
        self.authenticated = true;
    }

    fn set_channel_sender(&mut self, sender: mpsc::Sender<Vec<u8>>) {
        self.channel_sender = Some(sender);
    }
}

struct ServerState {
    database: Database,
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
        session_manager.start_cleanup_task().await;
        
        Self {
            database,
            auth_service,
            session_manager,
            clients: Arc::new(Mutex::new(HashMap::new())),
            id_counter: Arc::new(Mutex::new(0)),
        }
    }

    async fn session_count(&self) -> usize {
        self.session_manager.session_count().await
    }

    async fn table_count(&self) -> usize {
        self.session_manager.table_count().await
    }
}

struct SshServerHandler {
    state: Arc<ServerState>,
    client_id: usize,
    authenticated_user: Option<String>,
    session_id: Option<Uuid>,
    terminal_size: (u16, u16),
}

impl SshServerHandler {
    fn new(state: Arc<ServerState>, client_id: usize) -> Self {
        Self { 
            state, 
            client_id,
            authenticated_user: None,
            session_id: None,
            terminal_size: (80, 24), // Default terminal size
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
                        'f' | 'F' => {
                            // Fold action
                            let action = poker_engine::Action::Fold;
                            if let Err(e) = self.state.session_manager.process_game_action(&session_id, action).await {
                                error!("Failed to process fold action: {}", e);
                            }
                        }
                        'c' | 'C' => {
                            // Call/Check action
                            let action = poker_engine::Action::Call;
                            if let Err(e) = self.state.session_manager.process_game_action(&session_id, action).await {
                                // Try check if call fails
                                let action = poker_engine::Action::Check;
                                if let Err(e) = self.state.session_manager.process_game_action(&session_id, action).await {
                                    error!("Failed to process call/check action: {}", e);
                                }
                            }
                        }
                        'r' | 'R' => {
                            // Raise action (simplified - would need amount input in real implementation)
                            let action = poker_engine::Action::Raise(50); // Default raise amount
                            if let Err(e) = self.state.session_manager.process_game_action(&session_id, action).await {
                                error!("Failed to process raise action: {}", e);
                            }
                        }
                        'a' | 'A' => {
                            // All-in action
                            let action = poker_engine::Action::AllIn;
                            if let Err(e) = self.state.session_manager.process_game_action(&session_id, action).await {
                                error!("Failed to process all-in action: {}", e);
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
        if let Some(session_id) = self.session_id {
            // Get current table state
            if let Some(table_id) = self.state.session_manager.get_player_table(&session_id).await {
                if let Some(game_state) = self.state.session_manager.get_table_state(&table_id).await {
                    // Render the game state to terminal
                    let display = self.render_game_state(&game_state);
                    
                    // Send to terminal
                    session.data(0, CryptoVec::from(display.as_bytes()));
                }
            } else {
                // Show lobby/table selection
                let lobby_display = self.render_lobby().await;
                session.data(0, CryptoVec::from(lobby_display.as_bytes()));
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

#[russh::async_trait]
impl Handler for SshServerHandler {
    type Error = SshError;

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        session: &mut Session,
    ) -> Result<bool, Self::Error> {
        debug!("Opening session channel for client {}", self.client_id);
        
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
                    Ok(Auth::Reject)
                }
            }
            Ok(false) => {
                warn!("Password authentication failed for user: {}", user);
                Ok(Auth::Reject)
            }
            Err(e) => {
                error!("Authentication error for user {}: {}", user, e);
                Ok(Auth::Reject)
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
                    Ok(Auth::Reject)
                }
            }
            Ok(false) => {
                warn!("Public key authentication failed for user: {}", user);
                Ok(Auth::Reject)
            }
            Err(e) => {
                error!("Public key authentication error for user {}: {}", user, e);
                Ok(Auth::Reject)
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

    async fn channel_pty_request(
        &mut self,
        channel: ChannelId,
        term: &str,
        col_width: u32,
        row_height: u32,
        pix_width: u32,
        pix_height: u32,
        modes: &[(russh::ptyrequest::Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        debug!("PTY request - term: {}, size: {}x{}", term, col_width, row_height);
        
        self.terminal_size = (col_width as u16, row_height as u16);
        
        // Send initial display after PTY is established
        if self.authenticated_user.is_some() {
            self.update_terminal_display(session).await?;
        }
        
        Ok(())
    }

    async fn channel_window_change_request(
        &mut self,
        channel: ChannelId,
        col_width: u32,
        row_height: u32,
        pix_width: u32,
        pix_height: u32,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        debug!("Window size changed to {}x{}", col_width, row_height);
        
        self.terminal_size = (col_width as u16, row_height as u16);
        
        // Refresh display with new size
        if self.authenticated_user.is_some() {
            self.update_terminal_display(session).await?;
        }
        
        Ok(())
    }

    async fn channel_shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        debug!("Shell request on channel {}", channel);
        
        if self.authenticated_user.is_some() {
            // Start the interactive poker session
            self.update_terminal_display(session).await?;
        } else {
            session.data(channel, CryptoVec::from(b"Authentication required\r\n"));
        }
        
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

pub async fn run_server(database: Database, bind_address: &str, port: u16) -> Result<()> {
    info!("Setting up SSH server configuration");
    
    let server_config = Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(3600)), // 1 hour
        auth_rejection_time: std::time::Duration::from_secs(3),
        auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
        keys: vec![russh_keys::key::KeyPair::generate_ed25519().unwrap()],
        ..Default::default()
    };
    let server_config = Arc::new(server_config);
    let server = SshServer::new(database).await;

    info!("Starting SSH server on {}:{}", bind_address, port);
    info!("Authentication: password=true, pubkey=true");
    
    let bind_addr = format!("{}:{}", bind_address, port);

    russh::server::run(server_config, &bind_addr, server)
        .await
        .map_err(|e| anyhow::anyhow!("SSH server error: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::Database;
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
        
        assert_eq!(state.session_count().await, 0);
        assert_eq!(state.table_count().await, 0);
    }
} 