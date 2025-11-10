# Simplified SSH Poker Architecture

## Overview

We've dramatically simplified the SSH poker server architecture by removing complex event channels and the TUI bridge abstraction. The new approach follows the Charm.sh pattern of direct terminal rendering.

## Architecture Changes

### Before (Complex)
```
SSH Connection → SSH Handler → Event Channel → TUI Bridge → PokerApp → ratatui Backend → ANSI Output
```

### After (Simplified)
```
SSH Connection → SSH Handler → Direct ANSI Rendering
```

## Key Components

### 1. GameState Enum
Simple state machine for tracking the current screen:
- `Welcome` - Initial menu screen
- `Login` - Login form
- `Register` - Registration form  
- `Lobby` - Table selection
- `InGame` - Poker table view

### 2. Direct Rendering
Each state has a corresponding `render_*` method that returns a formatted ANSI string with:
- Unicode box drawing characters (╔═╗║╚╝)
- Emoji for visual appeal (🎰🔐📝🎲)
- Clear screen and cursor positioning
- Direct character output

### 3. Input Handling
- Character-by-character processing
- Direct state transitions
- Immediate re-rendering on state change
- No event queues or channels

## Benefits

1. **Simplicity**: ~300 lines vs ~1000+ lines of code
2. **Performance**: No async overhead, direct I/O
3. **Debuggability**: Easy to trace execution flow
4. **Maintainability**: Clear state transitions

## Example Flow

1. User connects via SSH
2. `shell_request` sends initial welcome screen
3. User presses 'G' for guest
4. `handle_input` transitions to `Lobby` state
5. `render()` generates new screen
6. Output sent directly to SSH session

## Next Steps

With this simplified foundation, we can now:
1. Enhance the rendering with rich poker table visuals
2. Add smooth animations using cursor positioning
3. Implement actual game logic
4. Apply casino color themes