#!/bin/bash
set -e

echo "🛡️  Guardian SIEM - Development Mode"
echo "===================================="

# Set default watch path if not set
export GUARDIAN_WATCH_PATH="${GUARDIAN_WATCH_PATH:-/tmp/guardian-test}"

echo ""
echo "📁 Watch path: $GUARDIAN_WATCH_PATH"
echo "   (Set GUARDIAN_WATCH_PATH to change)"

# Create watch directory if it doesn't exist
mkdir -p "$GUARDIAN_WATCH_PATH"

# Build daemon in debug mode
echo ""
echo "📦 Building Guardian Daemon (debug)..."
cargo build -p guardian-daemon

# Copy to Tauri binaries
echo ""
echo "📁 Preparing Tauri sidecar..."
mkdir -p guardian-sentinel/src-tauri/binaries
cp target/debug/guardian-daemon guardian-sentinel/src-tauri/binaries/
chmod +x guardian-sentinel/src-tauri/binaries/guardian-daemon

# Install npm dependencies if needed
cd guardian-sentinel
if [ ! -d "node_modules" ]; then
    echo ""
    echo "📥 Installing Node.js dependencies..."
    npm install
fi

# Run Tauri in dev mode
echo ""
echo "🚀 Starting Guardian Sentinel..."
echo ""
npm run tauri dev
