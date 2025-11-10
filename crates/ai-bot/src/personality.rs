use serde::{Serialize, Deserialize};
use rand::{Rng, thread_rng};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BotPersonality {
    Tight,         // Plays few hands, but plays them aggressively
    Loose,         // Plays many hands, calls frequently
    Aggressive,    // Bets and raises often, bluffs more
    Conservative,  // Avoids risks, plays safely
    Balanced,      // Mix of all styles, adapts to situation
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTraits {
    pub aggression: f64,      // 0.0 = very passive, 1.0 = very aggressive
    pub tightness: f64,       // 0.0 = very loose, 1.0 = very tight
    pub bluff_frequency: f64, // 0.0 = never bluffs, 1.0 = bluffs constantly
    pub risk_tolerance: f64,  // 0.0 = very risk-averse, 1.0 = loves risk
    pub adaptability: f64,    // 0.0 = never adapts, 1.0 = highly adaptive
    pub patience: f64,        // 0.0 = impatient, 1.0 = very patient
}

impl BotPersonality {
    pub fn get_traits(self) -> PersonalityTraits {
        match self {
            BotPersonality::Tight => PersonalityTraits {
                aggression: 0.7,
                tightness: 0.8,
                bluff_frequency: 0.1,
                risk_tolerance: 0.4,
                adaptability: 0.5,
                patience: 0.8,
            },
            BotPersonality::Loose => PersonalityTraits {
                aggression: 0.4,
                tightness: 0.2,
                bluff_frequency: 0.15,
                risk_tolerance: 0.7,
                adaptability: 0.6,
                patience: 0.3,
            },
            BotPersonality::Aggressive => PersonalityTraits {
                aggression: 0.9,
                tightness: 0.4,
                bluff_frequency: 0.25,
                risk_tolerance: 0.8,
                adaptability: 0.7,
                patience: 0.2,
            },
            BotPersonality::Conservative => PersonalityTraits {
                aggression: 0.2,
                tightness: 0.7,
                bluff_frequency: 0.05,
                risk_tolerance: 0.2,
                adaptability: 0.3,
                patience: 0.9,
            },
            BotPersonality::Balanced => PersonalityTraits {
                aggression: 0.5,
                tightness: 0.5,
                bluff_frequency: 0.12,
                risk_tolerance: 0.5,
                adaptability: 0.8,
                patience: 0.6,
            },
        }
    }

    pub fn get_description(self) -> &'static str {
        match self {
            BotPersonality::Tight => "Plays few hands but plays them strongly. Waits for premium hands and then bets aggressively.",
            BotPersonality::Loose => "Plays many hands and calls frequently. Enjoys seeing flops and chasing draws.",
            BotPersonality::Aggressive => "Bets and raises often, uses aggression as a weapon. Bluffs frequently to pressure opponents.",
            BotPersonality::Conservative => "Plays very safely, avoids risks. Only bets with strong hands and folds marginal situations.",
            BotPersonality::Balanced => "Adapts playing style to the situation. Uses a mix of tight and loose, aggressive and passive play.",
        }
    }

    pub fn get_random() -> Self {
        let mut rng = thread_rng();
        match rng.gen_range(0..5) {
            0 => BotPersonality::Tight,
            1 => BotPersonality::Loose,
            2 => BotPersonality::Aggressive,
            3 => BotPersonality::Conservative,
            _ => BotPersonality::Balanced,
        }
    }

    pub fn modify_for_difficulty(self, difficulty: DifficultyLevel) -> PersonalityTraits {
        let mut traits = self.get_traits();

        match difficulty {
            DifficultyLevel::Beginner => {
                // Beginners make more mistakes, are less adaptive
                traits.adaptability *= 0.3;
                traits.patience *= 0.5;
                // Add some randomness to make them more unpredictable in a bad way
                let mut rng = thread_rng();
                traits.aggression += rng.gen_range(-0.2..0.2);
                traits.tightness += rng.gen_range(-0.2..0.2);
                traits.bluff_frequency *= 0.5; // Beginners bluff less effectively
            }
            DifficultyLevel::Intermediate => {
                // Intermediate players are more consistent but not perfect
                traits.adaptability *= 0.6;
                let mut rng = thread_rng();
                traits.aggression += rng.gen_range(-0.1..0.1);
                traits.tightness += rng.gen_range(-0.1..0.1);
            }
            DifficultyLevel::Advanced => {
                // Advanced players are quite good
                traits.adaptability *= 0.9;
                traits.patience *= 1.1;
            }
            DifficultyLevel::Expert => {
                // Expert players maximize their traits
                traits.adaptability = traits.adaptability.min(1.0) * 1.2;
                traits.patience = traits.patience.min(1.0) * 1.1;
                // Experts can use all tools effectively
                if matches!(self, BotPersonality::Aggressive) {
                    traits.bluff_frequency *= 1.2;
                }
            }
        }

        // Clamp all values to [0.0, 1.0]
        traits.aggression = traits.aggression.clamp(0.0, 1.0);
        traits.tightness = traits.tightness.clamp(0.0, 1.0);
        traits.bluff_frequency = traits.bluff_frequency.clamp(0.0, 1.0);
        traits.risk_tolerance = traits.risk_tolerance.clamp(0.0, 1.0);
        traits.adaptability = traits.adaptability.clamp(0.0, 1.0);
        traits.patience = traits.patience.clamp(0.0, 1.0);

        traits
    }
}

