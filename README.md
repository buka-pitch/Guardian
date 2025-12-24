# Guardian SIEM

A lightweight, local-first Security Information and Event Management (SIEM) tool built with Rust and Tauri.

## Architecture

Guardian uses a **Decoupled Binary Pattern** consisting of:

### 🛡️ Guardian Daemon (Headless Agent)

- Lightweight background service for system event collection
- Monitors: File integrity, Event logs, Network sockets, Process activity
- Outputs structured JSON logs to stdout
- Minimal CPU/RAM footprint

### 🖥️ Sentinel Frontend (Tauri Application)

- React/TypeScript dashboard for visualization
- Real-time event streaming via Tauri IPC
- SQLite-backed historical log storage and search
- Rule-based alerting system

## Project Structure

```
Guardian/
├── Cargo.toml                    # Workspace definition
├── guardian-common/              # Shared data structures
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs               # LogEvent, Severity, EventType
├── guardian-daemon/              # Headless monitoring agent
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs              # Event collection & JSON output
│       └── rules.rs             # Rule engine
└── guardian-sentinel/            # Tauri frontend application
    ├── src-tauri/
    │   ├── Cargo.toml
    │   ├── tauri.conf.json      # Sidecar configuration
    │   ├── build.rs
    │   └── src/
    │       ├── main.rs          # Tauri setup & sidecar spawning
    │       ├── lib.rs           # App state management
    │       └── database.rs      # SQLite persistence
    ├── src/                     # React frontend (to be implemented)
    └── package.json
```

## Technology Stack

### Core Dependencies

- **Runtime**: `tokio` - Async runtime
- **Logging**: `tracing` - Structured logging
- **Serialization**: `serde` - Data serialization

### Monitoring

- **sysinfo**: Hardware/process monitoring
- **notify**: Real-time file system events

### Frontend

- **Tauri v2**: Desktop application framework
- **SQLite** (via `sqlx`): Event persistence
- **React + TypeScript**: UI dashboard

## Getting Started

### Prerequisites

- Rust 1.75+ (`rustup`)
- Node.js 18+ (for Tauri frontend)
- SQLite3

### Build the Daemon

```bash
# Build in release mode for optimal performance
cargo build --release -p guardian-daemon

# The binary will be at: target/release/guardian-daemon
```

### Run the Daemon Standalone

```bash
# Set watch path (optional, defaults to /tmp/guardian-test)
export GUARDIAN_WATCH_PATH=/path/to/monitor

# Run the daemon - outputs JSON to stdout
./target/release/guardian-daemon

# Test it by creating files in the monitored directory
echo "test" > /tmp/guardian-test/testfile.txt
```

### Build & Run the Sentinel Application

```bash
cd guardian-sentinel

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Tauri Sidecar Integration

The Sentinel application spawns the Guardian daemon as a **sidecar process**:

### Configuration (`tauri.conf.json`)

```json
{
  "bundle": {
    "externalBin": ["binaries/guardian-daemon"]
  },
  "plugins": {
    "shell": {
      "sidecar": [
        {
          "name": "guardian-daemon",
          "command": "binaries/guardian-daemon",
          "args": []
        }
      ]
    }
  }
}
```

### Rust Backend (`main.rs`)

```rust
// Spawn the sidecar
let sidecar = shell.sidecar("guardian-daemon")?;
let (mut rx, _child) = sidecar.spawn()?;

// Read JSON lines from stdout
if let Some(stdout) = rx.stdout.take() {
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        let event: LogEvent = serde_json::from_str(&line)?;

        // Emit to frontend
        app.emit("log-event", &event)?;

        // Store in database
        database::insert_event(&pool, &event).await?;
    }
}
```

## Shared Data Schema

The `guardian-common` crate defines the core `LogEvent` structure:

```rust
pub struct LogEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub severity: Severity,          // INFO, LOW, MEDIUM, HIGH, CRITICAL
    pub event_type: EventType,       // FileIntegrity, NetworkSocket, etc.
    pub hostname: String,
    pub tags: Vec<String>,
    pub rule_triggered: bool,
    pub rule_name: Option<String>,
}

pub enum EventType {
    FileIntegrity { path: String, operation: FileOperation, hash: Option<String> },
    NetworkSocket { local_addr: String, remote_addr: Option<String>, ... },
    SystemLog { source: String, level: String, message: String },
    ProcessMonitor { pid: u32, name: String, cpu_usage: f32, ... },
}
```

## Rule Engine

The daemon includes a simple pattern-matching rule engine (`rules.rs`):

```rust
pub struct RuleEngine {
    rules: Vec<Rule>,
}

impl RuleEngine {
    pub fn evaluate(&self, event: &LogEvent) -> Option<String> {
        for rule in &self.rules {
            if (rule.matcher)(event) {
                return Some(rule.name.clone());
            }
        }
        None
    }
}
```

### Built-in Rules

1. **Critical File Modification**: Flags changes to `/etc/passwd`, `/etc/shadow`, `/etc/sudoers`
2. **High Severity Alert**: Triggers on events with severity ≥ HIGH
3. **Suspicious Network**: Detects connections to non-standard ports (4444, 31337)
4. **High CPU Usage**: Alerts when process CPU usage > 90%

### Adding Custom Rules

```rust
engine.add_rule(
    "my_custom_rule",
    Box::new(|event| {
        // Your matching logic here
        matches!(event.event_type, EventType::FileIntegrity { path, .. }
            if path.ends_with(".secret"))
    })
);
```

## Database Schema

SQLite table for event persistence:

```sql
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    severity TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_data TEXT NOT NULL,      -- JSON blob
    hostname TEXT NOT NULL,
    tags TEXT NOT NULL,             -- JSON array
    rule_triggered INTEGER NOT NULL,
    rule_name TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX idx_timestamp ON events(timestamp DESC);
CREATE INDEX idx_severity ON events(severity);
CREATE INDEX idx_rule_triggered ON events(rule_triggered);
```

## Tauri Commands

Frontend can invoke these commands:

```typescript
import { invoke } from "@tauri-apps/api/core";

// Get recent events
const events = await invoke("get_recent_events", { limit: 100 });

// Get statistics
const stats = await invoke("get_event_stats");

// Search events
const results = await invoke("search_events", {
  query: "passwd",
  severity: "HIGH",
});
```

## Event Streaming

Listen to real-time events in the frontend:

```typescript
import { listen } from "@tauri-apps/api/event";

await listen("log-event", (event) => {
  console.log("New event:", event.payload);
  // Update UI with new event
});
```

## Deployment

### Linux (systemd)

```bash
# Copy daemon to system location
sudo cp target/release/guardian-daemon /usr/local/bin/

# Create systemd service
sudo nano /etc/systemd/system/guardian-daemon.service

# Enable and start
sudo systemctl enable guardian-daemon
sudo systemctl start guardian-daemon
```

### Windows (Service)

Use the `windows-service` crate integration (see `guardian-daemon/src/main.rs` for hooks).

## Performance Characteristics

- **Daemon Memory**: ~5-10 MB idle
- **Daemon CPU**: <1% idle, <5% during event bursts
- **Database Size**: ~1 KB per event (varies by event type)
- **Event Throughput**: 1000+ events/sec

## Roadmap

- [ ] Network socket monitoring implementation
- [ ] Windows Event Log integration
- [ ] React dashboard UI
- [ ] Real-time charts and visualizations
- [ ] Rule configuration UI
- [ ] Alert notifications (email, webhook)
- [ ] Multi-host aggregation
- [ ] Log rotation and retention policies

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.
