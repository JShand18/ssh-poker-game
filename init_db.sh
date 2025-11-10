#!/bin/bash

# Database initialization script for SSH Poker Game

set -e

echo "🗄️  Initializing SSH Poker Game Database"
echo "========================================"

DB_FILE="poker_game.db"

# Check if database already exists
if [ -f "$DB_FILE" ]; then
    echo "⚠️  Database file already exists: $DB_FILE"
    read -p "Do you want to reset it? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Removing existing database..."
        rm "$DB_FILE"
    else
        echo "Keeping existing database."
        exit 0
    fi
fi

echo "📝 Creating new SQLite database: $DB_FILE"

# Create empty database file
touch "$DB_FILE"

# Create a simple SQL script to initialize the database
cat > init_db.sql << 'EOF'
-- Users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    last_login TEXT
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_active ON users(is_active);

-- User sessions table
CREATE TABLE IF NOT EXISTS user_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Games table
CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    started_at TEXT,
    ended_at TEXT,
    state TEXT NOT NULL,
    pot_size INTEGER NOT NULL DEFAULT 0,
    current_round INTEGER NOT NULL DEFAULT 0
);

-- Game participants table
CREATE TABLE IF NOT EXISTS game_participants (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    chips INTEGER NOT NULL DEFAULT 1000,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    joined_at TEXT NOT NULL,
    left_at TEXT,
    FOREIGN KEY (game_id) REFERENCES games(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Player stats table
CREATE TABLE IF NOT EXISTS player_stats (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL UNIQUE,
    games_played INTEGER NOT NULL DEFAULT 0,
    games_won INTEGER NOT NULL DEFAULT 0,
    total_winnings INTEGER NOT NULL DEFAULT 0,
    total_losses INTEGER NOT NULL DEFAULT 0,
    highest_pot_won INTEGER NOT NULL DEFAULT 0,
    best_hand TEXT,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Game events table for audit logging
CREATE TABLE IF NOT EXISTS game_events (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL,
    user_id TEXT,
    event_type TEXT NOT NULL,
    event_data TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (game_id) REFERENCES games(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Insert test user (password: test123)
-- Using a simple hash for testing (in production, use proper password hashing)
INSERT INTO users (id, username, email, password_hash, created_at, updated_at, is_active)
VALUES (
    '550e8400-e29b-41d4-a716-446655440001',
    'test',
    'test@example.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY.Q8DtpefhDVqi', -- bcrypt hash of 'test123'
    datetime('now'),
    datetime('now'),
    1
);

-- Insert demo user (password: demo123)
INSERT INTO users (id, username, email, password_hash, created_at, updated_at, is_active)
VALUES (
    '550e8400-e29b-41d4-a716-446655440002',
    'demo',
    'demo@example.com',
    '$2b$12$5kJXD.0Q8RwB8tQpRXKZOu7xQNTqt3MH0VgGa3fYluXqAYqhBP2Oi', -- bcrypt hash of 'demo123'
    datetime('now'),
    datetime('now'),
    1
);

-- Insert admin user (password: admin123)
INSERT INTO users (id, username, email, password_hash, created_at, updated_at, is_active)
VALUES (
    '550e8400-e29b-41d4-a716-446655440003',
    'admin',
    'admin@example.com',
    '$2b$12$Yb0JGdGJL6uLfUy3YZrFNOZiZgTNPR/K6Sa8u0WKXGvYQs5.8eAVe', -- bcrypt hash of 'admin123'
    datetime('now'),
    datetime('now'),
    1
);

-- Initialize player stats for test users
INSERT INTO player_stats (id, user_id, updated_at)
VALUES 
    ('stat-001', '550e8400-e29b-41d4-a716-446655440001', datetime('now')),
    ('stat-002', '550e8400-e29b-41d4-a716-446655440002', datetime('now')),
    ('stat-003', '550e8400-e29b-41d4-a716-446655440003', datetime('now'));

SELECT 'Database initialized successfully!' as message;
EOF

# Run the SQL script
echo "🔧 Running database migrations..."
sqlite3 "$DB_FILE" < init_db.sql

# Clean up
rm init_db.sql

echo "✅ Database initialization complete!"
echo ""
echo "📊 Database Summary:"
sqlite3 "$DB_FILE" << EOF
.mode column
.headers on
SELECT COUNT(*) as user_count FROM users;
SELECT username, email, is_active FROM users;
EOF

echo ""
echo "🔐 Test Credentials:"
echo "  • Username: test    Password: test123"
echo "  • Username: demo    Password: demo123"
echo "  • Username: admin   Password: admin123"
echo ""
echo "✨ Database ready at: $DB_FILE"