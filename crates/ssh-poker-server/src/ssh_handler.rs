//! Direct SSH Session Handler with simplified rendering
//! 
//! This module provides a simplified SSH handler that directly renders
//! the poker TUI without complex event channels or bridges.

use async_trait::async_trait;
use data_store::Database;
use log::{debug, info};
use russh::{
    server::{Auth, Handler, Msg, Session},
    Channel, ChannelId, CryptoVec, Disconnect,
};
use russh_keys::key;
use std::sync::Arc;
use tokio::sync::Mutex;


use crate::{
    error::SshError,
    secure_auth::SecureAuthService,
    session::SessionManager,
};

/// Game state for each SSH session
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Welcome,
    Login { username: String },
    Register { username: String, email: String },
    Lobby { username: String },
    InGame { username: String, table_id: String },
}

/// Simplified SSH Session Handler with direct rendering
pub struct SshSessionHandler {
    /// Client ID
    client_id: usize,
    /// Current game state
    game_state: GameState,
    /// Terminal size
    terminal_size: (u16, u16),
    /// Channel ID for sending data
    channel_id: Option<ChannelId>,
    /// Input buffer for forms
    input_buffer: String,
    /// Database connection
    database: Database,
    /// Authentication service
    auth_service: Arc<Mutex<SecureAuthService>>,
    /// Session manager
    session_manager: Arc<SessionManager>,
}

impl SshSessionHandler {
    pub fn new(
        database: Database,
        auth_service: Arc<Mutex<SecureAuthService>>,
        session_manager: Arc<SessionManager>,
        client_id: usize,
    ) -> Self {
        Self {
            client_id,
            game_state: GameState::Welcome,
            terminal_size: (80, 24),
            channel_id: None,
            input_buffer: String::new(),
            database,
            auth_service,
            session_manager,
        }
    }

    /// Render the current game state directly to ANSI string
    fn render(&self) -> String {
        let mut output = String::new();
        
        // Clear screen and reset cursor
        output.push_str("\x1b[2J\x1b[H");
        
        match &self.game_state {
            GameState::Welcome => {
                output.push_str(&self.render_welcome());
            }
            GameState::Login { username } => {
                output.push_str(&self.render_login(username));
            }
            GameState::Register { username, email } => {
                output.push_str(&self.render_register(username, email));
            }
            GameState::Lobby { username } => {
                output.push_str(&self.render_lobby(username));
            }
            GameState::InGame { username, table_id } => {
                output.push_str(&self.render_game(username, table_id));
            }
        }
        
        output
    }

    /// Render welcome screen
    fn render_welcome(&self) -> String {
        format!(
            r#"
╔══════════════════════════════════════════════════════════╗
║                 🎰 SSH POKER CASINO 🎰                   ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║              Welcome to SSH Poker!                       ║
║                                                          ║
║              Choose an option:                           ║
║                                                          ║
║              [G] Play as Guest                           ║
║              [L] Login                                   ║
║              [R] Register                                ║
║              [Q] Quit                                    ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝

> "#
        )
    }

    /// Render login screen
    fn render_login(&self, username: &str) -> String {
        format!(
            r#"
╔══════════════════════════════════════════════════════════╗
║                    🔐 LOGIN 🔐                           ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Username: {}                                            ║
║                                                          ║
║  Password: ********                                      ║
║                                                          ║
║  [Enter] Submit   [Esc] Back                             ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝

> "#,
            format!("{:<20}", username)
        )
    }

    /// Render registration screen
    fn render_register(&self, username: &str, email: &str) -> String {
        format!(
            r#"
╔══════════════════════════════════════════════════════════╗
║                  📝 REGISTER 📝                          ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║  Username: {}                                            ║
║                                                          ║
║  Email: {}                                               ║
║                                                          ║
║  Password: ********                                      ║
║                                                          ║
║  [Enter] Submit   [Esc] Back                             ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝

> "#,
            format!("{:<20}", username),
            format!("{:<20}", email)
        )
    }

    /// Render lobby screen
    fn render_lobby(&self, username: &str) -> String {
        format!(
            r#"
╔══════════════════════════════════════════════════════════╗
║                    🎲 LOBBY 🎲                           ║
╠══════════════════════════════════════════════════════════╣
║  Welcome, {}!                                            ║
║                                                          ║
║  Available Tables:                                       ║
║  ┌─────────────────────────────────────────────┐        ║
║  │ Table 1: 3/6 players - $50 buy-in          │        ║
║  │ Table 2: 5/6 players - $100 buy-in         │        ║
║  │ Table 3: 2/6 players - $200 buy-in         │        ║
║  └─────────────────────────────────────────────┘        ║
║                                                          ║
║  [1-3] Join Table   [N] New Table   [Q] Quit            ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝

> "#,
            format!("{:<15}", username)
        )
    }