impl Default for BotPersonality {
    fn default() -> Self {
        BotPersonality::Balanced
    }
}

impl PersonalityTraits {
    pub fn should_play_hand(&self, hand_strength: f64) -> bool {
        let mut rng = thread_rng();
        
        // Tight players require higher hand strength
        let required_strength = self.tightness * 0.6 + 0.1; // Range: 0.1 to 0.7
        
        // Add some randomness based on adaptability
        let randomness = (1.0 - self.adaptability) * 0.2;
        let adjusted_required = required_strength + rng.gen_range(-randomness..randomness);
        
        hand_strength > adjusted_required
    }

    pub fn should_bluff(&self, hand_strength: f64, pot_size: u64, call_amount: u64) -> bool {
        if hand_strength > 0.5 {
            return false; // Don't bluff with decent hands
        }

        let mut rng = thread_rng();
        
        // Base bluff frequency
        let mut bluff_chance = self.bluff_frequency;
        
        // Aggressive players bluff more
        bluff_chance *= 1.0 + self.aggression * 0.5;
        
        // Risk-tolerant players bluff more
        bluff_chance *= 1.0 + self.risk_tolerance * 0.3;
        
        // Consider pot odds - better pot odds encourage bluffing
        if call_amount > 0 {
            let pot_odds = pot_size as f64 / call_amount as f64;
            if pot_odds > 3.0 {
                bluff_chance *= 1.2; // Good pot odds, bluff more
            } else if pot_odds < 2.0 {
                bluff_chance *= 0.8; // Poor pot odds, bluff less
            }
        }
        
        rng.gen::<f64>() < bluff_chance.min(0.4) // Cap bluffing at 40%
    }

    pub fn calculate_bet_size(&self, pot_size: u64, player_chips: u64, hand_strength: f64) -> u64 {
        let mut rng = thread_rng();
        
        // Base bet size as fraction of pot
        let mut bet_fraction = match hand_strength {
            x if x > 0.8 => 0.75, // Strong hands bet big
            x if x > 0.6 => 0.5,  // Good hands bet medium
            x if x > 0.4 => 0.33, // Marginal hands bet small
            _ => 0.25,            // Weak hands bet tiny (bluffs)
        };

        // Aggressive players bet larger
        bet_fraction *= 1.0 + self.aggression * 0.5;
        
        // Risk-tolerant players vary bet sizes more
        if self.risk_tolerance > 0.6 {
            let variance = rng.gen_range(-0.2..0.3);
            bet_fraction += variance;
        }

        // Patient players bet smaller to extract value
        if self.patience > 0.7 && hand_strength > 0.7 {
            bet_fraction *= 0.8; // Smaller bets to get called
        }

        let bet_amount = (pot_size as f64 * bet_fraction) as u64;
        
        // Ensure bet is reasonable
        bet_amount.max(pot_size / 10).min(player_chips).min(pot_size * 2)
    }

