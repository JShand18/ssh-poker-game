# Terminal-Based Multiplayer Poker Game

A production-ready SSH-accessible terminal-based multiplayer Texas Hold'em poker game implemented in Rust.

## Overview

This project implements a feature-rich, secure, and high-performance poker game server that players can access via SSH. The system supports multiple concurrent poker tables, AI bots with configurable difficulty levels, and comprehensive security features.

### Key Features

- **SSH Access**: Connect to the game server securely via any SSH client
- **Terminal UI**: Beautiful terminal-based user interface using ratatui
- **Multiplayer**: Support for up to 5 concurrent poker tables with 9 players each
- **AI Bots**: Intelligent computer players with configurable difficulty levels
- **Database Backend**: SQLite for persistent storage of user accounts and game statistics
- **Security**: Comprehensive security measures including encryption, anti-cheat mechanisms, and audit logging
- **Performance**: Optimized for low latency and high throughput

## Project Structure

```
ssh-poker-game/
├── crates/                   # Workspace members
│   ├── poker-engine/         # Core game logic
│   ├── ssh-poker-server/     # SSH poker server with Casino TUI
│   ├── poker-tui/            # Casino-themed terminal UI
│   ├── data-store/           # Database integration (SQLite)
│   ├── ai-bot/               # AI bot implementation
│   ├── hybrid-metrics/       # Prometheus/Datadog monitoring
│   ├── russh/                # SSH library (vendored)
│   └── russh-keys/           # SSH key handling (vendored)
├── docs/                     # Comprehensive documentation
├── test_ssh.sh               # Quick start script
├── run_server.sh             # Server launch script
├── init_db.sh                # Database initialization
└── README.md                 # This file
```

## Development Plan

This project follows a structured development plan with the following epics:

1. **Project Setup & Foundation** (Week 1-2)
2. **Core Game Logic** (Week 2-4)
3. **Terminal UI Development** (Week 3-5)
4. **SSH Server Implementation** (Week 4-6)
5. **Multiplayer Architecture** (Week 5-7)
6. **Database Integration** (Week 6-8)
7. **AI Bot Development** (Week 7-9)
8. **Security Hardening** (Week 8-10)
9. **Testing & Quality Assurance** (Week 9-11)
10. **Deployment & Monitoring** (Week 10-12)

## Getting Started

### Prerequisites

