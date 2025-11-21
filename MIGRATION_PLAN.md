# Migration Plan: Rust → Go + Charm.sh Infrastructure

## Executive Summary

This document outlines a potential migration from the current Rust-based implementation to Go using Charm.sh's Wish (SSH server) and Bubble Tea (TUI framework). The goal is to leverage battle-tested infrastructure for SSH and TUI handling while maintaining the poker game logic.

---

## Current State vs Proposed State

### Current Architecture (Rust)

| Component | Implementation | Lines of Code | Dependencies |
|-----------|---------------|---------------|--------------|
| SSH Server | russh (0.44) | ~2,000 | russh, russh-keys |
| TUI Framework | ratatui (0.28) | ~2,500 | ratatui, crossterm |
| SSH-TUI Bridge | Custom implementation | ~500 | Manual event mapping |
| Game Engine | Custom poker engine | ~4,000 | poker crate for evaluation |
| Database | sqlx + SQLite/PostgreSQL | ~1,500 | sqlx |
| AI Bots | Custom strategies | ~1,500 | - |
| Metrics | Prometheus + Datadog | ~500 | prometheus, dogstatsd |
| **Total** | 6 crates | **~12,000** | 25+ dependencies |

### Proposed Architecture (Go + Charm.sh)

| Component | Implementation | Estimated LOC | Dependencies |
|-----------|---------------|---------------|--------------|
| SSH Server | **Wish** | ~100-200 | github.com/charmbracelet/wish |
| TUI Framework | **Bubble Tea** | ~1,500-2,000 | github.com/charmbracelet/bubbletea |
| SSH-TUI Bridge | **Built-in Wish middleware** | **~0** | Automatic via wish/bubbletea |
| Game Engine | Port from Rust | ~3,000-4,000 | Custom or poker library |
| Database | database/sql or GORM | ~1,000-1,500 | lib/pq (PostgreSQL) or go-sqlite3 |
| AI Bots | Port from Rust | ~1,200-1,500 | - |
| Metrics | Prometheus client | ~400-500 | prometheus/client_golang |
| **Total** | Single module | **~7,200-10,000** | 10-15 dependencies |

---

## What Charm.sh Offers (Key Benefits)

### 1. **Wish (SSH Server)**

**Built-in Features:**
- Automatic SSH key generation (ed25519)
- Session management per SSH connection
- Middleware architecture (compose SSH handlers)
- Built-in Bubble Tea middleware - **eliminates custom SSH-TUI bridge**
- Automatic PTY handling and window resize events
- Authentication via authorized_keys or custom handlers
- Banner customization

**What This Replaces:**
- Your custom `SshSessionHandler` (~500 LOC)
- SSH-TUI bridge event mapping (~500 LOC)
- Session isolation logic (partially)
- ANSI escape sequence rendering (handled by Bubble Tea)

**Example:**
```go
s, err := wish.NewServer(
    wish.WithAddress(":2222"),
    wish.WithMiddleware(
        bubbletea.Middleware(func(s ssh.Session) (tea.Model, []tea.ProgramOption) {
            // Return your Bubble Tea app for this session
            return NewPokerApp(s), []tea.ProgramOption{tea.WithAltScreen()}
        }),
        logging.Middleware(),
    ),
)
```

### 2. **Bubble Tea (TUI Framework)**

**Architecture (The Elm Architecture):**
```go
type Model interface {
    Init() tea.Cmd                      // Initialize
    Update(tea.Msg) (tea.Model, tea.Cmd)  // Handle events
    View() string                       // Render
}
```

**Built-in Features:**
- Component library (github.com/charmbracelet/bubbles): tables, lists, text inputs, progress bars
- Lipgloss styling (like CSS for the terminal)
- Automatic input handling (keyboard, mouse, window resize)
- Command system for async operations
- Batch operations for concurrent updates

**What This Replaces:**
- Your ratatui-based TUI (~2,500 LOC simplified to ~1,500-2,000 LOC)
- Event system with channels (~200 LOC - now built-in)
- Custom components (~300 LOC - use bubbles library)

### 3. **Integrated Ecosystem**

**Charm.sh Stack:**
- **Wish**: SSH server
- **Bubble Tea**: TUI framework
- **Bubbles**: TUI components (text input, tables, spinners, etc.)
- **Lip Gloss**: Styling engine (colors, borders, layouts)
- **Harmonica**: Spring animations
- **Glamour**: Markdown rendering

**Key Advantage:** These are designed to work together seamlessly. Your custom SSH-TUI bridge becomes **0 lines of code** - it's handled automatically by the wish/bubbletea middleware.

---

## Migration Complexity Analysis

### Low Complexity (Direct Translation)

