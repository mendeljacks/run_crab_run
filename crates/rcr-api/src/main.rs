use anyhow::Result;
use rcr_api::AppState;
use rcr_db::Database;
use rcr_runner::JobExecutor;
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

    let executor = Arc::new(JobExecutor::new(db.clone()));
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