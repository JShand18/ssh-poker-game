use crate::{Action, Player, PlayerStatus, errors::{PokerError, Result}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents betting limits and rules for the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BettingRules {
    pub small_blind: u64,
    pub big_blind: u64,
    pub min_raise: u64,  // Minimum raise amount (typically big blind)
}

impl BettingRules {
    pub fn new(small_blind: u64, big_blind: u64) -> Self {
        Self {
            small_blind,
            big_blind,
            min_raise: big_blind,
        }
    }
}

/// Tracks the current betting state for a round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BettingRound {
    pub current_bet: u64,
    pub minimum_raise: u64,
    pub last_aggressor: Option<usize>,
    pub player_bets: HashMap<usize, u64>,
    pub total_pot: u64,
}

impl BettingRound {
    pub fn new() -> Self {
        Self {
            current_bet: 0,
            minimum_raise: 0,
            last_aggressor: None,
            player_bets: HashMap::new(),
            total_pot: 0,
        }
    }
    
    pub fn reset(&mut self) {
        self.current_bet = 0;
        self.minimum_raise = 0;
        self.last_aggressor = None;
        self.player_bets.clear();
    }
    
    pub fn player_bet_amount(&self, player_id: usize) -> u64 {
        self.player_bets.get(&player_id).copied().unwrap_or(0)
    }
    
    pub fn amount_to_call(&self, player_id: usize) -> u64 {
        self.current_bet.saturating_sub(self.player_bet_amount(player_id))
    }
}

/// Validates betting actions according to no-limit hold'em rules
pub struct BettingValidator {
    rules: BettingRules,
}

impl BettingValidator {
    pub fn new(rules: BettingRules) -> Self {
        Self { rules }
    }
    
    /// Validates if a betting action is legal
    pub fn validate_action(
        &self,
        action: &Action,
        player: &Player,
        round: &BettingRound,
    ) -> Result<()> {
        match action {
            Action::Fold => {
                // Always allowed if player can act
                if !player.can_act() {
                    return Err(PokerError::InvalidAction("Player cannot act".to_string()));
                }
                Ok(())
            }
            
            Action::Check => {
                // Only allowed if no bet to call
                if round.amount_to_call(player.id) > 0 {
                    return Err(PokerError::InvalidAction(
                        "Cannot check when there's a bet to call".to_string()
                    ));
                }
                Ok(())
            }
            
            Action::Call => {
                let call_amount = round.amount_to_call(player.id);
                if call_amount == 0 {
                    return Err(PokerError::InvalidAction(
                        "Nothing to call, use check instead".to_string()
                    ));
                }
                if call_amount > player.chips {
                    return Err(PokerError::InsufficientChips {
                        needed: call_amount,
                        available: player.chips,
                    });
                }
                Ok(())
            }
            
            Action::Bet(amount) => {
                // Bet is only valid when current bet is 0
                if round.current_bet > 0 {
                    return Err(PokerError::InvalidAction(
                        "Cannot bet when there's already a bet, use raise".to_string()
                    ));
                }
                
                // Minimum bet is the big blind
                if *amount < self.rules.big_blind {
                    return Err(PokerError::InvalidBetAmount(
                        format!("Minimum bet is {}", self.rules.big_blind)
                    ));
                }
                
                if *amount > player.chips {
                    return Err(PokerError::InsufficientChips {
                        needed: *amount,
                        available: player.chips,
                    });
                }
                
                Ok(())
            }
            
            Action::Raise(raise_amount) => {
                // Can only raise if there's a bet to raise
                if round.current_bet == 0 {
                    return Err(PokerError::InvalidAction(
                        "Cannot raise when there's no bet, use bet instead".to_string()
                    ));
                }
                
                // Minimum raise in no-limit is the size of the last bet/raise
                let min_raise = round.minimum_raise.max(self.rules.big_blind);
                if *raise_amount < min_raise {
                    return Err(PokerError::InvalidBetAmount(
                        format!("Minimum raise is {}", min_raise)
                    ));
                }
                
                let total_amount = round.amount_to_call(player.id) + raise_amount;
                if total_amount > player.chips {
                    return Err(PokerError::InsufficientChips {
                        needed: total_amount,
                        available: player.chips,
                    });
                }
                
                Ok(())
            }
            
            Action::AllIn => {
                // Always allowed if player has chips
                if player.chips == 0 {
                    return Err(PokerError::InvalidAction(
                        "Cannot go all-in with no chips".to_string()
                    ));
                }
                Ok(())
            }
        }
    }
    
