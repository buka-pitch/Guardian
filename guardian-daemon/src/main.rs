use anyhow::Result;
use guardian_common::{EventType, FileOperation, LogEvent, Severity};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

mod rules;
mod scanner;

use rules::RuleEngine;
use scanner::YaraScanner;
use sysinfo::System;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for internal logging (stderr)
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    info!("Guardian Daemon starting...");

    // Get hostname
    let hostname = hostname::get()
        .unwrap_or_else(|_| "unknown".into())
        .to_string_lossy()
        .to_string();

    // Create channel for events
    let (tx, mut rx) = mpsc::channel::<LogEvent>(1000);

    // Initialize rule engine
    let rule_engine = RuleEngine::new();

    // Initialize YARA scanner
    let scanner = match YaraScanner::new() {
        Ok(s) => Some(Arc::new(s)),
        Err(e) => {
            error!("Failed to initialize YARA scanner: {}", e);
            None
        }
    };

    // Spawn file monitor task
    let monitor_tx = tx.clone();
    let monitor_hostname = hostname.clone();
    let monitor_scanner = scanner.clone();
    
    tokio::task::spawn_blocking(move || {
        if let Err(e) = start_file_monitor(monitor_tx, monitor_hostname, monitor_scanner) {
            error!("File monitor error: {}", e);
        }
    });

    // Spawn system monitor task
    let sys_tx = tx.clone();
    let sys_hostname = hostname.clone();
    tokio::task::spawn_blocking(move || {
        monitor_system(sys_tx, sys_hostname);
    });

    info!("Guardian Daemon initialized. Monitoring events...");

    // Main event loop - process events and output to stdout
    while let Some(mut event) = rx.recv().await {
        // Apply rule engine
        if let Some(rule_name) = rule_engine.evaluate(&event) {
            event = event.with_rule(rule_name);
        }

        // Output JSON to stdout for Tauri to consume
        match event.to_json() {
            Ok(json) => println!("{}", json),
            Err(e) => warn!("Failed to serialize event: {}", e),
        }
    }

    Ok(())
}

/// Start file system monitoring
fn start_file_monitor(
    tx: mpsc::Sender<LogEvent>, 
    hostname: String,
    scanner: Option<Arc<YaraScanner>>
) -> Result<()> {
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();

    // Create watcher
    let mut watcher = notify::recommended_watcher(notify_tx)?;

    // Watch a specific path (configurable in production)
    let watch_path = std::env::var("GUARDIAN_WATCH_PATH").unwrap_or_else(|_| "/tmp/guardian-test".to_string());
    
    info!("Watching path: {}", watch_path);
    
    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&watch_path)?;
    
    watcher.watch(Path::new(&watch_path), RecursiveMode::Recursive)?;

    // Process file system events
    for res in notify_rx {
        match res {
            Ok(event) => {
                if let Some(log_event) = process_fs_event(event, &hostname, scanner.as_deref()) {
                    if tx.blocking_send(log_event).is_err() {
                        error!("Failed to send event - channel closed");
                        break;
                    }
                }
            }
            Err(e) => warn!("Watch error: {:?}", e),
        }
    }

    Ok(())
}

/// Convert notify events to LogEvents
fn process_fs_event(
    event: Event, 
    hostname: &str,
    scanner: Option<&YaraScanner>
) -> Option<LogEvent> {
    let operation = match event.kind {
        EventKind::Create(_) => FileOperation::Create,
        EventKind::Modify(_) => FileOperation::Modify,
        EventKind::Remove(_) => FileOperation::Delete,
        _ => return None,
    };

    let path = event.paths.first()?.to_string_lossy().to_string();

    // Default severity
    let mut severity = if path.contains("/etc") || path.contains("passwd") || path.contains("shadow") {
        Severity::High
    } else if path.ends_with(".conf") || path.ends_with(".cfg") {
        Severity::Medium
    } else {
        Severity::Low
    };

    let mut rules_matched = Vec::new();
    let mut matched_rule_name = None;

    // Scan file if scanner is available and event is Create/Modify
    if let Some(s) = scanner {
        if matches!(operation, FileOperation::Create | FileOperation::Modify) {
            // Only scan regular files
            if Path::new(&path).is_file() {
                let matches = s.scan_file(&path);
                if !matches.is_empty() {
                    severity = Severity::Critical;
                    matched_rule_name = Some(matches[0].clone()); // Use first match as main rule
                    rules_matched = matches;
                }
            }
        }
    }

    let mut log_event = LogEvent::new(
        severity,
        EventType::FileIntegrity {
            path: path.clone(),
            operation,
            hash: None, // TODO: Compute file hash for Create/Modify
        },
        hostname.to_string(),
    )
    .with_tag("file_monitor");

    // Add tags for YARA matches
    for rule in rules_matched {
        log_event = log_event.with_tag(format!("yara:{}", rule));
    }

    if let Some(rule) = matched_rule_name {
        log_event = log_event.with_rule(rule);
    }

    Some(log_event)
}

fn monitor_system(tx: mpsc::Sender<LogEvent>, hostname: String) {
    let mut sys = System::new_all();
    
    loop {
        sys.refresh_all();
        
        let pid = std::process::id();
        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let memory_usage = sys.used_memory();

        let event = LogEvent::new(
            Severity::Info,
            EventType::ProcessMonitor {
                pid,
                name: "system".to_string(), // aggregated system stats
                cpu_usage,
                memory_usage,
            },
            hostname.clone(),
        ).with_tag("system_monitor");

        // Use blocking send for the standalone thread
        if tx.blocking_send(event).is_err() {
            break;
        }

        std::thread::sleep(Duration::from_secs(1));
    }
}
