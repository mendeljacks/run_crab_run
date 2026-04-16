use anyhow::Result;
use rcr_db::Database;
use rcr_runner::JobExecutor;
use std::sync::Arc;
use tracing::info;

/// Environment variables:
///   RCR_DB_PATH      — SQLite database path (default: run_crab_run.db)

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rcr_runner=info".parse().unwrap()),
        )
        .init();

    let db_path = env("RCR_DB_PATH", "run_crab_run.db");

    info!("🦀 rcr-runner starting...");
    info!("Database: {}", db_path);

    let db = Database::new(&db_path).await?;

    let executor = Arc::new(JobExecutor::new(db.clone()));
    let scheduler = Arc::new(rcr_runner::Scheduler::new(db, executor));

    info!("🦀 Scheduler running");
    // run() blocks until the process is killed
    scheduler.run().await;

    #[allow(unreachable_code)]
    Ok(())
}

fn env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}