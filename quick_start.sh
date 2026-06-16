#!/bin/bash

# Quick Start Script for SSH Poker Game
# Builds and runs the server with demo user

set -e

echo "🎰 SSH Poker Game - Quick Start"
echo "================================"

# Build the project
echo "📦 Building SSH poker server..."
cargo build --release --bin ssh-poker-server

# Start the server
echo "🚀 Starting server on port 2222..."
echo "   Demo user: demo / demo123"
echo "   Connect with: ssh -p 2222 demo@localhost"
echo ""

# Get absolute path for database
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DB_FILE="${SCRIPT_DIR}/poker_game.db"

cargo run --release --bin ssh-poker-server -- \
    --port 2222 \
    --database "$DB_FILE" \
    --create-demo-user