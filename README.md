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

- Rust 1.70+ and Cargo
- PostgreSQL 14+
- OpenSSH or compatible SSH client

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/poker-game.git
   cd poker-game
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. Configure the database:
   ```
   cp config/database.example.toml config/database.toml
   # Edit config/database.toml with your PostgreSQL credentials
   ```

4. Start the server:
   ```
   ./target/release/poker-game
   ```

5. Connect to the server:
   ```
   ssh -p 2222 username@localhost
   ```

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for details on how to contribute to this project. All contributions are welcome!

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## Acknowledgments

- [russh](https://github.com/Eugeny/russh) - Rust SSH client & server library
- [ratatui](https://ratatui.rs) - Terminal UI library for Rust
- [tokio](https://tokio.rs) - Asynchronous runtime for Rust
- [rs-poker](https://github.com/elliottneilclark/rs-poker) - Poker evaluation library

## Project Status

This project is currently in development. See the [Project Board](https://github.com/yourusername/poker-game/projects/1) for current progress.
