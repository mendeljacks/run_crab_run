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

/// Should we send a notification for this run status + policy?
pub fn should_notify(policy: &str, status: RunStatus) -> bool {
    match policy {
        "always" => true,
        "failure" => matches!(status, RunStatus::Failed | RunStatus::Timeout),
        _ => false,
    }
}