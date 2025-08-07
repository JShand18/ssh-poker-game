//! Game views following Bubble Tea patterns
//! 
//! Provides different screens/views for the poker application

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style as RatatuiStyle,
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::{
    app::AppState,
    components::{ActionButtons, Card, GameTable, PlayerInfo},
    events::{GameEvent, InputEvent, NetworkEvent},
    themes::{CharmStyles, CardSuit},
};

/// Trait for application views (inspired by Bubble Tea)
pub trait View {
    /// Render the view
    fn render(&mut self, area: Rect, frame: &mut Frame, styles: &CharmStyles);
    
    /// Handle user input, returning optional state change
    fn handle_input(&mut self, input: &InputEvent, current_state: &AppState) -> Option<AppState>;
    
    /// Handle game events
    fn handle_game_event(&mut self, event: &GameEvent, current_state: &AppState);
    
    /// Handle network events
    fn handle_network_event(&mut self, event: &NetworkEvent, current_state: &AppState);
    
    /// Update view state (called on each tick)
    fn update(&mut self, current_state: &AppState);
}

/// Lobby/waiting room view
pub struct LobbyView {
    connected_players: Vec<String>,
    status_message: String,
}

impl LobbyView {
    pub fn new() -> Self {
        Self {
            connected_players: vec![
                "Alice".to_string(),
                "Bob".to_string(),
                "Charlie".to_string(),
            ],
            status_message: "Waiting for more players to join...".to_string(),
        }
    }
}

impl View for LobbyView {
    fn render(&mut self, area: Rect, frame: &mut Frame, styles: &CharmStyles) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(5),     // Player list  
                Constraint::Length(3),  // Status
                Constraint::Length(3),  // Instructions
            ])
            .split(area);
        
        // Title with casino styling
        let title_text = crate::themes::gradient_text(
            "🎰 Casino Poker Lobby",
            styles.palette.accent,      // Gold
            styles.palette.secondary,   // Forest green
        );
        
        let title = Paragraph::new(title_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(styles.border())
            );
        frame.render_widget(title, chunks[0]);
        
        // Player list
        let player_text: Vec<Line> = self.connected_players
            .iter()
            .enumerate()
            .map(|(i, player)| {
                Line::from(vec![
                    Span::styled(format!("{}. ", i + 1), styles.subtitle()),
                    Span::styled(format!("👤 {}", player), styles.player_name(false)),
                ])
            })
            .collect();
        
        let players = Paragraph::new(player_text)
            .block(
                Block::default()
                    .title("Connected Players")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(styles.border())
                    .title_style(styles.title())
            );
        frame.render_widget(players, chunks[1]);
        
        // Status message
        let status = Paragraph::new(self.status_message.clone())
            .style(styles.subtitle())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(styles.border())
            );
        frame.render_widget(status, chunks[2]);
        
        // Instructions
        let instructions = Text::from(vec![
            Line::from("Press F1 to start demo game"),
        ]);
        
        let help = Paragraph::new(instructions)
            .style(styles.subtitle())
            .alignment(Alignment::Center);
        frame.render_widget(help, chunks[3]);
    }
    
    fn handle_input(&mut self, input: &InputEvent, _current_state: &AppState) -> Option<AppState> {
        match input {
            InputEvent::Key(key) => {
                match key.code {
                    crossterm::event::KeyCode::F(1) => Some(AppState::InGame),
                    crossterm::event::KeyCode::Enter => Some(AppState::InGame),
                    _ => None,
                }
            }
            _ => None,
        }
    }
    
    fn handle_game_event(&mut self, event: &GameEvent, _current_state: &AppState) {
        match event {
            GameEvent::PlayerJoined { name, .. } => {
                if !self.connected_players.contains(name) {
                    self.connected_players.push(name.clone());
                }
                self.status_message = format!("{} joined the lobby", name);
            }
            GameEvent::PlayerLeft { player_id } => {
                // Remove player (simplified - would need proper ID mapping)
                self.status_message = format!("Player {} left the lobby", player_id);
            }
            _ => {}
        }
    }
    
    fn handle_network_event(&mut self, event: &NetworkEvent, _current_state: &AppState) {
        match event {
            NetworkEvent::Connected => {
                self.status_message = "Connected to server!".to_string();
            }
            NetworkEvent::Disconnected => {
                self.status_message = "Disconnected from server".to_string();
            }
            NetworkEvent::Error(msg) => {
                self.status_message = format!("Network error: {}", msg);
            }
        }
    }
    
    fn update(&mut self, _current_state: &AppState) {
        // Update lobby state periodically
        if self.connected_players.len() >= 2 && self.status_message.contains("Waiting") {
            self.status_message = "Ready to start game! Press Enter or F1".to_string();
        }
    }
}

/// In-game poker view
pub struct GameView {
    game_table: GameTable,
    players: Vec<PlayerInfo>,
    current_player: usize,
    action_buttons: ActionButtons,
    player_hand: Vec<Card>,
}

