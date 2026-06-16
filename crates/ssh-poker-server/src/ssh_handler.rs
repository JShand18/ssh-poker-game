//! SSH Session Handler with Casino TUI Integration
//!
//! This module provides an SSH handler that integrates with the poker-tui
//! to deliver a rich, casino-themed terminal interface over SSH.

use async_trait::async_trait;
use log::{debug, info};
use russh::{
    server::{Auth, Handler, Msg, Session},
    Channel, ChannelId, CryptoVec,
};
use russh_keys::key;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use crate::{
    error::SshError,
    secure_auth::SecureAuthService,
    session::SessionManager,
    ssh_tui_bridge::SshTuiBridge,
};

/// SSH Session Handler with Casino TUI
pub struct SshSessionHandler {
    /// Client ID
    client_id: usize,
    /// Terminal size
    terminal_size: (u16, u16),
    /// Channel ID for sending data
    channel_id: Option<ChannelId>,
    /// Authentication service
    auth_service: Arc<Mutex<SecureAuthService>>,
    /// Session manager
    session_manager: Arc<SessionManager>,
    /// TUI Bridge (created after authentication)
    tui_bridge: Option<Arc<Mutex<SshTuiBridge>>>,
    /// Output sender for TUI
    output_sender: Option<mpsc::UnboundedSender<Vec<u8>>>,
    /// Output receiver
    output_receiver: Option<mpsc::UnboundedReceiver<Vec<u8>>>,
    /// Authenticated username
    authenticated_user: Option<String>,
}

impl SshSessionHandler {
    pub fn new(
        auth_service: Arc<Mutex<SecureAuthService>>,
        session_manager: Arc<SessionManager>,
        client_id: usize,
    ) -> Self {
        // Create output channel for TUI
        let (output_sender, output_receiver) = mpsc::unbounded_channel();

        Self {
            client_id,
            terminal_size: (80, 24),
            channel_id: None,
            auth_service,
            session_manager,
            tui_bridge: None,
            output_sender: Some(output_sender),
            output_receiver: Some(output_receiver),
            authenticated_user: None,
        }
    }

    /// Initialize the TUI bridge after authentication
    async fn init_tui_bridge(&mut self) -> Result<(), SshError> {
        if self.tui_bridge.is_some() {
            return Ok(()); // Already initialized
        }

        if let Some(sender) = self.output_sender.clone() {
            match SshTuiBridge::new(sender) {
                Ok(mut bridge) => {
                    // Set the terminal size
                    bridge.set_terminal_size(self.terminal_size.0, self.terminal_size.1);

                    let bridge_arc = Arc::new(Mutex::new(bridge));
                    self.tui_bridge = Some(bridge_arc.clone());
                    info!("TUI bridge initialized for client {}", self.client_id);

                    // Start the TUI run loop in the background
                    self.start_tui_loop(bridge_arc).await;

                    Ok(())
                }
                Err(e) => {
                    info!("Failed to create TUI bridge: {}", e);
                    Err(SshError::Internal(format!("Failed to initialize TUI: {}", e)))
                }
            }
        } else {
            Err(SshError::Internal("Output sender not available".to_string()))
        }
    }

    /// Start the TUI run loop in the background
    async fn start_tui_loop(&mut self, bridge: Arc<Mutex<SshTuiBridge>>) {
        let client_id = self.client_id;

        tokio::spawn(async move {
            info!("Starting TUI loop for client {}", client_id);
            let mut bridge_lock = bridge.lock().await;
            if let Err(e) = bridge_lock.run().await {
                info!("TUI loop ended for client {}: {}", client_id, e);
            }
        });
    }

    /// Forward output from TUI to SSH channel
    async fn forward_tui_output(&mut self, session: &mut Session) {
        if let (Some(receiver), Some(channel_id)) = (&mut self.output_receiver, self.channel_id) {
            // Try to receive output without blocking
            if let Ok(output) = receiver.try_recv() {
                session.data(channel_id, CryptoVec::from_slice(&output));
            }
        }
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
        info!("Creating session for client {}", self.client_id);
        Ok(true)
    }