    /// Calculate the actual bet amount for an action
    pub fn calculate_bet_amount(
        &self,
        action: &Action,
        player: &Player,
        round: &BettingRound,
    ) -> u64 {
        match action {
            Action::Fold | Action::Check => 0,
            Action::Call => round.amount_to_call(player.id).min(player.chips),
            Action::Bet(amount) => (*amount).min(player.chips),
            Action::Raise(raise_amount) => {
                let call_amount = round.amount_to_call(player.id);
                (call_amount + raise_amount).min(player.chips)
            }
            Action::AllIn => player.chips,
        }
    }
    
    /// Get all valid actions for a player
    pub fn get_valid_actions(
        &self,
        player: &Player,
        round: &BettingRound,
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        
        if !player.can_act() {
            return actions;
        }
        
        // Fold is always available
        actions.push(Action::Fold);
        
        let to_call = round.amount_to_call(player.id);
        
        if to_call == 0 {
            // Can check
            actions.push(Action::Check);
            
            // If no current bet, can bet. If there IS a current bet, can raise
            if round.current_bet == 0 {
                // Can bet (if have enough chips)
                if player.chips >= self.rules.big_blind {
                    actions.push(Action::Bet(self.rules.big_blind));
                }
            } else {
                // Can raise even though we don't need to call (e.g., big blind)
                let min_raise = round.minimum_raise.max(self.rules.big_blind);
                if player.chips >= min_raise {
                    actions.push(Action::Raise(min_raise));
                }
            }
        } else {
            // Can call (if have enough chips)
            if player.chips >= to_call {
                actions.push(Action::Call);
            }
            
            // Can raise (if have enough chips)
            let min_raise = round.minimum_raise.max(self.rules.big_blind);
            if player.chips >= to_call + min_raise {
                actions.push(Action::Raise(min_raise));
            }
        }
        
        // All-in is available if player has chips
        if player.chips > 0 {
            actions.push(Action::AllIn);
        }
        
        actions
    }
}

