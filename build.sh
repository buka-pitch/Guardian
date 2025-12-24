#!/bin/bash
set -e

echo "🛡️  Guardian SIEM - Build Script"
echo "================================"

# Build the daemon
echo ""
echo "📦 Building Guardian Daemon & Bridge..."
cargo build --release -p guardian-daemon
cargo build --release -p guardian-bridge

echo ""
echo "✅ Daemon build complete!"
echo "📍 Binary location: target/release/guardian-daemon"
echo ""

# Test skipped to prevent hanging
# ./target/release/guardian-daemon --help 2>&1 | head -5 || echo "Daemon binary created successfully"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📦 To build the Tauri application, you need system dependencies:"
echo ""
echo "   Ubuntu/Debian:"
echo "   sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libsoup-3.0-dev"
echo ""
echo "   Fedora:"
echo "   sudo dnf install webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel patchelf libsoup3-devel"
echo ""
echo "   Arch:"
echo "   sudo pacman -S webkit2gtk-4.1 libappindicator-gtk3 librsvg patchelf libsoup3"
echo ""
echo "   Then run:"
echo "   cd guardian-sentinel && npm install && npm run tauri build"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if node_modules exists
cd guardian-sentinel
if [ ! -d "node_modules" ]; then
    echo "📥 Installing Node.js dependencies..."
    npm install
fi

# Build Tauri app
npm run tauri build
cd ..

echo ""
echo "✅ Build complete!"
echo ""
echo "📍 Daemon binary: target/release/guardian-daemon"
echo "📍 Tauri app: guardian-sentinel/src-tauri/target/release/bundle/"