    async fn auth_password(
        &mut self,
        user: &str,
        password: &str,
    ) -> Result<Auth, Self::Error> {
        info!("Password authentication attempt for user: {}", user);

        // Use the actual auth service for authentication
        let mut auth_service = self.auth_service.lock().await;
        match auth_service.authenticate_password(user, password).await {
            Ok(true) => {
                info!("Password authentication successful for user: {}", user);
                // Create a session for the authenticated user
                if let Ok(Some(db_user)) = auth_service.get_user(user).await {
                    let session_id = self.session_manager.create_session(db_user).await;
                    self.authenticated_user = Some(user.to_string());
                    info!("Session created for user {} with ID: {}", user, session_id);
                }
                Ok(Auth::Accept)
            }
            Ok(false) => {
                info!("Password authentication failed for user: {}", user);
                Ok(Auth::Reject { proceed_with_methods: None })
            }
            Err(e) => {
                info!("Authentication error for user {}: {}", user, e);
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

        // Use the actual auth service for authentication
        let mut auth_service = self.auth_service.lock().await;
        match auth_service.authenticate_publickey(user, public_key).await {
            Ok(true) => {
                info!("Public key authentication successful for user: {}", user);
                // Create a session for the authenticated user
                if let Ok(Some(db_user)) = auth_service.get_user(user).await {
                    let session_id = self.session_manager.create_session(db_user).await;
                    self.authenticated_user = Some(user.to_string());
                    info!("Session created for user {} with ID: {}", user, session_id);
                }
                Ok(Auth::Accept)
            }
            Ok(false) => {
                info!("Public key authentication failed for user: {}", user);
                Ok(Auth::Reject { proceed_with_methods: None })
            }
            Err(e) => {
                info!("Public key authentication error for user {}: {}", user, e);
                Ok(Auth::Reject { proceed_with_methods: None })
            }
        }
    }

    async fn auth_none(
        &mut self,
        user: &str,
    ) -> Result<Auth, Self::Error> {
        info!("Anonymous connection attempt for user: {}", user);

        // Allow anonymous access for now (TUI will handle auth)
        self.authenticated_user = Some(format!("guest_{}", self.client_id));
        debug!("Allowing anonymous access as guest_{}", self.client_id);

        Ok(Auth::Accept)
    }

    async fn data(
        &mut self,
        _channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Initialize TUI if not already done
        if self.tui_bridge.is_none() {
            self.init_tui_bridge().await?;
        }

        // Forward input to TUI bridge
        if let Some(bridge) = &self.tui_bridge {
            let mut bridge_lock = bridge.lock().await;
            if let Err(e) = bridge_lock.handle_ssh_input(data).await {
                debug!("Error handling SSH input: {}", e);
            }
        }

        // Forward any output from TUI to SSH
        self.forward_tui_output(session).await;

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

        // Update TUI bridge terminal size if it exists
        if let Some(bridge) = &self.tui_bridge {
            let mut bridge_lock = bridge.lock().await;
            bridge_lock.set_terminal_size(col_width as u16, row_height as u16);
        }

        // Forward any output
        self.forward_tui_output(session).await;

        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        debug!("Shell requested on channel {}", channel);
        self.channel_id = Some(channel);

        // Initialize TUI bridge now that we have a shell
        self.init_tui_bridge().await?;

        // Send initial output
        self.forward_tui_output(session).await;

        // Start a background task to continuously forward TUI output
        let receiver = self.output_receiver.take();

        if let Some(mut rx) = receiver {
            tokio::spawn(async move {
                while let Some(output) = rx.recv().await {
                    // Note: We can't directly access session here
                    // The output will be handled in the data() method
                    debug!("TUI output ready: {} bytes", output.len());
                }
            });
        }

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

        // Update TUI bridge terminal size
        if let Some(bridge) = &self.tui_bridge {
            let mut bridge_lock = bridge.lock().await;
            bridge_lock.set_terminal_size(col_width as u16, row_height as u16);
        }

        // Forward any output
        self.forward_tui_output(session).await;

        Ok(())
    }

    async fn channel_eof(
        &mut self,
        _channel: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        info!("Channel EOF for client {}", self.client_id);
        Ok(())
    }
}