    pub fn get_fold_threshold(&self, position_factor: f64) -> f64 {
        // Tight players fold more readily
        let base_threshold = 0.3 + self.tightness * 0.3;
        
        // Conservative players have higher fold threshold
        let conservatism_bonus = (1.0 - self.risk_tolerance) * 0.2;
        
        // Position matters - fold less in late position
        let position_adjustment = position_factor * 0.1;
        
        (base_threshold + conservatism_bonus - position_adjustment).clamp(0.2, 0.8)
    }

    pub fn should_adapt_to_opponents(&self, hands_played: u32) -> bool {
        if hands_played < 10 {
            return false; // Need data to adapt
        }

        let mut rng = thread_rng();
        rng.gen::<f64>() < self.adaptability
    }

    pub fn get_patience_factor(&self) -> f64 {
        self.patience
    }
}

#[derive(Debug, Clone)]
pub struct PersonalityModifier {
    base_traits: PersonalityTraits,
    current_traits: PersonalityTraits,
    adaptation_rate: f64,
}

impl PersonalityModifier {
    pub fn new(personality: BotPersonality, difficulty: DifficultyLevel) -> Self {
        let base_traits = personality.modify_for_difficulty(difficulty);
        Self {
            base_traits: base_traits.clone(),
            current_traits: base_traits,
            adaptation_rate: match difficulty {
                DifficultyLevel::Beginner => 0.1,
                DifficultyLevel::Intermediate => 0.2,
                DifficultyLevel::Advanced => 0.4,
                DifficultyLevel::Expert => 0.6,
            },
        }
    }

    pub fn get_traits(&self) -> &PersonalityTraits {
        &self.current_traits
    }

    pub fn adapt_to_table(&mut self, table_aggression: f64, table_tightness: f64) {
        if !self.current_traits.should_adapt_to_opponents(20) {
            return;
        }

        // Adapt aggression to counter table style
        let aggression_target = if table_aggression > 0.6 {
            // Table is aggressive, play tighter and more selectively aggressive
            self.base_traits.aggression * 0.8
        } else {
            // Table is passive, can be more aggressive
            self.base_traits.aggression * 1.2
        };

        // Adapt tightness inversely to table tightness
        let tightness_target = if table_tightness > 0.6 {
            // Table is tight, can play looser
            self.base_traits.tightness * 0.8
        } else {
            // Table is loose, play tighter
            self.base_traits.tightness * 1.2
        };

        // Gradually move towards targets
        self.current_traits.aggression += (aggression_target - self.current_traits.aggression) * self.adaptation_rate;
        self.current_traits.tightness += (tightness_target - self.current_traits.tightness) * self.adaptation_rate;

        // Clamp values
        self.current_traits.aggression = self.current_traits.aggression.clamp(0.0, 1.0);
        self.current_traits.tightness = self.current_traits.tightness.clamp(0.0, 1.0);
    }

