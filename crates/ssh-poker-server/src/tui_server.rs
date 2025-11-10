use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use log::{info, error};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use poker_engine::{GameState, Player, GamePhase};
use terminal_ui::{App, AppState};

/// Enhanced server that supports TUI mode
pub struct TuiServer {
    address: String,
    port: u16,
    game_sessions: Arc<Mutex<HashMap<SocketAddr, GameState>>>,
}

impl TuiServer {
    pub fn new(config: crate::ServerConfig) -> Self {
        Self {
            address: config.address,
            port: config.port,
            game_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Handle a client connection with TUI support
    async fn handle_tui_client(
        mut stream: TcpStream,
        addr: SocketAddr,
        game_sessions: Arc<Mutex<HashMap<SocketAddr, GameState>>>,
    ) -> Result<()> {
        info!("New TUI connection from: {}", addr);
        
        // Send TUI initialization sequence
        let init_msg = b"\x1b[2J\x1b[H"; // Clear screen and move cursor to top
        stream.write_all(init_msg).await?;
        
        // Send instructions
        let instructions = b"SSH Poker - Terminal UI Mode\r\n\r\nPress 'Enter' to start the graphical interface...\r\n";
        stream.write_all(instructions).await?;
        stream.flush().await?;
        
        // Wait for user to press enter
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        
        // Start the TUI session
        Self::run_tui_session(reader.into_inner(), addr, game_sessions).await?;
        
        Ok(())
    }
    
    /// Run a TUI session for a connected client
    async fn run_tui_session(
        mut stream: TcpStream,
        addr: SocketAddr,
        game_sessions: Arc<Mutex<HashMap<SocketAddr, GameState>>>,
    ) -> Result<()> {
        // Initialize terminal mode commands
        let enter_tui = b"\x1b[?1049h\x1b[?25l"; // Enter alternate screen, hide cursor
        stream.write_all(enter_tui).await?;
        
        // Create app state
        let mut app = App::new();
        
        // Main TUI loop
        loop {
            // Render the UI to a string buffer
            let ui_output = Self::render_tui_to_string(&app);
            stream.write_all(ui_output.as_bytes()).await?;
            stream.flush().await?;
            
            // Read input (simplified - in real implementation would handle escape sequences)
            let mut input_buffer = [0; 1024];
            match stream.try_read(&mut input_buffer) {
                Ok(n) if n > 0 => {
                    let input = String::from_utf8_lossy(&input_buffer[..n]);
                    
                    // Process input
                    for ch in input.chars() {
                        match ch {
                            'q' => {
                                // Exit TUI
                                let exit_tui = b"\x1b[?1049l\x1b[?25h"; // Leave alternate screen, show cursor
                                stream.write_all(exit_tui).await?;
                                return Ok(());
                            }
                            'n' => {
                                // Start new game
                                Self::start_tui_game(&mut app, &addr, &game_sessions);
                            }
                            'h' => app.show_help(),
                            '\r' | '\n' => app.process_input(),
                            c if c.is_ascii() => app.on_char(c),
                            _ => {}
                        }
                    }
                }
                _ => {
                    // No input available
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }
            }
        }
    }
    
    /// Render the TUI to a string for network transmission
    fn render_tui_to_string(app: &App) -> String {
        // This is a simplified version - in reality, we'd need to properly
        // serialize the TUI output for network transmission
        let mut output = String::new();
        
        // Clear screen
        output.push_str("\x1b[2J\x1b[H");
        
        // Draw title
        output.push_str("╔══════════════════════════════════════╗\r\n");
        output.push_str("║          🎰 SSH POKER 🎰             ║\r\n");
        output.push_str("╚══════════════════════════════════════╝\r\n\r\n");
        
        match app.state() {
            AppState::MainMenu => {
                output.push_str("Main Menu:\r\n");
                output.push_str("  [N] New Game\r\n");
                output.push_str("  [H] Help\r\n");
                output.push_str("  [Q] Quit\r\n");
            }
            AppState::InGame => {
                if let Some(game) = &app.game {
                    output.push_str(&format!("=== {} ===\r\n", match game.current_phase {
                        GamePhase::PreFlop => "Pre-Flop",
                        GamePhase::Flop => "Flop",
                        GamePhase::Turn => "Turn",
                        GamePhase::River => "River",
                        GamePhase::Showdown => "Showdown",
                    }));
                    
                    // Show community cards
                    if !game.community_cards.is_empty() {
                        output.push_str("Community: ");
                        for card in &game.community_cards {
                            output.push_str(&format!("{} ", card));
                        }
                        output.push_str("\r\n");
                    }
                    
                    // Show pot
                    output.push_str(&format!("Pot: ${}\r\n\r\n", game.pots[0].amount));
                    
                    // Show players
                    for (i, player) in game.players.iter().enumerate() {
                        let marker = if i == game.current_player_index { "→ " } else { "  " };
                        output.push_str(&format!(
                            "{}{} - Chips: ${} - Bet: ${}\r\n",
                            marker,
                            player.name,
                            player.chips,
                            player.current_bet
                        ));
                        
                        // Show hole cards for human player
                        if i == 0 && player.hole_cards.is_some() {
                            let cards = player.hole_cards.unwrap();
                            output.push_str(&format!("   Your cards: {} {}\r\n", cards[0], cards[1]));
                        }
                    }
                }
            }
            AppState::Help => {
                output.push_str("Help:\r\n");
                output.push_str("  Use keyboard to navigate\r\n");
                output.push_str("  Press any key to return\r\n");
            }
            AppState::GameOver => {
                output.push_str("Game Over!\r\n");
                output.push_str("Press [N] for new game or [Q] to quit\r\n");
            }
        }
        
        output
    }
    
    fn start_tui_game(
        app: &mut App,
        addr: &SocketAddr,
        game_sessions: &Arc<Mutex<HashMap<SocketAddr, GameState>>>,
    ) {
        // Create a new game
        let players = vec![
            Player::new(0, "You".to_string(), 1000),
            Player::new(1, "AI Bot".to_string(), 1000),
        ];
        
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();
        
        // Update app state
        app.game = Some(game.clone());
        app.state = AppState::InGame;
        
        // Store game in sessions
        let mut sessions = game_sessions.lock().unwrap();
        sessions.insert(*addr, game);
    }
}

/// Run the TUI-enabled server
pub async fn run_tui_server(config: crate::ServerConfig) -> Result<()> {
    let _ = env_logger::try_init();
    
    let server = TuiServer::new(config);
    let addr = format!("{}:{}", server.address, server.port);
    info!("Starting TUI-enabled server on {}", addr);
    
    let listener = TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);
    
    let game_sessions = server.game_sessions.clone();
    
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let sessions = game_sessions.clone();
                tokio::spawn(async move {
                    if let Err(e) = TuiServer::handle_tui_client(stream, addr, sessions).await {
                        error!("Error handling TUI client {}: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
            }
        }
    }
} 