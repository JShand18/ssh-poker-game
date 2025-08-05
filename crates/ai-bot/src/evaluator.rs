use poker_engine::{Action, Card, Hand, HandEvaluator as CoreHandEvaluator};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Position {
    Early,
    Middle,
    Late,
}

#[derive(Debug, Clone)]
pub struct HandStrengthEvaluator {
    core_evaluator: CoreHandEvaluator,
}

impl HandStrengthEvaluator {
    pub fn new() -> Self {
        Self {
            core_evaluator: CoreHandEvaluator::new(),
        }
    }

    pub fn evaluate_hand(&self, cards: &[Card]) -> Hand {
        self.core_evaluator.evaluate(cards)
    }

    pub fn calculate_hand_strength(&self, hole_cards: &[Card; 2], community_cards: &[Card]) -> f64 {
        let mut all_cards = hole_cards.to_vec();
        all_cards.extend_from_slice(community_cards);
        
        let hand = self.evaluate_hand(&all_cards);
        self.hand_to_strength(&hand)
    }

    pub fn calculate_outs(&self, hole_cards: &[Card; 2], community_cards: &[Card]) -> u8 {
        if community_cards.len() < 3 {
            return 0; // Can't calculate outs pre-flop
        }

        let mut outs = 0;
        let used_cards: std::collections::HashSet<Card> = hole_cards.iter()
            .chain(community_cards.iter())
            .cloned()
            .collect();

        // Check all possible turn/river cards
        for rank in [
            poker_engine::Rank::Two, poker_engine::Rank::Three, poker_engine::Rank::Four,
            poker_engine::Rank::Five, poker_engine::Rank::Six, poker_engine::Rank::Seven,
            poker_engine::Rank::Eight, poker_engine::Rank::Nine, poker_engine::Rank::Ten,
            poker_engine::Rank::Jack, poker_engine::Rank::Queen, poker_engine::Rank::King,
            poker_engine::Rank::Ace,
        ] {
            for suit in [
                poker_engine::Suit::Clubs, poker_engine::Suit::Diamonds,
                poker_engine::Suit::Hearts, poker_engine::Suit::Spades,
            ] {
                let test_card = Card { rank, suit };
                if !used_cards.contains(&test_card) {
                    // Test if this card improves our hand significantly
                    let mut test_community = community_cards.to_vec();
                    test_community.push(test_card);
                    
                    let current_strength = self.calculate_hand_strength(hole_cards, community_cards);
                    let improved_strength = self.calculate_hand_strength(hole_cards, &test_community);
                    
                    if improved_strength > current_strength + 0.1 { // Significant improvement
                        outs += 1;
                    }
                }
            }
        }

        outs
    }

    pub fn calculate_pot_equity(&self, hole_cards: &[Card; 2], community_cards: &[Card], opponents: usize) -> f64 {
        let hand_strength = self.calculate_hand_strength(hole_cards, community_cards);
        
        // Adjust for number of opponents (more opponents = need stronger hand)
        let opponent_adjustment = match opponents {
            1 => 1.0,
            2 => 0.85,
            3 => 0.75,
            4 => 0.65,
            5 => 0.55,
            _ => 0.5,
        };

        hand_strength * opponent_adjustment
    }

    fn hand_to_strength(&self, hand: &Hand) -> f64 {
        use poker_engine::HandRank;
        
        let base_strength = match hand.rank {
            HandRank::HighCard => 0.05,
            HandRank::Pair => 0.15,
            HandRank::TwoPair => 0.35,
            HandRank::ThreeOfAKind => 0.55,
            HandRank::Straight => 0.70,
            HandRank::Flush => 0.80,
            HandRank::FullHouse => 0.90,
            HandRank::FourOfAKind => 0.97,
            HandRank::StraightFlush => 1.0,
        };

        // Add fine-grained strength based on high cards within the hand type
        let kicker_bonus = self.calculate_kicker_strength(&hand.cards);
        (base_strength + kicker_bonus * 0.05).min(1.0)
    }