1. **Game Engine Logic** ⭐️⭐️☆☆☆
   - Pure business logic, minimal changes
   - Card, Deck, Hand evaluation → straightforward port
   - FSM (Finite State Machine) → Go state machine pattern
   - Betting rules and pot management → direct translation
   - **Estimated Effort:** 2-3 days

2. **Database Layer** ⭐️⭐️☆☆☆
   - SQLite/PostgreSQL support available in Go
   - Use `database/sql` with drivers or GORM ORM
   - Similar async patterns with goroutines
   - **Estimated Effort:** 1-2 days

3. **AI Bot Strategies** ⭐️⭐️☆☆☆
   - Pure logic, minimal dependencies
   - Strategy pattern translates well to Go interfaces
   - **Estimated Effort:** 1-2 days

### Medium Complexity (Redesign Required)

4. **TUI Components** ⭐️⭐️⭐️☆☆
   - Redesign for Elm architecture (Init/Update/View)
   - Casino theming with Lip Gloss instead of owo-colors
   - Replace custom components with Bubbles library
   - Poker table renderer needs rewrite for Lip Gloss
   - **Estimated Effort:** 3-5 days

5. **Authentication & Sessions** ⭐️⭐️⭐️☆☆
   - Wish handles SSH sessions automatically
   - Need to implement game session management (player → table mapping)
   - Argon2 password hashing available in Go (golang.org/x/crypto/argon2)
   - **Estimated Effort:** 2-3 days

### Low Complexity (Simplified by Charm.sh)

6. **SSH Server** ⭐️☆☆☆☆
   - Replaced by Wish with middleware (~100 LOC)
   - **Estimated Effort:** 0.5 days

7. **SSH-TUI Bridge** ⭐️☆☆☆☆
   - Eliminated entirely - built into Wish
   - **Estimated Effort:** 0 days (net savings)

8. **Metrics** ⭐️⭐️☆☆☆
   - Prometheus client for Go is mature
   - Datadog integration available
   - **Estimated Effort:** 1 day

---

## Pros and Cons Analysis

### Pros of Migration to Go + Charm.sh

#### 1. **Drastically Simplified SSH Integration**
- ✅ Wish middleware eliminates ~1,000 LOC of custom SSH handling
- ✅ Automatic per-session TUI isolation
- ✅ Built-in PTY and window resize handling
- ✅ No manual ANSI escape sequence rendering

#### 2. **Battle-Tested Infrastructure**
- ✅ Wish and Bubble Tea power hundreds of production SSH apps
- ✅ Active maintenance and large community (14k+ stars on Bubble Tea)
- ✅ Well-documented with extensive examples
- ✅ Used by major projects (Soft Serve, Glow, Skate, etc.)

#### 3. **Simpler Codebase**
- ✅ Estimated 40% reduction in LOC (12,000 → 7,200-10,000)
- ✅ Single language/ecosystem instead of Rust crates
- ✅ Less "glue code" between SSH and TUI layers
- ✅ Cleaner Elm architecture for UI state management

#### 4. **Developer Experience**
- ✅ Go's simpler syntax and faster compile times
- ✅ Easier for contributors to onboard (Go more widely known)
- ✅ Rich ecosystem of Charm.sh components (Bubbles, Lip Gloss, etc.)
- ✅ Better SSH app examples and tutorials

#### 5. **Deployment**
- ✅ Single static binary (like Rust)
- ✅ Excellent cross-compilation support
- ✅ Lower memory footprint (Go vs Rust is comparable)

### Cons of Migration to Go + Charm.sh

#### 1. **Rewrite Effort**
- ❌ ~2-3 weeks of full-time development
- ❌ Need to port ~12,000 lines of working code
- ❌ Risk of introducing bugs during migration
- ❌ Testing and validation required for all components

#### 2. **Loss of Rust's Type Safety**
- ❌ Rust's borrow checker catches memory bugs at compile time
- ❌ Go's error handling is more manual (`if err != nil`)
- ❌ Less compile-time guarantees overall
- ❌ Runtime panics vs Rust's safer Result/Option types

#### 3. **Performance Considerations**
- ❌ Rust is generally faster for CPU-bound tasks
- ❌ Go's garbage collector adds latency (though minimal for this use case)
- ❌ Poker hand evaluation might be slightly slower (depends on library)

#### 4. **Current Investment**
- ❌ ~12,000 LOC of working Rust code
- ❌ Existing dependencies and configurations
- ❌ Familiarity with Rust ecosystem
- ❌ Sunk cost in current architecture

#### 5. **Poker Library Ecosystem**
- ❌ Need to find or write poker hand evaluator in Go
- ❌ Rust's `poker` crate (0.4.1) is well-tested - may need to port or find equivalent

---