    /// Render game table (simplified for now)
    fn render_game(&self, username: &str, table_id: &str) -> String {
        format!(
            r#"
╔══════════════════════════════════════════════════════════╗
║                 Table {} - Texas Hold'em                 ║
╠══════════════════════════════════════════════════════════╣
║                                                          ║
║                    Player 2                              ║
║                    🂠 🂠 $500                             ║
║                                                          ║
║        Player 1              Player 3                    ║
║        🂠 🂠 $750            🂠 🂠 $300                    ║
║                                                          ║
║                   POT: $150                              ║
║                 [ 🃁 ] [ 🃇 ] [ 🃍 ]                       ║
║                                                          ║
║                    {} (You)                              ║
║                  [ 🂱 ] [ 🂷 ]                            ║
║                    $1000                                 ║
║                                                          ║
║  [F]old  [C]all  [R]aise  [A]ll-in                      ║
╚══════════════════════════════════════════════════════════╝

> "#,
            table_id,
            username
        )
    }

    /// Handle character input and update game state
    async fn handle_input(&mut self, ch: char, session: &mut Session) -> Result<(), SshError> {
        match &mut self.game_state {
            GameState::Welcome => {
                match ch.to_ascii_lowercase() {
                    'g' => {
                        self.game_state = GameState::Lobby { 
                            username: format!("Guest_{}", self.client_id) 
                        };
                    }
                    'l' => {
                        self.game_state = GameState::Login { 
                            username: String::new() 
                        };
                        self.input_buffer.clear();
                    }
                    'r' => {
                        self.game_state = GameState::Register { 
                            username: String::new(), 
                            email: String::new() 
                        };
                        self.input_buffer.clear();
                    }
                    'q' => {
                        if let Some(channel) = self.channel_id {
                            let _ = session.data(
                                channel, 
                                CryptoVec::from_slice(b"\r\nThanks for playing! Goodbye!\r\n")
                            );
                        }
                        session.disconnect(Disconnect::ByApplication, "User quit", "");
                        return Ok(());
                    }
                    _ => {}
                }
            }
            GameState::Login { username } => {
                match ch {
                    '\x1b' => { // ESC
                        self.game_state = GameState::Welcome;
                    }
                    '\r' | '\n' => { // Enter
                        // TODO: Implement actual authentication
                        if !username.is_empty() {
                            self.game_state = GameState::Lobby { 
                                username: username.clone() 
                            };
                        }
                    }
                    '\x7f' | '\x08' => { // Backspace
                        username.pop();
                    }
                    c if c.is_alphanumeric() || c == '_' => {
                        if username.len() < 20 {
                            username.push(c);
                        }
                    }
                    _ => {}
                }
            }
            GameState::Lobby { username } => {
                match ch {
                    '1'..='3' => {
                        let table_id = format!("Table {}", ch);
                        self.game_state = GameState::InGame { 
                            username: username.clone(), 
                            table_id 
                        };
                    }
                    'q' | 'Q' => {
                        self.game_state = GameState::Welcome;
                    }
                    _ => {}
                }
            }
            GameState::InGame { .. } => {
                match ch.to_ascii_lowercase() {
                    'f' => info!("Player folded"),
                    'c' => info!("Player called"),
                    'r' => info!("Player raised"),
                    'a' => info!("Player went all-in"),
                    'q' => {
                        self.game_state = GameState::Welcome;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        
        // Send updated render to client
        if let Some(channel) = self.channel_id {
            let output = self.render();
            let _ = session.data(channel, CryptoVec::from_slice(output.as_bytes()));
        }
        
        Ok(())
    }




}

#[async_trait]
impl Handler for SshSessionHandler {
    type Error = SshError;

    async fn channel_open_session(
        &mut self,
        _channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        debug!("Opening session channel for client {}", self.client_id);
        
        // Create anonymous session
        info!("Creating session for client {}", self.client_id);
        
        Ok(true)
    }

    async fn auth_password(
        &mut self,
        user: &str,
        _password: &str,
    ) -> Result<Auth, Self::Error> {
        info!("Auto-accepting SSH connection for user: {} (auth will be handled in TUI)", user);
        
        debug!("Password auth attempt for user: {}", user);
        
        Ok(Auth::Accept)
    }

    async fn auth_publickey(
        &mut self,
        user: &str,
        _public_key: &key::PublicKey,
    ) -> Result<Auth, Self::Error> {
        info!("Auto-accepting SSH connection for user: {} (auth will be handled in TUI)", user);
        
        debug!("Password auth attempt for user: {}", user);
        
        Ok(Auth::Accept)
    }

    async fn auth_none(
        &mut self,
        user: &str,
    ) -> Result<Auth, Self::Error> {
        info!("Anonymous SSH connection for user: {} - proceeding to TUI", user);
        
        debug!("Password auth attempt for user: {}", user);
        
        Ok(Auth::Accept)
    }

    async fn data(
        &mut self,
        _channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if let Ok(input_str) = String::from_utf8(data.to_vec()) {
            for ch in input_str.chars() {
                self.handle_input(ch, session).await?;
            }
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
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        debug!("PTY request - size: {}x{}", col_width, row_height);
        self.terminal_size = (col_width as u16, row_height as u16);
        

        
        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        info!("Shell requested on channel {} - starting poker TUI", channel);
        self.channel_id = Some(channel);
        
        // Send initial render
        let output = self.render();
        session.data(channel, CryptoVec::from_slice(output.as_bytes()));
        
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
        
        // Re-render with new size
        if let Some(channel) = self.channel_id {
            let output = self.render();
            session.data(channel, CryptoVec::from_slice(output.as_bytes()));
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