//! Beautiful poker UI components using ratatui
//! 
//! Provides reusable, casino-themed components for the poker game

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},

    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Clear, Paragraph, Wrap
    },
    Frame,
};

use crate::themes::{CasinoStyles, CardSuit};

/// Poker card component
#[derive(Debug, Clone)]
pub struct Card {
    pub rank: String,
    pub suit: CardSuit,
    pub hidden: bool,
}

impl Card {
    pub fn new(rank: String, suit: CardSuit) -> Self {
        Self {
            rank,
            suit,
            hidden: false,
        }
    }
    
    pub fn hidden() -> Self {
        Self {
            rank: "?".to_string(),
            suit: CardSuit::Spades,
            hidden: true,
        }
    }
    
    /// Render the card as a widget
    pub fn render(&self, styles: &CasinoStyles) -> Paragraph {
        let suit_symbol = match self.suit {
            CardSuit::Spades => "♠",
            CardSuit::Hearts => "♥", 
            CardSuit::Diamonds => "♦",
            CardSuit::Clubs => "♣",
        };
        
        let card_text = if self.hidden {
            "┌─────┐\n│ ??? │\n│  ?  │\n│ ??? │\n└─────┘".to_string()
        } else {
            format!(
                "┌─────┐\n│ {}{}  │\n│  {}  │\n│  {} {} │\n└─────┘", 
                self.rank, suit_symbol, suit_symbol, suit_symbol, self.rank
            )
        };
        
        let style = if self.hidden {
            styles.border()
        } else {
            styles.card(self.suit)
        };
        
        Paragraph::new(card_text)
            .style(style)
            .alignment(Alignment::Center)
    }
}

/// Player info component
#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub name: String,
    pub chips: u64,
    pub bet: u64,
    pub is_active: bool,
    pub is_dealer: bool,
    pub cards: Vec<Card>,
}

impl PlayerInfo {
    pub fn new(name: String, chips: u64) -> Self {
        Self {
            name,
            chips,
            bet: 0,
            is_active: false,
            is_dealer: false,
            cards: Vec::new(),
        }
    }
    
    /// Render player information panel
    pub fn render(&self, area: Rect, frame: &mut Frame, styles: &CasinoStyles) {
        let title = if self.is_dealer {
            format!("👑 {} (Dealer)", self.name)
        } else {
            self.name.clone()
        };
        
        let name_style = styles.player_name(self.is_active);
        let border_style = if self.is_active {
            styles.button_primary()
        } else {
            styles.border()
        };
        
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title_style(name_style);
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        // Player stats
        let stats_text = vec![
            Line::from(vec![
                Span::styled("Chips: ", styles.subtitle()),
                Span::styled(format!("${}", self.chips), styles.chips()),
            ]),
            Line::from(vec![
                Span::styled("Bet: ", styles.subtitle()),
                Span::styled(format!("${}", self.bet), styles.warning()),
            ]),
        ];
        
        let stats = Paragraph::new(stats_text)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(stats, inner);
    }
}

/// Action buttons component
#[derive(Debug, Clone)]
pub struct ActionButtons {
    pub available_actions: Vec<String>,
    pub selected_index: usize,
}

impl ActionButtons {
    pub fn new(actions: Vec<String>) -> Self {
        Self {
            available_actions: actions,
            selected_index: 0,
        }
    }
    
    pub fn next(&mut self) {
        if !self.available_actions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.available_actions.len();
        }
    }
    
    pub fn previous(&mut self) {
        if !self.available_actions.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.available_actions.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }
    
    pub fn selected_action(&self) -> Option<&String> {
        self.available_actions.get(self.selected_index)
    }
    
    /// Render action buttons
    pub fn render(&self, area: Rect, frame: &mut Frame, styles: &CasinoStyles) {
        let button_height = 3;
        let button_spacing = 1;
        let total_height = self.available_actions.len() * (button_height + button_spacing);
        
        if total_height == 0 {
            return;
        }
        
        let constraints: Vec<Constraint> = (0..self.available_actions.len())
            .map(|_| Constraint::Length(button_height as u16))
            .collect();
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);
        
        for (i, action) in self.available_actions.iter().enumerate() {
            if let Some(chunk) = chunks.get(i) {
                let is_selected = i == self.selected_index;
                let style = if is_selected {
                    styles.button_primary()
                } else {
                    styles.button_secondary()
                };
                
                let button_text = if is_selected {
                    format!("▶ {} ◀", action)
                } else {
                    format!("  {}  ", action)
                };
                
                let button = Paragraph::new(button_text)
                    .style(style)
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(if is_selected { 
                                styles.button_primary() 
                            } else { 
                                styles.border() 
                            })
                    );
                
                frame.render_widget(button, *chunk);
            }
        }
    }
}

