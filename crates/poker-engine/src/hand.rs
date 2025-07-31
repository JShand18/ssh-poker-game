use crate::{Card, Rank, Suit};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandRank {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
}

impl fmt::Display for HandRank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                HandRank::HighCard => "High Card",
                HandRank::OnePair => "One Pair",
                HandRank::TwoPair => "Two Pair",
                HandRank::ThreeOfAKind => "Three of a Kind",
                HandRank::Straight => "Straight",
                HandRank::Flush => "Flush",
                HandRank::FullHouse => "Full House",
                HandRank::FourOfAKind => "Four of a Kind",
                HandRank::StraightFlush => "Straight Flush",
            }
        )
    }
}

/// A thread-safe hand evaluator that reuses the underlying poker evaluator
pub struct HandEvaluator {
    evaluator: poker::Evaluator,
}

impl HandEvaluator {
    pub fn new() -> Self {
        Self {
            evaluator: poker::Evaluator::new(),
        }
    }
    
    pub fn evaluate(&self, cards: &[Card]) -> Hand {
        // Convert our cards to poker crate cards
        let poker_cards: Vec<poker::Card> = cards.iter().map(|c| c.into()).collect();
        
        // Evaluate the hand
        let eval = self.evaluator.evaluate(&poker_cards).unwrap();
        
        // Convert evaluation to our HandRank
        let rank = match eval.class() {
            poker::EvalClass::StraightFlush { .. } => HandRank::StraightFlush,
            poker::EvalClass::FourOfAKind { .. } => HandRank::FourOfAKind,
            poker::EvalClass::FullHouse { .. } => HandRank::FullHouse,
            poker::EvalClass::Flush { .. } => HandRank::Flush,
            poker::EvalClass::Straight { .. } => HandRank::Straight,
            poker::EvalClass::ThreeOfAKind { .. } => HandRank::ThreeOfAKind,
            poker::EvalClass::TwoPair { .. } => HandRank::TwoPair,
            poker::EvalClass::Pair { .. } => HandRank::OnePair,
            poker::EvalClass::HighCard { .. } => HandRank::HighCard,
        };
        
        Hand {
            cards: cards.to_vec(),
            rank,
            eval,
        }
    }
}

impl Default for HandEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hand {
    cards: Vec<Card>,
    rank: HandRank,
    eval: poker::Eval,
}

impl Hand {
    /// Legacy method that creates a new evaluator each time - prefer using HandEvaluator
    pub fn evaluate(cards: &[Card]) -> Self {
        let evaluator = HandEvaluator::new();
        evaluator.evaluate(cards)
    }
    
    pub fn rank(&self) -> HandRank {
        self.rank
    }
    
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }
}

impl From<&Card> for poker::Card {
    fn from(card: &Card) -> Self {
        let rank = match card.rank {
            Rank::Two => poker::Rank::Two,
            Rank::Three => poker::Rank::Three,
            Rank::Four => poker::Rank::Four,
            Rank::Five => poker::Rank::Five,
            Rank::Six => poker::Rank::Six,
            Rank::Seven => poker::Rank::Seven,
            Rank::Eight => poker::Rank::Eight,
            Rank::Nine => poker::Rank::Nine,
            Rank::Ten => poker::Rank::Ten,
            Rank::Jack => poker::Rank::Jack,
            Rank::Queen => poker::Rank::Queen,
            Rank::King => poker::Rank::King,
            Rank::Ace => poker::Rank::Ace,
        };
        
        let suit = match card.suit {
            Suit::Spades => poker::Suit::Spades,
            Suit::Hearts => poker::Suit::Hearts,
            Suit::Diamonds => poker::Suit::Diamonds,
            Suit::Clubs => poker::Suit::Clubs,
        };
        
        poker::Card::new(rank, suit)
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Use the poker crate's evaluation comparison
        self.eval.cmp(&other.eval)
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn royal_flush() {
        let cards = vec![
            Card { rank: Rank::Ace, suit: Suit::Hearts },
            Card { rank: Rank::King, suit: Suit::Hearts },
            Card { rank: Rank::Queen, suit: Suit::Hearts },
            Card { rank: Rank::Jack, suit: Suit::Hearts },
            Card { rank: Rank::Ten, suit: Suit::Hearts },
        ];
        let hand = Hand::evaluate(&cards);
        assert_eq!(hand.rank(), HandRank::StraightFlush);
    }

    #[test]
    fn four_of_a_kind() {
        let cards = vec![
            Card { rank: Rank::Seven, suit: Suit::Hearts },
            Card { rank: Rank::Seven, suit: Suit::Diamonds },
            Card { rank: Rank::Seven, suit: Suit::Clubs },
            Card { rank: Rank::Seven, suit: Suit::Spades },
            Card { rank: Rank::Two, suit: Suit::Hearts },
        ];
        let hand = Hand::evaluate(&cards);
        assert_eq!(hand.rank(), HandRank::FourOfAKind);
    }

    #[test]
    fn full_house() {
        let cards = vec![
            Card { rank: Rank::Three, suit: Suit::Hearts },
            Card { rank: Rank::Three, suit: Suit::Diamonds },
            Card { rank: Rank::Three, suit: Suit::Clubs },
            Card { rank: Rank::Two, suit: Suit::Spades },
            Card { rank: Rank::Two, suit: Suit::Hearts },
        ];
        let hand = Hand::evaluate(&cards);
        assert_eq!(hand.rank(), HandRank::FullHouse);
    }

    #[test]
    fn hand_comparison() {
        let flush_cards = vec![
            Card { rank: Rank::Ace, suit: Suit::Hearts },
            Card { rank: Rank::King, suit: Suit::Hearts },
            Card { rank: Rank::Ten, suit: Suit::Hearts },
            Card { rank: Rank::Five, suit: Suit::Hearts },
            Card { rank: Rank::Two, suit: Suit::Hearts },
        ];
        let flush = Hand::evaluate(&flush_cards);

        let pair_cards = vec![
            Card { rank: Rank::Seven, suit: Suit::Hearts },
            Card { rank: Rank::Seven, suit: Suit::Diamonds },
            Card { rank: Rank::King, suit: Suit::Clubs },
            Card { rank: Rank::Ten, suit: Suit::Spades },
            Card { rank: Rank::Two, suit: Suit::Hearts },
        ];
        let pair = Hand::evaluate(&pair_cards);

        assert!(flush > pair);
    }

    #[test]
    #[ignore] // This test is very slow and should only be run manually
    fn evaluate_all_5_card_hands() {
        use crate::deck::Deck;
        use itertools::Itertools;

        let deck = Deck::new();
        let cards = deck.as_ref();

        let mut count = 0;
        for hand_cards in cards.iter().combinations(5) {
            let hand_slice: Vec<Card> = hand_cards.into_iter().cloned().collect();
            let _hand = Hand::evaluate(&hand_slice);
            count += 1;
        }

        assert_eq!(count, 2_598_960);
    }
} 