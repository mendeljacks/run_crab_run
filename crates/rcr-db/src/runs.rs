use crate::Database;
use rcr_core::models::run::{CreateRun, Run, RunStatus, RunsFilter, RunSummary};
use rcr_core::models::trigger::Trigger;
use rcr_core::Error;

impl Database {
    pub async fn create_run(&self, create: CreateRun) -> Result<Run, Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        let trigger_str = create.trigger.to_string();
        let webhook_args_str = create
            .webhook_args
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap());

        sqlx::query(
            "INSERT INTO runs (id, job_id, trigger, status, started_at, webhook_args) VALUES (?, ?, ?, 'running', ?, ?)"
        )
        .bind(&id)
        .bind(&create.job_id)
        .bind(&trigger_str)
        .bind(now.to_rfc3339())
        .bind(&webhook_args_str)
        .execute(self.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(Run {
            id,
            job_id: create.job_id,
            trigger: create.trigger,
            status: RunStatus::Running,
            exit_code: None,
            stdout: None,
            stderr: None,
            error_message: None,
            cpu_pct: None,
            mem_kb: None,
            started_at: now,
            finished_at: None,
            duration_ms: None,
            webhook_args: create.webhook_args,
        })
    }

    pub async fn update_run_completed(
        &self,
        id: &str,
        status: RunStatus,
        exit_code: Option<i32>,
        stdout: Option<String>,
        stderr: Option<String>,
        error_message: Option<String>,
        cpu_pct: Option<f32>,
        mem_kb: Option<i64>,
        duration_ms: Option<i64>,
    ) -> Result<(), Error> {
        let now = chrono::Utc::now();
        let status_str = status.to_string();

        sqlx::query(
            "UPDATE runs SET status=?, exit_code=?, stdout=?, stderr=?, error_message=?, cpu_pct=?, mem_kb=?, finished_at=?, duration_ms=? WHERE id=?"
        )
        .bind(&status_str)
        .bind(exit_code)
        .bind(stdout)
        .bind(stderr)
        .bind(&error_message)
        .bind(cpu_pct)
        .bind(mem_kb)
        .bind(now.to_rfc3339())
        .bind(duration_ms)
        .bind(id)
        .execute(self.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_run(&self, id: &str) -> Result<Run, Error> {
        let row = sqlx::query_as::<_, RunRow>("SELECT * FROM runs WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::RunNotFound(id.to_string()),
                other => Error::Database(other.to_string()),
            })?;
        row.into_model()
    }

    pub async fn list_runs(&self, filter: RunsFilter) -> Result<Vec<RunSummary>, Error> {
        let mut query_str = String::from(
            "SELECT r.id, r.job_id, j.name as job_name, r.trigger, r.status, r.exit_code, r.duration_ms, r.cpu_pct, r.mem_kb, r.started_at, r.finished_at FROM runs r JOIN jobs j ON r.job_id = j.id WHERE 1=1"
        );

        if filter.job_id.is_some() {
            query_str.push_str(" AND r.job_id = ?");
        }
        if filter.status.is_some() {
            query_str.push_str(" AND r.status = ?");
        }

        query_str.push_str(" ORDER BY r.started_at DESC");

        if filter.limit.is_some() {
            query_str.push_str(" LIMIT ?");
        }
        if filter.offset.is_some() {
            query_str.push_str(" OFFSET ?");
        }

        let mut q = sqlx::query_as::<_, RunSummaryRow>(&query_str);

        if let Some(ref job_id) = filter.job_id {
            q = q.bind(job_id);
        }
        if let Some(ref status) = filter.status {
            q = q.bind(status.to_string());
        }
        if let Some(limit) = filter.limit {
            q = q.bind(limit);
        }
        if let Some(offset) = filter.offset {
            q = q.bind(offset);
        }

        let rows = q
            .fetch_all(self.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        rows.into_iter().map(|r| r.into_model()).collect()
    }

    pub async fn delete_run(&self, id: &str) -> Result<(), Error> {
        let result = sqlx::query("DELETE FROM runs WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(Error::RunNotFound(id.to_string()));
        }
        Ok(())
    }

    pub async fn cancel_run(&self, id: &str) -> Result<(), Error> {
        let now = chrono::Utc::now();
        sqlx::query(
            "UPDATE runs SET status='failed', error_message='cancelled', finished_at=? WHERE id=? AND status='running'"
        )
        .bind(now.to_rfc3339())
        .bind(id)
        .execute(self.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct RunRow {
    id: String,
    job_id: String,
    trigger: String,
    status: String,
    exit_code: Option<i32>,
    stdout: Option<String>,
    stderr: Option<String>,
    error_message: Option<String>,
    cpu_pct: Option<f32>,
    mem_kb: Option<i64>,
    started_at: String,
    finished_at: Option<String>,
    duration_ms: Option<i64>,
    webhook_args: Option<String>,
}

impl RunRow {
    fn into_model(self) -> Result<Run, Error> {
        Ok(Run {
            id: self.id,
            job_id: self.job_id,
            trigger: parse_trigger(&self.trigger)?,
            status: parse_status(&self.status)?,
            exit_code: self.exit_code,
            stdout: self.stdout,
            stderr: self.stderr,
            error_message: self.error_message,
            cpu_pct: self.cpu_pct,
            mem_kb: self.mem_kb,
            started_at: chrono::DateTime::parse_from_rfc3339(&self.started_at)
                .map_err(|e| Error::Database(e.to_string()))?
                .into(),
            finished_at: self.finished_at
                .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| Error::Database(e.to_string()))
                    .map(|d| d.into()))
                .transpose()?,
            duration_ms: self.duration_ms,
            webhook_args: self.webhook_args
                .and_then(|s| serde_json::from_str(&s).ok()),
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
struct RunSummaryRow {
    id: String,
    job_id: String,
    job_name: String,
    trigger: String,
    status: String,
    exit_code: Option<i32>,
    duration_ms: Option<i64>,
    cpu_pct: Option<f32>,
    mem_kb: Option<i64>,
    started_at: String,
    finished_at: Option<String>,
}

impl RunSummaryRow {
    fn into_model(self) -> Result<RunSummary, Error> {
        Ok(RunSummary {
            id: self.id,
            job_id: self.job_id,
            job_name: self.job_name,
            trigger: parse_trigger(&self.trigger)?,
            status: parse_status(&self.status)?,
            exit_code: self.exit_code,
            duration_ms: self.duration_ms,
            cpu_pct: self.cpu_pct,
            mem_kb: self.mem_kb,
            started_at: chrono::DateTime::parse_from_rfc3339(&self.started_at)
                .map_err(|e| Error::Database(e.to_string()))?
                .into(),
            finished_at: self.finished_at
                .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| Error::Database(e.to_string()))
                    .map(|d| d.into()))
                .transpose()?,
        })
    }
}

fn parse_trigger(s: &str) -> Result<Trigger, Error> {
    if s == "schedule" {
        Ok(Trigger::Schedule)
    } else if s == "manual" {
        Ok(Trigger::Manual)
    } else if let Some(name) = s.strip_prefix("webhook:") {
        Ok(Trigger::Webhook { name: name.to_string() })
    } else {
        Err(Error::Database(format!("Invalid trigger: {}", s)))
    }
}

fn parse_status(s: &str) -> Result<RunStatus, Error> {
    match s {
        "running" => Ok(RunStatus::Running),
        "success" => Ok(RunStatus::Success),
        "failed" => Ok(RunStatus::Failed),
        "timeout" => Ok(RunStatus::Timeout),
        "skipped" => Ok(RunStatus::Skipped),
        other => Err(Error::Database(format!("Invalid status: {}", other))),
    }
}