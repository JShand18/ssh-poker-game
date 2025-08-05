# Terminal-Based Multiplayer Poker Game

A production-ready SSH-accessible terminal-based multiplayer Texas Hold'em poker game implemented in Rust.

## Overview

This project implements a feature-rich, secure, and high-performance poker game server that players can access via SSH. The system supports multiple concurrent poker tables, AI bots with configurable difficulty levels, and comprehensive security features.

### Key Features

- **SSH Access**: Connect to the game server securely via any SSH client
- **Terminal UI**: Beautiful terminal-based user interface using ratatui
- **Multiplayer**: Support for up to 5 concurrent poker tables with 9 players each
- **AI Bots**: Intelligent computer players with configurable difficulty levels
- **Database Backend**: PostgreSQL for persistent storage of user accounts and game statistics
- **Security**: Comprehensive security measures including encryption, anti-cheat mechanisms, and audit logging
- **Performance**: Optimized for low latency and high throughput

## Project Structure

```
poker-game/
├── .github/
│   └── ISSUE_TEMPLATE/       # GitHub issue templates
├── src/
│   ├── game/                 # Core game logic
│   ├── ui/                   # Terminal user interface
│   ├── server/               # SSH server implementation
│   ├── db/                   # Database integration
│   ├── ai/                   # AI bot implementation
│   ├── security/             # Security implementations
│   └── main.rs               # Application entry point
├── tests/                    # Tests
├── docs/                     # Documentation
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

- **Rust 1.70+** and Cargo (install from [rustup.rs](https://rustup.rs/))
- **SSH client** (OpenSSH, PuTTY, or any compatible client)
- **Terminal** with ANSI color support (most modern terminals)

### Quick Start (Recommended)

1. **Clone and run:**
   ```bash
   git clone <your-repo-url>
   cd ssh-poker-game
   ./run_server.sh
   ```

   The script will:
   - Build the project automatically
   - Create a SQLite database
   - Set up a demo user (username: `demo`, password: `demo123`)
   - Start the server on port 2222

2. **Connect and play:**
   ```bash
   ssh -p 2222 demo@localhost
   ```

### Manual Installation

1. **Build the project:**
   ```bash
   cargo build --release
   ```

2. **Start the server:**
   ```bash
   # Basic server (creates demo user on first run)
   ./target/release/ssh-poker-server --create-demo-user
   
   # Or with custom settings
   ./target/release/ssh-poker-server \
     --port 2222 \
     --address 0.0.0.0 \
     --database poker_game.db \
     --create-demo-user \
     --verbose
   ```

3. **Connect to play:**
   ```bash
   ssh -p 2222 demo@localhost
   # Use password: demo123
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

- `--port` - Server port (default: 2222)
- `--address` - Bind address (default: 0.0.0.0)
- `--database` - Database file path (default: poker_game.db)
- `--create-demo-user` - Create demo user on startup
- `--verbose` - Enable debug logging
## Acknowledgments

- [russh](https://github.com/Eugeny/russh) - Rust SSH client & server library
- [ratatui](https://ratatui.rs) - Terminal UI library for Rust
- [tokio](https://tokio.rs) - Asynchronous runtime for Rust
- [rs-poker](https://github.com/elliottneilclark/rs-poker) - Poker evaluation library

## Project Status

This project is currently in development. See the [Project Board](https://github.com/users/JShand18/projects/2) for current progress.
