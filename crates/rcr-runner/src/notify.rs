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

        let status_color = match status {
            RunStatus::Success => "#059669",
            RunStatus::Failed => "#dc2626",
            RunStatus::Timeout => "#d97706",
            _ => "#64748b",
        };

        let subject = format!("[Run Crab Run] {} — {}", job_name, status_str);

        let exit_code_str = exit_code.map(|c| c.to_string()).unwrap_or_else(|| "N/A".into());
        let short_id = if run_id.len() > 8 { &run_id[..8] } else { run_id };

        let stdout_display = if stdout.len() > 4000 { &stdout[..4000] } else { stdout };
        let stderr_display = if stderr.len() > 4000 { &stderr[..4000] } else { stderr };

        let html_body = format!(r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 0; background: #f8f9fa;">
<table width="100%%" cellpadding="0" cellspacing="0" style="background: #f8f9fa; padding: 2rem;">
<tr><td align="center">
<table width="600" cellpadding="0" cellspacing="0" style="background: #fff; border: 1px solid #e2e8f0; border-radius: 12px; overflow: hidden;">
<tr><td style="background: {status_color}; padding: 1.5rem; text-align: center;">
<h1 style="color: #fff; margin: 0; font-size: 1.5rem;">🦀 {status_str}</h1>
</td></tr>
<tr><td style="padding: 1.5rem;">
<table width="100%%" cellpadding="0" cellspacing="0" style="margin-bottom: 1.5rem;">
<tr><td style="padding: 0.5rem 0; border-bottom: 1px solid #f1f5f9;"><strong style="color: #64748b; font-size: 0.75rem; text-transform: uppercase;">Job</strong></td>
<td style="padding: 0.5rem 0; border-bottom: 1px solid #f1f5f9; text-align: right; font-weight: 600;">{job_name}</td></tr>
<tr><td style="padding: 0.5rem 0; border-bottom: 1px solid #f1f5f9;"><strong style="color: #64748b; font-size: 0.75rem; text-transform: uppercase;">Run ID</strong></td>
<td style="padding: 0.5rem 0; border-bottom: 1px solid #f1f5f9; text-align: right; font-family: monospace;">{short_id}</td></tr>
<tr><td style="padding: 0.5rem 0; border-bottom: 1px solid #f1f5f9;"><strong style="color: #64748b; font-size: 0.75rem; text-transform: uppercase;">Exit Code</strong></td>
<td style="padding: 0.5rem 0; border-bottom: 1px solid #f1f5f9; text-align: right;">{exit_code_str}</td></tr>
</table>
<h2 style="font-size: 0.9rem; color: #1a202c; margin: 0 0 0.5rem;">Output</h2>
<pre style="background: #0f172a; color: #e2e8f0; padding: 1rem; border-radius: 8px; font-size: 0.78rem; overflow-x: auto; max-height: 300px; overflow-y: auto; white-space: pre-wrap; word-break: break-all;">{stdout_display}</pre>
{stderr_section}
</td></tr>
<tr><td style="background: #f8f9fa; padding: 1rem; text-align: center; font-size: 0.75rem; color: #94a3b8;">
🦀 Run Crab Run — Job Execution Notification
</td></tr>
</table>
</td></tr>
</table>
</body>
</html>"#,
            status_color = status_color,
            status_str = status_str,
            job_name = html_escape(job_name),
            short_id = short_id,
            exit_code_str = exit_code_str,
            stdout_display = html_escape(stdout_display),
            stderr_section = if stderr_display.is_empty() {
                String::new()
            } else {
                format!(r#"<h2 style="font-size: 0.9rem; color: #1a202c; margin: 1rem 0 0.5rem;">Errors</h2>
<pre style="background: #0f172a; color: #fca5a5; padding: 1rem; border-radius: 8px; font-size: 0.78rem; border: 2px solid #dc2626; overflow-x: auto; max-height: 200px; overflow-y: auto; white-space: pre-wrap; word-break: break-all;">{stderr_display}</pre>"#, stderr_display = html_escape(stderr_display))
            }
        );

        let text_body = format!(
            "Job: {}\nRun ID: {}\nStatus: {}\nExit code: {}\n\nStdout (last 2000 chars):\n{}\n\nStderr (last 2000 chars):\n{}",
            job_name,
            run_id,
            status_str,
            exit_code_str,
            &stdout[..stdout.len().min(2000)],
            &stderr[..stderr.len().min(2000)],
        );

        let smtp_host = self.smtp_host.clone();
        let smtp_port = self.smtp_port;
        let smtp_user = self.smtp_user.clone();
        let smtp_pass = self.smtp_pass.clone();
        let from_address = self.from_address.clone();
        let to_email = to_email.to_string();

        std::thread::spawn(move || {
            let _ = send_email_blocking(&smtp_host, smtp_port, &smtp_user, &smtp_pass, &from_address, &to_email, &subject, &html_body, &text_body);
        });

        Ok(())
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn send_email_blocking(
    smtp_host: &str,
    smtp_port: u16,
    smtp_user: &str,
    smtp_pass: &str,
    from: &str,
    to: &str,
    subject: &str,
    html_body: &str,
    text_body: &str,
) -> Result<(), String> {
    use lettre::{Message, SmtpTransport, Transport};
    use lettre::message::{MultiPart, SinglePart};
    use lettre::transport::smtp::authentication::Credentials;

    let email = Message::builder()
        .from(from.parse().map_err(|e| format!("Invalid from: {}", e))?)
        .to(to.parse().map_err(|e| format!("Invalid to: {}", e))?)
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::plain(text_body.to_string()))
                .singlepart(SinglePart::html(html_body.to_string()))
        )
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