## Architectural Comparison

### Current Rust Flow

```
SSH Client
    ↓
russh Server (SshSessionHandler)
    ↓
SSH-TUI Bridge (manual event mapping)
    ↓
ratatui TUI (PokerApp + Views)
    ↓
poker-engine (Game logic)
    ↓
sqlx Database
```

### Proposed Go + Charm.sh Flow

```
SSH Client
    ↓
Wish Server (middleware architecture)
    ↓
    ├── bubbletea.Middleware (automatic SSH-TUI bridge)
    ├── logging.Middleware
    └── auth.Middleware
    ↓
Bubble Tea App (Elm architecture)
    ↓
Poker Engine (ported from Rust)
    ↓
database/sql or GORM
```

**Key Difference:** The SSH-TUI bridge layer is eliminated. Wish's `bubbletea.Middleware` automatically connects SSH sessions to Bubble Tea programs.

---

## Migration Strategy (If Proceeding)

### Phase 1: Proof of Concept (3-5 days)
**Goal:** Validate that Charm.sh meets requirements

1. Create minimal Go project with Wish + Bubble Tea
2. Implement basic SSH server with simple TUI (login screen)
3. Verify session isolation works correctly
4. Test with multiple concurrent SSH clients
5. Implement basic casino-themed styling with Lip Gloss

**Deliverable:** Working SSH server with simple TUI demonstrating the architecture

**Decision Point:** If PoC is successful, proceed to Phase 2. Otherwise, stick with Rust.

### Phase 2: Core Game Engine (4-6 days)
**Goal:** Port poker game logic

1. Port card, deck, and hand evaluation
   - Research Go poker libraries or port Rust logic
2. Port FSM (state machine)
3. Port betting system and pot management
4. Port player management
5. Write unit tests for all game logic

**Deliverable:** Working poker engine with test coverage

### Phase 3: Database Layer (2-3 days)
**Goal:** Implement persistence

1. Set up database/sql or GORM with SQLite/PostgreSQL
2. Port user models and operations
3. Port authentication (Argon2 password hashing)
4. Implement session management
5. Migration scripts for schema

**Deliverable:** Database layer with auth and session management

### Phase 4: TUI Implementation (5-7 days)
**Goal:** Build casino-themed UI

1. Design Elm architecture for PokerApp
   - Init, Update, View functions
2. Implement views:
   - AuthView (login/register)
   - LobbyView (table listing)
   - GameView (poker table)
3. Build poker table renderer with Lip Gloss
4. Integrate Bubbles components (text input, tables)
5. Casino theming and styling
6. Event handling (keyboard input)

**Deliverable:** Full TUI with all views

### Phase 5: AI Bots (2-3 days)
**Goal:** Port AI players

1. Port bot strategies and personalities
2. Port hand strength evaluator
3. Port opponent modeling
4. Integration with game engine

**Deliverable:** Working AI bots

### Phase 6: Integration & Testing (3-4 days)
**Goal:** Connect all pieces

1. Wire up Wish middleware to Bubble Tea app
2. Implement game orchestration (table management)
3. Multi-player session handling
4. Integration testing with multiple clients
5. Load testing (9 players × 5 tables)

**Deliverable:** Fully integrated system

### Phase 7: Observability & Deployment (2-3 days)
**Goal:** Production readiness

1. Prometheus metrics integration
2. Logging setup
3. Health check endpoints
4. Docker containerization
5. Deployment scripts

**Deliverable:** Production-ready application

### Total Estimated Timeline: 21-31 days (3-4.5 weeks)

---

## Incremental Approach (Recommended)

Instead of a full rewrite, consider a **hybrid approach**:

### Option A: Rust + Charm-inspired TUI
- Keep Rust backend (SSH server, game engine, database)
- Redesign TUI to follow Elm architecture principles
- Keep current russh + ratatui stack
- Improve SSH-TUI bridge with cleaner event handling
- **Effort:** 1-2 weeks
- **Risk:** Low
- **Benefit:** Better architecture without full rewrite

### Option B: Go SSH Server + Rust Game Engine
- Migrate SSH server to Wish + Bubble Tea
- Keep poker engine in Rust, expose as a library (FFI)
- Call Rust from Go via cgo or gRPC
- **Effort:** 2-3 weeks
- **Risk:** Medium
- **Benefit:** Best of both worlds (Charm.sh for SSH/TUI, Rust for game logic)

### Option C: Full Migration (as described above)
- Complete rewrite to Go
- **Effort:** 3-4.5 weeks
- **Risk:** High
- **Benefit:** Unified codebase, simpler architecture

---

## Code Comparison: Rust vs Go + Charm.sh

### SSH Server Setup