impl GameView {
    pub fn new() -> Self {
        // Demo game state
        let mut game_table = GameTable::new();
        game_table.pot = 150;
        game_table.community_cards = vec![
            Card::new("A".to_string(), CardSuit::Hearts),
            Card::new("K".to_string(), CardSuit::Spades),
            Card::new("Q".to_string(), CardSuit::Diamonds),
        ];
        game_table.current_round = "Flop".to_string();
        
        let players = vec![
            PlayerInfo {
                name: "You".to_string(),
                chips: 1000,
                bet: 50,
                is_active: true,
                is_dealer: false,
                cards: vec![],
            },
            PlayerInfo {
                name: "Alice".to_string(), 
                chips: 850,
                bet: 50,
                is_active: false,
                is_dealer: true,
                cards: vec![],
            },
            PlayerInfo {
                name: "Bob".to_string(),
                chips: 1200,
                bet: 50,
                is_active: false,
                is_dealer: false,
                cards: vec![],
            },
        ];
        
        let action_buttons = ActionButtons::new(vec![
            "Call $50".to_string(),
            "Raise".to_string(),
            "Fold".to_string(),
        ]);
        
        let player_hand = vec![
            Card::new("J".to_string(), CardSuit::Hearts),
            Card::new("10".to_string(), CardSuit::Hearts),
        ];
        
        Self {
            game_table,
            players,
            current_player: 0,
            action_buttons,
            player_hand,
        }
    }
}

impl View for GameView {
    fn render(&mut self, area: Rect, frame: &mut Frame, styles: &CharmStyles) {
        // Fill entire background with dark green baize
        let background_block = Block::default()
            .style(RatatuiStyle::default().bg(styles.palette.background));
        frame.render_widget(background_block, area);
        
        // Traditional poker table layout
        let table_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),     // Top players
                Constraint::Min(10),       // Central table area
                Constraint::Length(8),     // Bottom area (your seat)
                Constraint::Length(4),     // Action buttons
            ])
            .split(area);
        
        // Render top players (opponents across from you)
        self.render_top_players(table_layout[0], frame, styles);
        
        // Render central poker table with community cards and pot
        self.render_poker_table(table_layout[1], frame, styles);
        
        // Render your position at bottom of table
        self.render_your_seat(table_layout[2], frame, styles);
        
        // Render action buttons at the bottom
        self.action_buttons.render(table_layout[3], frame, styles);
    }
    
    fn handle_input(&mut self, input: &InputEvent, _current_state: &AppState) -> Option<AppState> {
        match input {
            InputEvent::Key(key) => {
                match key.code {
                    crossterm::event::KeyCode::Up => {
                        self.action_buttons.previous();
                        None
                    }
                    crossterm::event::KeyCode::Down => {
                        self.action_buttons.next();
                        None
                    }
                    crossterm::event::KeyCode::Enter => {
                        if let Some(action) = self.action_buttons.selected_action() {
                            // Handle selected action
                            log::info!("Player selected action: {}", action);
                            // In a real game, this would send the action to the game engine
                        }
                        None
                    }
                    crossterm::event::KeyCode::Esc => Some(AppState::Lobby),
                    _ => None,
                }
            }
            _ => None,
        }
    }
    
    fn handle_game_event(&mut self, event: &GameEvent, _current_state: &AppState) {
        match event {
            GameEvent::PlayerAction { player_id, action } => {
                log::info!("Player {} took action: {}", player_id, action);
                // Update game state based on action
            }
            GameEvent::StateChanged { new_state } => {
                self.game_table.current_round = new_state.clone();
            }
            GameEvent::RoundComplete { winner, pot } => {
                log::info!("Round complete! Winner: {}, Pot: ${}", winner, pot);
            }
            _ => {}
        }
    }
    
    fn handle_network_event(&mut self, _event: &NetworkEvent, _current_state: &AppState) {
        // Handle network events specific to gameplay
    }
    
    fn update(&mut self, _current_state: &AppState) {
        // Update game state, animations, etc.
    }
}

impl GameView {
    /// Render players positioned at the top of the table (opponents)
    fn render_top_players(&self, area: Rect, frame: &mut Frame, styles: &CharmStyles) {
        if self.players.len() <= 1 {
            return;
        }
        
        // Skip the first player (that's you at bottom)
        let opponents: Vec<&PlayerInfo> = self.players.iter().skip(1).collect();
        
        if opponents.is_empty() {
            return;
        }
        
        let player_width = area.width / opponents.len() as u16;
        let constraints: Vec<Constraint> = opponents
            .iter()
            .map(|_| Constraint::Length(player_width))
            .collect();
        
        let player_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);
        
