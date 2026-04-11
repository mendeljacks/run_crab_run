use anyhow::Result;
use rcr_api::AppState;
use rcr_core::models::config::AppConfig;
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
                .unwrap_or_else(|_| "run_crab_run=info".parse().unwrap()),
        )
        .init();

    info!("🦀 run-crab-run starting...");

    // Load config
    let config_path = std::env::args().nth(1).unwrap_or_else(|| "config/default.toml".into());
    let config_str = std::fs::read_to_string(&config_path)?;
    let config: AppConfig = toml::from_str(&config_str)?;

    info!("Database: {}", config.database.path);
    info!("Listening on {}:{}", config.server.host, config.server.port);

    // Initialize database
    let db = Database::new(&config.database.path).await?;
    db.run_migrations().await?;
    info!("Database initialized");

    // Initialize notifier
    let notifier: Arc<dyn Notifier> = if let Some(ref email_config) = config.email {
        info!("Email notifier configured (SMTP: {}:{})", email_config.smtp_host, email_config.smtp_port);
        Arc::new(EmailNotifier::new(
            email_config.smtp_host.clone(),
            email_config.smtp_port,
            email_config.smtp_user.clone(),
            email_config.smtp_pass.clone(),
            email_config.from_address.clone(),
        ))
    } else {
        info!("No email config, notifications disabled");
        Arc::new(NoopNotifier)
    };

    // Initialize executor
    let executor = Arc::new(JobExecutor::new(db.clone(), notifier));

    // Initialize scheduler
    let scheduler = Arc::new(rcr_runner::Scheduler::new(db.clone(), executor.clone()));
    let scheduler_handle = tokio::spawn(async move {
        scheduler.run().await;
    });

    // Build and run API server
    let state = AppState {
        db,
        executor,
    };

    let app = rcr_api::routes::router(state);

    let listener = tokio::net::TcpListener::bind(
        format!("{}:{}", config.server.host, config.server.port)
    ).await?;

    info!("🦀 Server running on {}:{}", config.server.host, config.server.port);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Shutting down...");
    scheduler_handle.abort();

    Ok(())
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