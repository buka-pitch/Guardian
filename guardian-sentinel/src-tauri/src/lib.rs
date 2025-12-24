pub mod database;

use anyhow::Result;
use guardian_common::LogEvent;
use sqlx::SqlitePool;
use std::path::PathBuf;

/// Application state
pub struct AppState {
    db_path: PathBuf,
    pool: Option<SqlitePool>,
}

impl AppState {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            pool: None,
        }
    }

    /// Initialize the database connection
    pub async fn init_db(&mut self) -> Result<()> {
        let pool = database::init_database(&self.db_path).await?;
        self.pool = Some(pool);
        Ok(())
    }

    /// Get the database pool
    fn pool(&self) -> Result<&SqlitePool> {
        self.pool
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
    }

    /// Store an event in the database
    pub async fn store_event(&self, event: &LogEvent) -> Result<()> {
        database::insert_event(self.pool()?, event).await
    }

    /// Get recent events
    pub async fn get_recent_events(&self, limit: i64) -> Result<Vec<LogEvent>> {
        database::get_recent_events(self.pool()?, limit).await
    }

    /// Get event statistics
    pub async fn get_event_stats(&self) -> Result<serde_json::Value> {
        database::get_event_stats(self.pool()?).await
    }

    /// Search events
    pub async fn search_events(
        &self,
        query: &str,
        severity: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<LogEvent>> {
        database::search_events(self.pool()?, query, severity, limit, offset).await
    }
}
