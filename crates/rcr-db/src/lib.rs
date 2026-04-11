pub mod jobs;
pub mod runs;
pub mod webhooks;
pub mod migrations;

use rcr_core::Error;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::SqlitePool;
use std::str::FromStr;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_path: &str) -> Result<Self, Error> {
        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", database_path))
            .map_err(|e| Error::Database(e.to_string()))?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal);

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<(), Error> {
        // Ensure tracking table exists
        sqlx::query(migrations::MIGRATION_001)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        // Mark migration 001 as applied (the IF NOT EXISTS makes it safe to re-run)
        sqlx::query("INSERT OR IGNORE INTO _migrations (id) VALUES ('001')")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        // Future migrations go here:
        // let applied: Vec<String> = sqlx::query_scalar::<_, String>(
        //     "SELECT id FROM _migrations ORDER BY id"
        // ).fetch_all(&self.pool).await.map_err(|e| Error::Database(e.to_string()))?;
        //
        // if !applied.iter().any(|a| a == "002") {
        //     sqlx::query(migrations::MIGRATION_002) ...
        //     sqlx::query("INSERT INTO _migrations (id) VALUES ('002')") ...
        // }

        Ok(())
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}