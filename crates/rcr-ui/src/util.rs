use chrono::{DateTime, Utc};
use rcr_core::models::{RunStatus, Trigger};

/// Format a duration in milliseconds to a human-readable string.
pub fn format_duration(ms: Option<i64>) -> String {
    match ms {
        None => "—".to_string(),
        Some(ms) if ms < 1000 => format!("{}ms", ms),
        Some(ms) if ms < 60000 => format!("{:.1}s", ms as f64 / 1000.0),
        Some(ms) => format!("{:.1}m", ms as f64 / 60000.0),
    }
}

/// Format a `DateTime` as a relative time string (e.g., "5 minutes ago").
pub fn format_time_ago(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now - *dt;
    let secs = diff.num_seconds();

    if secs < 0 {
        return "just now".to_string();
    }

    if secs < 60 {
        format!("{} second{} ago", secs, if secs == 1 { "" } else { "s" })
    } else if secs < 3600 {
        let mins = secs / 60;
        format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if secs < 86400 {
        let hours = secs / 3600;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else {
        let days = secs / 86400;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    }
}

/// Format a `DateTime` as an absolute time string.
pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Shorten a UUID to the first 8 characters.
pub fn short_id(id: &str) -> String {
    if id.len() > 8 {
        id[..8].to_string()
    } else {
        id.to_string()
    }
}

/// Get the CSS class and label for a run status badge.
pub fn status_badge(status: &RunStatus) -> (&'static str, &'static str) {
    match status {
        RunStatus::Success => ("badge badge-success", "✓ success"),
        RunStatus::Running => ("badge badge-primary", "↻ running"),
        RunStatus::Failed => ("badge badge-danger", "✕ failed"),
        RunStatus::Timeout => ("badge badge-warning", "⏱ timeout"),
        RunStatus::Skipped => ("badge badge-neutral", "− skipped"),
    }
}

/// Get the CSS class and label for an enabled/disabled badge.
pub fn enabled_badge(enabled: bool) -> (&'static str, &'static str) {
    if enabled {
        ("badge badge-success", "enabled")
    } else {
        ("badge badge-neutral", "disabled")
    }
}

/// Format a `Trigger` for display.
pub fn format_trigger(trigger: &Trigger) -> String {
    match trigger {
        Trigger::Schedule => "📅 schedule".to_string(),
        Trigger::Manual => "⚡ manual".to_string(),
    }
}

/// Format memory in KB to a human-readable string.
pub fn format_memory(kb: Option<i64>) -> String {
    match kb {
        None => "—".to_string(),
        Some(kb) => format!("{:.1} MB", kb as f64 / 1024.0),
    }
}

/// Format a percentage.
pub fn format_percent(pct: Option<f32>) -> String {
    match pct {
        None => "—".to_string(),
        Some(p) => format!("{:.1}%", p),
    }
}