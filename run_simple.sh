#!/bin/bash

# SSH Poker Game - Simple Server Script
set -e

echo "🎮 SSH Poker Game - Simple Version"
echo "================================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "✅ Rust found"

# Build just the poker-engine for now
echo "🔨 Building poker engine..."
cargo build --release -p poker-engine

if [ $? -ne 0 ]; then
    echo "❌ Build failed. Please check the error messages above."
    exit 1
fi

echo "✅ Build successful"

# Run the CLI version of poker for demonstration
echo ""
echo "🎮 Running poker CLI demo..."
echo ""
./target/release/poker-cli

echo ""
echo "👋 Demo finished!"