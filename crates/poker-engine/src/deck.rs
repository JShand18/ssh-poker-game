use crate::card::{Card, Rank, Suit};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(52);
        for &suit in &[Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
            for &rank in &[
                Rank::Two,
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
                Rank::Ten,
                Rank::Jack,
                Rank::Queen,
                Rank::King,
                Rank::Ace,
            ] {
                cards.push(Card::new(rank, suit));
            }
        }
        Deck { cards }
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl AsRef<[Card]> for Deck {
    fn as_ref(&self) -> &[Card] {
        &self.cards
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn new_deck_has_52_cards() {
        let deck = Deck::new();
        assert_eq!(deck.len(), 52);
    }

    #[test]
    fn shuffle_changes_card_order() {
        let mut deck = Deck::new();
        let original_order = deck.as_ref().to_vec();
        deck.shuffle();
        assert_ne!(original_order, deck.as_ref());
    }

    #[test]
    fn drawing_a_card_reduces_deck_size() {
        let mut deck = Deck::new();
        let original_size = deck.len();
        let card = deck.draw();
        assert!(card.is_some());
        assert_eq!(deck.len(), original_size - 1);
    }

    #[test]
    fn drawing_all_cards_empties_deck() {
        let mut deck = Deck::new();
        for _ in 0..52 {
            assert!(deck.draw().is_some());
        }
        assert!(deck.draw().is_none());
        assert!(deck.is_empty());
    }

    #[test]
    fn shuffled_deck_has_52_unique_cards() {
        let mut deck = Deck::new();
        deck.shuffle();
        let mut drawn_cards = HashSet::new();
        for _ in 0..52 {
            if let Some(card) = deck.draw() {
                assert!(drawn_cards.insert(card));
            }
        }
        assert_eq!(drawn_cards.len(), 52);
    }
} 