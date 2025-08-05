use poker_engine::{Action, GamePhase};
use crate::{BotError, Result};
use crate::evaluator::Position;
use async_trait::async_trait;
use rand::{Rng, thread_rng};

#[derive(Debug, Clone)]
pub struct DecisionContext {
    pub hand_strength: f64,
    pub pot_odds: f64,
    pub position: Position,
    pub aggression_level: f64,
    pub risk_tolerance: f64,
    pub valid_actions: Vec<Action>,
    pub game_phase: GamePhase,
    pub opponents_count: usize,
    pub pot_size: u64,
    pub current_bet: u64,
    pub player_chips: u64,
}

#[async_trait]
pub trait PokerStrategy: Send + Sync {
    fn decide_action(&mut self, context: &mut DecisionContext) -> Result<Action>;
    fn get_strategy_name(&self) -> &'static str;
}

pub struct TightStrategy {
    name: &'static str,
}

impl TightStrategy {
    pub fn new() -> Self {
        Self {
            name: "Tight",
        }
    }
}

#[async_trait]
impl PokerStrategy for TightStrategy {
    fn decide_action(&mut self, context: &mut DecisionContext) -> Result<Action> {
        // Tight strategy: only play strong hands, fold weak hands
        if context.hand_strength < 0.4 {
            if context.valid_actions.contains(&Action::Fold) {
                return Ok(Action::Fold);
            } else if context.valid_actions.contains(&Action::Check) {
                return Ok(Action::Check);
            }
        }

        if context.hand_strength > 0.7 {
            // Strong hand - bet or raise
            if context.valid_actions.contains(&Action::Raise(50)) {
                let raise_amount = (context.pot_size / 2).max(50).min(context.player_chips);
                return Ok(Action::Raise(raise_amount));
            } else if context.valid_actions.contains(&Action::Bet(50)) {
                let bet_amount = (context.pot_size / 3).max(50).min(context.player_chips);
                return Ok(Action::Bet(bet_amount));
            }
        }

        // Moderate hand - call or check
        if context.valid_actions.contains(&Action::Call) {
            Ok(Action::Call)
        } else if context.valid_actions.contains(&Action::Check) {
            Ok(Action::Check)
        } else {
            Ok(context.valid_actions[0].clone())
        }
    }

    fn get_strategy_name(&self) -> &'static str {
        self.name
    }
}

pub struct LooseStrategy {
    name: &'static str,
}

impl LooseStrategy {
    pub fn new() -> Self {
        Self {
            name: "Loose",
        }
    }
}

#[async_trait]
impl PokerStrategy for LooseStrategy {
    fn decide_action(&mut self, context: &mut DecisionContext) -> Result<Action> {
        // Loose strategy: play many hands, rarely fold
        if context.hand_strength < 0.15 && context.current_bet > context.pot_size / 4 {
            // Only fold very weak hands when facing large bets
            if context.valid_actions.contains(&Action::Fold) {
                return Ok(Action::Fold);
            }
        }

        if context.hand_strength > 0.5 {
            // Decent hand - be aggressive
            if context.valid_actions.contains(&Action::Raise(50)) {
                let raise_amount = (context.pot_size / 2).max(50).min(context.player_chips);
                return Ok(Action::Raise(raise_amount));
            } else if context.valid_actions.contains(&Action::Bet(50)) {
                let bet_amount = (context.pot_size / 3).max(50).min(context.player_chips);
                return Ok(Action::Bet(bet_amount));
            }
        }

        // Default to calling or checking
        if context.valid_actions.contains(&Action::Call) {
            Ok(Action::Call)
        } else if context.valid_actions.contains(&Action::Check) {
            Ok(Action::Check)
        } else {
            Ok(context.valid_actions[0].clone())
        }
    }

    fn get_strategy_name(&self) -> &'static str {
        self.name
    }
}

pub struct AggressiveStrategy {
    name: &'static str,
}