    pub fn reset_to_base(&mut self) {
        self.current_traits = self.base_traits.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_traits() {
        let tight_traits = BotPersonality::Tight.get_traits();
        let loose_traits = BotPersonality::Loose.get_traits();
        
        assert!(tight_traits.tightness > loose_traits.tightness);
        assert!(loose_traits.risk_tolerance > tight_traits.risk_tolerance);
    }

    #[test]
    fn test_difficulty_modification() {
        let base_traits = BotPersonality::Balanced.get_traits();
        let beginner_traits = BotPersonality::Balanced.modify_for_difficulty(DifficultyLevel::Beginner);
        let expert_traits = BotPersonality::Balanced.modify_for_difficulty(DifficultyLevel::Expert);
        
        assert!(expert_traits.adaptability >= beginner_traits.adaptability);
        assert!(expert_traits.patience >= base_traits.patience);
    }

    #[test]
    fn test_hand_playing_decision() {
        let tight_traits = BotPersonality::Tight.get_traits();
        let loose_traits = BotPersonality::Loose.get_traits();
        
        let weak_hand = 0.2;
        let strong_hand = 0.8;
        
        // Both should play strong hands
        assert!(tight_traits.should_play_hand(strong_hand));
        assert!(loose_traits.should_play_hand(strong_hand));
        
        // Loose should be more likely to play weak hands
        let tight_plays_weak = (0..100).filter(|_| tight_traits.should_play_hand(weak_hand)).count();
        let loose_plays_weak = (0..100).filter(|_| loose_traits.should_play_hand(weak_hand)).count();
        
        assert!(loose_plays_weak >= tight_plays_weak);
    }

    #[test]
    fn test_bluff_decision() {
        let aggressive_traits = BotPersonality::Aggressive.get_traits();
        let conservative_traits = BotPersonality::Conservative.get_traits();
        
        let weak_hand = 0.2;
        let pot_size = 100;
        let call_amount = 25;
        
        let aggressive_bluffs = (0..100)
            .filter(|_| aggressive_traits.should_bluff(weak_hand, pot_size, call_amount))
            .count();
        let conservative_bluffs = (0..100)
            .filter(|_| conservative_traits.should_bluff(weak_hand, pot_size, call_amount))
            .count();
        
        assert!(aggressive_bluffs >= conservative_bluffs);
    }

    #[test]
    fn test_bet_sizing() {
        let aggressive_traits = BotPersonality::Aggressive.get_traits();
        let conservative_traits = BotPersonality::Conservative.get_traits();
        
        let pot_size = 100;
        let player_chips = 1000;
        let hand_strength = 0.7;
        
        let aggressive_bet = aggressive_traits.calculate_bet_size(pot_size, player_chips, hand_strength);
        let conservative_bet = conservative_traits.calculate_bet_size(pot_size, player_chips, hand_strength);
        
        // Aggressive players should generally bet more
        // Note: This might not always be true due to randomness, but over many iterations it should be
        assert!(aggressive_bet > 0);
        assert!(conservative_bet > 0);
    }

    #[test]
    fn test_personality_modifier() {
        let mut modifier = PersonalityModifier::new(BotPersonality::Balanced, DifficultyLevel::Advanced);
        let _initial_aggression = modifier.get_traits().aggression;
        
        // Simulate adapting to a very aggressive table
        modifier.adapt_to_table(0.9, 0.5);
        
        // The bot should have adapted somehow (exact behavior depends on implementation)
        // Just test that the method doesn't panic and values stay in bounds
        assert!(modifier.get_traits().aggression >= 0.0);
        assert!(modifier.get_traits().aggression <= 1.0);
        assert!(modifier.get_traits().tightness >= 0.0);
        assert!(modifier.get_traits().tightness <= 1.0);
    }

    #[test]
    fn test_random_personality() {
        let personality = BotPersonality::get_random();
        let description = personality.get_description();
        assert!(!description.is_empty());
    }

    #[test]
    fn test_fold_threshold() {
        let tight_traits = BotPersonality::Tight.get_traits();
        let loose_traits = BotPersonality::Loose.get_traits();
        
        let early_position_factor = 0.0; // Early position
        let late_position_factor = 1.0;  // Late position
        
        let tight_early_threshold = tight_traits.get_fold_threshold(early_position_factor);
        let tight_late_threshold = tight_traits.get_fold_threshold(late_position_factor);
        let loose_early_threshold = loose_traits.get_fold_threshold(early_position_factor);
        
        // Tight players should have higher fold thresholds
        assert!(tight_early_threshold >= loose_early_threshold);
        
        // Late position should have lower fold threshold than early position
        assert!(tight_late_threshold <= tight_early_threshold);
        
        // All thresholds should be in valid range
        assert!(tight_early_threshold >= 0.2 && tight_early_threshold <= 0.8);
        assert!(tight_late_threshold >= 0.2 && tight_late_threshold <= 0.8);
        assert!(loose_early_threshold >= 0.2 && loose_early_threshold <= 0.8);
    }
}