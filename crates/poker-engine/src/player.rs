use crate::{Card, errors::{PokerError, Result}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerStatus {
    Active,
    Folded,
    AllIn,
    SittingOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub chips: u64,
    pub hole_cards: Option<[Card; 2]>,
    pub status: PlayerStatus,
    pub current_bet: u64,
    pub total_bet_this_round: u64,
}

impl Player {
    pub fn new(id: usize, name: String, chips: u64) -> Self {
        Self {
            id,
            name,
            chips,
            hole_cards: None,
            status: PlayerStatus::Active,
            current_bet: 0,
            total_bet_this_round: 0,
        }
    }

    pub fn deal_hole_cards(&mut self, cards: [Card; 2]) {
        self.hole_cards = Some(cards);
    }

    pub fn fold(&mut self) {
        self.status = PlayerStatus::Folded;
    }

    pub fn bet(&mut self, amount: u64) -> Result<()> {
        if amount > self.chips {
            return Err(PokerError::InsufficientChips {
                needed: amount,
                available: self.chips,
            });
        }

        self.chips -= amount;
        self.current_bet += amount;
        self.total_bet_this_round += amount;

        if self.chips == 0 {
            self.status = PlayerStatus::AllIn;
        }

        Ok(())
    }

    pub fn reset_current_bet(&mut self) {
        self.current_bet = 0;
    }

    pub fn reset_for_new_hand(&mut self) {
        self.hole_cards = None;
        self.status = if self.chips > 0 {
            PlayerStatus::Active
        } else {
            PlayerStatus::SittingOut
        };
        self.current_bet = 0;
        self.total_bet_this_round = 0;
    }

    pub fn win_chips(&mut self, amount: u64) {
        self.chips += amount;
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, PlayerStatus::Active | PlayerStatus::AllIn)
    }

    pub fn can_act(&self) -> bool {
        matches!(self.status, PlayerStatus::Active) && self.chips > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rank, Suit};

    #[test]
    fn test_player_creation() {
        let player = Player::new(1, "Alice".to_string(), 1000);
        assert_eq!(player.id, 1);
        assert_eq!(player.name, "Alice");
        assert_eq!(player.chips, 1000);
        assert_eq!(player.status, PlayerStatus::Active);
    }
    
    #[test]
    fn test_deal_hole_cards() {
        let mut player = Player::new(1, "Alice".to_string(), 1000);
        let cards = [
            Card { rank: crate::Rank::Ace, suit: crate::Suit::Spades },
            Card { rank: crate::Rank::King, suit: crate::Suit::Spades },
        ];
        player.deal_hole_cards(cards);
        assert_eq!(player.hole_cards, Some(cards));
    }
    
    #[test]
    fn test_fold() {
        let mut player = Player::new(1, "Alice".to_string(), 1000);
        player.fold();
        assert_eq!(player.status, PlayerStatus::Folded);
        assert!(!player.is_active());
    }
    
    #[test]
    fn test_betting() {
        let mut player = Player::new(1, "Alice".to_string(), 1000);
        assert!(player.bet(100).is_ok());
        assert_eq!(player.chips, 900);
        assert_eq!(player.current_bet, 100);
        assert_eq!(player.total_bet_this_round, 100);
    }
    
    #[test]
    fn test_insufficient_chips() {
        let mut player = Player::new(1, "Alice".to_string(), 100);
        let result = player.bet(200);
        assert!(result.is_err());
        assert_eq!(player.chips, 100); // Chips unchanged
    }
    
    #[test]
    fn test_all_in() {
        let mut player = Player::new(1, "Alice".to_string(), 100);
        assert!(player.bet(100).is_ok());
        assert_eq!(player.chips, 0);
        assert_eq!(player.status, PlayerStatus::AllIn);
    }
    
    #[test]
    fn test_win_chips() {
        let mut player = Player::new(1, "Alice".to_string(), 100);
        player.win_chips(200);
        assert_eq!(player.chips, 300);
    }
    
    #[test]
    fn test_reset_for_new_hand() {
        let mut player = Player::new(1, "Alice".to_string(), 100);
        player.current_bet = 50;
        player.total_bet_this_round = 50;
        player.status = PlayerStatus::Folded;
        player.hole_cards = Some([
            Card { rank: crate::Rank::Ace, suit: crate::Suit::Spades },
            Card { rank: crate::Rank::King, suit: crate::Suit::Spades },
        ]);
        
        player.reset_for_new_hand();
        
        assert_eq!(player.current_bet, 0);
        assert_eq!(player.total_bet_this_round, 0);
        assert_eq!(player.status, PlayerStatus::Active);
        assert!(player.hole_cards.is_none());
    }
    
    #[test]
    fn test_reset_for_new_hand_no_chips() {
        let mut player = Player::new(1, "Alice".to_string(), 0);
        player.reset_for_new_hand();
        assert_eq!(player.status, PlayerStatus::SittingOut);
    }
    
    #[test]
    fn test_multiple_bets() {
        let mut player = Player::new(1, "Alice".to_string(), 1000);
        
        // First bet
        assert!(player.bet(100).is_ok());
        assert_eq!(player.chips, 900);
        assert_eq!(player.current_bet, 100);
        
        // Second bet (raise)
        assert!(player.bet(200).is_ok());
        assert_eq!(player.chips, 700);
        assert_eq!(player.current_bet, 300);
        assert_eq!(player.total_bet_this_round, 300);
    }
    
    #[test]
    fn test_can_act() {
        let mut player = Player::new(1, "Alice".to_string(), 1000);
        assert!(player.can_act());
        
        player.status = PlayerStatus::Folded;
        assert!(!player.can_act());
        
        player.status = PlayerStatus::AllIn;
        assert!(!player.can_act());
        
        player.status = PlayerStatus::SittingOut;
        assert!(!player.can_act());
    }
    
    #[test]
    fn test_edge_case_zero_bet() {
        let mut player = Player::new(1, "Alice".to_string(), 1000);
        assert!(player.bet(0).is_ok());
        assert_eq!(player.chips, 1000);
        assert_eq!(player.current_bet, 0);
    }
    
    #[test]
    fn test_exact_chips_bet() {
        let mut player = Player::new(1, "Alice".to_string(), 100);
        assert!(player.bet(100).is_ok());
        assert_eq!(player.chips, 0);
        assert_eq!(player.status, PlayerStatus::AllIn);
        assert_eq!(player.current_bet, 100);
    }
} 