impl AggressiveStrategy {
    pub fn new() -> Self {
        Self {
            name: "Aggressive",
        }
    }
}

#[async_trait]
impl PokerStrategy for AggressiveStrategy {
    fn decide_action(&mut self, context: &mut DecisionContext) -> Result<Action> {
        let mut rng = thread_rng();
        
        // Aggressive strategy: bet and raise frequently
        if context.hand_strength > 0.3 || rng.gen::<f64>() < 0.3 {
            // Bet or raise with decent hands or as a bluff
            if context.valid_actions.contains(&Action::Raise(50)) {
                let raise_amount = (context.pot_size * 2 / 3).max(100).min(context.player_chips);
                return Ok(Action::Raise(raise_amount));
            } else if context.valid_actions.contains(&Action::Bet(50)) {
                let bet_amount = (context.pot_size / 2).max(75).min(context.player_chips);
                return Ok(Action::Bet(bet_amount));
            }
        }

        if context.hand_strength < 0.2 && context.current_bet > context.pot_size / 2 {
            // Fold very weak hands against large bets
            if context.valid_actions.contains(&Action::Fold) {
                return Ok(Action::Fold);
            }
        }

        // Call or check otherwise
        if context.valid_actions.contains(&Action::Call) {
            Ok(Action::Call)
        } else if context.valid_actions.contains(&Action::Check) {
            Ok(Action::Check)
        } else {
            Ok(context.valid_actions[0].clone())
        }
    }

    fn get_strategy_name(&self) -> &'static str {
        self.name
    }
}

pub struct ConservativeStrategy {
    name: &'static str,
}

impl ConservativeStrategy {
    pub fn new() -> Self {
        Self {
            name: "Conservative",
        }
    }
}

#[async_trait]
impl PokerStrategy for ConservativeStrategy {
    fn decide_action(&mut self, context: &mut DecisionContext) -> Result<Action> {
        // Conservative strategy: avoid big risks, play safely
        if context.hand_strength < 0.5 {
            if context.current_bet > context.pot_size / 3 {
                // Fold weak hands against moderate bets
                if context.valid_actions.contains(&Action::Fold) {
                    return Ok(Action::Fold);
                }
            }
            // Check or call small bets
            if context.valid_actions.contains(&Action::Check) {
                return Ok(Action::Check);
            } else if context.valid_actions.contains(&Action::Call) && context.current_bet <= context.pot_size / 4 {
                return Ok(Action::Call);
            }
        }

        if context.hand_strength > 0.8 {
            // Only bet with very strong hands
            if context.valid_actions.contains(&Action::Bet(50)) {
                let bet_amount = (context.pot_size / 4).max(25).min(context.player_chips);
                return Ok(Action::Bet(bet_amount));
            } else if context.valid_actions.contains(&Action::Raise(50)) {
                let raise_amount = (context.pot_size / 3).max(50).min(context.player_chips);
                return Ok(Action::Raise(raise_amount));
            }
        }

        // Default to passive play
        if context.valid_actions.contains(&Action::Check) {
            Ok(Action::Check)
        } else if context.valid_actions.contains(&Action::Call) {
            Ok(Action::Call)
        } else {
            Ok(Action::Fold)
        }
    }

    fn get_strategy_name(&self) -> &'static str {
        self.name
    }
}

pub struct BalancedStrategy {
    name: &'static str,
}

impl BalancedStrategy {
    pub fn new() -> Self {
        Self {
            name: "Balanced",
        }
    }
}

