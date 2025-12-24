use anyhow::Result;
use guardian_common::LogEvent;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::io::{self, BufRead};
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    info!("Guardian Event Bridge starting...");

    // Get database path from environment or use default
    let db_path = std::env::var("GUARDIAN_DB_PATH")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").expect("HOME not set");
            format!("{}/.local/share/com.guardian.sentinel/guardian.db", home)
        });

    info!("Connecting to database: {}", db_path);

    // Ensure parent directory exists
    let db_path_buf = PathBuf::from(&db_path);
    if let Some(parent) = db_path_buf.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Connect to database
    let db_url = format!("sqlite://{}?mode=rwc", db_path);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    info!("Database connected successfully");

    // Read JSON events from stdin and store in database
    let stdin = io::stdin();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = line?;
        
        // Skip non-JSON lines (log messages)
        if !line.trim_start().starts_with('{') {
            continue;
        }

        match LogEvent::from_json(&line) {
            Ok(event) => {
                info!("Received event: {:?} - {:?}", event.severity, event.event_type);
                
                if let Err(e) = insert_event(&pool, &event).await {
                    error!("Failed to store event: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to parse event JSON: {} - Line: {}", e, line);
            }
        }
    }

    Ok(())
}

/// Insert a log event into the database
async fn insert_event(pool: &SqlitePool, event: &LogEvent) -> Result<()> {
    let event_type = serde_json::to_string(&event.event_type)?;
    let tags = serde_json::to_string(&event.tags)?;

    sqlx::query(
        r#"
        INSERT INTO events (id, timestamp, severity, event_type, event_data, hostname, tags, rule_triggered, rule_name)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(event.id.to_string())
    .bind(event.timestamp.to_rfc3339())
    .bind(serde_json::to_string(&event.severity).unwrap_or_default().trim_matches('"').to_string())
    .bind(serde_json::to_string(&event.event_type).unwrap_or_default())
    .bind(event_type)
    .bind(&event.hostname)
    .bind(tags)
    .bind(event.rule_triggered as i32)
    .bind(&event.rule_name)
    .execute(pool)
    .await?;

    Ok(())
}
