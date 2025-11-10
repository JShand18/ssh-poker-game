# Design Patterns: Elements of Reusable Object-Oriented Software - Pattern to Code Mapping

This document maps the 23 patterns from the Gang of Four book to existing implementations and potential applications in the SSH Poker Game codebase.

## Creational Patterns

### 1. **Factory Method**
**Intent**: Define an interface for creating objects, but let subclasses decide which class to instantiate.

#### Current Implementation:
- Not explicitly implemented yet

#### Potential Application:
```rust
// In crates/ai-bot/src/factory.rs
pub trait BotFactory {
    fn create_bot(&self, config: BotConfig) -> Box<dyn PokerBot>;
}

pub struct DifficultyBasedFactory;
impl BotFactory for DifficultyBasedFactory {
    fn create_bot(&self, config: BotConfig) -> Box<dyn PokerBot> {
        match config.difficulty {
            BotDifficulty::Beginner => Box::new(BeginnerBot::new(config)),
            BotDifficulty::Expert => Box::new(ExpertBot::new(config)),
            // ...
        }
    }
}
```

#### Relevant Code:
- `crates/ai-bot/src/lib.rs:17-23` - BotDifficulty enum
- Future: Bot creation logic

---

### 2. **Abstract Factory**
**Intent**: Provide interface for creating families of related objects.

#### Current Implementation:
- Not implemented