/// Manages side pots when players go all-in
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidePot {
    pub amount: u64,
    pub eligible_players: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PotManager {
    pub main_pot: u64,
    pub side_pots: Vec<SidePot>,
}

impl PotManager {
    pub fn new() -> Self {
        Self {
            main_pot: 0,
            side_pots: Vec::new(),
        }
    }
    
    /// Calculate side pots based on player contributions
    pub fn calculate_side_pots(&mut self, players: &[Player], round: &BettingRound) {
        // Helper function to get player's total bet amount
        // Uses total_bet_this_round if set, otherwise falls back to round data
        let get_player_total_bet = |player: &Player| -> u64 {
            if player.total_bet_this_round > 0 {
                player.total_bet_this_round
            } else {
                round.player_bet_amount(player.id)
            }
        };
        
        // Get all unique bet amounts from players who are all-in
        let mut all_in_amounts: Vec<(usize, u64)> = players
            .iter()
            .filter(|p| p.status == PlayerStatus::AllIn)
            .map(|p| (p.id, get_player_total_bet(p)))
            .collect();
        
        // Sort by bet amount
        all_in_amounts.sort_by_key(|&(_, amount)| amount);
        
        self.side_pots.clear();
        let mut processed_amount = 0;
        
        for (_, all_in_amount) in all_in_amounts {
            if all_in_amount <= processed_amount {
                continue;
            }
            
            let pot_contribution = all_in_amount - processed_amount;
            let mut pot_amount = 0;
            let mut eligible_players = Vec::new();
            
            // Calculate who contributes to this pot
            for player in players {
                let player_total_bet = get_player_total_bet(player);
                if player_total_bet > processed_amount {
                    let contribution = (player_total_bet - processed_amount).min(pot_contribution);
                    pot_amount += contribution;
                    
                    // Only non-folded players are eligible to win
                    if player.status != PlayerStatus::Folded && player.status != PlayerStatus::SittingOut {
                        if player_total_bet >= all_in_amount {
                            eligible_players.push(player.id);
                        }
                    }
                }
            }
            
            if pot_amount > 0 && !eligible_players.is_empty() {
                self.side_pots.push(SidePot {
                    amount: pot_amount,
                    eligible_players,
                });
            }
            
            processed_amount = all_in_amount;
        }
        
        // Main pot is everything above the highest all-in
        self.main_pot = 0;
        let mut main_pot_eligible = Vec::new();
        
        for player in players {
            let player_total_bet = get_player_total_bet(player);
            if player_total_bet > processed_amount {
                self.main_pot += player_total_bet - processed_amount;
            }
            
            // Track who is eligible for main pot
            if player.status != PlayerStatus::Folded && 
               player.status != PlayerStatus::SittingOut && 
               player_total_bet > processed_amount {
                main_pot_eligible.push(player.id);
            }
        }
        
        // If there's only a main pot (no all-ins), calculate total from all players
        if self.side_pots.is_empty() && self.main_pot == 0 {
            self.main_pot = players.iter()
                .map(|p| get_player_total_bet(p))
                .sum();
        }
    }
    
    pub fn total_pot(&self) -> u64 {
        self.main_pot + self.side_pots.iter().map(|sp| sp.amount).sum::<u64>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{Player, PlayerStatus};

    #[test]
    fn test_betting_validator_check() {
        let rules = BettingRules::new(10, 20);
        let validator = BettingValidator::new(rules);
        let player = Player::new(0, "Test".to_string(), 1000);
        let round = BettingRound::new();

        // Check is valid when current bet is 0
        assert!(validator.validate_action(&Action::Check, &player, &round).is_ok());

        // Check is invalid when there's a bet
        let mut round_with_bet = round.clone();
        round_with_bet.current_bet = 50;
        assert!(validator.validate_action(&Action::Check, &player, &round_with_bet).is_err());
    }

    #[test]
    fn test_betting_validator_bet() {
        let rules = BettingRules::new(10, 20);
        let validator = BettingValidator::new(rules);
        let player = Player::new(0, "Test".to_string(), 1000);
        let round = BettingRound::new();

        // Valid bet
        assert!(validator.validate_action(&Action::Bet(50), &player, &round).is_ok());

        // Bet below minimum
        assert!(validator.validate_action(&Action::Bet(10), &player, &round).is_err());

        // Bet with insufficient chips
        let poor_player = Player::new(1, "Poor".to_string(), 30);
        assert!(validator.validate_action(&Action::Bet(50), &poor_player, &round).is_err());
    }

    #[test]
    fn test_betting_validator_raise() {
        let rules = BettingRules::new(10, 20);
        let validator = BettingValidator::new(rules);
        let player = Player::new(0, "Test".to_string(), 1000);
        
        let mut round = BettingRound::new();
        round.current_bet = 50;
        round.minimum_raise = 50;

        // Valid raise
        assert!(validator.validate_action(&Action::Raise(50), &player, &round).is_ok());

        // Raise below minimum
        assert!(validator.validate_action(&Action::Raise(30), &player, &round).is_err());

        // Raise with no current bet
        let round_no_bet = BettingRound::new();
        assert!(validator.validate_action(&Action::Raise(50), &player, &round_no_bet).is_err());
    }
    
    #[test]
    fn test_pot_manager_simple() {
        let mut pot_manager = PotManager::new();
        let players = vec![
            Player::new(0, "Alice".to_string(), 1000),
            Player::new(1, "Bob".to_string(), 1000),
            Player::new(2, "Charlie".to_string(), 1000),
        ];
        
        let mut round = BettingRound::new();
        round.player_bets.insert(0, 100);
        round.player_bets.insert(1, 100);
        round.player_bets.insert(2, 100);
        
        pot_manager.calculate_side_pots(&players, &round);
        
        assert_eq!(pot_manager.main_pot, 300);
        assert_eq!(pot_manager.side_pots.len(), 0);
        assert_eq!(pot_manager.total_pot(), 300);
    }
    
    #[test]
    fn test_pot_manager_with_all_in() {
        let mut pot_manager = PotManager::new();
        let mut players = vec![
            Player::new(0, "Alice".to_string(), 100),
            Player::new(1, "Bob".to_string(), 500),
            Player::new(2, "Charlie".to_string(), 1000),
        ];
        
        // Simulate all-in scenario
        players[0].chips = 0; // Alice is all-in
        players[0].status = PlayerStatus::AllIn;
        players[1].chips = 400; // Bob has 400 left after betting 100
        players[2].chips = 700; // Charlie has 700 left after betting 300
        
        let mut round = BettingRound::new();
        round.player_bets.insert(0, 100);  // Alice all-in for 100
        round.player_bets.insert(1, 100);  // Bob matches
        round.player_bets.insert(2, 300);  // Charlie raises
        
        pot_manager.calculate_side_pots(&players, &round);
        
        // Side pot 1: 100 * 3 = 300 (all players)
        assert_eq!(pot_manager.side_pots.len(), 1);
        assert_eq!(pot_manager.side_pots[0].amount, 300);
        assert_eq!(pot_manager.side_pots[0].eligible_players, vec![0, 1, 2]);
        
        // Main pot: 200 (only Bob and Charlie)
        assert_eq!(pot_manager.main_pot, 200);
        assert_eq!(pot_manager.total_pot(), 500);
    }
    
    #[test]
    fn test_pot_manager_multiple_all_ins() {
        let mut pot_manager = PotManager::new();
        let mut players = vec![
            Player::new(0, "Alice".to_string(), 50),
            Player::new(1, "Bob".to_string(), 150),
            Player::new(2, "Charlie".to_string(), 300),
            Player::new(3, "Dave".to_string(), 500),
        ];
        
        // Simulate multiple all-ins
        players[0].chips = 0;  // Alice all-in for 50
        players[0].status = PlayerStatus::AllIn;
        players[1].chips = 0;  // Bob all-in for 150
        players[1].status = PlayerStatus::AllIn;
        players[2].chips = 0;  // Charlie all-in for 300
        players[2].status = PlayerStatus::AllIn;
        players[3].chips = 200; // Dave has 200 left
        
        let mut round = BettingRound::new();
        round.player_bets.insert(0, 50);   // Alice
        round.player_bets.insert(1, 150);  // Bob
        round.player_bets.insert(2, 300);  // Charlie
        round.player_bets.insert(3, 300);  // Dave matches Charlie
        
        pot_manager.calculate_side_pots(&players, &round);
        
        // Side pot 1: 50 * 4 = 200 (all players)
        assert_eq!(pot_manager.side_pots[0].amount, 200);
        assert_eq!(pot_manager.side_pots[0].eligible_players.len(), 4);
        
        // Side pot 2: (150-50) * 3 = 300 (Bob, Charlie, Dave)
        assert_eq!(pot_manager.side_pots[1].amount, 300);
        assert_eq!(pot_manager.side_pots[1].eligible_players.len(), 3);
        
        // Side pot 3: (300-150) * 2 = 300 (Charlie, Dave)
        assert_eq!(pot_manager.side_pots[2].amount, 300);
        assert_eq!(pot_manager.side_pots[2].eligible_players.len(), 2);
        
        // No main pot (everyone is all-in)
        assert_eq!(pot_manager.main_pot, 0);
        assert_eq!(pot_manager.total_pot(), 800);
    }
    
    #[test]
    fn test_betting_round_edge_cases() {
        let mut round = BettingRound::new();
        
        // Test with no bets
        assert_eq!(round.amount_to_call(0), 0);
        assert_eq!(round.player_bet_amount(999), 0); // Non-existent player
        
        // Test player who hasn't bet
        round.player_bets.insert(0, 100);
        assert_eq!(round.amount_to_call(1), 100);
        
        // Test player who has partially bet
        round.player_bets.insert(1, 50);
        round.current_bet = 100;
        assert_eq!(round.amount_to_call(1), 50);
    }
    
    #[test]
    fn test_betting_validator_edge_cases() {
        let rules = BettingRules::new(10, 20);
        let validator = BettingValidator::new(rules);
        
        // Test with player who has exactly the big blind
        let poor_player = Player::new(0, "Poor".to_string(), 20);
        let round = BettingRound::new();
        assert!(validator.validate_action(&Action::Bet(20), &poor_player, &round).is_ok());
        assert!(validator.validate_action(&Action::Bet(21), &poor_player, &round).is_err());
        
        // Test all-in validation
        assert!(validator.validate_action(&Action::AllIn, &poor_player, &round).is_ok());
        
        // Test folded player trying to act
        let mut folded_player = Player::new(1, "Folded".to_string(), 1000);
        folded_player.status = PlayerStatus::Folded;
        assert!(validator.validate_action(&Action::Check, &folded_player, &round).is_err());
    }
    
    #[test]
    fn test_pot_manager_single_player_all_in() {
        let mut pot_manager = PotManager::new();
        let mut players = vec![
            Player::new(0, "Alice".to_string(), 100),
            Player::new(1, "Bob".to_string(), 1000),
        ];
        
        // Alice goes all-in, Bob has plenty
        players[0].chips = 0;
        players[0].status = PlayerStatus::AllIn;
        players[1].chips = 900;
        
        let mut round = BettingRound::new();
        round.player_bets.insert(0, 100);
        round.player_bets.insert(1, 100);
        
        pot_manager.calculate_side_pots(&players, &round);
        
        // Should have one side pot with both players
        assert_eq!(pot_manager.side_pots.len(), 1);
        assert_eq!(pot_manager.side_pots[0].amount, 200);
        assert_eq!(pot_manager.side_pots[0].eligible_players, vec![0, 1]);
        assert_eq!(pot_manager.main_pot, 0);
    }
    
    #[test]
    fn test_complex_pot_scenario() {
        let mut pot_manager = PotManager::new();
        let mut players = vec![
            Player::new(0, "Alice".to_string(), 25),
            Player::new(1, "Bob".to_string(), 100),
            Player::new(2, "Charlie".to_string(), 75),
            Player::new(3, "Dave".to_string(), 200),
            Player::new(4, "Eve".to_string(), 300),
        ];
        
        // Complex all-in scenario
        players[0].chips = 0;  // Alice all-in for 25
        players[0].status = PlayerStatus::AllIn;
        players[1].chips = 0;  // Bob all-in for 100
        players[1].status = PlayerStatus::AllIn;
        players[2].chips = 0;  // Charlie all-in for 75
        players[2].status = PlayerStatus::AllIn;
        players[3].chips = 0;  // Dave all-in for 200
        players[3].status = PlayerStatus::AllIn;
        players[4].chips = 100; // Eve has 100 left after calling 200
        
        let mut round = BettingRound::new();
        round.player_bets.insert(0, 25);
        round.player_bets.insert(1, 100);
        round.player_bets.insert(2, 75);
        round.player_bets.insert(3, 200);
        round.player_bets.insert(4, 200);
        
        pot_manager.calculate_side_pots(&players, &round);
        
        // Verify the complex side pot structure
        assert_eq!(pot_manager.side_pots[0].amount, 125); // 25 * 5
        assert_eq!(pot_manager.side_pots[0].eligible_players.len(), 5);
        
        assert_eq!(pot_manager.side_pots[1].amount, 200); // (75-25) * 4
        assert_eq!(pot_manager.side_pots[1].eligible_players.len(), 4);
        
        assert_eq!(pot_manager.side_pots[2].amount, 75); // (100-75) * 3
        assert_eq!(pot_manager.side_pots[2].eligible_players.len(), 3);
        
        assert_eq!(pot_manager.side_pots[3].amount, 200); // (200-100) * 2
        assert_eq!(pot_manager.side_pots[3].eligible_players.len(), 2);
        
        assert_eq!(pot_manager.main_pot, 0);
        assert_eq!(pot_manager.total_pot(), 600);
    }
    
    #[test]
    fn test_pot_manager_with_folded_players() {
        let mut pot_manager = PotManager::new();
        let mut players = vec![
            Player::new(0, "Alice".to_string(), 100),
            Player::new(1, "Bob".to_string(), 100),
            Player::new(2, "Charlie".to_string(), 100),
        ];
        
        // Charlie folded after betting
        players[2].status = PlayerStatus::Folded;
        
        let mut round = BettingRound::new();
        round.player_bets.insert(0, 50);
        round.player_bets.insert(1, 50);
        round.player_bets.insert(2, 20); // Charlie bet 20 then folded
        
        pot_manager.calculate_side_pots(&players, &round);
        
        // Main pot should include Charlie's dead money
        assert_eq!(pot_manager.main_pot, 120);
        assert_eq!(pot_manager.side_pots.len(), 0);
    }
} 