#[async_trait]
impl PokerStrategy for BalancedStrategy {
    fn decide_action(&mut self, context: &mut DecisionContext) -> Result<Action> {
        let mut rng = thread_rng();
        
        // Position-based adjustments
        let position_modifier = match context.position {
            Position::Early => -0.1,   // Play tighter in early position
            Position::Middle => 0.0,   // Neutral
            Position::Late => 0.1,     // Play looser in late position
        };

        let adjusted_hand_strength = (context.hand_strength + position_modifier).max(0.0).min(1.0);

        // Phase-based strategy
        match context.game_phase {
            GamePhase::PreFlop => self.preflop_strategy(context, adjusted_hand_strength),
            GamePhase::Flop => self.postflop_strategy(context, adjusted_hand_strength),
            GamePhase::Turn => self.turn_strategy(context, adjusted_hand_strength),
            GamePhase::River => self.river_strategy(context, adjusted_hand_strength),
            GamePhase::Showdown => {
                // Should not be making decisions in showdown
                if context.valid_actions.contains(&Action::Check) {
                    Ok(Action::Check)
                } else {
                    Ok(context.valid_actions[0].clone())
                }
            }
        }
    }

    fn get_strategy_name(&self) -> &'static str {
        self.name
    }
}

impl BalancedStrategy {
    fn preflop_strategy(&self, context: &DecisionContext, hand_strength: f64) -> Result<Action> {
        if hand_strength < 0.3 {
            // Weak hands - fold or check
            if context.current_bet > 0 && context.valid_actions.contains(&Action::Fold) {
                return Ok(Action::Fold);
            } else if context.valid_actions.contains(&Action::Check) {
                return Ok(Action::Check);
            }
        }

        if hand_strength > 0.7 {
            // Strong hands - raise for value
            if context.valid_actions.contains(&Action::Raise(50)) {
                let raise_amount = (context.pot_size / 2).max(50).min(context.player_chips / 10);
                return Ok(Action::Raise(raise_amount));
            } else if context.valid_actions.contains(&Action::Bet(50)) {
                let bet_amount = (context.pot_size / 3).max(50).min(context.player_chips / 10);
                return Ok(Action::Bet(bet_amount));
            }
        }

        // Medium hands - call or check
        if context.valid_actions.contains(&Action::Call) {
            Ok(Action::Call)
        } else if context.valid_actions.contains(&Action::Check) {
            Ok(Action::Check)
        } else {
            Ok(context.valid_actions[0].clone())
        }
    }

    fn postflop_strategy(&self, context: &DecisionContext, hand_strength: f64) -> Result<Action> {
        let mut rng = thread_rng();
        
        if hand_strength > 0.6 {
            // Good hands - bet for value
            if context.valid_actions.contains(&Action::Bet(50)) {
                let bet_amount = (context.pot_size * 2 / 3).max(50).min(context.player_chips / 5);
                return Ok(Action::Bet(bet_amount));
            } else if context.valid_actions.contains(&Action::Raise(50)) {
                let raise_amount = (context.pot_size / 2).max(50).min(context.player_chips / 5);
                return Ok(Action::Raise(raise_amount));
            }
        }

        if hand_strength < 0.3 {
            // Weak hands - check or fold
            if context.current_bet > context.pot_size / 3 && context.valid_actions.contains(&Action::Fold) {
                return Ok(Action::Fold);
            } else if context.valid_actions.contains(&Action::Check) {
                return Ok(Action::Check);
            }
        }

        // Bluff occasionally with weak hands
        if hand_strength < 0.4 && rng.gen::<f64>() < 0.15 {
            if context.valid_actions.contains(&Action::Bet(50)) {
                let bluff_amount = (context.pot_size / 2).max(25).min(context.player_chips / 10);
                return Ok(Action::Bet(bluff_amount));
            }
        }

        // Default to call or check
        if context.valid_actions.contains(&Action::Call) {
            Ok(Action::Call)
        } else if context.valid_actions.contains(&Action::Check) {
            Ok(Action::Check)
        } else {
            Ok(context.valid_actions[0].clone())
        }
    }

