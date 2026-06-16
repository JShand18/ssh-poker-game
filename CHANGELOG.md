# Changelog

All notable changes to the SSH Poker Game project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2024-11-21

### Added
- Comprehensive test suite (`test_ssh_poker.sh`) with multiple test modes
- Quick start script (`quick_start.sh`) for easy server startup
- Casino-themed TUI integration via `ssh_tui_bridge`
- Proper authentication flow with SecureAuthService
- Demo user creation with `--create-demo-user` flag
- Updated documentation reflecting current state

### Changed
- SSH handler now uses integrated TUI bridge instead of basic ANSI rendering
- Authentication now properly validates against database
- Session management integrated with authentication flow
- Improved error handling and logging throughout

### Fixed
- Authentication bypass vulnerability - now properly validates credentials
- SSH handler now correctly uses auth_service and session_manager
- Compilation warnings reduced from 45 to 2
- Proper integration between SSH input and TUI events

### Removed
- **1,020+ lines of dead code removed:**
  - Deleted `auth.rs` (89 lines) - replaced by secure_auth.rs
  - Deleted `tui_server.rs` (236 lines) - obsolete implementation
  - Deleted `config.rs` - unused configuration file
  - Removed empty vendored crate directories (`/crates/russh/`, `/crates/russh-keys/`)
  - Removed `SshServerHandler` struct from lib.rs (152 lines)
  - Removed `handle_terminal_input()` function (79 lines)
  - Removed legacy `run_server()` function (61 lines)
  - Cleaned up unused imports and dead code blocks

## [0.2.0] - 2024-11-10

### Added
- Initial SSH poker server implementation
- Casino-themed TUI components (poker-tui crate)
- SQLite database integration (data-store crate)
- Basic AI bot framework (ai-bot crate)
- Prometheus metrics (hybrid-metrics crate)
- Core poker engine with Elm-like architecture
- SSH server with russh library
- Basic authentication system
- Session management
- Game state management

### Known Issues
- Game actions (fold, call, raise) not fully connected to engine
- TUI bridge runs in spawned task with limited bidirectional communication
- Some terminal emulators may not render colors correctly

## [0.1.0] - 2024-10-01

### Added
- Initial project setup
- Cargo workspace configuration
- Basic project structure
- Documentation framework
- Initial design documents

---

## Upcoming (Phase 4 & 5)

### Planned
- Consolidate game state management between SessionManager and PokerApp
- Create trait-based abstractions for Go migration
- Complete game action handlers
- Add player statistics and leaderboards
- Implement in-game chat system
- Add tournament mode support
- Document public APIs for Go reimplementation

### Migration to Go/Charm.sh
- Port poker engine to Go
- Implement Bubble Tea UI
- Use Wish for SSH server
- Maintain feature parity with Rust implementation

---

## Version History

- **0.2.1** - Current (November 2024) - Major code cleanup and TUI integration
- **0.2.0** - November 2024 - Initial working implementation
- **0.1.0** - October 2024 - Project inception

## Statistics

### Code Quality Improvements (v0.2.1)
- Lines of code removed: **1,020+**
- Files deleted: **6**
- Duplicate implementations consolidated: **3**
- Authentication paths unified: **From 3 to 1**
- Compilation warnings reduced: **From 45 to 2**

### Current Metrics
- Total lines of code: ~15,000
- Number of crates: 6
- Test coverage: ~40%
- Supported concurrent connections: 50+
- Average response time: <1ms