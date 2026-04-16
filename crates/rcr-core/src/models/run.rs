use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::trigger::Trigger;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Running,
    Success,
    Failed,
    Timeout,
    Skipped,
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunStatus::Running => write!(f, "running"),
            RunStatus::Success => write!(f, "success"),
            RunStatus::Failed => write!(f, "failed"),
            RunStatus::Timeout => write!(f, "timeout"),
            RunStatus::Skipped => write!(f, "skipped"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub job_id: String,
    pub job_name: Option<String>,
    pub command: Option<String>,
    pub trigger: Trigger,
    pub status: RunStatus,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub error_message: Option<String>,
    pub cpu_pct: Option<f32>,
    pub mem_kb: Option<i64>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRun {
    pub job_id: String,
    pub command: String,
    pub trigger: Trigger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub id: String,
    pub job_id: String,
    pub job_name: String,
    pub command: String,
    pub trigger: Trigger,
    pub status: RunStatus,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<i64>,
    pub cpu_pct: Option<f32>,
    pub mem_kb: Option<i64>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunsFilter {
    pub job_id: Option<String>,
    pub status: Option<RunStatus>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunsResponse {
    pub runs: Vec<RunSummary>,
    pub total: i64,
}