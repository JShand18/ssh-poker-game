use crate::themes::CasinoStyles;
use poker_engine::{GameState, Player, Card, GamePhase, Action};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Clear},
    style::{Color, Style, Modifier},
    layout::{Alignment, Rect},
    text::{Line, Span},
};
use std::collections::HashMap;

const _TABLE_WIDTH: u16 = 80;
const _TABLE_HEIGHT: u16 = 24;

pub struct PokerTableRenderer {
    seat_positions: HashMap<usize, (u16, u16)>, // seat_id -> (x, y) coordinates
    animation_frame: u8,
    _styles: CasinoStyles,
}

impl PokerTableRenderer {
    pub fn new() -> Self {
        let mut seat_positions = HashMap::new();
        
        // Define 6-max table seat positions in a circular arrangement
        // Seats are numbered 0-5, with seat 0 being the user's position (bottom center)
        seat_positions.insert(0, (40, 20)); // User seat (bottom center)
        seat_positions.insert(1, (65, 18)); // Bottom right
        seat_positions.insert(2, (70, 10)); // Right
        seat_positions.insert(3, (40, 3));  // Top center
        seat_positions.insert(4, (10, 10)); // Left
        seat_positions.insert(5, (15, 18)); // Bottom left

        Self {
            seat_positions,
            animation_frame: 0,
            _styles: CasinoStyles::new(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, game_state: &GameState, user_player_id: usize) {
        // Clear the entire area
        frame.render_widget(Clear, area);
        
        // Render the main table background
        self.render_table_background(frame, area);
        
        // Render community cards in the center
        self.render_community_cards(frame, area, &game_state.community_cards, game_state.current_phase);
        
        // Render pot information
        self.render_pot_info(frame, area, game_state);
        
        // Render each player's seat
        for (seat_id, player) in game_state.players.iter().enumerate() {
            self.render_player_seat(frame, area, seat_id, player, seat_id == user_player_id, game_state);
        }
        
        // Render current action indicator
        if let Some(_current_player) = game_state.get_current_player() {
            self.render_action_indicator(frame, area, game_state.current_player_index);
        }
        
        // Render available actions for user
        if game_state.current_player_index == user_player_id {
            self.render_user_actions(frame, area, &game_state.get_valid_actions());
        }
        
        // Render game phase indicator
        self.render_phase_indicator(frame, area, game_state.current_phase);
        
        // Update animation frame
        self.animation_frame = (self.animation_frame + 1) % 60;
    }

    fn render_table_background(&self, frame: &mut Frame, area: Rect) {
        let table_art = vec![
            "╔════════════════════════════════════════════════════════════════════════════╗",
            "║                                                                            ║",
            "║     ┌─────────┐                                       ┌─────────┐        ║",
            "║     │ SEAT 4  │                 ┌─────────┐           │ SEAT 3  │        ║",
            "║     │         │                 │ SEAT 2  │           │         │        ║",
            "║     └─────────┘                 │         │           └─────────┘        ║",
            "║                                 └─────────┘                              ║",
            "║                                                                          ║",
            "║                           ╔═══════════════╗                             ║",
            "║                           ║               ║                             ║",
            "║                           ║  COMMUNITY    ║                             ║",
            "║                           ║     CARDS     ║                             ║",
            "║                           ║               ║                             ║",
            "║                           ╚═══════════════╝                             ║",
            "║                                                                          ║",
            "║                                 POT: $                                   ║",
            "║                                                                          ║",
            "║     ┌─────────┐                                       ┌─────────┐        ║",
            "║     │ SEAT 5  │                 ┌─────────┐           │ SEAT 1  │        ║",
            "║     │         │                 │ SEAT 0  │           │         │        ║",
            "║     └─────────┘                 │  (YOU)  │           └─────────┘        ║",
            "║                                 └─────────┘                              ║",
            "║                                                                          ║",
            "╚════════════════════════════════════════════════════════════════════════════╝",
        ];

        let table_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let table_lines: Vec<Line> = table_art.iter()
            .map(|line| Line::from(Span::styled(*line, Style::default().fg(Color::Green))))
            .collect();

        let table_paragraph = Paragraph::new(table_lines)
            .block(table_block)
            .alignment(Alignment::Center);

        frame.render_widget(table_paragraph, area);
    }

    fn render_community_cards(&self, frame: &mut Frame, area: Rect, community_cards: &[Card], phase: GamePhase) {
        let card_area = Rect {
            x: area.x + 25,
            y: area.y + 9,
            width: 30,
            height: 5,
        };

        let cards_to_show = match phase {
            GamePhase::PreFlop => 0,
            GamePhase::Flop => 3,
            GamePhase::Turn => 4,
            GamePhase::River => 5,
            GamePhase::Showdown => community_cards.len(),
        };

        let mut card_display = String::new();
        
        // Show revealed cards
        for (i, card) in community_cards.iter().take(cards_to_show).enumerate() {
            if i > 0 {
                card_display.push(' ');
            }
            card_display.push_str(&self.format_card_ascii(card));
        }
        
        // Show face-down cards for unrevealed community cards
        for i in cards_to_show..5 {
            if i > 0 || !card_display.is_empty() {
                card_display.push(' ');
            }
            card_display.push_str("┌─────┐\n│ ??? │\n│  ?  │\n│ ??? │\n└─────┘");
        }

        let community_paragraph = Paragraph::new(card_display)
            .block(Block::default().borders(Borders::ALL).title("Community Cards"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));

        frame.render_widget(community_paragraph, card_area);
    }

    fn render_pot_info(&self, frame: &mut Frame, area: Rect, game_state: &GameState) {
        let pot_area = Rect {
            x: area.x + 35,
            y: area.y + 15,
            width: 20,
            height: 3,
        };

        let total_pot = game_state.pot_manager.total_pot();
        let pot_text = format!("💰 POT: ${}", total_pot);
        
        let pot_paragraph = Paragraph::new(pot_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

        frame.render_widget(pot_paragraph, pot_area);
    }

    fn render_player_seat(&self, frame: &mut Frame, area: Rect, seat_id: usize, player: &Player, is_user: bool, game_state: &GameState) {
        if let Some(&(x_offset, y_offset)) = self.seat_positions.get(&seat_id) {
            let seat_area = Rect {
                x: area.x + x_offset - 10,
                y: area.y + y_offset - 3,
                width: 20,
                height: 6,
            };

            let mut seat_content = Vec::new();
            
            // Player name and chips
            let name_line = if is_user {
                format!("👤 {} (YOU)", player.name)
            } else {
                format!("🤖 {}", player.name)
            };
            seat_content.push(Line::from(Span::styled(name_line, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
            
            let chips_line = format!("💰 ${}", player.chips);
            seat_content.push(Line::from(Span::styled(chips_line, Style::default().fg(Color::Green))));

            // Player status
            let status_style = match player.status {
                poker_engine::PlayerStatus::Active => Style::default().fg(Color::Green),
                poker_engine::PlayerStatus::Folded => Style::default().fg(Color::Red),
                poker_engine::PlayerStatus::AllIn => Style::default().fg(Color::Yellow),
                poker_engine::PlayerStatus::SittingOut => Style::default().fg(Color::Gray),
            };
            
            let status_text = format!("{:?}", player.status);
            seat_content.push(Line::from(Span::styled(status_text, status_style)));

            // Current bet
            if player.current_bet > 0 {
                let bet_line = format!("Bet: ${}", player.current_bet);
                seat_content.push(Line::from(Span::styled(bet_line, Style::default().fg(Color::Yellow))));
            }

            // Hole cards
            if let Some(hole_cards) = &player.hole_cards {
                if is_user {
                    // Show user's cards
                    let cards_line = format!("{} {}", 
                        self.format_card_small(&hole_cards[0]),
                        self.format_card_small(&hole_cards[1])
                    );
                    seat_content.push(Line::from(cards_line));
                } else {
                    // Show face-down cards for other players
                    let cards_line = "🂠 🂠".to_string();
                    seat_content.push(Line::from(Span::styled(cards_line, Style::default().fg(Color::Blue))));
                }
            }

            let border_style = if seat_id == game_state.current_player_index {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if is_user {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };

            let seat_block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(format!("Seat {}", seat_id + 1));

            let seat_paragraph = Paragraph::new(seat_content)
                .block(seat_block)
                .alignment(Alignment::Center);

            frame.render_widget(seat_paragraph, seat_area);
        }
    }

    fn render_action_indicator(&self, frame: &mut Frame, area: Rect, current_player_index: usize) {
        if let Some(&(x_offset, y_offset)) = self.seat_positions.get(&current_player_index) {
            let indicator_area = Rect {
                x: area.x + x_offset - 5,
                y: area.y + y_offset - 4,
                width: 10,
                height: 1,
            };

            let indicator_text = if self.animation_frame < 30 { ">>> ACTION <<<" } else { "             " };
            let indicator = Paragraph::new(indicator_text)
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD | Modifier::RAPID_BLINK));

            frame.render_widget(indicator, indicator_area);
        }
    }

    fn render_user_actions(&self, frame: &mut Frame, area: Rect, valid_actions: &[Action]) {
        let actions_area = Rect {
            x: area.x + 2,
            y: area.y + area.height - 6,
            width: area.width - 4,
            height: 5,
        };

        let mut action_lines = Vec::new();
        action_lines.push(Line::from(Span::styled("Available Actions:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
        action_lines.push(Line::from(""));

        let mut action_text = String::new();
        for (i, action) in valid_actions.iter().enumerate() {
            if i > 0 {
                action_text.push_str("  |  ");
            }
            
            let (key, description) = self.action_to_key_description(action);
            action_text.push_str(&format!("({}) {}", key, description));
        }

        action_lines.push(Line::from(Span::styled(action_text, Style::default().fg(Color::Green))));
        action_lines.push(Line::from(""));
        action_lines.push(Line::from(Span::styled("Press the corresponding key to make your move!", Style::default().fg(Color::White).add_modifier(Modifier::ITALIC))));

        let actions_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title("Your Turn");

        let actions_paragraph = Paragraph::new(action_lines)
            .block(actions_block)
            .alignment(Alignment::Center);

        frame.render_widget(actions_paragraph, actions_area);
    }

    fn render_phase_indicator(&self, frame: &mut Frame, area: Rect, phase: GamePhase) {
        let phase_area = Rect {
            x: area.x + area.width - 20,
            y: area.y + 2,
            width: 18,
            height: 3,
        };

        let phase_text = match phase {
            GamePhase::PreFlop => "PRE-FLOP",
            GamePhase::Flop => "FLOP",
            GamePhase::Turn => "TURN",
            GamePhase::River => "RIVER",
            GamePhase::Showdown => "SHOWDOWN",
        };

        let phase_color = match phase {
            GamePhase::PreFlop => Color::Blue,
            GamePhase::Flop => Color::Green,
            GamePhase::Turn => Color::Yellow,
            GamePhase::River => Color::Red,
            GamePhase::Showdown => Color::Magenta,
        };

        let phase_paragraph = Paragraph::new(phase_text)
            .block(Block::default().borders(Borders::ALL).title("Phase"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(phase_color).add_modifier(Modifier::BOLD));

        frame.render_widget(phase_paragraph, phase_area);
    }

    fn format_card_ascii(&self, card: &Card) -> String {
        let rank_str = match card.rank {
            poker_engine::Rank::Two => "2",
            poker_engine::Rank::Three => "3",
            poker_engine::Rank::Four => "4",
            poker_engine::Rank::Five => "5",
            poker_engine::Rank::Six => "6",
            poker_engine::Rank::Seven => "7",
            poker_engine::Rank::Eight => "8",
            poker_engine::Rank::Nine => "9",
            poker_engine::Rank::Ten => "T",
            poker_engine::Rank::Jack => "J",
            poker_engine::Rank::Queen => "Q",
            poker_engine::Rank::King => "K",
            poker_engine::Rank::Ace => "A",
        };

        let suit_str = match card.suit {
            poker_engine::Suit::Hearts => "♥",
            poker_engine::Suit::Diamonds => "♦",
            poker_engine::Suit::Clubs => "♣",
            poker_engine::Suit::Spades => "♠",
        };

        format!("┌─────┐\n│ {}{}  │\n│  {}  │\n│  {}{} │\n└─────┘", 
                rank_str, suit_str, suit_str, rank_str, suit_str)
    }

    fn format_card_small(&self, card: &Card) -> String {
        let rank_str = match card.rank {
            poker_engine::Rank::Two => "2",
            poker_engine::Rank::Three => "3",
            poker_engine::Rank::Four => "4",
            poker_engine::Rank::Five => "5",
            poker_engine::Rank::Six => "6",
            poker_engine::Rank::Seven => "7",
            poker_engine::Rank::Eight => "8",
            poker_engine::Rank::Nine => "9",
            poker_engine::Rank::Ten => "T",
            poker_engine::Rank::Jack => "J",
            poker_engine::Rank::Queen => "Q",
            poker_engine::Rank::King => "K",
            poker_engine::Rank::Ace => "A",
        };

        let suit_str = match card.suit {
            poker_engine::Suit::Hearts => "♥",
            poker_engine::Suit::Diamonds => "♦",
            poker_engine::Suit::Clubs => "♣",
            poker_engine::Suit::Spades => "♠",
        };

        format!("{}{}", rank_str, suit_str)
    }

    fn action_to_key_description(&self, action: &Action) -> (char, &'static str) {
        match action {
            Action::Fold => ('F', "Fold"),
            Action::Check => ('C', "Check"),
            Action::Call => ('C', "Call"),
            Action::Bet(_) => ('B', "Bet"),
            Action::Raise(_) => ('R', "Raise"),
            Action::AllIn => ('A', "All-In"),
        }
    }

    pub fn render_showdown_results(&self, frame: &mut Frame, area: Rect, winners: &[(usize, u64)], game_state: &GameState) {
        let results_area = Rect {
            x: area.x + area.width / 4,
            y: area.y + area.height / 3,
            width: area.width / 2,
            height: area.height / 3,
        };

        frame.render_widget(Clear, results_area);

        let mut result_lines = Vec::new();
        result_lines.push(Line::from(Span::styled("🎉 SHOWDOWN RESULTS 🎉", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
        result_lines.push(Line::from(""));

        for (player_id, winnings) in winners {
            if let Some(player) = game_state.players.get(*player_id) {
                let result_text = format!("{} wins ${}", player.name, winnings);
                result_lines.push(Line::from(Span::styled(result_text, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))));
                
                // Show winning hand if available
                if let Some(hole_cards) = &player.hole_cards {
                    let hand_text = format!("  with {} {}", 
                        self.format_card_small(&hole_cards[0]),
                        self.format_card_small(&hole_cards[1])
                    );
                    result_lines.push(Line::from(Span::styled(hand_text, Style::default().fg(Color::White))));
                }
            }
        }

        result_lines.push(Line::from(""));
        result_lines.push(Line::from(Span::styled("Press any key to continue...", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))));

        let results_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title("Hand Results");

        let results_paragraph = Paragraph::new(result_lines)
            .block(results_block)
            .alignment(Alignment::Center);

        frame.render_widget(results_paragraph, results_area);
    }

    pub fn render_lobby(&self, frame: &mut Frame, area: Rect, tables: &[(uuid::Uuid, String, usize, usize)]) {
        frame.render_widget(Clear, area);

        let lobby_art = vec![
            "╔══════════════════════════════════════════════════════════════════════════════╗",
            "║                                                                              ║",
            "║    ♠♥♦♣  WELCOME TO THE SSH POKER LOBBY  ♣♦♥♠                                ║",
            "║                                                                              ║",
            "║      ┌─────────────────────────────────────────────────────────────┐        ║",
            "║      │                    AVAILABLE TABLES                         │        ║",
            "║      └─────────────────────────────────────────────────────────────┘        ║",
            "║                                                                              ║",
            "╚══════════════════════════════════════════════════════════════════════════════╝",
        ];

        let mut lobby_lines: Vec<Line> = lobby_art.iter()
            .map(|line| Line::from(Span::styled(*line, Style::default().fg(Color::Green))))
            .collect();

        // Add table listings
        lobby_lines.push(Line::from(""));
        
        if tables.is_empty() {
            lobby_lines.push(Line::from(Span::styled("No active tables available.", Style::default().fg(Color::Yellow))));
            lobby_lines.push(Line::from(""));
            lobby_lines.push(Line::from(Span::styled("Press 'N' to create a new table!", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
        } else {
            for (i, (_table_id, name, players, max_players)) in tables.iter().enumerate() {
                let table_line = format!("{}. {} ({}/{} players)", i + 1, name, players, max_players);
                let color = if *players < *max_players { Color::Green } else { Color::Red };
                lobby_lines.push(Line::from(Span::styled(table_line, Style::default().fg(color))));
            }
            
            lobby_lines.push(Line::from(""));
            lobby_lines.push(Line::from(Span::styled("Press number to join table, 'N' to create new table", Style::default().fg(Color::Cyan))));
        }

        lobby_lines.push(Line::from(""));
        lobby_lines.push(Line::from(Span::styled("Commands: (Q)uit", Style::default().fg(Color::Gray))));

        let lobby_paragraph = Paragraph::new(lobby_lines)
            .alignment(Alignment::Center)
            .style(Style::default().bg(Color::Black));

        frame.render_widget(lobby_paragraph, area);
    }

    pub fn render_waiting_for_players(&self, frame: &mut Frame, area: Rect, current_players: usize, min_players: usize) {
        let waiting_area = Rect {
            x: area.x + area.width / 4,
            y: area.y + area.height / 2 - 3,
            width: area.width / 2,
            height: 6,
        };

        frame.render_widget(Clear, waiting_area);

        let dots = match self.animation_frame / 15 {
            0 => "   ",
            1 => ".  ",
            2 => ".. ",
            _ => "...",
        };

        let waiting_lines = vec![
            Line::from(Span::styled("⏳ WAITING FOR PLAYERS ⏳", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled(format!("Current: {} / {} minimum", current_players, min_players), Style::default().fg(Color::White))),
            Line::from(""),
            Line::from(Span::styled(format!("Please wait{}", dots), Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))),
        ];

        let waiting_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title("Table Status");

        let waiting_paragraph = Paragraph::new(waiting_lines)
            .block(waiting_block)
            .alignment(Alignment::Center);

        frame.render_widget(waiting_paragraph, waiting_area);
    }
}

impl Default for PokerTableRenderer {
    fn default() -> Self {
        Self::new()
    }
}