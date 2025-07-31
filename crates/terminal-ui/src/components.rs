// Components module for reusable UI elements
// This will be expanded as we build more complex UI components

use ratatui::style::{Color, Style};

/// Helper function to style cards based on suit
pub fn get_card_style(card: &poker_engine::Card) -> Style {
    match card.suit {
        poker_engine::Suit::Hearts | poker_engine::Suit::Diamonds => {
            Style::default().fg(Color::Red)
        }
        _ => Style::default().fg(Color::White),
    }
}

/// Format a card with unicode symbols
pub fn format_card_unicode(card: &poker_engine::Card) -> String {
    let suit = match card.suit {
        poker_engine::Suit::Clubs => "♣",
        poker_engine::Suit::Diamonds => "♦",
        poker_engine::Suit::Hearts => "♥",
        poker_engine::Suit::Spades => "♠",
    };
    
    let rank = match card.rank {
        poker_engine::Rank::Two => "2",
        poker_engine::Rank::Three => "3",
        poker_engine::Rank::Four => "4",
        poker_engine::Rank::Five => "5",
        poker_engine::Rank::Six => "6",
        poker_engine::Rank::Seven => "7",
        poker_engine::Rank::Eight => "8",
        poker_engine::Rank::Nine => "9",
        poker_engine::Rank::Ten => "10",
        poker_engine::Rank::Jack => "J",
        poker_engine::Rank::Queen => "Q",
        poker_engine::Rank::King => "K",
        poker_engine::Rank::Ace => "A",
    };
    
    format!("{}{}", rank, suit)
} 