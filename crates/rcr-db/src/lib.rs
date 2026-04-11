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
        sqlx::query(migrations::MIGRATION_001)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        sqlx::query(migrations::MIGRATION_002)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        sqlx::query(migrations::MIGRATION_003)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}