#### Potential Application:
- Creating different UI renderers (terminal, web, mobile)
- Creating game variations (Texas Hold'em, Omaha, etc.)

#### Relevant Code:
- `crates/charm-tui/src/` - Could have abstract UI factory
- `crates/terminal-ui/src/` - Alternative UI implementation

---

### 3. **Builder**
**Intent**: Separate construction of complex object from its representation.

#### Current Implementation:
- Partially seen in GameState construction

#### Potential Application:
```rust
// Enhanced GameState builder
pub struct GameStateBuilder {
    players: Option<Vec<Player>>,
    small_blind: Option<u64>,
    big_blind: Option<u64>,
    // ...
}

impl GameStateBuilder {
    pub fn with_players(mut self, players: Vec<Player>) -> Self {
        self.players = Some(players);
        self
    }
    
    pub fn build(self) -> Result<GameState> {
        // Validation and construction
    }
}
```

#### Relevant Code:
- `crates/poker-engine/src/game.rs:60-103` - GameState::new()
- Could be refactored to builder pattern

---

### 4. **Prototype**
**Intent**: Create objects by cloning a prototypical instance.

#### Current Implementation:
- Clone trait implemented on many types

#### Relevant Code:
- `crates/poker-engine/src/game.rs:36` - #[derive(Clone)]
- `crates/poker-engine/src/player.rs:12` - #[derive(Clone)]
- Used for game state snapshots

---

### 5. **Singleton**
**Intent**: Ensure a class has only one instance with global access.

#### Current Implementation:
- Not used (anti-pattern in Rust)
- Arc<T> provides controlled shared access instead

#### Better Rust Approach:
- `crates/ssh-server/src/lib.rs:63-75` - Shared state with Arc
- `crates/ssh-server/src/session.rs` - SessionManager as shared resource

---

## Structural Patterns

### 6. **Adapter**
**Intent**: Convert interface of a class into another interface clients expect.

#### Current Implementation:
- SSH to terminal UI adaptation

#### Relevant Code:
- `crates/ssh-server/src/charm_handler.rs` - Adapts SSH to Charm TUI
- `crates/wish-server/src/ssh_tui_bridge.rs` - Bridge between SSH and TUI

---

### 7. **Bridge**
**Intent**: Decouple abstraction from implementation.

#### Potential Application:
- Proposed in design improvements

#### Relevant Code:
- `docs/design_improvements.md:38-42` - HandEvaluatorTrait
- Separates hand evaluation interface from implementation

---

### 8. **Composite**
**Intent**: Compose objects into tree structures.

#### Current Implementation:
- UI component hierarchy

#### Relevant Code:
- `crates/charm-tui/src/components.rs` - UI component tree
- `crates/charm-tui/src/views.rs` - View hierarchy

---

### 9. **Decorator**
**Intent**: Attach additional responsibilities dynamically.

#### Potential Application:
- Logging wrapper for game actions
- Metrics collection decorator

#### Example:
```rust
pub struct LoggingGameEngine<T: GameEngine> {
    inner: T,
}

impl<T: GameEngine> GameEngine for LoggingGameEngine<T> {
    fn process_action(&mut self, action: Action) -> Result<()> {
        info!("Processing action: {:?}", action);
        self.inner.process_action(action)
    }
}
```

---

### 10. **Facade**
**Intent**: Provide unified interface to a subsystem.

#### Current Implementation:
- GameState acts as facade to game subsystems

#### Relevant Code:
- `crates/poker-engine/src/game.rs` - GameState as facade
- Hides complexity of betting, hand evaluation, pot management

---

### 11. **Flyweight**
**Intent**: Use sharing to support large numbers of fine-grained objects.

#### Current Implementation:
- Card representation could use this

#### Relevant Code:
- `crates/poker-engine/src/card.rs` - Currently copies Card
- Could be optimized with flyweight for card instances

---

### 12. **Proxy**
**Intent**: Provide placeholder/surrogate for another object.

#### Potential Application:
- Remote player proxy for network games
- Lazy loading of player statistics

---

## Behavioral Patterns

### 13. **Chain of Responsibility**
**Intent**: Chain receiving objects and pass request along until handled.

#### Potential Application:
- Input validation chain
- Action processing pipeline

---

### 14. **Command**
**Intent**: Encapsulate request as object.

#### Proposed Implementation:
- `docs/design_improvements.md:49-63` - Command pattern for GameState

#### Relevant Code:
- `crates/poker-engine/src/game.rs:20-28` - Action enum (commands)
- Could be enhanced with command pattern

---

### 15. **Iterator**
**Intent**: Provide way to access elements sequentially.

#### Current Implementation:
- Rust's built-in Iterator trait used extensively

#### Relevant Code:
- `crates/poker-engine/src/game.rs:621-650` - Iterator usage
- `crates/poker-engine/src/deck.rs:54-58` - AsRef for iteration

---

### 16. **Mediator**
**Intent**: Define object that encapsulates how objects interact.

#### Current Implementation:
- SessionManager mediates between sessions

#### Relevant Code:
- `crates/ssh-server/src/session.rs` - SessionManager
- Coordinates multiple poker tables and players

---

### 17. **Memento**
**Intent**: Capture object's internal state for later restoration.

#### Potential Application:
- Game replay system
- Undo functionality

#### Related:
- Serialize/Deserialize traits enable state capture

---

### 18. **Observer**
**Intent**: Define one-to-many dependency between objects.

#### Potential Application:
- Game state change notifications
- Real-time UI updates

#### Example:
```rust
pub trait GameObserver {
    fn on_action(&mut self, action: &Action);
    fn on_phase_change(&mut self, new_phase: GamePhase);
}
```

---

### 19. **State**
**Intent**: Allow object to alter behavior when internal state changes.

#### Current Implementation:
- FSM implementation

#### Relevant Code:
- `crates/poker-engine/src/fsm.rs` - Game state machine
- `crates/poker-engine/src/game.rs:12-18` - GamePhase enum
- Perfect example of State pattern

---

### 20. **Strategy**
**Intent**: Define family of algorithms, make them interchangeable.

#### Current Implementation:
- AI bot strategies

#### Relevant Code:
- `crates/ai-bot/src/strategy.rs:22-26` - PokerStrategy trait
- `crates/ai-bot/src/strategy.rs:28-41` - TightStrategy
- `crates/ai-bot/src/personality.rs` - Different personalities
- Excellent implementation of Strategy pattern

---

### 21. **Template Method**
**Intent**: Define skeleton of algorithm, subclasses override specific steps.

#### Potential Application:
```rust
pub trait GamePhaseHandler {
    fn handle_phase(&mut self, state: &mut GameState) {
        self.validate_state(state);
        self.process_phase(state);
        self.transition_next(state);
    }
    
    fn validate_state(&self, state: &GameState);
    fn process_phase(&mut self, state: &mut GameState);
    fn transition_next(&self, state: &mut GameState);
}
```

---

### 22. **Visitor**
**Intent**: Represent operation to be performed on object structure elements.

#### Potential Application:
- Hand evaluation visitor
- Game statistics collector

---

### 23. **Interpreter**
**Intent**: Define grammar and interpreter for language.

#### Potential Application:
- Poker hand notation parser
- Betting command interpreter

---

## Summary of Pattern Usage

### Currently Implemented:
1. **State Pattern** - Game FSM (`fsm.rs`)
2. **Strategy Pattern** - AI strategies (`strategy.rs`)
3. **Facade Pattern** - GameState
4. **Adapter Pattern** - SSH handlers
5. **Mediator Pattern** - SessionManager
6. **Iterator Pattern** - Throughout (Rust built-in)
7. **Composite Pattern** - UI components

### Proposed in Design Docs:
1. **Command Pattern** - For GameState refactoring
2. **Bridge Pattern** - For HandEvaluator abstraction

### Good Candidates for Implementation:
1. **Factory Method** - For bot creation
2. **Builder** - For complex object construction
3. **Observer** - For real-time updates
4. **Template Method** - For game phase handling
5. **Decorator** - For logging/metrics

### Rust-Specific Considerations:
- Singleton → Use Arc<T> instead
- Prototype → Use Clone trait
- Many patterns simplified by Rust's trait system
- Ownership system influences pattern implementation