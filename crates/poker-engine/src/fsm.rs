use crate::{GameState, GamePhase, errors::{PokerError, Result}};
use serde::{Deserialize, Serialize};

/// Extended game phase that includes waiting state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    /// Waiting for players to join
    Waiting,
    /// Pre-flop betting round
    PreFlop,
    /// Flop betting round
    Flop,
    /// Turn betting round
    Turn,
    /// River betting round
    River,
    /// Showdown - reveal cards and determine winner
    Showdown,
    /// Hand complete - distribute winnings
    HandComplete,
}

/// Events that can trigger state transitions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Enough players have joined to start
    PlayersReady,
    /// Betting round completed
    BettingRoundComplete,
    /// All but one player folded
    AllButOneFolded,
    /// Showdown complete, winner determined
    ShowdownComplete,
    /// Hand winnings distributed
    WinningsDistributed,
}

/// The finite state machine for managing game flow
pub struct GameFSM {
    current_state: State,
}

impl GameFSM {
    pub fn new() -> Self {
        Self {
            current_state: State::Waiting,
        }
    }
    
    pub fn current_state(&self) -> State {
        self.current_state
    }
    
    /// Transition to a new state based on an event
    pub fn transition(&mut self, event: Event) -> Result<State> {
        let new_state = match (self.current_state, event) {
            // From Waiting
            (State::Waiting, Event::PlayersReady) => State::PreFlop,
            
            // From PreFlop
            (State::PreFlop, Event::BettingRoundComplete) => State::Flop,
            (State::PreFlop, Event::AllButOneFolded) => State::HandComplete,
            
            // From Flop
            (State::Flop, Event::BettingRoundComplete) => State::Turn,
            (State::Flop, Event::AllButOneFolded) => State::HandComplete,
            
            // From Turn
            (State::Turn, Event::BettingRoundComplete) => State::River,
            (State::Turn, Event::AllButOneFolded) => State::HandComplete,
            
            // From River
            (State::River, Event::BettingRoundComplete) => State::Showdown,
            (State::River, Event::AllButOneFolded) => State::HandComplete,
            
            // From Showdown
            (State::Showdown, Event::ShowdownComplete) => State::HandComplete,
            
            // From HandComplete
            (State::HandComplete, Event::WinningsDistributed) => State::Waiting,
            
            // Invalid transitions
            (state, event) => {
                return Err(PokerError::InvalidGameState(
                    format!("Invalid transition from {:?} with event {:?}", state, event)
                ));
            }
        };
        
        self.current_state = new_state;
        Ok(new_state)
    }
    
    /// Check if a specific action is valid in the current state
    pub fn is_valid_action(&self, action: &str) -> bool {
        match self.current_state {
            State::Waiting => matches!(action, "join" | "leave" | "start"),
            State::PreFlop | State::Flop | State::Turn | State::River => {
                matches!(action, "fold" | "check" | "call" | "bet" | "raise" | "all_in")
            }
            State::Showdown => matches!(action, "show" | "muck"),
            State::HandComplete => matches!(action, "continue" | "leave"),
        }
    }
    
    /// Get the next expected events for the current state
    pub fn expected_events(&self) -> Vec<Event> {
        match self.current_state {
            State::Waiting => vec![Event::PlayersReady],
            State::PreFlop | State::Flop | State::Turn | State::River => {
                vec![Event::BettingRoundComplete, Event::AllButOneFolded]
            }
            State::Showdown => vec![Event::ShowdownComplete],
            State::HandComplete => vec![Event::WinningsDistributed],
        }
    }
}

impl Default for GameFSM {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait to integrate FSM with GameState
pub trait GameStateFSM {
    fn get_fsm_state(&self) -> State;
    fn should_transition(&self) -> Option<Event>;
    fn apply_transition(&mut self, event: Event) -> Result<()>;
}

impl GameStateFSM for GameState {
    fn get_fsm_state(&self) -> State {
        // Map GamePhase to FSM State
        match self.current_phase {
            GamePhase::PreFlop => State::PreFlop,
            GamePhase::Flop => State::Flop,
            GamePhase::Turn => State::Turn,
            GamePhase::River => State::River,
            GamePhase::Showdown => State::Showdown,
        }
    }
    
