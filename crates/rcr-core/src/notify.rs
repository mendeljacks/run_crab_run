use crate::models::run::RunStatus;

/// Trait for sending notifications about job completions.
/// Implementations: Email (SMTP), Webhook (Slack/Discord), etc.
pub trait Notifier: Send + Sync {
    fn notify(
        &self,
        job_name: &str,
        run_id: &str,
        status: RunStatus,
        exit_code: Option<i32>,
        stdout: &str,
        stderr: &str,
        to: &str,
    ) -> Result<(), String>;
}