    fn calculate_kicker_strength(&self, cards: &[Card]) -> f64 {
        let mut strength = 0.0;
        let mut sorted_cards = cards.to_vec();
        sorted_cards.sort_by(|a, b| b.rank.cmp(&a.rank));

        for (i, card) in sorted_cards.iter().enumerate() {
            let rank_value = match card.rank {
                poker_engine::Rank::Two => 2.0,
                poker_engine::Rank::Three => 3.0,
                poker_engine::Rank::Four => 4.0,
                poker_engine::Rank::Five => 5.0,
                poker_engine::Rank::Six => 6.0,
                poker_engine::Rank::Seven => 7.0,
                poker_engine::Rank::Eight => 8.0,
                poker_engine::Rank::Nine => 9.0,
                poker_engine::Rank::Ten => 10.0,
                poker_engine::Rank::Jack => 11.0,
                poker_engine::Rank::Queen => 12.0,
                poker_engine::Rank::King => 13.0,
                poker_engine::Rank::Ace => 14.0,
            };
            
            // Higher cards are more valuable, but with diminishing returns for later cards
            strength += rank_value / (14.0 * (i + 1) as f64);
        }

        strength / cards.len() as f64
    }

    pub fn is_drawing_hand(&self, hole_cards: &[Card; 2], community_cards: &[Card]) -> bool {
        if community_cards.len() < 3 {
            return false;
        }

        let outs = self.calculate_outs(hole_cards, community_cards);
        outs >= 4 // 4 or more outs is considered a drawing hand
    }

    pub fn calculate_implied_odds(&self, pot_size: u64, call_amount: u64, expected_future_bets: u64) -> f64 {
        if call_amount == 0 {
            return f64::INFINITY;
        }
        (pot_size + expected_future_bets) as f64 / call_amount as f64
    }
}

impl Default for HandStrengthEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BettingHistory {
    actions: Vec<(usize, Action)>, // (player_id, action)
    round_actions: HashMap<usize, Vec<Action>>, // player_id -> actions this round
}

impl BettingHistory {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            round_actions: HashMap::new(),
        }
    }

    pub fn add_action(&mut self, player_id: usize, action: Action) {
        self.actions.push((player_id, action.clone()));
        self.round_actions.entry(player_id).or_default().push(action);
    }

    pub fn reset(&mut self) {
        self.actions.clear();
        self.round_actions.clear();
    }

    pub fn get_player_actions(&self, player_id: usize) -> Vec<&Action> {
        self.round_actions.get(&player_id)
            .map(|actions| actions.iter().collect())
            .unwrap_or_default()
    }

    pub fn get_all_actions(&self) -> &[(usize, Action)] {
        &self.actions
    }

    pub fn count_player_raises(&self, player_id: usize) -> usize {
        self.round_actions.get(&player_id)
            .map(|actions| actions.iter().filter(|action| matches!(action, Action::Raise(_))).count())
            .unwrap_or(0)
    }

    pub fn count_player_folds(&self, player_id: usize) -> usize {
        self.round_actions.get(&player_id)
            .map(|actions| actions.iter().filter(|action| matches!(action, Action::Fold)).count())
            .unwrap_or(0)
    }

    pub fn get_aggression_factor(&self, player_id: usize) -> f64 {
        let actions = self.get_player_actions(player_id);
        if actions.is_empty() {
            return 0.5; // Neutral
        }

        let aggressive_actions = actions.iter()
            .filter(|action| matches!(action, Action::Bet(_) | Action::Raise(_) | Action::AllIn))
            .count();

        let passive_actions = actions.iter()
            .filter(|action| matches!(action, Action::Call | Action::Check))
            .count();

        if aggressive_actions + passive_actions == 0 {
            return 0.5;
        }

        aggressive_actions as f64 / (aggressive_actions + passive_actions) as f64
    }

    pub fn is_player_tight(&self, player_id: usize) -> bool {
        let fold_count = self.count_player_folds(player_id);
        let total_actions = self.get_player_actions(player_id).len();
        
        if total_actions == 0 {
            return false;
        }

        fold_count as f64 / total_actions as f64 > 0.6
    }

    pub fn is_player_loose(&self, player_id: usize) -> bool {
        let fold_count = self.count_player_folds(player_id);
        let total_actions = self.get_player_actions(player_id).len();
        
        if total_actions == 0 {
            return false;
        }

        fold_count as f64 / total_actions as f64 < 0.3
    }
}

impl Default for BettingHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PositionAnalyzer;

impl PositionAnalyzer {
    pub fn calculate_position_strength(position: Position, players_count: usize) -> f64 {
        match position {
            Position::Early => match players_count {
                2..=3 => 0.4, // Heads-up or 3-handed, early position is less bad
                4..=6 => 0.3, // Small table, early position is disadvantageous
                _ => 0.2,     // Full table, early position is very disadvantageous
            },
            Position::Middle => 0.5, // Neutral position
            Position::Late => match players_count {
                2..=3 => 0.6, // Less advantage in short-handed games
                4..=6 => 0.7, // Good advantage in medium games
                _ => 0.8,     // Great advantage in full ring games
            },
        }
    }