**Current Rust (russh):**
```rust
// ~100+ LOC to set up SSH server
let config = russh::server::Config {
    inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
    auth_rejection_time: std::time::Duration::from_secs(1),
    keys: vec![russh_keys::key::KeyPair::generate_ed25519().unwrap()],
    ..Default::default()
};

let sh = SshServer {
    server_state: Arc::new(Mutex::new(ServerState::new())),
};

russh::server::run(Arc::new(config), "0.0.0.0:2222", sh).await?;
```

**Proposed Go (Wish):**
```go
// ~20 LOC for SSH server with TUI
s, err := wish.NewServer(
    wish.WithAddress(":2222"),
    wish.WithHostKeyPath(".ssh/poker_host_key"),
    wish.WithMiddleware(
        bubbletea.Middleware(func(s ssh.Session) (tea.Model, []tea.ProgramOption) {
            return NewPokerApp(s), []tea.ProgramOption{
                tea.WithAltScreen(),
                tea.WithMouseCellMotion(),
            }
        }),
        logging.Middleware(),
    ),
)
if err != nil {
    log.Fatal(err)
}

log.Fatal(s.ListenAndServe())
```

### TUI App Structure

**Current Rust (ratatui):**
```rust
// Separate event loop, manual SSH integration
pub struct PokerApp {
    state: AppState,
    current_view: ViewType,
    event_receiver: mpsc::Receiver<AppEvent>,
}

impl PokerApp {
    pub async fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if let Ok(event) = self.event_receiver.try_recv() {
                self.handle_event(event).await?;
            }
        }
    }
}
```

**Proposed Go (Bubble Tea):**
```go
// Clean Elm architecture, automatic SSH integration via Wish
type PokerApp struct {
    state       AppState
    currentView ViewType
}

func (m PokerApp) Init() tea.Cmd {
    return nil // Initialize
}

func (m PokerApp) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
    // Handle events (keyboard, etc.)
    switch msg := msg.(type) {
    case tea.KeyMsg:
        return m.handleKeyPress(msg)
    }
    return m, nil
}

func (m PokerApp) View() string {
    // Render UI (returns string)
    return m.renderCurrentView()
}
```

**Key Difference:** Bubble Tea's architecture is cleaner and more declarative. No manual event loop required.

---

## Recommendation

### If you want production-ready SSH poker game ASAP:
**Stick with Rust** and focus on:
- Completing SSH-TUI integration
- Polishing the existing architecture
- Adding features (more bot strategies, table limits, tournaments, etc.)

**Timeline:** 1-2 weeks to production
**Risk:** Low

### If you want to leverage Charm.sh ecosystem and simplify long-term maintenance:
**Migrate to Go + Charm.sh** using the phased approach

**Timeline:** 3-4.5 weeks to production
**Risk:** Medium

### If you want a middle ground:
**Option A (Rust + Charm-inspired architecture)** - refactor TUI to Elm-style without changing languages

**Timeline:** 1-2 weeks
**Risk:** Low

---

## Next Steps

1. **Decision:** Evaluate pros/cons and choose an approach
2. **Proof of Concept:** If migrating to Go, build a minimal PoC first (Phase 1)
3. **Validate Assumptions:** Test Wish + Bubble Tea with multiple concurrent SSH sessions
4. **Commit:** If PoC is successful, proceed with full migration
5. **Iterate:** Use phased approach to minimize risk

---

## Questions to Consider

1. **Timeline:** Do you have 3-4.5 weeks for a full rewrite, or do you need something production-ready sooner?
2. **Team:** Is your team more familiar with Rust or Go?
3. **Performance:** Is raw performance critical, or is developer productivity more important?
4. **Maintenance:** Who will maintain this long-term? (Go may be easier to onboard new contributors)
5. **Features:** Are there missing features in the Rust version that would require significant effort anyway?
6. **Ecosystem:** Do you want to leverage Charm.sh's ecosystem (Soft Serve, Glow, etc.) for future integrations?

---

## Resources

### Charm.sh Documentation
- Wish: https://github.com/charmbracelet/wish
- Bubble Tea: https://github.com/charmbracelet/bubbletea
- Bubbles: https://github.com/charmbracelet/bubbles
- Lip Gloss: https://github.com/charmbracelet/lipgloss

### Examples
- Soft Serve (Git SSH server): https://github.com/charmbracelet/soft-serve
- Glow (Markdown reader): https://github.com/charmbracelet/glow
- SSH Examples: https://github.com/charmbracelet/wish/tree/main/examples

### Go Poker Libraries
- https://github.com/loganjspears/joker (Hand evaluation)
- https://github.com/chehsunliu/poker (Alternative)

---

**Created:** 2025-11-21
**Author:** Claude Code
**Status:** Planning Document
