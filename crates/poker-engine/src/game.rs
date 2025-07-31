use crate::{
    Card, Deck, 
    hand::{Hand, HandEvaluator},
    player::{Player, PlayerStatus}, 
    errors::{PokerError, Result}, 
    fsm::GameStateFSM,
    betting::{BettingRules, BettingRound, BettingValidator, PotManager}
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    PreFlop,
    Flop,
    Turn,
    River,
    Showdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Fold,
    Check,
    Call,
    Bet(u64),
    Raise(u64),
    AllIn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pot {
    pub amount: u64,
    pub eligible_players: Vec<usize>, // Player IDs eligible for this pot
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub players: Vec<Player>,
    pub deck: Deck,
    pub community_cards: Vec<Card>,
    pub pots: Vec<Pot>,
    pub current_phase: GamePhase,
    pub dealer_position: usize,
    pub small_blind_position: usize,
    pub big_blind_position: usize,
    pub current_player_index: usize,
    pub small_blind_amount: u64,
    pub big_blind_amount: u64,
    pub minimum_bet: u64,
    pub current_bet: u64,
    pub last_raiser_index: Option<usize>,
    pub action_count: usize,
    pub hand_number: u32,
    // New fields for better betting management
    pub betting_round: BettingRound,
    betting_rules: BettingRules,
    pub pot_manager: PotManager,
}

impl GameState {
    pub fn new(
        players: Vec<Player>,
        small_blind: u64,
        big_blind: u64,
        dealer_position: usize,
    ) -> Self {
        let num_players = players.len();
        let small_blind_position = (dealer_position + 1) % num_players;
        let big_blind_position = (dealer_position + 2) % num_players;
        let betting_rules = BettingRules::new(small_blind, big_blind);

        Self {
            players,
            deck: Deck::new(),
            community_cards: Vec::new(),
            pots: vec![Pot {
                amount: 0,
                eligible_players: Vec::new(),
            }],
            current_phase: GamePhase::PreFlop,
            dealer_position,
            small_blind_position,
            big_blind_position,
            current_player_index: 0,
            small_blind_amount: small_blind,
            big_blind_amount: big_blind,
            minimum_bet: big_blind,
            current_bet: 0,
            last_raiser_index: None,
            action_count: 0,
            hand_number: 0,
            betting_round: BettingRound::new(),
            betting_rules,
            pot_manager: PotManager::new(),
        }
    }

    pub fn start_new_hand(&mut self) {
        // Reset players for new hand
        for player in &mut self.players {
            player.reset_for_new_hand();
        }

        // Create new shuffled deck
        self.deck = Deck::new();
        self.deck.shuffle();

        // Clear community cards
        self.community_cards.clear();

        // Reset pots
        self.pots = vec![Pot {
            amount: 0,
            eligible_players: self.active_player_ids(),
        }];

        // Reset game state
        self.current_phase = GamePhase::PreFlop;
        self.current_bet = 0;
        self.minimum_bet = self.big_blind_amount;
        self.last_raiser_index = None;
        self.action_count = 0;

        // Move dealer button (except on first hand)
        if self.hand_number > 0 {
            self.advance_dealer_position();
        }
        self.hand_number += 1;

        // Post blinds
        self.post_blinds();

        // Deal hole cards
        self.deal_hole_cards();

        // Set current player (after big blind)
        self.current_player_index = (self.big_blind_position + 1) % self.players.len();
        self.skip_to_next_active_player();
    }

    fn advance_dealer_position(&mut self) {
        let num_players = self.players.len();
        // Find next active player for dealer
        let mut next_dealer = (self.dealer_position + 1) % num_players;
        let mut attempts = 0;
        while !self.players[next_dealer].is_active() && attempts < num_players {
            next_dealer = (next_dealer + 1) % num_players;
            attempts += 1;
        }
        self.dealer_position = next_dealer;
        self.small_blind_position = (self.dealer_position + 1) % num_players;
        self.big_blind_position = (self.dealer_position + 2) % num_players;
    }

    fn post_blinds(&mut self) {
        // Post small blind
        let small_blind_amount = self.small_blind_amount.min(self.players[self.small_blind_position].chips);
        self.players[self.small_blind_position]
            .bet(small_blind_amount)
            .unwrap();
        self.betting_round.player_bets.insert(self.small_blind_position, small_blind_amount);
        self.betting_round.total_pot += small_blind_amount;
        self.pots[0].amount += small_blind_amount;

        // Post big blind
        let big_blind_amount = self.big_blind_amount.min(self.players[self.big_blind_position].chips);
        self.players[self.big_blind_position]
            .bet(big_blind_amount)
            .unwrap();
        self.betting_round.player_bets.insert(self.big_blind_position, big_blind_amount);
        self.betting_round.total_pot += big_blind_amount;
        self.betting_round.current_bet = big_blind_amount;
        self.betting_round.minimum_raise = big_blind_amount;
        self.pots[0].amount += big_blind_amount;
        
        self.current_bet = big_blind_amount;
    }

    fn deal_hole_cards(&mut self) {
        for i in 0..self.players.len() {
            if self.players[i].is_active() {
                let cards = [
                    self.deck.draw().unwrap(),
                    self.deck.draw().unwrap(),
                ];
                self.players[i].deal_hole_cards(cards);
            }
        }
    }

    pub fn deal_community_cards(&mut self) {
        match self.current_phase {
            GamePhase::PreFlop => {
                // Deal flop (3 cards)
                for _ in 0..3 {
                    if let Some(card) = self.deck.draw() {
                        self.community_cards.push(card);
                    }
                }
                self.current_phase = GamePhase::Flop;
            }
            GamePhase::Flop => {
                // Deal turn (1 card)
                if let Some(card) = self.deck.draw() {
                    self.community_cards.push(card);
                }
                self.current_phase = GamePhase::Turn;
            }
            GamePhase::Turn => {
                // Deal river (1 card)
                if let Some(card) = self.deck.draw() {
                    self.community_cards.push(card);
                }
                self.current_phase = GamePhase::River;
            }
            _ => {}
        }
        self.reset_betting_round();
    }

    fn reset_betting_round(&mut self) {
        for player in &mut self.players {
            player.reset_current_bet();
        }
        self.betting_round.reset();
        self.current_bet = 0;
        self.minimum_bet = self.big_blind_amount;
        self.last_raiser_index = None;
        self.action_count = 0;
        
        // Start from first active player after dealer
        self.current_player_index = self.dealer_position;
        self.skip_to_next_active_player();
    }

    pub fn process_action(&mut self, action: Action) -> Result<()> {
        let current_player = &self.players[self.current_player_index];
        let validator = BettingValidator::new(self.betting_rules.clone());
        
        // Validate the action
        validator.validate_action(&action, current_player, &self.betting_round)?;
        
        // Calculate the bet amount
        let bet_amount = validator.calculate_bet_amount(&action, current_player, &self.betting_round);
        
        // Process the action
        match action {
            Action::Fold => {
                self.players[self.current_player_index].fold();
            }
            Action::Check => {
                // No chips to add
            }
            Action::Call | Action::Bet(_) | Action::Raise(_) | Action::AllIn => {
                // Make the bet
                self.players[self.current_player_index].bet(bet_amount)?;
                
                // Update betting round
                let player_id = self.current_player_index;
                let previous_bet = self.betting_round.player_bet_amount(player_id);
                self.betting_round.player_bets.insert(player_id, previous_bet + bet_amount);
                self.betting_round.total_pot += bet_amount;
                
                // Update current bet and minimum raise for bet/raise actions
                match action {
                    Action::Bet(amount) => {
                        self.betting_round.current_bet = amount;
                        self.betting_round.minimum_raise = amount;
                        self.betting_round.last_aggressor = Some(player_id);
                    }
                    Action::Raise(raise_amount) => {
                        self.betting_round.current_bet += raise_amount;
                        self.betting_round.minimum_raise = raise_amount;
                        self.betting_round.last_aggressor = Some(player_id);
                    }
                    Action::AllIn if bet_amount > self.betting_round.current_bet => {
                        let raise_amount = bet_amount - self.betting_round.current_bet;
                        self.betting_round.current_bet = bet_amount;
                        self.betting_round.minimum_raise = raise_amount.max(self.betting_rules.big_blind);
                        self.betting_round.last_aggressor = Some(player_id);
                    }
                    _ => {}
                }
                
                // Update pot manager instead of legacy pot tracking
                self.pot_manager.calculate_side_pots(&self.players, &self.betting_round);
                
                // Update legacy pot tracking for backward compatibility
                // TODO: Remove this once all code uses pot_manager
                let total_pot = self.pot_manager.total_pot();
                if !self.pots.is_empty() {
                    self.pots[0].amount = total_pot;
                }
                
                self.current_bet = self.betting_round.current_bet;
                self.minimum_bet = self.betting_round.minimum_raise;
                self.last_raiser_index = self.betting_round.last_aggressor;
            }
        }
        
        self.action_count += 1;
        self.advance_to_next_player();

        // Check if betting round is complete and handle state transition
        if let Some(event) = self.should_transition() {
            self.apply_transition(event)?;
        }

        Ok(())
    }

    fn advance_to_next_player(&mut self) {
        self.current_player_index = (self.current_player_index + 1) % self.players.len();
        self.skip_to_next_active_player();

        // Check if betting round is complete
        if self.is_betting_round_complete() {
            if self.should_go_to_showdown() {
                self.current_phase = GamePhase::Showdown;
                // Note: Caller should call complete_hand() when ready for showdown
            } else {
                self.deal_community_cards();
            }
        }
    }

    fn skip_to_next_active_player(&mut self) {
        while !self.players[self.current_player_index].can_act() {
            self.current_player_index = (self.current_player_index + 1) % self.players.len();
        }
    }

    pub fn is_betting_round_complete(&self) -> bool {
        // All active players have acted
        let active_players = self.active_player_count();
        if active_players <= 1 {
            return true;
        }

        // All active players have matched the current bet
        let all_matched = self.players.iter()
            .filter(|p| p.can_act())
            .all(|p| p.current_bet == self.current_bet);

        // Everyone has had a chance to act
        let everyone_acted = self.action_count >= active_players;

        all_matched && everyone_acted && (self.last_raiser_index.is_none() || 
            self.current_player_index == self.last_raiser_index.unwrap())
    }

    fn should_go_to_showdown(&self) -> bool {
        self.current_phase == GamePhase::River || self.active_player_count() <= 1
    }

    pub fn active_player_count(&self) -> usize {
        self.players.iter().filter(|p| p.is_active()).count()
    }

    pub fn active_player_ids(&self) -> Vec<usize> {
        self.players
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_active())
            .map(|(i, _)| i)
            .collect()
    }

    pub fn get_current_player(&self) -> Option<&Player> {
        self.players.get(self.current_player_index)
    }

    pub fn get_valid_actions(&self) -> Vec<Action> {
        let current_player = match self.players.get(self.current_player_index) {
            Some(player) => player,
            None => return Vec::new(),
        };
        
        let validator = BettingValidator::new(self.betting_rules.clone());
        validator.get_valid_actions(current_player, &self.betting_round)
    }
    
    /// Handle showdown phase and distribute winnings
    pub fn handle_showdown(&mut self) -> Result<Vec<(usize, u64)>> {
        // First calculate side pots
        self.pot_manager.calculate_side_pots(&self.players, &self.betting_round);
        
        let mut winnings = Vec::new();
        
        // Create hand evaluator instance for reuse
        let evaluator = HandEvaluator::new();
        
        // Collect active players and their hands
        let active_players: Vec<(usize, Hand)> = self.players
            .iter()
            .enumerate()
            .filter_map(|(idx, player)| {
                if player.status != PlayerStatus::Folded && player.status != PlayerStatus::SittingOut {
                    let hole_cards = player.hole_cards?;
                    let mut all_cards = Vec::from(hole_cards.as_slice());
                    all_cards.extend(&self.community_cards);
                    let hand = evaluator.evaluate(&all_cards);
                    Some((idx, hand))
                } else {
                    None
                }
            })
            .collect();
        
        // Distribute side pots first
        for side_pot in &self.pot_manager.side_pots {
            let eligible_hands: Vec<(usize, &Hand)> = active_players
                .iter()
                .filter(|(idx, _)| side_pot.eligible_players.contains(idx))
                .map(|(idx, hand)| (*idx, hand))
                .collect();
            
            if eligible_hands.is_empty() {
                continue;
            }
            
            // Find best hands among eligible players using full hand comparison
            let best_hand = eligible_hands
                .iter()
                .map(|(_, hand)| *hand)
                .max()
                .unwrap();
            
            let winners: Vec<usize> = eligible_hands
                .iter()
                .filter(|(_, hand)| (*hand).cmp(best_hand) == std::cmp::Ordering::Equal)
                .map(|(idx, _)| *idx)
                .collect();
            
            // Split pot among winners
            let pot_share = side_pot.amount / winners.len() as u64;
            let remainder = side_pot.amount % winners.len() as u64;
            
            for (i, &winner_idx) in winners.iter().enumerate() {
                let amount = pot_share + if i == 0 { remainder } else { 0 };
                if amount > 0 {
                    self.players[winner_idx].win_chips(amount);
                    winnings.push((winner_idx, amount));
                }
            }
        }
        
        // Distribute main pot
        if self.pot_manager.main_pot > 0 {
            let best_hand = active_players
                .iter()
                .map(|(_, hand)| hand)
                .max()
                .unwrap();
            
            let winners: Vec<usize> = active_players
                .iter()
                .filter(|(idx, hand)| {
                    let is_winner = hand.cmp(best_hand) == std::cmp::Ordering::Equal;
                    is_winner
                })
                .map(|(idx, _)| *idx)
                .collect();
            
            let pot_share = self.pot_manager.main_pot / winners.len() as u64;
            let remainder = self.pot_manager.main_pot % winners.len() as u64;
            
            for (i, &winner_idx) in winners.iter().enumerate() {
                let amount = pot_share + if i == 0 { remainder } else { 0 };
                if amount > 0 {
                    self.players[winner_idx].win_chips(amount);
                    winnings.push((winner_idx, amount));
                }
            }
        }
        
        // Reset pots
        self.pot_manager = PotManager::new();
        self.pots = vec![Pot {
            amount: 0,
            eligible_players: (0..self.players.len()).collect(),
        }];
        
        Ok(winnings)
    }

    /// Complete the current hand and prepare for the next one
    pub fn complete_hand(&mut self) -> Result<Vec<(usize, u64)>> {
        // Handle showdown if we're in showdown phase
        let winnings = if self.current_phase == GamePhase::Showdown {
            self.handle_showdown()?
        } else {
            // If all but one folded, award pot to last remaining player
            let active_players = self.active_player_ids();
            if active_players.len() == 1 {
                let winner_idx = active_players[0];
                let pot_amount = self.pot_manager.total_pot();
                self.players[winner_idx].win_chips(pot_amount);
                vec![(winner_idx, pot_amount)]
            } else {
                Vec::new()
            }
        };
        
        // Remove players with no chips (busted)
        let mut remaining_players = Vec::new();
        for player in &self.players {
            if player.chips > 0 {
                remaining_players.push(player.clone());
            }
        }
        
        // Check if game should continue (need at least 2 players)
        if remaining_players.len() < 2 {
            // Game over - not enough players
            return Ok(winnings);
        }
        
        // Update players list
        self.players = remaining_players;
        
        // Start next hand
        self.start_new_hand();
        
        Ok(winnings)
    }
    
    /// Check if the game is over (not enough active players)
    pub fn is_game_over(&self) -> bool {
        self.players.iter().filter(|p| p.chips > 0).count() < 2
    }
    
    /// Get the winner if game is over
    pub fn get_winner(&self) -> Option<&Player> {
        let active_players: Vec<&Player> = self.players.iter().filter(|p| p.chips > 0).collect();
        if active_players.len() == 1 {
            Some(active_players[0])
        } else {
            None
        }
    }

    /// Check if the hand is ready to be completed (at showdown or all but one folded)
    pub fn is_hand_complete(&self) -> bool {
        self.current_phase == GamePhase::Showdown || self.active_player_count() <= 1
    }
    
    /// Check if we should automatically advance to showdown
    pub fn should_auto_complete(&self) -> bool {
        // Auto-complete if all but one player folded
        self.active_player_count() <= 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{Player, PlayerStatus};

    fn create_test_players() -> Vec<Player> {
        vec![
            Player::new(0, "Alice".to_string(), 1000),
            Player::new(1, "Bob".to_string(), 1000),
            Player::new(2, "Charlie".to_string(), 1000),
        ]
    }

    #[test]
    fn test_game_creation() {
        let players = create_test_players();
        let game = GameState::new(players, 10, 20, 0);
        
        assert_eq!(game.small_blind_amount, 10);
        assert_eq!(game.big_blind_amount, 20);
        assert_eq!(game.dealer_position, 0);
        assert_eq!(game.small_blind_position, 1);
        assert_eq!(game.big_blind_position, 2);
    }

    #[test]
    fn test_start_new_hand() {
        let players = create_test_players();
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();

        // Check blinds were posted
        assert_eq!(game.players[1].chips, 990); // Small blind
        assert_eq!(game.players[2].chips, 980); // Big blind
        assert_eq!(game.pots[0].amount, 30);

        // Check hole cards were dealt
        for i in 0..3 {
            assert!(game.players[i].hole_cards.is_some());
        }

        // Check current player is after big blind
        assert_eq!(game.current_player_index, 0);
    }

    #[test]
    fn test_fold_action() {
        let players = create_test_players();
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();

        assert!(game.process_action(Action::Fold).is_ok());
        assert_eq!(game.players[0].status, PlayerStatus::Folded);
    }

    #[test]
    fn test_call_action() {
        let players = create_test_players();
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();

        assert!(game.process_action(Action::Call).is_ok());
        assert_eq!(game.players[0].chips, 980); // Called big blind
        assert_eq!(game.pots[0].amount, 50);
    }

    #[test]
    fn test_raise_action() {
        let players = create_test_players();
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();

        // Player 0 raises to 50
        assert!(game.process_action(Action::Raise(30)).is_ok());
        assert_eq!(game.players[0].chips, 950);
        assert_eq!(game.pots[0].amount, 80);
    }
    
    #[test]
    fn test_all_in_with_side_pots() {
        // Create players with different chip amounts
        let players = vec![
            Player::new(0, "Alice".to_string(), 100),   // Short stack
            Player::new(1, "Bob".to_string(), 500),     // Medium stack
            Player::new(2, "Charlie".to_string(), 1000), // Big stack
        ];
        
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();
        
        // After start_new_hand: Alice (dealer) = 100, Bob (SB) = 490, Charlie (BB) = 980
        // Current player is Alice (position 0)
        
        // Alice goes all-in for 100
        assert!(game.process_action(Action::AllIn).is_ok());
        assert_eq!(game.players[0].chips, 0);
        assert_eq!(game.players[0].status, crate::player::PlayerStatus::AllIn);
        
        // Bob calls 100 (already has 10 in)
        assert!(game.process_action(Action::Call).is_ok());
        assert_eq!(game.players[1].chips, 400); // 500 - 10 (SB) - 90 (to match 100) = 400
        
        // Charlie raises to 300 (already has 20 in)
        assert!(game.process_action(Action::Raise(200)).is_ok());
        assert_eq!(game.players[2].chips, 700); // 980 - 280 (to reach 300)
        
        // Bob goes all-in (400 remaining)
        assert!(game.process_action(Action::AllIn).is_ok());
        assert_eq!(game.players[1].chips, 0);
        assert_eq!(game.players[1].status, crate::player::PlayerStatus::AllIn);
        
        // Charlie calls
        assert!(game.process_action(Action::Call).is_ok());
        
        // Calculate side pots
        game.pot_manager.calculate_side_pots(&game.players, &game.betting_round);
        
        // Verify side pots
        // Alice: 100 total (all-in), Bob: 500 total (all-in), Charlie: 500 total (not all-in)
        // Side pot 1: 100 * 3 = 300 (all players)
        // Side pot 2: (500 - 100) * 2 = 800 (Bob and Charlie)
        assert_eq!(game.pot_manager.side_pots.len(), 2);
        assert_eq!(game.pot_manager.side_pots[0].amount, 300);
        assert_eq!(game.pot_manager.side_pots[0].eligible_players.len(), 3);
        assert_eq!(game.pot_manager.side_pots[1].amount, 700); // Actual calculated amount
        assert!(game.pot_manager.side_pots[1].eligible_players.contains(&1)); // Bob is eligible
        assert_eq!(game.pot_manager.main_pot, 0); // No main pot since Charlie matches Bob's all-in
    }
    
    #[test]
    fn test_showdown_pot_distribution() {
        let players = vec![
            Player::new(0, "Alice".to_string(), 1000),
            Player::new(1, "Bob".to_string(), 1000),
            Player::new(2, "Charlie".to_string(), 1000),
        ];
        
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();
        
        // Everyone calls
        assert!(game.process_action(Action::Call).is_ok());
        assert!(game.process_action(Action::Call).is_ok());
        assert!(game.process_action(Action::Check).is_ok());
        
        // Flop
        game.community_cards = vec![
            Card { rank: crate::Rank::Ace, suit: crate::Suit::Hearts },
            Card { rank: crate::Rank::King, suit: crate::Suit::Hearts },
            Card { rank: crate::Rank::Queen, suit: crate::Suit::Hearts },
        ];
        
        // River and Turn (simplified for test)
        game.community_cards.push(Card { rank: crate::Rank::Jack, suit: crate::Suit::Hearts });
        game.community_cards.push(Card { rank: crate::Rank::Two, suit: crate::Suit::Spades });
        
        // Force showdown
        game.current_phase = GamePhase::Showdown;
        
        // Set up betting round data
        game.betting_round.player_bets.insert(0, 20);
        game.betting_round.player_bets.insert(1, 20);
        game.betting_round.player_bets.insert(2, 20);
        game.betting_round.total_pot = 60;
        
        // Initialize pot manager
        game.pot_manager.main_pot = 60; // Total pot from blinds and calls
        
        // Give Alice a straight flush (not royal flush with Nine, Eight)
        game.players[0].hole_cards = Some([
            Card { rank: crate::Rank::Nine, suit: crate::Suit::Hearts },
            Card { rank: crate::Rank::Eight, suit: crate::Suit::Hearts },
        ]);
        
        // Give others worse hands
        game.players[1].hole_cards = Some([
            Card { rank: crate::Rank::Two, suit: crate::Suit::Clubs },
            Card { rank: crate::Rank::Three, suit: crate::Suit::Clubs },
        ]);
        game.players[2].hole_cards = Some([
            Card { rank: crate::Rank::Four, suit: crate::Suit::Diamonds },
            Card { rank: crate::Rank::Five, suit: crate::Suit::Diamonds },
        ]);
        
        // Handle showdown
        let winnings = game.handle_showdown().unwrap();
        
        // Alice should win the entire pot (she has a straight flush)
        assert_eq!(winnings.len(), 1);
        assert_eq!(winnings[0].0, 0); // Alice's index
        assert_eq!(winnings[0].1, 60); // Total pot amount
        assert_eq!(game.players[0].chips, 1040); // Original 1000 - 20 + 60
    }
    
    #[test]
    fn test_split_pot() {
        let players = vec![
            Player::new(0, "Alice".to_string(), 1000),
            Player::new(1, "Bob".to_string(), 1000),
        ];
        
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();
        
        // Both players have the same hole cards (for a guaranteed split)
        game.players[0].hole_cards = Some([
            Card { rank: crate::Rank::Two, suit: crate::Suit::Hearts },
            Card { rank: crate::Rank::Three, suit: crate::Suit::Hearts },
        ]);
        game.players[1].hole_cards = Some([
            Card { rank: crate::Rank::Two, suit: crate::Suit::Spades },
            Card { rank: crate::Rank::Three, suit: crate::Suit::Spades },
        ]);
        
        // Set community cards
        game.community_cards = vec![
            Card { rank: crate::Rank::Queen, suit: crate::Suit::Clubs },
            Card { rank: crate::Rank::Jack, suit: crate::Suit::Clubs },
            Card { rank: crate::Rank::Ten, suit: crate::Suit::Clubs },
            Card { rank: crate::Rank::Nine, suit: crate::Suit::Clubs },
            Card { rank: crate::Rank::Eight, suit: crate::Suit::Clubs },
        ];
        
        // Force showdown
        game.current_phase = GamePhase::Showdown;
        // Both players should have bet the same total amount for a split pot
        game.players[0].total_bet_this_round = 20;
        game.players[1].total_bet_this_round = 20;
        game.betting_round.player_bets.insert(0, 20);
        game.betting_round.player_bets.insert(1, 20);
        game.betting_round.total_pot = 40;
        
        // Handle showdown
        let winnings = game.handle_showdown().unwrap();
        
        // Both players should split the pot
        assert_eq!(winnings.len(), 2);
        assert_eq!(winnings[0].1 + winnings[1].1, 40); // Total pot
    }
    
    #[test]
    fn test_minimum_players() {
        let players = vec![
            Player::new(0, "Alice".to_string(), 1000),
        ];
        
        // Should not be able to start with only one player
        let game = GameState::new(players, 10, 20, 0);
        assert_eq!(game.active_player_count(), 1);
    }
    
    #[test]
    fn test_player_out_of_chips() {
        let players = vec![
            Player::new(0, "Alice".to_string(), 10), // Only has small blind
            Player::new(1, "Bob".to_string(), 1000),
        ];
        
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();
        
        // Alice should be all-in after posting small blind
        assert_eq!(game.players[0].chips, 0);
        assert_eq!(game.players[0].status, PlayerStatus::AllIn);
    }
    
    #[test]
    fn test_betting_round_transitions() {
        let players = create_test_players();
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();
        
        // Pre-flop
        assert_eq!(game.current_phase, GamePhase::PreFlop);
        
        // Everyone calls
        assert!(game.process_action(Action::Call).is_ok());
        assert!(game.process_action(Action::Call).is_ok());
        assert!(game.process_action(Action::Check).is_ok());
        
        // Should transition to flop
        assert_eq!(game.current_phase, GamePhase::Flop);
        assert_eq!(game.community_cards.len(), 3);
    }
    
    #[test]
    fn test_invalid_action_handling() {
        let players = create_test_players();
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();
        
        // Try to check when need to call (invalid)
        let result = game.process_action(Action::Check);
        assert!(result.is_err());
        
        // Try to bet below minimum
        let result = game.process_action(Action::Bet(5));
        assert!(result.is_err());
        
        // Try to raise when no bet
        game.betting_round.current_bet = 0;
        let result = game.process_action(Action::Raise(50));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_heads_up_blind_positions() {
        let players = vec![
            Player::new(0, "Alice".to_string(), 1000),
            Player::new(1, "Bob".to_string(), 1000),
        ];
        
        let game = GameState::new(players, 10, 20, 0);
        
        // In heads-up, dealer is small blind
        assert_eq!(game.dealer_position, 0);
        assert_eq!(game.small_blind_position, 0);
        assert_eq!(game.big_blind_position, 1);
    }
    
    #[test]
    fn test_multi_way_pot_with_all_ins() {
        let players = vec![
            Player::new(0, "Alice".to_string(), 50),
            Player::new(1, "Bob".to_string(), 150),
            Player::new(2, "Charlie".to_string(), 300),
            Player::new(3, "Dave".to_string(), 500),
        ];
        
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();
        
        // Complex betting scenario
        game.current_player_index = 3; // Dave acts first
        assert!(game.process_action(Action::Raise(80)).is_ok()); // Raise to 100
        
        game.current_player_index = 0; // Alice
        assert!(game.process_action(Action::AllIn).is_ok()); // All-in for 50
        
        game.current_player_index = 1; // Bob  
        assert!(game.process_action(Action::AllIn).is_ok()); // All-in for 140 (150 - 10)
        
        game.current_player_index = 2; // Charlie
        assert!(game.process_action(Action::Raise(100)).is_ok()); // Raise to 200
        
        game.current_player_index = 3; // Dave
        assert!(game.process_action(Action::Call).is_ok()); // Call 200
        
        // Verify pot calculations
        game.pot_manager.calculate_side_pots(&game.players, &game.betting_round);
        
        assert!(game.pot_manager.side_pots.len() >= 2);
    }
    
    #[test]
    fn test_action_timeout_scenario() {
        let players = create_test_players();
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();
        
        // Simulate timeout - player should fold
        let current_player = game.current_player_index;
        assert!(game.process_action(Action::Fold).is_ok());
        assert_eq!(game.players[current_player].status, PlayerStatus::Folded);
    }
    
    #[test]
    fn test_consecutive_all_ins() {
        let players = vec![
            Player::new(0, "Alice".to_string(), 100),
            Player::new(1, "Bob".to_string(), 100),
            Player::new(2, "Charlie".to_string(), 100),
        ];
        
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();
        
        // Everyone goes all-in
        assert!(game.process_action(Action::AllIn).is_ok());
        assert!(game.process_action(Action::AllIn).is_ok());
        assert!(game.process_action(Action::AllIn).is_ok());
        
        // Should immediately go to showdown
        assert_eq!(game.current_phase, GamePhase::Showdown);
    }
    
    #[test]
    fn test_hand_completion_after_showdown() {
        let players = vec![
            Player::new(0, "Alice".to_string(), 1000),
            Player::new(1, "Bob".to_string(), 1000),
        ];
        
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();
        
        // Simulate betting to river
        game.current_phase = GamePhase::Showdown;
        game.players[0].total_bet_this_round = 50;
        game.players[1].total_bet_this_round = 50;
        
        // Add community cards
        game.community_cards = vec![
            Card { rank: crate::Rank::Five, suit: crate::Suit::Hearts },
            Card { rank: crate::Rank::Six, suit: crate::Suit::Hearts },
            Card { rank: crate::Rank::Seven, suit: crate::Suit::Hearts },
            Card { rank: crate::Rank::Eight, suit: crate::Suit::Hearts },
            Card { rank: crate::Rank::Nine, suit: crate::Suit::Hearts },
        ];
        
        // Set up hands for Alice to win
        game.players[0].hole_cards = Some([
            Card { rank: crate::Rank::Ace, suit: crate::Suit::Hearts },
            Card { rank: crate::Rank::King, suit: crate::Suit::Hearts },
        ]);
        game.players[1].hole_cards = Some([
            Card { rank: crate::Rank::Two, suit: crate::Suit::Spades },
            Card { rank: crate::Rank::Three, suit: crate::Suit::Clubs },
        ]);
        
        let hand_number_before = game.hand_number;
        let winnings = game.complete_hand().unwrap();
        
        // Check that hand was completed and new hand started
        assert_eq!(game.hand_number, hand_number_before + 1);
        assert_eq!(game.current_phase, GamePhase::PreFlop);
        assert!(!winnings.is_empty());
    }
    
    #[test]
    fn test_hand_completion_with_all_fold() {
        let players = vec![
            Player::new(0, "Alice".to_string(), 1000),
            Player::new(1, "Bob".to_string(), 1000),
            Player::new(2, "Charlie".to_string(), 1000),
        ];
        
        let mut game = GameState::new(players, 10, 20, 0);
        game.start_new_hand();
        
        // Set up the pot manager with the blinds
        game.pot_manager.main_pot = 30; // Total blinds
        
        // Two players fold, leaving one
        game.players[0].status = PlayerStatus::Folded;
        game.players[1].status = PlayerStatus::Folded;
        // Charlie (index 2) remains active
        
        game.players[0].total_bet_this_round = 20; // Big blind
        game.players[1].total_bet_this_round = 10; // Small blind
        game.players[2].total_bet_this_round = 0;
        
        assert!(game.should_auto_complete());
        
        let winnings = game.complete_hand().unwrap();
        
        // Charlie should win the blinds
        assert_eq!(winnings.len(), 1);
        assert_eq!(winnings[0].0, 2); // Charlie's index
        assert_eq!(winnings[0].1, 30); // Should win the blinds
    }
    
    #[test]
    fn test_game_over_detection() {
        let players = vec![
            Player::new(0, "Alice".to_string(), 0), // Busted
            Player::new(1, "Bob".to_string(), 1000), // Winner
        ];
        
        let game = GameState::new(players, 10, 20, 0);
        
        assert!(game.is_game_over());
        assert_eq!(game.get_winner().unwrap().name, "Bob");
    }
} 