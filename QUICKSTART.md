# Guardian SIEM - Quick Start Guide

## Prerequisites

- **Rust**: 1.75 or later (`rustup` recommended)
- **Node.js**: 18 or later
- **SQLite3**: For database operations

## Installation

```bash
# Clone or navigate to the project
cd Guardian

# Run the development script
./dev.sh
```

This will:

1. Build the daemon in debug mode
2. Install Node.js dependencies
3. Launch the Tauri application

## Manual Setup

### 1. Build the Daemon

```bash
cargo build --release -p guardian-daemon
```

### 2. Test the Daemon Standalone

```bash
# Set watch path (optional)
export GUARDIAN_WATCH_PATH=/tmp/guardian-test

# Run daemon
./target/release/guardian-daemon

# In another terminal, create a test file
echo "test" > /tmp/guardian-test/testfile.txt
```

You should see JSON output like:

```json
{"id":"...","timestamp":"...","severity":"LOW","type":"file_integrity",...}
```

### 3. Run the Full Application

```bash
cd guardian-sentinel

# Install dependencies
npm install

# Run in dev mode
npm run tauri dev
```

## Production Build

```bash
# Use the build script
./build.sh

# Or manually:
cargo build --release --workspace
cd guardian-sentinel && npm run tauri build
```

## Testing

```bash
# Run all tests
cargo test --workspace

# Test specific crate
cargo test -p guardian-common
cargo test -p guardian-daemon
```

## Configuration

### Environment Variables

- `GUARDIAN_WATCH_PATH`: Directory to monitor (default: `/tmp/guardian-test`)
- `RUST_LOG`: Logging level (default: `info`)

### Example

```bash
export GUARDIAN_WATCH_PATH=/var/log
export RUST_LOG=debug
./target/release/guardian-daemon
```

## Troubleshooting

### Daemon not outputting events

1. Check the watch path exists: `ls $GUARDIAN_WATCH_PATH`
2. Verify permissions: `ls -la $GUARDIAN_WATCH_PATH`
3. Enable debug logging: `RUST_LOG=debug ./target/release/guardian-daemon`

### Tauri app not showing events

1. Check browser console for errors (Ctrl+Shift+I)
2. Verify daemon binary is in `guardian-sentinel/src-tauri/binaries/`
3. Check Tauri logs in the terminal

### Database errors

1. Delete the database: `rm -rf ~/.local/share/com.guardian.sentinel/`
2. Restart the application

## Next Steps

- Read the [README.md](README.md) for architecture details
- Review the [walkthrough.md](.gemini/antigravity/brain/5aa7fecb-f118-44a2-aac2-d39e01859ec7/walkthrough.md) for implementation details
- Customize rules in `guardian-daemon/src/rules.rs`
- Extend event types in `guardian-common/src/lib.rs`
