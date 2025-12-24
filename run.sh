#!/bin/bash
set -e

# Default settings
# WATCH_PATH=${GUARDIAN_WATCH_PATH:-/}
WATCH_PATH=${GUARDIAN_WATCH_PATH:-$HOME/projects/Guardian}
# 
DB_PATH=${GUARDIAN_DB_PATH:-$HOME/.local/share/com.guardian.sentinel/guardian.db}

echo "🛡️  Guardian SIEM - Launcher"
echo "============================="
echo "👀 Watching: $WATCH_PATH"
echo "💾 Database: $DB_PATH"
echo ""

# Ensure release binaries exist
if [ ! -f "target/release/guardian-daemon" ] || [ ! -f "target/release/guardian-bridge" ]; then
    echo "❌ Binaries not found! Running build..."
    ./build.sh
fi

echo "🚀 Starting Daemon -> Bridge pipeline..."
export GUARDIAN_WATCH_PATH="$WATCH_PATH"
export GUARDIAN_DB_PATH="$DB_PATH"

# Start daemon piped to bridge in background
./target/release/guardian-daemon 2>&1 | ./target/release/guardian-bridge &
PID=$!

echo "✅ Pipeline active (PID: $PID)"
echo ""

if [ "$1" == "--ui" ]; then
    echo "🖥️  Starting UI..."
    ./target/release/guardian-sentinel
    
    # Kill pipeline when UI exits
    kill $PID
else
    echo "ℹ️  Run with --ui to start the dashboard"
    echo "   Pipeline running in background. Press Ctrl+C to stop."
    wait $PID
fi