        for (i, player) in opponents.iter().enumerate() {
            if let Some(player_area) = player_areas.get(i) {
                self.render_compact_player(player, *player_area, frame, styles);
            }
        }
    }
    
    /// Render the central poker table with oval felt, community cards, and pot
    fn render_poker_table(&self, area: Rect, frame: &mut Frame, styles: &CharmStyles) {
        // Create poker table felt background
        let felt_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(styles.border())
            .style(RatatuiStyle::default().bg(styles.palette.secondary)); // Forest green felt
        
        let felt_area = felt_block.inner(area);
        frame.render_widget(felt_block, area);
        
        // Split felt area into sections
        let felt_sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),     // Pot display
                Constraint::Min(6),        // Community cards
                Constraint::Length(2),     // Round info
            ])
            .split(felt_area);
        
        // Pot in center with casino styling
        let pot_text = vec![
            Line::from(vec![
                Span::styled("💰 POT: $", styles.chips()),
                Span::styled(format!("{}", self.game_table.pot), styles.chips()),
            ])
        ];
        
        let pot_display = Paragraph::new(pot_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(styles.border())
                    .style(RatatuiStyle::default().bg(styles.palette.surface))
            );
        frame.render_widget(pot_display, felt_sections[0]);
        
        // Community cards in the center
        if !self.game_table.community_cards.is_empty() {
            let card_width = 8;
            let total_cards = self.game_table.community_cards.len();
            let total_width = total_cards * card_width;
            
            if total_width <= felt_sections[1].width as usize {
                // Center the cards
                let start_x = (felt_sections[1].width as usize - total_width) / 2;
                
                for (i, card) in self.game_table.community_cards.iter().enumerate() {
                    let card_area = Rect {
                        x: felt_sections[1].x + start_x as u16 + (i * card_width) as u16,
                        y: felt_sections[1].y + 1,
                        width: card_width as u16 - 1,
                        height: felt_sections[1].height - 2,
                    };
                    
                    let card_widget = card.render(styles);
                    frame.render_widget(card_widget, card_area);
                }
            }
        } else {
            // Show "Waiting for cards..." message
            let waiting_msg = Paragraph::new("Waiting for cards to be dealt...")
                .style(styles.subtitle())
                .alignment(Alignment::Center);
            frame.render_widget(waiting_msg, felt_sections[1]);
        }
        
        // Round info at bottom
        let round_info = Paragraph::new(format!("Round: {}", self.game_table.current_round))
            .style(styles.subtitle())
            .alignment(Alignment::Center);
        frame.render_widget(round_info, felt_sections[2]);
    }
    
    /// Render your seat at the bottom of the table
    fn render_your_seat(&self, area: Rect, frame: &mut Frame, styles: &CharmStyles) {
        if let Some(you) = self.players.first() {
            // Split into your info and your cards
            let your_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30), // Your info
                    Constraint::Percentage(70), // Your cards
                ])
                .split(area);
            
            // Render your player info
            self.render_compact_player(you, your_layout[0], frame, styles);
            
            // Render your hand
            if !self.player_hand.is_empty() {
                let hand_block = Block::default()
                    .title("Your Hand")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(styles.border())
                    .title_style(styles.title());
                
                let hand_area = hand_block.inner(your_layout[1]);
                frame.render_widget(hand_block, your_layout[1]);
                
                let card_width = 8;
                let total_width = self.player_hand.len() * card_width;
                
                if total_width <= hand_area.width as usize {
                    let start_x = (hand_area.width as usize - total_width) / 2;
                    
                    for (i, card) in self.player_hand.iter().enumerate() {
                        let card_area = Rect {
                            x: hand_area.x + start_x as u16 + (i * card_width) as u16,
                            y: hand_area.y,
                            width: card_width as u16 - 1,
                            height: hand_area.height,
                        };
                        
                        let card_widget = card.render(styles);
                        frame.render_widget(card_widget, card_area);
                    }
                }
            }
        }
    }
    
    /// Render a compact player display suitable for table positions
    fn render_compact_player(&self, player: &PlayerInfo, area: Rect, frame: &mut Frame, styles: &CharmStyles) {
        let dealer_indicator = if player.is_dealer { "👑 " } else { "" };
        let name_style = styles.player_name(player.is_active);
        
        let border_style = if player.is_active {
            styles.border()
        } else {
            styles.subtitle()
        };
        
        let player_text = vec![
            Line::from(vec![
                Span::styled(format!("{}{}", dealer_indicator, player.name), name_style),
            ]),
            Line::from(vec![
                Span::styled("💰", styles.chips()),
                Span::styled(format!("${}", player.chips), styles.chips()),
            ]),
            Line::from(vec![
                Span::styled("Bet: ", styles.subtitle()),
                Span::styled(format!("${}", player.bet), styles.warning()),
            ]),
        ];
        
        let player_widget = Paragraph::new(player_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_style)
                    .style(if player.is_active {
                        RatatuiStyle::default().bg(styles.palette.surface)
                    } else {
                        RatatuiStyle::default()
                    })
            );
        
        frame.render_widget(player_widget, area);
    }
}

/// View state management
#[derive(Debug, Clone)]
pub struct ViewState {
    pub current_view: String,
    pub data: serde_json::Value,
}

impl ViewState {
    pub fn new(view_name: &str) -> Self {
        Self {
            current_view: view_name.to_string(),
            data: serde_json::Value::Null,
        }
    }
}