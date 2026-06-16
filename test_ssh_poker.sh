#!/bin/bash

# SSH Poker Game Test Script
# Tests the SSH poker server functionality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SSH_PORT=2222

# Ensure we're in the right directory
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Use absolute paths
DB_FILE="${SCRIPT_DIR}/test_poker.db"
LOG_FILE="${SCRIPT_DIR}/test_server.log"
PID_FILE="${SCRIPT_DIR}/test_server.pid"

# Functions
print_status() {
    echo -e "${BLUE}[*]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

cleanup() {
    print_status "Cleaning up..."

    # Stop server if running
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if kill -0 "$PID" 2>/dev/null; then
            print_status "Stopping server (PID: $PID)..."
            kill "$PID"
            sleep 2
            if kill -0 "$PID" 2>/dev/null; then
                print_warning "Server still running, forcing stop..."
                kill -9 "$PID"
            fi
        fi
        rm -f "$PID_FILE"
    fi

    # Clean up test files
    rm -f "$DB_FILE"
    rm -f "$LOG_FILE"

    print_success "Cleanup complete"
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

# Main test script
main() {
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║         SSH Poker Game - Server Test Suite              ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo

    # Clean any previous test artifacts
    cleanup 2>/dev/null || true

    # Ensure we can write to current directory
    touch "$DB_FILE" 2>/dev/null && rm -f "$DB_FILE"
    if [ $? -ne 0 ]; then
        print_error "Cannot write to current directory. Check permissions."
        exit 1
    fi

    # Step 1: Build the project
    print_status "Building SSH poker server..."
    if cargo build --bin ssh-poker-server 2>&1 | tee build.log | grep -q "Finished"; then
        print_success "Build successful"
    else
        print_error "Build failed. Check build.log for details"
        exit 1
    fi

    # Step 2: Start the server in background
    print_status "Starting SSH poker server on port $SSH_PORT..."
    RUST_LOG=info cargo run --bin ssh-poker-server -- \
        --port $SSH_PORT \
        --database "$DB_FILE" \
        --create-demo-user \
        > "$LOG_FILE" 2>&1 &

    SERVER_PID=$!
    echo $SERVER_PID > "$PID_FILE"

    # Wait for server to start
    print_status "Waiting for server to start..."
    for i in {1..10}; do
        if nc -z localhost $SSH_PORT 2>/dev/null; then
            print_success "Server is listening on port $SSH_PORT (PID: $SERVER_PID)"
            break
        fi
        if [ $i -eq 10 ]; then
            print_error "Server failed to start. Check $LOG_FILE for details"
            tail -20 "$LOG_FILE"
            exit 1
        fi
        sleep 1
    done

    # Step 3: Test SSH connection
    print_status "Testing SSH connectivity..."

    # Test 1: Anonymous connection (should be allowed)
    print_status "Test 1: Anonymous connection..."
    if timeout 5 ssh -p $SSH_PORT -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
        -o PasswordAuthentication=no -o PubkeyAuthentication=no \
        guest@localhost exit 2>/dev/null; then
        print_success "Anonymous connection successful"
    else
        print_warning "Anonymous connection failed (expected if auth is enforced)"
    fi

    # Test 2: Demo user connection
    print_status "Test 2: Demo user authentication..."
    if command -v sshpass &> /dev/null; then
        if timeout 5 sshpass -p "demo123" ssh -p $SSH_PORT \
            -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
            demo@localhost exit 2>/dev/null; then
            print_success "Demo user authentication successful"
        else
            print_error "Demo user authentication failed"
        fi
    else
        print_warning "sshpass not installed, skipping password auth test"
        print_status "Install sshpass with: brew install hudochenkov/sshpass/sshpass (macOS) or apt install sshpass (Linux)"
    fi

    # Step 4: Check server logs for errors
    print_status "Checking server logs..."
    if grep -q "ERROR\|PANIC" "$LOG_FILE"; then
        print_warning "Errors found in server logs:"
        grep "ERROR\|PANIC" "$LOG_FILE" | head -5
    else
        print_success "No errors in server logs"
    fi

    # Step 5: Test metrics endpoint
    print_status "Testing metrics endpoint..."
    if curl -s http://localhost:9090/metrics > /dev/null 2>&1; then
        print_success "Metrics endpoint is accessible"
    else
        print_warning "Metrics endpoint not accessible (may be disabled)"
    fi

    # Step 6: Database verification
    print_status "Verifying database..."
    if [ -f "$DB_FILE" ]; then
        print_success "Database file created: $DB_FILE"

        # Check if we can query the database
        if command -v sqlite3 &> /dev/null; then
            TABLE_COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';" 2>/dev/null || echo "0")
            print_status "Database contains $TABLE_COUNT tables"
        fi
    else
        print_error "Database file not created"
    fi

    # Step 7: Performance check
    print_status "Checking server performance..."
    if kill -0 "$SERVER_PID" 2>/dev/null; then
        CPU_USAGE=$(ps -p "$SERVER_PID" -o %cpu= | tr -d ' ')
        MEM_USAGE=$(ps -p "$SERVER_PID" -o %mem= | tr -d ' ')
        print_status "CPU Usage: ${CPU_USAGE}%"
        print_status "Memory Usage: ${MEM_USAGE}%"

        if (( $(echo "$CPU_USAGE > 50" | bc -l) )); then
            print_warning "High CPU usage detected"
        else
            print_success "CPU usage is normal"
        fi
    fi

    echo
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║                    Test Summary                         ║"
    echo "╚══════════════════════════════════════════════════════════╝"

    # Summary
    print_success "Server is running successfully"
    print_status "Server PID: $SERVER_PID"
    print_status "Log file: $LOG_FILE"
    print_status "Database: $DB_FILE"
    echo
    print_status "To connect manually, use:"
    echo "    ssh -p $SSH_PORT demo@localhost  (password: demo123)"
    echo "    ssh -p $SSH_PORT guest@localhost (anonymous)"
    echo
    print_status "To view logs:"
    echo "    tail -f $LOG_FILE"
    echo
    print_status "To stop the server:"
    echo "    kill $SERVER_PID"
    echo
}

# Interactive test mode
interactive_test() {
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║            SSH Poker - Interactive Test                 ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo
    print_status "Starting interactive test session..."
    print_status "This will connect you to the poker server"
    echo

    read -p "Username (default: demo): " USERNAME
    USERNAME=${USERNAME:-demo}

    print_status "Connecting to SSH poker server as $USERNAME..."
    ssh -p $SSH_PORT -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null "$USERNAME@localhost"
}

# Parse arguments
case "${1:-}" in
    "interactive")
        interactive_test
        ;;
    "clean")
        cleanup
        ;;
    *)
        main
        ;;
esac