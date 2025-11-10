# SSH Poker Game - Architecture Consolidation Summary

## Overview

This document summarizes the architectural consolidation performed to simplify the codebase and remove duplicate implementations.

## Key Changes

### 1. TUI Consolidation

**Before:**
- `poker-tui` crate (formerly charm-tui) - Using ratatui + owo-colors
- `terminal-ui` crate - Using ratatui (older version)

**After:**
- Single `poker-tui` crate combining the best of both:
  - Casino theme system from poker-tui (dark red, forest green, gold)
  - Poker table rendering from terminal-ui
  - Modern dependencies (ratatui 0.28)
  - owo-colors for enhanced terminal styling

### 2. SSH Server Consolidation

**Before:**
- `ssh-server` crate - Using russh (generic name)
- `wish-server` crate - Also using russh (duplicate)
- `ssh-poker-server` crate - Just a binary wrapper

**After:**
- Single `ssh-poker-server` crate:
  - Better descriptive name
  - Contains all SSH functionality
  - Includes ssh_tui_bridge module for TUI integration
  - Uses russh for SSH protocol implementation
  - Nice CLI with clap and colored output

### 3. Technology Stack Clarification

**Important Discovery:** Charm.sh provides Go libraries (Bubble Tea, Wish), not Rust libraries. Our Rust implementation uses:
- **TUI**: `ratatui` - The most popular Rust TUI framework
- **SSH**: `russh` - Async SSH implementation for Rust
- **Styling**: `owo-colors` - Zero-cost terminal styling

This is actually a good architecture choice because:
- Maintains Rust's memory safety and performance benefits
- Uses mature, well-maintained Rust libraries
- Avoids cross-language complexity

## Remaining Work

The consolidation exposed that the SSH↔TUI integration is incomplete:

1. **SshSessionHandler** (formerly CharmSshHandler) has TODO: "Implement proper rendering of TUI to SSH terminal"
2. **ssh_tui_bridge.rs** shows placeholder text instead of actual TUI rendering
3. Need to implement `ratatui::backend::Backend` trait for SSH

## Next Steps

The main remaining work is to complete the SSH↔TUI integration:

1. Implement proper SSH backend for ratatui in `ssh_tui_bridge.rs`
2. Connect the poker table renderer to SSH sessions in `SshSessionHandler`
3. Remove placeholder rendering and show actual game
4. Test the complete SSH→TUI flow

These are tracked in the TODO system as:
- "Implement proper ratatui Backend trait for SSH in ssh_tui_bridge.rs"
- "Replace placeholder rendering in SshSessionHandler with actual TUI"

## Files Removed

- `/crates/charm-tui/` (renamed to poker-tui)
- `/crates/terminal-ui/` (merged into poker-tui)
- `/crates/wish-server/` (merged into ssh-poker-server)

## Dependencies Updated

All references updated from:
- `charm-tui` → `poker-tui` (renamed)
- `wish-server` → removed (merged into ssh-poker-server)
- `ssh-server` → `ssh-poker-server`
- `terminal-ui` → removed

## Benefits

1. **Reduced Complexity**: From 3 SSH/TUI crates to 2 focused crates
2. **Clear Architecture**: One TUI crate, one SSH crate
3. **No Duplication**: Single implementation for each concern
4. **Better Integration**: SSH and TUI modules now in proper locations