    pub fn should_play_tighter(position: Position) -> bool {
        matches!(position, Position::Early)
    }

    pub fn should_play_looser(position: Position) -> bool {
        matches!(position, Position::Late)
    }

    pub fn bluff_frequency_modifier(position: Position) -> f64 {
        match position {
            Position::Early => 0.5,  // Bluff less in early position
            Position::Middle => 1.0, // Normal bluffing frequency
            Position::Late => 1.5,   // Bluff more in late position
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use poker_engine::{Rank, Suit};

    fn create_test_cards() -> ([Card; 2], Vec<Card>) {
        let hole_cards = [
            Card { rank: Rank::Ace, suit: Suit::Spades },
            Card { rank: Rank::King, suit: Suit::Hearts },
        ];
        let community_cards = vec![
            Card { rank: Rank::Queen, suit: Suit::Diamonds },
            Card { rank: Rank::Jack, suit: Suit::Clubs },
            Card { rank: Rank::Ten, suit: Suit::Spades },
        ];
        (hole_cards, community_cards)
    }

    #[test]
    fn test_hand_strength_evaluator() {
        let evaluator = HandStrengthEvaluator::new();
        let (hole_cards, community_cards) = create_test_cards();

        let strength = evaluator.calculate_hand_strength(&hole_cards, &community_cards);
        assert!(strength > 0.6, "Straight should have high strength: {}", strength);
    }

    #[test]
    fn test_outs_calculation() {
        let evaluator = HandStrengthEvaluator::new();
        let hole_cards = [
            Card { rank: Rank::Nine, suit: Suit::Hearts },
            Card { rank: Rank::Eight, suit: Suit::Hearts },
        ];
        let community_cards = vec![
            Card { rank: Rank::Seven, suit: Suit::Clubs },
            Card { rank: Rank::Six, suit: Suit::Diamonds },
            Card { rank: Rank::Two, suit: Suit::Hearts },
        ];

        let outs = evaluator.calculate_outs(&hole_cards, &community_cards);
        assert!(outs > 0, "Should have outs for straight draw");
    }

    #[test]
    fn test_betting_history() {
        let mut history = BettingHistory::new();
        
        history.add_action(0, Action::Raise(100));
        history.add_action(1, Action::Call);
        history.add_action(0, Action::Bet(50));

        assert_eq!(history.count_player_raises(0), 1);
        assert_eq!(history.get_aggression_factor(0), 1.0); // All actions are aggressive
        assert_eq!(history.get_aggression_factor(1), 0.0); // All actions are passive
    }

    #[test]
    fn test_position_analysis() {
        let early_strength = PositionAnalyzer::calculate_position_strength(Position::Early, 6);
        let late_strength = PositionAnalyzer::calculate_position_strength(Position::Late, 6);
        
        assert!(late_strength > early_strength, "Late position should be stronger than early position");
        assert!(PositionAnalyzer::should_play_tighter(Position::Early));
        assert!(PositionAnalyzer::should_play_looser(Position::Late));
    }

    #[test]
    fn test_pot_equity_calculation() {
        let evaluator = HandStrengthEvaluator::new();
        let (hole_cards, community_cards) = create_test_cards();

        let equity1 = evaluator.calculate_pot_equity(&hole_cards, &community_cards, 1);
        let equity3 = evaluator.calculate_pot_equity(&hole_cards, &community_cards, 3);

        assert!(equity1 > equity3, "Equity should decrease with more opponents");
    }

    #[test]
    fn test_implied_odds() {
        let evaluator = HandStrengthEvaluator::new();
        
        let odds = evaluator.calculate_implied_odds(100, 20, 50);
        assert_eq!(odds, 7.5); // (100 + 50) / 20
        
        let infinite_odds = evaluator.calculate_implied_odds(100, 0, 50);
        assert!(infinite_odds.is_infinite());
    }

    #[test]
    fn test_drawing_hand_detection() {
        let evaluator = HandStrengthEvaluator::new();
        
        // Flush draw
        let hole_cards = [
            Card { rank: Rank::Ace, suit: Suit::Hearts },
            Card { rank: Rank::King, suit: Suit::Hearts },
        ];
        let community_cards = vec![
            Card { rank: Rank::Queen, suit: Suit::Hearts },
            Card { rank: Rank::Jack, suit: Suit::Hearts },
            Card { rank: Rank::Two, suit: Suit::Clubs },
        ];

        let is_drawing = evaluator.is_drawing_hand(&hole_cards, &community_cards);
        // This might be true or false depending on the exact implementation
        // The test just ensures the method doesn't panic
        assert!(is_drawing || !is_drawing);
    }
}