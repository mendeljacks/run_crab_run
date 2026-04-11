use rcr_core::models::run::RunStatus;
use rcr_core::notify::Notifier;

pub struct EmailNotifier {
    smtp_host: String,
    smtp_port: u16,
    smtp_user: String,
    smtp_pass: String,
    from_address: String,
}

impl EmailNotifier {
    pub fn new(
        smtp_host: String,
        smtp_port: u16,
        smtp_user: String,
        smtp_pass: String,
        from_address: String,
    ) -> Self {
        Self {
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_pass,
            from_address,
        }
    }
}

impl Notifier for EmailNotifier {
    fn notify(
        &self,
        job_name: &str,
        run_id: &str,
        status: RunStatus,
        exit_code: Option<i32>,
        stdout: &str,
        stderr: &str,
        to_email: &str,
    ) -> Result<(), String> {
        let status_str = match status {
            RunStatus::Success => "✅ SUCCESS",
            RunStatus::Failed => "❌ FAILED",
            RunStatus::Timeout => "⏱️ TIMED OUT",
            _ => "ℹ️ OTHER",
        };

        let subject = format!("[Run Crab Run] {} — {}", job_name, status_str);
        let body = format!(
            "Job: {}\n\
             Run ID: {}\n\
             Status: {}\n\
             Exit code: {}\n\n\
             Stdout (last 2000 chars):\n{}\n\n\
             Stderr (last 2000 chars):\n{}",
            job_name,
            run_id,
            status_str,
            exit_code.map(|c| c.to_string()).unwrap_or_else(|| "N/A".into()),
            &stdout[..stdout.len().min(2000)],
            &stderr[..stderr.len().min(2000)],
        );

        // Send email in a blocking thread to avoid async context issues
        let smtp_host = self.smtp_host.clone();
        let smtp_port = self.smtp_port;
        let smtp_user = self.smtp_user.clone();
        let smtp_pass = self.smtp_pass.clone();
        let from_address = self.from_address.clone();
        let to_email = to_email.to_string();

        std::thread::spawn(move || {
            let _ = send_email_blocking(&smtp_host, smtp_port, &smtp_user, &smtp_pass, &from_address, &to_email, &subject, &body);
        });

        Ok(())
    }
}

fn send_email_blocking(
    smtp_host: &str,
    smtp_port: u16,
    smtp_user: &str,
    smtp_pass: &str,
    from: &str,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), String> {
    use lettre::{Message, SmtpTransport, Transport};
    use lettre::transport::smtp::authentication::Credentials;

    let email = Message::builder()
        .from(from.parse().map_err(|e| format!("Invalid from: {}", e))?)
        .to(to.parse().map_err(|e| format!("Invalid to: {}", e))?)
        .subject(subject)
        .body(body.to_string())
        .map_err(|e| format!("Build error: {}", e))?;

    let creds = Credentials::new(smtp_user.to_string(), smtp_pass.to_string());

    let mailer = SmtpTransport::relay(smtp_host)
        .map_err(|e| format!("Relay error: {}", e))?
        .credentials(creds)
        .port(smtp_port)
        .build();

    mailer.send(&email).map_err(|e| format!("Send error: {}", e))?;
    Ok(())
}

/// A no-op notifier for when email is not configured.
pub struct NoopNotifier;

impl Notifier for NoopNotifier {
    fn notify(
        &self,
        _job_name: &str,
        _run_id: &str,
        _status: RunStatus,
        _exit_code: Option<i32>,
        _stdout: &str,
        _stderr: &str,
        _to: &str,
    ) -> Result<(), String> {
        Ok(())
    }
}