    fn should_transition(&self) -> Option<Event> {
        // Check if only one player remains active
        if self.active_player_count() <= 1 {
            return Some(Event::AllButOneFolded);
        }
        
        // Check if betting round is complete
        if self.is_betting_round_complete() {
            return Some(Event::BettingRoundComplete);
        }
        
        None
    }
    
    fn apply_transition(&mut self, event: Event) -> Result<()> {
        match event {
            Event::BettingRoundComplete => {
                match self.current_phase {
                    GamePhase::PreFlop => {
                        self.deal_community_cards(); // Deals flop
                    }
                    GamePhase::Flop => {
                        self.deal_community_cards(); // Deals turn
                    }
                    GamePhase::Turn => {
                        self.deal_community_cards(); // Deals river
                    }
                    GamePhase::River => {
                        self.current_phase = GamePhase::Showdown;
                    }
                    GamePhase::Showdown => {
                        // Hand is complete
                    }
                }
            }
            Event::AllButOneFolded => {
                // Move directly to hand complete
                self.current_phase = GamePhase::Showdown;
            }
            _ => {
                return Err(PokerError::InvalidGameState(
                    format!("Cannot apply event {:?} in current state", event)
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let fsm = GameFSM::new();
        assert_eq!(fsm.current_state(), State::Waiting);
    }

    #[test]
    fn test_valid_transitions() {
        let mut fsm = GameFSM::new();
        
        // Waiting -> PreFlop
        assert!(fsm.transition(Event::PlayersReady).is_ok());
        assert_eq!(fsm.current_state(), State::PreFlop);
        
        // PreFlop -> Flop
        assert!(fsm.transition(Event::BettingRoundComplete).is_ok());
        assert_eq!(fsm.current_state(), State::Flop);
        
        // Flop -> Turn
        assert!(fsm.transition(Event::BettingRoundComplete).is_ok());
        assert_eq!(fsm.current_state(), State::Turn);
        
        // Turn -> River
        assert!(fsm.transition(Event::BettingRoundComplete).is_ok());
        assert_eq!(fsm.current_state(), State::River);
        
        // River -> Showdown
        assert!(fsm.transition(Event::BettingRoundComplete).is_ok());
        assert_eq!(fsm.current_state(), State::Showdown);
        
        // Showdown -> HandComplete
        assert!(fsm.transition(Event::ShowdownComplete).is_ok());
        assert_eq!(fsm.current_state(), State::HandComplete);
        
        // HandComplete -> Waiting
        assert!(fsm.transition(Event::WinningsDistributed).is_ok());
        assert_eq!(fsm.current_state(), State::Waiting);
    }

    #[test]
    fn test_fold_out_transition() {
        let mut fsm = GameFSM::new();
        
        // Start game
        assert!(fsm.transition(Event::PlayersReady).is_ok());
        
        // All but one fold during preflop
        assert!(fsm.transition(Event::AllButOneFolded).is_ok());
        assert_eq!(fsm.current_state(), State::HandComplete);
    }

    #[test]
    fn test_invalid_transition() {
        let mut fsm = GameFSM::new();
        
        // Cannot go from Waiting to Flop directly
        assert!(fsm.transition(Event::BettingRoundComplete).is_err());
    }

    #[test]
    fn test_action_validation() {
        let fsm = GameFSM::new();
        
        // In waiting state
        assert!(fsm.is_valid_action("join"));
        assert!(fsm.is_valid_action("start"));
        assert!(!fsm.is_valid_action("fold"));
        assert!(!fsm.is_valid_action("bet"));
    }
} 