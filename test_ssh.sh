#!/bin/bash

echo "Starting SSH Poker Server with Casino TUI..."
echo "============================================"
echo ""
echo "Server will run on localhost:2222"
echo ""
echo "To connect:"
echo "  ssh -p 2222 localhost"
echo ""
echo "Default test credentials:"
echo "  Username: test"
echo "  Password: test123"
echo ""
echo "Controls in game:"
echo "  • 1 or F1: Toggle between Lobby and Game"
echo "  • WASD or Arrows: Navigate"
echo "  • Enter: Select action"
echo "  • Esc: Go back"
echo "  • q: Quit"
echo ""
echo "Press Ctrl+C to stop the server"
echo "============================================"
echo ""

# Run the SSH server with casino TUI
cargo run --bin ssh-poker-server -- --address 0.0.0.0 --port 2222