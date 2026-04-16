use anyhow::Result;
use rcr_core::notify::Notifier;
use rcr_db::Database;
use rcr_runner::JobExecutor;
use rcr_runner::notify::{EmailNotifier, NoopNotifier};
use std::sync::Arc;
use tracing::info;

/// Environment variables:
///   RCR_DB_PATH      — SQLite database path (default: run_crab_run.db)
///   RCR_EMAIL_SMTP_HOST / RCR_EMAIL_SMTP_PORT / RCR_EMAIL_SMTP_USER /
///   RCR_EMAIL_SMTP_PASS / RCR_EMAIL_FROM — optional email notification

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

    let notifier: Arc<dyn Notifier> = match std::env::var("RCR_EMAIL_SMTP_HOST") {
        Ok(smtp_host) => {
            let smtp_port: u16 = env_parse("RCR_EMAIL_SMTP_PORT", 587);
            let smtp_user = env("RCR_EMAIL_SMTP_USER", "");
            let smtp_pass = env("RCR_EMAIL_SMTP_PASS", "");
            let from = env("RCR_EMAIL_FROM", "");
            info!("Email notifier configured (SMTP: {}:{})", smtp_host, smtp_port);
            Arc::new(EmailNotifier::new(smtp_host, smtp_port, smtp_user, smtp_pass, from))
        }
        Err(_) => {
            info!("No email config, notifications disabled");
            Arc::new(NoopNotifier)
        }
    };

    let executor = Arc::new(JobExecutor::new(db.clone(), notifier));
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

fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}