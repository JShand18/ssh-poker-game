use poker_engine::{Action, GameState, Card, Rank, Hand};
use rand::{Rng, thread_rng};
use log::{debug, info};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use async_trait::async_trait;

pub mod strategy;
pub mod evaluator;
pub mod personality;

pub use strategy::{PokerStrategy, TightStrategy, LooseStrategy, AggressiveStrategy, ConservativeStrategy};
pub use evaluator::{HandStrengthEvaluator, Position, BettingHistory};
pub use personality::{BotPersonality, DifficultyLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BotDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub difficulty: BotDifficulty,
    pub personality: BotPersonality,
    pub aggression_level: f64,  // 0.0 to 1.0
    pub bluff_frequency: f64,   // 0.0 to 1.0
    pub risk_tolerance: f64,    // 0.0 to 1.0
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            difficulty: BotDifficulty::Intermediate,
            personality: BotPersonality::Balanced,
            aggression_level: 0.5,
            bluff_frequency: 0.1,
            risk_tolerance: 0.5,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BotError {
    #[error("Invalid game state")]
    InvalidGameState,
    #[error("No valid actions available")]
    NoValidActions,
    #[error("Strategy error: {0}")]
    StrategyError(String),
    #[error("Hand evaluation error: {0}")]
    HandEvaluationError(String),
}

pub type Result<T> = std::result::Result<T, BotError>;

#[async_trait]
pub trait PokerBot: Send + Sync {
    async fn decide_action(&mut self, game_state: &GameState, player_id: usize) -> Result<Action>;
    fn get_difficulty(&self) -> BotDifficulty;
    fn get_name(&self) -> &str;
    fn update_from_action(&mut self, player_id: usize, action: &Action);
    fn reset_for_new_hand(&mut self);
}

pub struct AIBot {
    name: String,
    config: BotConfig,
    strategy: Box<dyn PokerStrategy>,
    hand_evaluator: HandStrengthEvaluator,
    betting_history: BettingHistory,
    opponent_models: HashMap<usize, OpponentModel>,
}

#[derive(Debug, Clone)]
struct OpponentModel {
    aggression_factor: f64,
    tightness_factor: f64,
    bluff_frequency: f64,
    hands_played: u32,
    total_actions: u32,
    fold_frequency: f64,
    raise_frequency: f64,
}

impl Default for OpponentModel {
    fn default() -> Self {
        Self {
            aggression_factor: 0.5,
            tightness_factor: 0.5,
            bluff_frequency: 0.1,
            hands_played: 0,
            total_actions: 0,
            fold_frequency: 0.3,
            raise_frequency: 0.2,
        }
    }
}

impl AIBot {
    pub fn new(name: String, config: BotConfig) -> Self {
        let strategy: Box<dyn PokerStrategy> = match config.personality {
            BotPersonality::Tight => Box::new(TightStrategy::new()),
            BotPersonality::Loose => Box::new(LooseStrategy::new()),
            BotPersonality::Aggressive => Box::new(AggressiveStrategy::new()),
            BotPersonality::Conservative => Box::new(ConservativeStrategy::new()),
            BotPersonality::Balanced => Box::new(strategy::BalancedStrategy::new()),
        };

        Self {
            name,
            config,
            strategy,
            hand_evaluator: HandStrengthEvaluator::new(),
            betting_history: BettingHistory::new(),
            opponent_models: HashMap::new(),
        }
    }

    pub fn with_difficulty(name: String, difficulty: BotDifficulty) -> Self {
        let config = match difficulty {
            BotDifficulty::Beginner => BotConfig {
                difficulty,
                personality: BotPersonality::Conservative,
                aggression_level: 0.2,
                bluff_frequency: 0.05,
                risk_tolerance: 0.3,
            },
            BotDifficulty::Intermediate => BotConfig {
                difficulty,
                personality: BotPersonality::Balanced,
                aggression_level: 0.5,
                bluff_frequency: 0.1,
                risk_tolerance: 0.5,
            },
            BotDifficulty::Advanced => BotConfig {
                difficulty,
                personality: BotPersonality::Aggressive,
                aggression_level: 0.7,
                bluff_frequency: 0.15,
                risk_tolerance: 0.7,
            },
            BotDifficulty::Expert => BotConfig {
                difficulty,
                personality: BotPersonality::Tight,
                aggression_level: 0.8,
                bluff_frequency: 0.2,
                risk_tolerance: 0.6,
            },
        };

        Self::new(name, config)
    }

    fn evaluate_hand_strength(&self, hole_cards: &[Card; 2], community_cards: &[Card]) -> Result<f64> {
        let mut all_cards = hole_cards.to_vec();
        all_cards.extend_from_slice(community_cards);
        
        if all_cards.len() < 2 {
            return Err(BotError::HandEvaluationError("Not enough cards".to_string()));
        }

        // Calculate hand strength based on current cards
        let hand_strength = match community_cards.len() {
            0 => self.evaluate_preflop_strength(hole_cards),
            3 => self.evaluate_flop_strength(hole_cards, community_cards),
            4 => self.evaluate_turn_strength(hole_cards, community_cards),
            5 => self.evaluate_river_strength(hole_cards, community_cards),
            _ => 0.5, // Default middle strength
        };

        Ok(hand_strength)
    }

    fn evaluate_preflop_strength(&self, hole_cards: &[Card; 2]) -> f64 {
        let card1 = &hole_cards[0];
        let card2 = &hole_cards[1];
        
        // Basic hand strength evaluation
        let is_pair = card1.rank == card2.rank;
        let is_suited = card1.suit == card2.suit;
        let is_connected = ((card1.rank as u8).abs_diff(card2.rank as u8)) <= 1;
        
        let high_card_bonus = match (card1.rank, card2.rank) {
            (Rank::Ace, _) | (_, Rank::Ace) => 0.3,
            (Rank::King, _) | (_, Rank::King) => 0.2,
            (Rank::Queen, _) | (_, Rank::Queen) => 0.15,
            (Rank::Jack, _) | (_, Rank::Jack) => 0.1,
            _ => 0.0,
        };

        let mut strength: f64 = 0.3; // Base strength

        if is_pair {
            strength += match card1.rank {
                Rank::Ace | Rank::King | Rank::Queen => 0.4,
                Rank::Jack | Rank::Ten | Rank::Nine => 0.3,
                _ => 0.2,
            };
        }

        if is_suited {
            strength += 0.1;
        }

        if is_connected {
            strength += 0.05;
        }

        strength += high_card_bonus;
        strength.min(1.0).max(0.0)
    }

    fn evaluate_flop_strength(&self, hole_cards: &[Card; 2], community_cards: &[Card]) -> f64 {
        let mut all_cards = hole_cards.to_vec();
        all_cards.extend_from_slice(community_cards);
        let hand = self.hand_evaluator.evaluate_hand(&all_cards);
        
        // Convert hand rank to strength (higher rank = higher strength)
        self.hand_rank_to_strength(&hand)
    }

    fn evaluate_turn_strength(&self, hole_cards: &[Card; 2], community_cards: &[Card]) -> f64 {
        let mut all_cards = hole_cards.to_vec();
        all_cards.extend_from_slice(community_cards);
        let hand = self.hand_evaluator.evaluate_hand(&all_cards);
        
        self.hand_rank_to_strength(&hand)
    }

    fn evaluate_river_strength(&self, hole_cards: &[Card; 2], community_cards: &[Card]) -> f64 {
        let mut all_cards = hole_cards.to_vec();
        all_cards.extend_from_slice(community_cards);
        let hand = self.hand_evaluator.evaluate_hand(&all_cards);
        
        self.hand_rank_to_strength(&hand)
    }

    fn hand_rank_to_strength(&self, hand: &Hand) -> f64 {
        match hand.rank() {
            poker_engine::HandRank::HighCard => 0.1,
            poker_engine::HandRank::OnePair => 0.2,
            poker_engine::HandRank::TwoPair => 0.4,
            poker_engine::HandRank::ThreeOfAKind => 0.6,
            poker_engine::HandRank::Straight => 0.7,
            poker_engine::HandRank::Flush => 0.8,
            poker_engine::HandRank::FullHouse => 0.9,
            poker_engine::HandRank::FourOfAKind => 0.95,
            poker_engine::HandRank::StraightFlush => 1.0,
        }
    }

    fn calculate_pot_odds(&self, game_state: &GameState, call_amount: u64) -> f64 {
        let pot_size = game_state.pot_manager.total_pot();
        if call_amount == 0 {
            return 1.0; // No cost to call
        }
        pot_size as f64 / (pot_size + call_amount) as f64
    }

    fn should_bluff(&self, hand_strength: f64) -> bool {
        if hand_strength > 0.5 {
            return false; // No need to bluff with a good hand
        }

        let mut rng = thread_rng();
        let bluff_chance = self.config.bluff_frequency * (1.0 - hand_strength);
        rng.gen::<f64>() < bluff_chance
    }

    fn update_opponent_model(&mut self, player_id: usize, action: &Action) {
        let model = self.opponent_models.entry(player_id).or_default();
        model.total_actions += 1;

        match action {
            Action::Fold => {
                model.fold_frequency = (model.fold_frequency * (model.total_actions - 1) as f64 + 1.0) / model.total_actions as f64;
                // Folding more indicates tighter play
                model.tightness_factor = (model.tightness_factor * 0.9 + 0.1).min(1.0);
            }
            Action::Raise(_) | Action::Bet(_) => {
                model.raise_frequency = (model.raise_frequency * (model.total_actions - 1) as f64 + 1.0) / model.total_actions as f64;
                model.aggression_factor = (model.aggression_factor * 0.9 + 0.1).min(1.0);
                // Betting/raising indicates looser play
                model.tightness_factor = (model.tightness_factor * 0.95).max(0.0);
                // Update bluff frequency estimate
                model.bluff_frequency = (model.bluff_frequency * 0.95 + 0.05 * 0.15).min(0.4);
            }
            Action::Call => {
                // Calling suggests moderate play
                model.tightness_factor = (model.tightness_factor * 0.98 + 0.02 * 0.5).clamp(0.0, 1.0);
            }
            Action::Check => {
                // Checking suggests passive play
                model.aggression_factor = (model.aggression_factor * 0.95).max(0.0);
            }
            Action::AllIn => {
                model.aggression_factor = (model.aggression_factor * 0.8 + 0.2).min(1.0);
                model.raise_frequency = (model.raise_frequency * 0.9 + 0.1).min(1.0);
                // All-in could be very tight (nuts) or very loose (bluff)
                model.bluff_frequency = (model.bluff_frequency * 0.9 + 0.1 * 0.25).min(0.5);
            }
        }
    }
}

#[async_trait]
impl PokerBot for AIBot {
    async fn decide_action(&mut self, game_state: &GameState, player_id: usize) -> Result<Action> {
        debug!("AI Bot {} deciding action for player {}", self.name, player_id);

        // Get current player
        let player = game_state.players.get(player_id)
            .ok_or(BotError::InvalidGameState)?;

        // Get hole cards
        let hole_cards = player.hole_cards
            .ok_or(BotError::InvalidGameState)?;

        // Evaluate hand strength
        let hand_strength = self.evaluate_hand_strength(&hole_cards, &game_state.community_cards)?;
        debug!("Hand strength evaluated as: {:.2}", hand_strength);

        // Get valid actions
        let valid_actions = game_state.get_valid_actions();
        if valid_actions.is_empty() {
            return Err(BotError::NoValidActions);
        }

        // Calculate pot odds if calling is an option
        let pot_odds = if valid_actions.contains(&Action::Call) {
            self.calculate_pot_odds(game_state, game_state.current_bet - player.current_bet)
        } else {
            0.0
        };

        // Determine position (early, middle, late)
        let position = self.determine_position(game_state, player_id);

        // Calculate if we should bluff
        let should_bluff = self.should_bluff(hand_strength);
        
        // Get average opponent stats
        let (avg_aggression, avg_tightness) = if self.opponent_models.is_empty() {
            (None, None)
        } else {
            let total_models = self.opponent_models.len() as f64;
            let sum_aggression: f64 = self.opponent_models.values()
                .map(|m| m.aggression_factor)
                .sum();
            let sum_tightness: f64 = self.opponent_models.values()
                .map(|m| m.tightness_factor)
                .sum();
            (Some(sum_aggression / total_models), Some(sum_tightness / total_models))
        };

        // Use strategy to decide action
        let mut decision_context = strategy::DecisionContext {
            hand_strength,
            pot_odds,
            position,
            aggression_level: self.config.aggression_level,
            risk_tolerance: self.config.risk_tolerance,
            valid_actions: valid_actions.clone(),
            game_phase: game_state.current_phase,
            opponents_count: game_state.active_player_count() - 1,
            pot_size: game_state.pot_manager.total_pot(),
            current_bet: game_state.current_bet,
            player_chips: player.chips,
            should_bluff,
            opponent_aggression: avg_aggression,
            opponent_tightness: avg_tightness,
        };

        let action = self.strategy.decide_action(&mut decision_context)?;

        // Apply difficulty-based modifications
        let final_action = self.apply_difficulty_modifications(action, hand_strength, &valid_actions);

        info!("AI Bot {} chose action: {:?}", self.name, final_action);
        Ok(final_action)
    }

    fn get_difficulty(&self) -> BotDifficulty {
        self.config.difficulty
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn update_from_action(&mut self, player_id: usize, action: &Action) {
        self.update_opponent_model(player_id, action);
        self.betting_history.add_action(player_id, action.clone());
    }

    fn reset_for_new_hand(&mut self) {
        self.betting_history.reset();
        for model in self.opponent_models.values_mut() {
            model.hands_played += 1;
        }
    }
}

impl AIBot {
    fn determine_position(&self, game_state: &GameState, player_id: usize) -> Position {
        let total_players = game_state.players.len();
        let dealer_pos = game_state.dealer_position;
        
        let position_index = (player_id + total_players - dealer_pos - 1) % total_players;
        
        match position_index {
            0..=2 => Position::Early,
            _ if position_index >= total_players - 2 => Position::Late,
            _ => Position::Middle,
        }
    }

    fn apply_difficulty_modifications(&self, action: Action, hand_strength: f64, valid_actions: &[Action]) -> Action {
        let mut rng = thread_rng();

        match self.config.difficulty {
            BotDifficulty::Beginner => {
                // Beginners make suboptimal plays 30% of the time
                if rng.gen::<f64>() < 0.3 {
                    self.make_suboptimal_play(hand_strength, valid_actions)
                } else {
                    action
                }
            }
            BotDifficulty::Intermediate => {
                // Intermediate players make minor mistakes 15% of the time
                if rng.gen::<f64>() < 0.15 {
                    self.make_minor_mistake(action, valid_actions)
                } else {
                    action
                }
            }
            BotDifficulty::Advanced | BotDifficulty::Expert => {
                // Advanced and expert players play optimally
                action
            }
        }
    }

    fn make_suboptimal_play(&self, hand_strength: f64, valid_actions: &[Action]) -> Action {
        let mut rng = thread_rng();
        
        // Beginners tend to call too much and fold too little
        if hand_strength < 0.3 && valid_actions.contains(&Action::Call) {
            if rng.gen::<f64>() < 0.4 {
                return Action::Call; // Should probably fold but calls instead
            }
        }
        
        // Or they might fold decent hands
        if hand_strength > 0.6 && valid_actions.contains(&Action::Fold) {
            if rng.gen::<f64>() < 0.2 {
                return Action::Fold; // Should play but folds instead
            }
        }
        
        // Default to a random valid action
        valid_actions[rng.gen_range(0..valid_actions.len())].clone()
    }

    fn make_minor_mistake(&self, optimal_action: Action, valid_actions: &[Action]) -> Action {
        let mut rng = thread_rng();
        
        match optimal_action {
            Action::Raise(amount) => {
                // Sometimes bet instead of raise, or raise less
                if valid_actions.contains(&Action::Bet(amount / 2)) {
                    Action::Bet(amount / 2)
                } else if valid_actions.contains(&Action::Call) {
                    Action::Call
                } else {
                    optimal_action
                }
            }
            Action::Bet(_amount) => {
                // Sometimes check instead of bet
                if valid_actions.contains(&Action::Check) && rng.gen::<f64>() < 0.3 {
                    Action::Check
                } else {
                    optimal_action
                }
            }
            _ => optimal_action,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use poker_engine::{Player, GameState, Suit};

    fn create_test_game_state() -> GameState {
        let players = vec![
            Player::new(0, "Human".to_string(), 1000),
            Player::new(1, "AI".to_string(), 1000),
        ];
        GameState::new(players, 10, 20, 0)
    }

    #[tokio::test]
    async fn test_ai_bot_creation() {
        let bot = AIBot::with_difficulty("TestBot".to_string(), BotDifficulty::Intermediate);
        assert_eq!(bot.get_name(), "TestBot");
        assert_eq!(bot.get_difficulty(), BotDifficulty::Intermediate);
    }

    #[tokio::test]
    async fn test_hand_strength_evaluation() {
        let bot = AIBot::with_difficulty("TestBot".to_string(), BotDifficulty::Expert);
        
        // Test pocket aces
        let hole_cards = [
            Card { rank: Rank::Ace, suit: Suit::Spades },
            Card { rank: Rank::Ace, suit: Suit::Hearts },
        ];
        let community_cards = vec![];
        
        let strength = bot.evaluate_hand_strength(&hole_cards, &community_cards).unwrap();
        assert!(strength > 0.7, "Pocket aces should have high strength");
    }

    #[tokio::test]
    async fn test_decision_making() {
        let mut bot = AIBot::with_difficulty("TestBot".to_string(), BotDifficulty::Expert);
        let mut game_state = create_test_game_state();
        
        // Give the bot some cards
        let hole_cards = [
            Card { rank: Rank::King, suit: Suit::Spades },
            Card { rank: Rank::Queen, suit: Suit::Hearts },
        ];
        game_state.players[1].deal_hole_cards(hole_cards);
        
        game_state.start_new_hand();
        
        let action = bot.decide_action(&game_state, 1).await;
        assert!(action.is_ok(), "Bot should be able to decide an action");
    }

    #[test]
    fn test_pot_odds_calculation() {
        let bot = AIBot::with_difficulty("TestBot".to_string(), BotDifficulty::Expert);
        let game_state = create_test_game_state();
        
        let pot_odds = bot.calculate_pot_odds(&game_state, 50);
        assert!(pot_odds >= 0.0 && pot_odds <= 1.0, "Pot odds should be between 0 and 1");
    }

    #[test]
    fn test_opponent_modeling() {
        let mut bot = AIBot::with_difficulty("TestBot".to_string(), BotDifficulty::Expert);
        
        // Simulate an aggressive opponent
        bot.update_from_action(0, &Action::Raise(100));
        bot.update_from_action(0, &Action::Bet(50));
        bot.update_from_action(0, &Action::AllIn);
        
        let model = bot.opponent_models.get(&0).unwrap();
        assert!(model.aggression_factor > 0.5, "Opponent should be modeled as aggressive");
    }
}