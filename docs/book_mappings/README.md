# Book to Code Mappings - Study Guide

This directory contains comprehensive mappings between three foundational programming books and the SSH Poker Game codebase. These mappings will help you connect theoretical concepts to practical implementations.

## 📚 Books Covered

1. **The Rust Programming Language** ("The Rust Book")
   - Foundation for all Rust concepts
   - [Full mapping](./rust_book_mapping.md)

2. **Design Patterns: Elements of Reusable Object-Oriented Software** (Gang of Four)
   - Classic software design patterns
   - [Full mapping](./design_patterns_mapping.md)

3. **Rust for Embedded Systems**
   - Systems programming concepts
   - [Full mapping](./embedded_rust_mapping.md)

## 🎯 Suggested Learning Path

### Phase 1: Rust Fundamentals (Weeks 1-3)
Start with the Rust Book, focusing on:

1. **Chapters 1-6**: Basic concepts
   - Work through examples in `crates/poker-engine/src/card.rs` and `player.rs`
   - Implement simple modifications to understand ownership

2. **Chapters 7-9**: Project organization and error handling
   - Study the workspace structure
   - Examine error handling in `crates/poker-engine/src/errors.rs`

3. **Chapters 10-11**: Traits and testing
   - Implement a new trait for the poker engine
   - Add tests to existing modules

### Phase 2: Advanced Rust (Weeks 4-5)
Continue with advanced Rust topics:

1. **Chapters 13-15**: Functional features and smart pointers
   - Study iterator usage in hand evaluation
   - Understand Arc/Mutex patterns in the server

2. **Chapters 16-17**: Concurrency and async
   - Deep dive into `crates/ssh-server/src/lib.rs`
   - Trace async flow from connection to game action

### Phase 3: Design Patterns (Weeks 6-7)
Apply design patterns to the codebase:

1. **Behavioral Patterns**:
   - Study State pattern in `fsm.rs`
   - Examine Strategy pattern in AI bots

2. **Structural Patterns**:
   - Understand Adapter pattern in SSH handlers
   - Consider implementing Decorator for logging

3. **Creational Patterns**:
   - Design a Factory for bot creation
   - Refactor GameState with Builder pattern

### Phase 4: Systems Programming (Weeks 8-9)
Apply embedded concepts to server programming:

1. **Memory Management**:
   - Profile the poker engine's memory usage
   - Optimize hot paths for zero allocation

2. **State Machines**:
   - Extend the FSM with new game variants
   - Implement compile-time state validation

3. **Real-time Considerations**:
   - Add timeout handling throughout
   - Implement backpressure mechanisms

## 💡 Hands-On Exercises

### Week 1-2: Ownership Mastery
```rust
// Exercise: Modify the deck to support card borrowing
// Current: pub fn draw(&mut self) -> Option<Card>
// Goal: Support peeking without taking ownership
```

### Week 3-4: Trait Design
```rust
// Exercise: Create a new trait for game variants
pub trait PokerVariant {
    fn deal_hole_cards(&self, num_players: usize) -> Vec<Vec<Card>>;
    fn community_card_rounds(&self) -> Vec<usize>;
}
```

### Week 5-6: Pattern Implementation
```rust
// Exercise: Implement Observer pattern for game events
pub trait GameObserver {
    fn on_player_action(&mut self, player_id: usize, action: Action);
    fn on_game_phase_change(&mut self, old: GamePhase, new: GamePhase);
}
```

### Week 7-8: Performance Optimization
```rust
// Exercise: Benchmark and optimize hand evaluation
// Goal: Reduce evaluation time by 50%
// Hint: Consider lookup tables and bit manipulation
```

### Week 9: Integration Project
Combine all learnings:
1. Add a new AI personality using Strategy pattern
2. Implement async tournament mode
3. Add real-time statistics using Observer pattern
4. Optimize for 1000+ concurrent games

## 📊 Progress Tracking

Create a personal progress tracker:

```markdown
## My Learning Progress

### Rust Book
- [ ] Chapter 1: Getting Started
- [ ] Chapter 2: Guessing Game
- [ ] Chapter 3: Common Concepts
... (continue for all chapters)

### Design Patterns Implemented
- [x] State Pattern (already in codebase)
- [x] Strategy Pattern (already in codebase)
- [ ] Factory Method (exercise)
- [ ] Builder (exercise)
... (continue for patterns you want to implement)

### Embedded Concepts Applied
- [ ] Zero-allocation hot paths
- [ ] Compile-time state machines
- [ ] Resource pooling
... (continue for concepts)
```

## 🔗 Cross-References

### When Reading About Ownership (Rust Ch. 4)
- See Design Pattern: Prototype (cloning)
- See Embedded: Resource Management
- Code: `deck.rs:draw()`, `player.rs:deal_hole_cards()`

### When Reading About Traits (Rust Ch. 10)
- See Design Pattern: Strategy, Bridge
- See Embedded: Hardware Abstraction Layer concepts
- Code: `PokerBot` trait, proposed `GameEngine` trait

### When Reading About Concurrency (Rust Ch. 16)
- See Design Pattern: Mediator (SessionManager)
- See Embedded: Interrupt handling, RTOS concepts
- Code: SSH server connection handling

### When Reading State Pattern (GoF)
- See Rust: Enums and Pattern Matching (Ch. 6)
- See Embedded: State Machines
- Code: `fsm.rs`, `GamePhase` enum

## 🎓 Additional Resources

### Video Tutorials
- [Rust in Motion](https://www.manning.com/livevideo/rust-in-motion) - Visual learning
- [Design Patterns in Rust](https://www.youtube.com/playlist?list=PLza5oFLQGTl2Z5T8g1pRkIynR3E0_pc7U)

### Practice Projects
1. **Mini Project 1**: Add Omaha poker variant
2. **Mini Project 2**: Implement replay system using Memento pattern
3. **Mini Project 3**: Create tournament mode with brackets

### Community
- [Rust Users Forum](https://users.rust-lang.org/) - Ask questions
- [/r/rust](https://www.reddit.com/r/rust/) - Community discussions
- Project issues - Contribute to this codebase!

## 📝 Notes Section

Use this space to track insights, questions, and discoveries:

```markdown
## My Notes

### Interesting Discoveries
- The poker engine uses zero heap allocations during hand evaluation
- Arc<Mutex<T>> pattern is used extensively for shared state
- The FSM prevents invalid game states at compile time

### Questions to Explore
- How would I implement undo/redo functionality?
- What's the performance impact of async vs threads here?
- How can I make the AI bots learn from games?

### Ideas for Improvements
- Add spectator mode using Observer pattern
- Implement hand history using Command pattern
- Create a plugin system for custom game rules
```

---

Remember: The best way to learn is by doing. Don't just read the code—modify it, break it, fix it, and make it better. Each concept you learn from the books has a practical application in this codebase. Happy learning! 🚀