- **Rust 1.75+** and Cargo (install from [rustup.rs](https://rustup.rs/))
- **SSH client** (OpenSSH, PuTTY, or any compatible client)
- **Terminal** with ANSI color support and minimum size 80x24
- **SQLite3** (automatically handled by the application)

### Quick Start (Recommended)

1. **Clone and run:**
   ```bash
   git clone <your-repo-url>
   cd ssh-poker-game
   ./test_ssh.sh
   ```

   The script will:
   - Build the project in release mode
   - Create a SQLite database (poker_game.db)
   - Start the server on port 2222 with Casino TUI

2. **Connect and play:**
   ```bash
   ssh -p 2222 localhost
   ```
   
   Use any username and password (e.g., `test`/`test123`) for testing.

### Manual Installation

1. **Build the project:**
   ```bash
   cargo build --release
   ```

2. **Start the server:**
   ```bash
   # Basic server
   ./target/release/ssh-poker-server
   
   # Or with custom settings
   ./target/release/ssh-poker-server \
     --port 2222 \
     --address 0.0.0.0 \
     --database poker_game.db
   ```

3. **Connect to play:**
   ```bash
   ssh -p 2222 localhost
   # Use any username/password for testing (e.g., test/test123)
   ```

### Creating Additional Users

To create more users, you can:

1. **Use the built-in registration** (when connecting via SSH)
2. **Add users via the database** (SQLite tools)
3. **Extend the server** with user management commands

### Server Options

```bash
./target/release/ssh-poker-server --help
```

- `--port` or `-p` - Server port (default: 2222)
- `--address` or `-a` - Bind address (default: 0.0.0.0)
- `--database` or `-d` - Database file path (default: poker_game.db)
- `--help` or `-h` - Show help message
## Acknowledgments

- [russh](https://github.com/Eugeny/russh) - Rust SSH client & server library
- [ratatui](https://ratatui.rs) - Terminal UI library for Rust
- [tokio](https://tokio.rs) - Asynchronous runtime for Rust
- [rs-poker](https://github.com/elliottneilclark/rs-poker) - Poker evaluation library

## Project Status

This project is currently in development. See the [Project Board](https://github.com/users/JShand18/projects/2) for current progress.

## Documentation

The project includes comprehensive documentation:

- [Executive Summary](docs/executive_summary.md) - High-level project overview
- [Requirements](docs/requirements.md) - Detailed feature requirements
- [Specifications](docs/specifications.md) - Technical specifications
- [Project Details](docs/project_details.md) - Full project plan with resource allocation
- [Implementation Strategy](docs/implementation_strategy.md) - Detailed implementation guide
- [Tasks](docs/tasks.md) - Development task breakdown
- [Design Improvements](docs/design_improvements.md) - Architectural improvement proposals
- [Cargo Configuration](docs/cargo_configuration.md) - Build and dependency management
- [Architecture Diagrams](docs/architecture_diagrams.md) - Visual system architecture with Mermaid diagrams
- [Architecture ASCII](docs/architecture_ascii.md) - ASCII-art architecture diagrams
- [Current State Summary](docs/current_state_summary.md) - Current implementation status

### 📚 Learning Resources

- [Book Mappings](docs/book_mappings/README.md) - Comprehensive mappings between foundational programming books and this codebase:
  - **The Rust Programming Language** - Chapter-by-chapter code mappings
  - **Design Patterns (Gang of Four)** - Pattern implementations and opportunities
  - **Rust for Embedded Systems** - Systems programming concepts in practice

# Command Reference
🎮 SSH Poker Game - Complete Command Reference

  🚀 Quick Start Scripts

  Primary Scripts

  # Start SSH server with Casino TUI (recommended)
  ./test_ssh.sh
  # Runs on port 2222 with the new casino-themed interface

  # Start SSH server with automatic setup
  ./run_server.sh
  # Builds release version, creates database, adds demo user

  # Run simple poker CLI demo
  ./run_simple.sh
  # Builds and runs just the poker engine CLI

  📦 Cargo Commands

  Running Binaries

  # Run SSH server with Casino TUI
  cargo run --bin ssh-poker-server

  # Run SSH server with arguments
  cargo run --bin ssh-poker-server -- --address 0.0.0.0 --port 2222

  # Run poker CLI demo
  cargo run --bin poker-cli

  Building

  # Build everything in debug mode
  cargo build

  # Build everything in release mode (optimized)
  cargo build --release

  # Build specific crate
  cargo build -p poker-engine
  cargo build -p ssh-poker-server
  cargo build -p poker-tui
  cargo build -p data-store
  cargo build -p hybrid-metrics
  cargo build -p ai-bot

  Testing

  # Run all tests
  cargo test

  # Run tests for specific crate
  cargo test -p poker-engine
  cargo test -p ssh-poker-server

  # Run tests with output
  cargo test -- --nocapture

  # Run specific test
  cargo test test_name

  Code Quality

  # Check for compilation errors without building
  cargo check

  # Check all workspace members
  cargo check --workspace

  # Format code
  cargo fmt

  # Format and check
  cargo fmt -- --check

  # Run clippy linter
  cargo clippy

  # Run clippy on everything
  cargo clippy --workspace --all-features --all-targets

  # Fix auto-fixable issues
  cargo fix

  # Clean build artifacts
  cargo clean

  Documentation

  # Build and open documentation
  cargo doc --open

  # Build docs for all dependencies
  cargo doc --no-deps

  # Build and open docs for specific crate
  cargo doc -p poker-engine --open

  🔧 Development Commands

  Database Operations

  # Connect to PostgreSQL (if using PostgreSQL backend)
  psql -U postgres -d poker_game

  # Connect to SQLite (if using SQLite)
  sqlite3 poker_game.db

  Monitoring & Metrics

  # Prometheus metrics are exposed on port 9090 when server runs
  curl http://localhost:9090/metrics

  # Check server health
  curl http://localhost:9090/health

  🎯 SSH Connection Commands

  Connecting to the Server

  # Basic connection (default port 2222)
  ssh -p 2222 localhost

  # With username
  ssh -p 2222 test@localhost

  # With verbose output (debugging)
  ssh -vvv -p 2222 localhost

  # Specify different host
  ssh -p 2222 user@your-server.com

  🎮 In-Game Commands (Once Connected)

  Navigation

  - 1 or F1 - Toggle between Lobby and Game
  - W/A/S/D or Arrow Keys - Navigate
  - Enter - Select/Confirm
  - Esc - Go back
  - q - Quit/Disconnect

  Game Actions

  - n - Create new table (in lobby)
  - 1-9 - Join table by number
  - f - Fold
  - c - Call/Check
  - r - Raise
  - a - All-in

  🛠️ Advanced Commands

  Running with Custom Configuration

  # Set environment variables
  RUST_LOG=debug cargo run --bin ssh-poker-server
  RUST_BACKTRACE=1 cargo run --bin ssh-poker-server

  # With custom database
  DATABASE_URL=postgres://user:pass@localhost/poker cargo run --bin ssh-poker-server

  # Enable Datadog metrics (if configured)
  DATADOG_ENABLED=true cargo run --bin ssh-poker-server

  Benchmarking

  # Run benchmarks
  cargo bench

  # Run specific benchmark
  cargo bench bench_name

  Dependency Management

  # Update dependencies
  cargo update

  # Add a new dependency
  cargo add tokio

  # Check for outdated dependencies
  cargo outdated

  # Audit for security vulnerabilities
  cargo audit

  📊 Workspace Information

  Available Crates

  - poker-engine - Core game logic
  - ssh-poker-server - SSH server with Casino TUI
  - poker-tui - Casino-themed terminal UI
  - data-store - Database integration
  - hybrid-metrics - Prometheus/Datadog monitoring
  - ai-bot - AI player implementation

  List All Workspace Members

  cargo metadata --no-deps --format-version 1 | jq -r '.workspace_members[]'

  🔍 Troubleshooting Commands

  # Check if port is in use
  lsof -i :2222

  # Kill process on port
  kill $(lsof -t -i:2222)

  # Check SSH server logs
  RUST_LOG=debug ./test_ssh.sh

  # Test database connection
  cargo run --bin ssh-poker-server -- --test-db-connection

  📝 Git Commands (for version control)

  # Check status
  git status

  # Stage changes
  git add .

  # Commit with message
  git commit -m "Update message"

  # Push to remote
  git push origin main

  This comprehensive list covers all the major commands for developing, testing, running, and maintaining the SSH Poker Game application. The most common workflow is to use ./test_ssh.sh to start the
  server and then connect with ssh -p 2222 localhost.