/// Game table component showing pot, community cards, etc.
#[derive(Debug, Clone)]
pub struct GameTable {
    pub pot: u64,
    pub community_cards: Vec<Card>,
    pub current_round: String,
}

impl GameTable {
    pub fn new() -> Self {
        Self {
            pot: 0,
            community_cards: Vec::new(),
            current_round: "Pre-flop".to_string(),
        }
    }
    
    /// Render the game table
    pub fn render(&self, area: Rect, frame: &mut Frame, styles: &CasinoStyles) {
        let block = Block::default()
            .title("🃏 Poker Table")
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(styles.border())
            .title_style(styles.title());
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Pot info
                Constraint::Min(6),    // Community cards
                Constraint::Length(2), // Round info
            ])
            .split(inner);
        
        // Pot display
        let pot_text = vec![
            Line::from(vec![
                Span::styled("💰 Pot: ", styles.subtitle()),
                Span::styled(format!("${}", self.pot), styles.chips()),
            ])
        ];
        
        let pot_widget = Paragraph::new(pot_text)
            .alignment(Alignment::Center);
        frame.render_widget(pot_widget, chunks[0]);
        
        // Community cards
        if !self.community_cards.is_empty() {
            let card_width = 7; // Width of each card
            let card_spacing = 2;
            let total_width = self.community_cards.len() * card_width + 
                             (self.community_cards.len().saturating_sub(1)) * card_spacing;
            
            if total_width <= chunks[1].width as usize {
                let cards_area = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        self.community_cards.iter()
                            .map(|_| Constraint::Length(card_width as u16))
                            .collect::<Vec<_>>()
                    )
                    .split(chunks[1]);
                
                for (i, card) in self.community_cards.iter().enumerate() {
                    if let Some(card_area) = cards_area.get(i) {
                        let card_widget = card.render(styles);
                        frame.render_widget(card_widget, *card_area);
                    }
                }
            }
        }
        
        // Round info
        let round_text = Text::from(Line::from(vec![
            Span::styled("Round: ", styles.subtitle()),
            Span::styled(&self.current_round, styles.warning()),
        ]));
        
        let round_widget = Paragraph::new(round_text)
            .alignment(Alignment::Center);
        frame.render_widget(round_widget, chunks[2]);
    }
}

impl Default for GameTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Modal dialog component
pub struct Modal<'a> {
    title: &'a str,
    content: Text<'a>,
    width: u16,
    height: u16,
}

impl<'a> Modal<'a> {
    pub fn new(title: &'a str, content: Text<'a>) -> Self {
        Self {
            title,
            content,
            width: 50,
            height: 20,
        }
    }
    
    pub fn size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    
    /// Render modal in the center of the screen
    pub fn render(&self, area: Rect, frame: &mut Frame, styles: &CasinoStyles) {
        let popup_area = center_rect(self.width, self.height, area);
        
        // Clear the background
        frame.render_widget(Clear, popup_area);
        
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(styles.button_primary())
            .title_style(styles.title());
        
        let paragraph = Paragraph::new(self.content.clone())
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);
        
        frame.render_widget(paragraph, popup_area);
    }
}

/// Helper function to center a rectangle
fn center_rect(width: u16, height: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height) / 2),
            Constraint::Length(height),
            Constraint::Percentage((100 - height) / 2),
        ])
        .split(area);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width) / 2),
            Constraint::Length(width),
            Constraint::Percentage((100 - width) / 2),
        ])
        .split(popup_layout[1])[1]
}