use anyhow::Result;
use rcr_api::AppState;
use rcr_core::notify::Notifier;
use rcr_db::Database;
use rcr_runner::JobExecutor;
use rcr_runner::notify::{EmailNotifier, NoopNotifier};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rcr_api=info".parse().unwrap()),
        )
        .init();

    let host = env("RCR_HOST", "0.0.0.0");
    let port: u16 = env_parse("RCR_PORT", 3001);
    let db_path = env("RCR_DB_PATH", "run_crab_run.db");

    info!("🦀 rcr-api starting...");
    info!("Database: {}", db_path);
    info!("Listening on {}:{}", host, port);

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
    let state = AppState { db, executor };

    let app = rcr_api::routes::router(state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    info!("🦀 API server running on {}:{}", host, port);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Shutting down...");
    Ok(())
}

fn env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { info!("Received Ctrl+C"); },
        _ = terminate => { info!("Received SIGTERM"); },
    }
}