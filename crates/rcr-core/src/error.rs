use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Job not found: {0}")]
    JobNotFound(String),

    #[error("Run not found: {0}")]
    RunNotFound(String),

    #[error("Job already running: {0}")]
    JobAlreadyRunning(String),

    #[error("Invalid RRULE: {0}")]
    InvalidRrule(String),

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Webhook secret mismatch")]
    WebhookSecretMismatch,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;