    fn turn_strategy(&self, context: &DecisionContext, hand_strength: f64) -> Result<Action> {
        // Similar to flop but more cautious
        if hand_strength > 0.7 {
            if context.valid_actions.contains(&Action::Bet(50)) {
                let bet_amount = (context.pot_size / 2).max(50).min(context.player_chips / 4);
                return Ok(Action::Bet(bet_amount));
            }
        }

        if hand_strength < 0.35 && context.current_bet > context.pot_size / 4 {
            if context.valid_actions.contains(&Action::Fold) {
                return Ok(Action::Fold);
            }
        }

        if context.valid_actions.contains(&Action::Call) {
            Ok(Action::Call)
        } else if context.valid_actions.contains(&Action::Check) {
            Ok(Action::Check)
        } else {
            Ok(context.valid_actions[0].clone())
        }
    }

    fn river_strategy(&self, context: &DecisionContext, hand_strength: f64) -> Result<Action> {
        // River play - more straightforward
        if hand_strength > 0.75 {
            // Strong hands - bet for value
            if context.valid_actions.contains(&Action::Bet(50)) {
                let bet_amount = (context.pot_size * 2 / 3).max(50).min(context.player_chips / 3);
                return Ok(Action::Bet(bet_amount));
            }
        }

        if hand_strength < 0.4 {
            // Weak hands - usually fold to bets
            if context.current_bet > 0 && context.valid_actions.contains(&Action::Fold) {
                return Ok(Action::Fold);
            }
        }

        // Check or call with medium hands
        if context.valid_actions.contains(&Action::Check) {
            Ok(Action::Check)
        } else if context.valid_actions.contains(&Action::Call) {
            Ok(Action::Call)
        } else {
            Ok(context.valid_actions[0].clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> DecisionContext {
        DecisionContext {
            hand_strength: 0.5,
            pot_odds: 0.3,
            position: Position::Middle,
            aggression_level: 0.5,
            risk_tolerance: 0.5,
            valid_actions: vec![Action::Fold, Action::Call, Action::Check, Action::Bet(50)],
            game_phase: GamePhase::Flop,
            opponents_count: 2,
            pot_size: 100,
            current_bet: 20,
            player_chips: 1000,
        }
    }

    #[test]
    fn test_tight_strategy() {
        let mut strategy = TightStrategy::new();
        let mut context = create_test_context();
        
        // Test with weak hand
        context.hand_strength = 0.2;
        let action = strategy.decide_action(&mut context).unwrap();
        assert!(matches!(action, Action::Fold), "Tight strategy should fold weak hands");

        // Test with strong hand
        context.hand_strength = 0.8;
        let action = strategy.decide_action(&mut context).unwrap();
        assert!(matches!(action, Action::Raise(_) | Action::Bet(_)), "Tight strategy should bet strong hands");
    }

    #[test]
    fn test_aggressive_strategy() {
        let mut strategy = AggressiveStrategy::new();
        let mut context = create_test_context();
        
        context.hand_strength = 0.4;
        let action = strategy.decide_action(&mut context).unwrap();
        // Aggressive strategy should often bet or raise even with medium hands
        assert!(!matches!(action, Action::Fold), "Aggressive strategy should rarely fold medium hands");
    }

    #[test]
    fn test_balanced_strategy() {
        let mut strategy = BalancedStrategy::new();
        let mut context = create_test_context();
        
        // Test different game phases
        context.game_phase = GamePhase::PreFlop;
        let _action = strategy.decide_action(&mut context).unwrap();
        
        context.game_phase = GamePhase::River;
        let _action = strategy.decide_action(&mut context).unwrap();
        
        // Should not panic and should return valid actions
    }

    #[test]
    fn test_conservative_strategy() {
        let mut strategy = ConservativeStrategy::new();
        let mut context = create_test_context();
        
        // Test with medium hand and large bet
        context.hand_strength = 0.4;
        context.current_bet = 60; // Large relative to pot
        let action = strategy.decide_action(&mut context).unwrap();
        assert!(matches!(action, Action::Fold | Action::Check), "Conservative strategy should avoid large bets");
    }
}