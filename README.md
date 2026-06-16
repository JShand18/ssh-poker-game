# 🎰 SSH Poker Game - Casino Terminal Experience

A production-ready SSH-accessible multiplayer Texas Hold'em poker game with a rich casino-themed terminal UI, implemented in Rust.

## ✨ Recent Updates (November 2024)

- **Major Code Cleanup**: Removed 1,000+ lines of dead code and consolidated duplicate implementations
- **Casino TUI Integration**: Rich terminal interface with casino styling now fully integrated
- **Enhanced Security**: Proper authentication with SecureAuthService and database-backed sessions
- **Improved Architecture**: Cleaner separation of concerns and preparation for Go/Charm.sh migration

## 🎮 Overview

This project implements a feature-rich, secure poker game server accessible via SSH. Players connect using any SSH client to enjoy a casino-style poker experience directly in their terminal.

### 🌟 Key Features

- **🔐 SSH Access**: Secure connection via any SSH client
- **🎨 Casino TUI**: Beautiful casino-themed terminal interface using ratatui
- **👥 Multiplayer**: Support for multiple concurrent tables and players
- **🤖 AI Bots**: Intelligent computer players with configurable difficulty
- **💾 Database Backend**: SQLite for persistent storage and user management
- **🔒 Security**: Argon2 password hashing, rate limiting, and session management
- **📊 Metrics**: Prometheus metrics endpoint for monitoring

## 📁 Project Structure

```
ssh-poker-game/
├── crates/                      # Workspace members
│   ├── poker-engine/            # Core game logic (Elm-like architecture)
│   ├── ssh-poker-server/        # SSH server with integrated Casino TUI
│   │   ├── src/
│   │   │   ├── lib.rs          # Main server entry point
│   │   │   ├── ssh_handler.rs  # SSH session handler with TUI bridge
│   │   │   ├── ssh_tui_bridge.rs # Bridge between SSH and poker-tui
│   │   │   ├── secure_auth.rs  # Database-backed authentication
│   │   │   ├── session.rs      # Session and table management
│   │   │   └── error.rs        # Error types
│   ├── poker-tui/               # Casino-themed terminal UI components
│   ├── data-store/              # Database layer (SQLite via sqlx)
│   ├── ai-bot/                  # AI opponent implementation
│   └── hybrid-metrics/          # Prometheus/Datadog monitoring
├── docs/                        # Comprehensive documentation
├── test_ssh_poker.sh           # Comprehensive test suite
├── quick_start.sh              # Quick server startup script
├── MIGRATION_PLAN.md           # Go/Charm.sh migration roadmap
└── README.md                   # This file
```

## 🚀 Getting Started

### Prerequisites

