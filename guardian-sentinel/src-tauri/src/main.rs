// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use guardian_common::LogEvent;
use guardian_sentinel_lib::AppState;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tracing::{error, info};
#[allow(unused_imports)]
use tauri_plugin_shell::ShellExt;

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();

            // Initialize database
            let db_path = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir")
                .join("guardian.db");

            info!("Database path: {:?}", db_path);

            // Create app state
            let state = Arc::new(Mutex::new(AppState::new(db_path)));
            app.manage(state.clone());

            // Initialize database in background
            let init_state = state.clone();
            tauri::async_runtime::spawn(async move {
                let mut state = init_state.lock().await;
                if let Err(e) = state.init_db().await {
                    error!("Failed to initialize database: {}", e);
                }
            });

            // Spawn the guardian daemon sidecar
            tauri::async_runtime::spawn(async move {
                if let Err(e) = spawn_daemon(handle, state).await {
                    error!("Daemon error: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_recent_events,
            get_event_stats,
            search_events
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Spawn the guardian daemon and process its output
async fn spawn_daemon(
    app: tauri::AppHandle,
    state: Arc<Mutex<AppState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Spawning guardian daemon...");

    // Get the path to the sidecar binary
    // In dev: cargo run --bin guardian-daemon
    // In prod: bundled sidecar
    // For this prototype, we'll try to run the binary directly from target/debug for simplicity in dev mode
    // In a real Tauri app, you'd use the sidecar feature properly
    
    let mut cmd = tauri_plugin_shell::ShellExt::shell(&app)
        .sidecar("guardian-daemon")
        .or_else(|_| {
            // Fallback for dev mode if sidecar isn't configured in tauri.conf.json
            // We'll run the binary directly relative to the project root
             Ok(tauri_plugin_shell::ShellExt::shell(&app)
                .command("../../target/debug/guardian-daemon"))
        })?;

    let (mut rx, _child) = cmd.spawn()?;

    // Process output in background
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                tauri_plugin_shell::process::CommandEvent::Stdout(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    for event_str in line.lines() {
                        if event_str.trim().is_empty() { continue; }
                        
                        // Try to parse as LogEvent
                        if let Ok(log_event) = serde_json::from_str::<LogEvent>(event_str) {
                            // Store in DB
                            let state_lock = state.lock().await;
                            if let Err(e) = state_lock.store_event(&log_event).await {
                                error!("Failed to store event: {}", e);
                            }
                            drop(state_lock);
                            
                            // Emit to frontend
                            if let Err(e) = app.emit("realtime-event", &log_event) {
                                error!("Failed to emit event: {}", e);
                            }
                        } else {
                            // Log raw output if it's not JSON
                             info!("Daemon: {}", event_str);
                        }
                    }
                }
                tauri_plugin_shell::process::CommandEvent::Stderr(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    info!("Daemon Log: {}", line.trim());
                }
                _ => {}
            }
        }
    });

    info!("Guardian daemon started");

    Ok(())
}

/// Tauri command to get recent events
#[tauri::command]
async fn get_recent_events(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    limit: Option<i64>,
) -> Result<Vec<LogEvent>, String> {
    let state = state.lock().await;
    state
        .get_recent_events(limit.unwrap_or(100))
        .await
        .map_err(|e| e.to_string())
}

/// Tauri command to get event statistics
#[tauri::command]
async fn get_event_stats(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<serde_json::Value, String> {
    let state = state.lock().await;
    state.get_event_stats().await.map_err(|e| e.to_string())
}

/// Tauri command to search events
#[tauri::command]
async fn search_events(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    query: String,
    severity: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<LogEvent>, String> {
    let state = state.lock().await;
    state
        .search_events(
            &query,
            severity.as_deref(),
            limit.unwrap_or(100),
            offset.unwrap_or(0),
        )
        .await
        .map_err(|e| e.to_string())
}
