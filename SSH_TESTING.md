# SSH Poker Server - Testing Guide

## Quick Start

### 1. Start the SSH Server
```bash
./test_ssh.sh
```

Or manually:
```bash
cargo run --bin ssh-poker-server -- --address 0.0.0.0 --port 2222
```

### 2. Connect via SSH
```bash
ssh -p 2222 localhost
```

### 3. Test Credentials
The server currently accepts any password for testing. Use:
- Username: `test` 
- Password: `test123`

## Casino TUI Controls

Once connected, you'll see the beautiful casino-themed interface with:
- **Dark red and forest green** color scheme (traditional casino colors)
- **Gold accents** for important UI elements
- **Traditional poker table layout**

### Navigation
- `1` or `F1`: Toggle between Lobby and Game views
- `W/A/S/D` or Arrow Keys: Navigate menus
- `Enter`: Select/confirm action
- `Esc`: Go back
- `q`: Quit/disconnect

### In-Game Actions
- `f`: Fold
- `c`: Call/Check
- `r`: Raise (default amount)
- `a`: All-in
- `n`: Create new table (in lobby)
- `1-9`: Join table by number (in lobby)

## Architecture Overview

The SSH server now integrates with the `charm-tui` crate which provides:
- Beautiful casino-style terminal UI
- Event-driven architecture (Bubble Tea inspired)
- Thread-safe View trait for SSH compatibility
- Traditional poker table layout with player positions

## Troubleshooting

### Connection Refused
- Ensure the server is running on port 2222
- Check that no other service is using that port
- Try `lsof -i :2222` to see if the port is in use

### Authentication Failed
- The server uses the `SecureAuthService` for authentication
- Currently configured for password authentication
- Public key auth is also supported but requires setup

### Display Issues
- Ensure your terminal supports ANSI colors
- Try a different terminal emulator if colors don't appear correctly
- The TUI requires a minimum terminal size of 80x24

## Development Notes

The SSH server uses:
- `russh` for SSH protocol handling
- `charm-tui` for the casino-themed terminal interface
- `data-store` for PostgreSQL database integration
- `hybrid-metrics` for Prometheus/Datadog monitoring

The main integration point is in `crates/ssh-poker-server/src/charm_handler.rs` which bridges SSH input/output with the charm-tui poker application.