- **Rust 1.75+** - Install from [rustup.rs](https://rustup.rs/)
- **SSH client** - OpenSSH, PuTTY, or any compatible client
- **Terminal** - Minimum size 80x24 with ANSI color support

### 🎯 Quick Start (Recommended)

```bash
# Clone the repository
git clone <your-repo-url>
cd ssh-poker-game

# Run the quick start script
./quick_start.sh
```

This will:
- Build the project in release mode
- Create a SQLite database
- Create a demo user (demo/demo123)
- Start the server on port 2222

### 🔌 Connecting to the Server

```bash
# Connect as demo user (password: demo123)
ssh -p 2222 demo@localhost

# Connect as anonymous guest
ssh -p 2222 guest@localhost
```

### 🛠️ Manual Setup

#### Build the Project

```bash
# Debug build (faster compilation, slower runtime)
cargo build --bin ssh-poker-server

# Release build (optimized for production)
cargo build --release --bin ssh-poker-server
```

#### Run the Server

```bash
# Basic server startup
cargo run --bin ssh-poker-server

# With custom options
cargo run --bin ssh-poker-server -- \
  --port 2222 \
  --address 0.0.0.0 \
  --database poker_game.db \
  --create-demo-user
```

#### Server Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--port` | `-p` | 2222 | SSH server port |
| `--address` | `-a` | 0.0.0.0 | Bind address |
| `--database` | `-d` | poker_game.db | SQLite database path |
| `--create-demo-user` | | false | Create demo user on startup |
| `--verbose` | `-v` | false | Enable debug logging |

## 🧪 Testing

### Run the Test Suite

```bash
# Run comprehensive test suite
./test_ssh_poker.sh

# Interactive test mode
./test_ssh_poker.sh interactive

# Clean up test artifacts
./test_ssh_poker.sh clean
```

### Run Unit Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p poker-engine
cargo test -p ssh-poker-server

# With output
cargo test -- --nocapture
```

## 🎮 Gameplay

### In-Game Controls

Once connected via SSH:

#### Navigation
- **Arrow Keys** or **W/A/S/D** - Navigate menus
- **Enter** - Select/Confirm
- **Esc** - Go back
- **Q** - Quit to main menu

#### Poker Actions
- **F** - Fold
- **C** - Call/Check
- **R** - Raise
- **A** - All-in

#### Lobby
- **1-9** - Join table by number
- **N** - Create new table

## 🔧 Development

### Building Components

```bash
# Build everything
cargo build --workspace

# Build specific crate
cargo build -p poker-engine
cargo build -p poker-tui
cargo build -p ssh-poker-server
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy linter
cargo clippy --workspace --all-features

# Check for compilation errors
cargo check --workspace

# Security audit
cargo audit
```

### Monitoring

When the server is running, metrics are available at:

```bash
# Prometheus metrics
curl http://localhost:9090/metrics

# Health check
curl http://localhost:9090/health
```

### Environment Variables

```bash
# Set log level
RUST_LOG=debug cargo run --bin ssh-poker-server

# Enable backtrace for debugging
RUST_BACKTRACE=1 cargo run --bin ssh-poker-server
```

## 📚 Documentation

Comprehensive documentation is available in the `docs/` directory:

- [Executive Summary](docs/executive_summary.md) - High-level overview
- [Current State](docs/current_state_summary.md) - Implementation status
- [Architecture](docs/simplified_architecture.md) - System architecture
- [Migration Plan](MIGRATION_PLAN.md) - Go/Charm.sh migration roadmap

### Learning Resources

- [Book Mappings](docs/book_mappings/README.md) - Code examples mapped to programming books
- [Gameplay Guide](GAMEPLAY.md) - Detailed poker rules and gameplay

## 🏗️ Architecture Highlights

### Clean Architecture
- **Elm-like game engine** - Functional, immutable game state management
- **Casino TUI** - Rich terminal interface with themes and animations
- **SSH Bridge** - Seamless integration between SSH and TUI events

### Security Features
- **Argon2** password hashing
- **Rate limiting** on authentication attempts
- **Session management** with automatic cleanup
- **Database-backed** user authentication

### Recent Improvements (November 2024)
- Removed 1,020+ lines of dead code
- Consolidated 3 authentication systems into 1
- Integrated casino-themed TUI via ssh_tui_bridge
- Fixed authentication to use SecureAuthService properly
- Created comprehensive test infrastructure

## 🚦 Troubleshooting

### Port Already in Use
```bash
# Check what's using port 2222
lsof -i :2222

# Kill process on port
kill $(lsof -t -i:2222)
```

### Connection Issues
```bash
# Test with verbose SSH output
ssh -vvv -p 2222 localhost

# Check server logs
RUST_LOG=debug ./quick_start.sh
```

### Database Issues
```bash
# Connect to SQLite database
sqlite3 poker_game.db

# Show tables
.tables

# Check users
SELECT * FROM users;
```

## 🤝 Contributing

Contributions are welcome! Please ensure:
1. Code passes `cargo fmt` and `cargo clippy`
2. Tests pass with `cargo test`
3. Documentation is updated for new features

## 📜 License

MIT License - See LICENSE file for details

## 🙏 Acknowledgments

- [russh](https://github.com/warp-tech/russh) - Rust SSH library
- [ratatui](https://ratatui.rs) - Terminal UI framework
- [tokio](https://tokio.rs) - Async runtime
- [sqlx](https://github.com/launchbadge/sqlx) - Async SQL toolkit

## 📊 Project Status

**Current Phase**: Code cleanup complete, TUI integrated, preparing for state consolidation

**Next Steps**:
- Phase 4: Consolidate game state management
- Phase 5: Prepare for Go/Charm.sh migration

See the [Migration Plan](MIGRATION_PLAN.md) for the roadmap to Go with Charm.sh.