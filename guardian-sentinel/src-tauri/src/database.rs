use anyhow::Result;
use guardian_common::LogEvent;
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use std::path::Path;
use tracing::info;

/// Initialize the SQLite database
pub async fn init_database(db_path: &Path) -> Result<SqlitePool> {
    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    info!("Connecting to database: {}", db_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Run migrations
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS events (
            id TEXT PRIMARY KEY NOT NULL,
            timestamp TEXT NOT NULL,
            severity TEXT NOT NULL,
            event_type TEXT NOT NULL,
            event_data TEXT NOT NULL,
            hostname TEXT NOT NULL,
            tags TEXT NOT NULL,
            rule_triggered INTEGER NOT NULL DEFAULT 0,
            rule_name TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Create indexes for common queries
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_timestamp ON events(timestamp DESC)")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_severity ON events(severity)")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_rule_triggered ON events(rule_triggered)")
        .execute(&pool)
        .await?;

    info!("Database initialized successfully");

    Ok(pool)
}

/// Insert a log event into the database
pub async fn insert_event(pool: &SqlitePool, event: &LogEvent) -> Result<()> {
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
    .bind(serde_json::to_string(&event.event_type)?)
    .bind(event_type)
    .bind(&event.hostname)
    .bind(tags)
    .bind(event.rule_triggered as i32)
    .bind(&event.rule_name)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get recent events
pub async fn get_recent_events(pool: &SqlitePool, limit: i64) -> Result<Vec<LogEvent>> {
    let rows = sqlx::query(
        r#"
        SELECT id, timestamp, severity, event_data, hostname, tags, rule_triggered, rule_name
        FROM events
        ORDER BY timestamp DESC
        LIMIT ?
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut events = Vec::new();
    for row in rows {
        let event_json = format!(
            r#"{{
                "id": "{}",
                "timestamp": "{}",
                "severity": "{}",
                {},
                "hostname": "{}",
                "tags": {},
                "rule_triggered": {},
                "rule_name": {}
            }}"#,
            row.get::<String, _>("id"),
            row.get::<String, _>("timestamp"),
            row.get::<String, _>("severity"),
            row.get::<String, _>("event_data")
                .trim_start_matches('{')
                .trim_end_matches('}'),
            row.get::<String, _>("hostname"),
            row.get::<String, _>("tags"),
            row.get::<i32, _>("rule_triggered") != 0,
            row.get::<Option<String>, _>("rule_name")
                .map(|s| format!("\"{}\"", s))
                .unwrap_or_else(|| "null".to_string())
        );

        if let Ok(event) = serde_json::from_str::<LogEvent>(&event_json) {
            events.push(event);
        }
    }

    Ok(events)
}

/// Get event statistics
pub async fn get_event_stats(pool: &SqlitePool) -> Result<serde_json::Value> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
        .fetch_one(pool)
        .await?;

    let by_severity = sqlx::query(
        r#"
        SELECT severity, COUNT(*) as count
        FROM events
        GROUP BY severity
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut severity_counts = serde_json::Map::new();
    for row in by_severity {
        severity_counts.insert(
            row.get::<String, _>("severity"),
            serde_json::json!(row.get::<i64, _>("count")),
        );
    }

    let rules_triggered: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE rule_triggered = 1")
            .fetch_one(pool)
            .await?;

    Ok(serde_json::json!({
        "total": total,
        "by_severity": severity_counts,
        "rules_triggered": rules_triggered
    }))
}

/// Search events
pub async fn search_events(
    pool: &SqlitePool,
    query: &str,
    severity: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<LogEvent>> {
    let mut sql = String::from(
        r#"
        SELECT id, timestamp, severity, event_data, hostname, tags, rule_triggered, rule_name
        FROM events
        WHERE (event_data LIKE ? OR hostname LIKE ? OR tags LIKE ?)
        "#,
    );

    if severity.is_some() {
        sql.push_str(" AND severity = ?");
    }

    sql.push_str(" ORDER BY timestamp DESC LIMIT ? OFFSET ?");

    let search_pattern = format!("%{}%", query);
    let mut query_builder = sqlx::query(&sql)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern); // Bind for tags

    if let Some(sev) = severity {
        query_builder = query_builder.bind(sev);
    }

    query_builder = query_builder.bind(limit).bind(offset);

    let rows = query_builder.fetch_all(pool).await?;

    let mut events = Vec::new();
    for row in rows {
        let event_json = format!(
            r#"{{
                "id": "{}",
                "timestamp": "{}",
                "severity": "{}",
                {},
                "hostname": "{}",
                "tags": {},
                "rule_triggered": {},
                "rule_name": {}
            }}"#,
            row.get::<String, _>("id"),
            row.get::<String, _>("timestamp"),
            row.get::<String, _>("severity"),
            row.get::<String, _>("event_data")
                .trim_start_matches('{')
                .trim_end_matches('}'),
            row.get::<String, _>("hostname"),
            row.get::<String, _>("tags"),
            row.get::<i32, _>("rule_triggered") != 0,
            row.get::<Option<String>, _>("rule_name")
                .map(|s| format!("\"{}\"", s))
                .unwrap_or_else(|| "null".to_string())
        );

        match serde_json::from_str::<LogEvent>(&event_json) {
            Ok(event) => events.push(event),
            Err(e) => tracing::error!("Failed to deserialize event: {}", e),
        }
    }

    Ok(events)
}
