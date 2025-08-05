#!/bin/bash

# SSH Poker Game Server - Quick Start Script
set -e

echo "🎮 SSH Poker Game - Quick Start"
echo "================================"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "✅ Rust found"

# Build the project
echo "🔨 Building the project..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed. Please check the error messages above."
    exit 1
fi

echo "✅ Build successful"

# Check if demo user should be created
if [ ! -f "poker_game.db" ]; then
    echo "🆕 First run detected - creating demo user"
    CREATE_DEMO="--create-demo-user"
else
    CREATE_DEMO=""
fi

echo ""
echo "🚀 Starting SSH Poker Server..."
echo ""

# Run the server
./target/release/ssh-poker-server $CREATE_DEMO --verbose

echo ""
echo "👋 Server stopped. Thanks for playing!"