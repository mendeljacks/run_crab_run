use crate::Database;
use rcr_core::models::run::{CreateRun, Run, RunStatus, RunsFilter, RunsResponse, RunSummary};
use rcr_core::models::trigger::Trigger;
use rcr_core::Error;

impl Database {
    pub async fn create_run(&self, create: CreateRun) -> Result<Run, Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        let trigger_str = create.trigger.to_string();

        sqlx::query(
            "INSERT INTO runs (id, job_id, command, trigger, status, started_at) VALUES (?, ?, ?, ?, 'running', ?)"
        )
        .bind(&id)
        .bind(&create.job_id)
        .bind(&create.command)
        .bind(&trigger_str)
        .bind(now.to_rfc3339())
        .execute(self.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(Run {
            id,
            job_id: create.job_id,
            job_name: None, // populated on read via join
            command: Some(create.command),
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

    pub async fn update_run_status(&self, id: &str, status: RunStatus) -> Result<(), Error> {
        let status_str = status.to_string();
        sqlx::query("UPDATE runs SET status=? WHERE id=?")
            .bind(&status_str)
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn get_run(&self, id: &str) -> Result<Run, Error> {
        let row = sqlx::query_as::<_, RunRow>(
            "SELECT r.id, r.job_id, j.name as job_name, r.command, r.trigger, r.status, r.exit_code, r.stdout, r.stderr, r.error_message, r.cpu_pct, r.mem_kb, r.started_at, r.finished_at, r.duration_ms FROM runs r JOIN jobs j ON r.job_id = j.id WHERE r.id = ?"
        )
        .bind(id)
        .fetch_one(self.pool())
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::RunNotFound(id.to_string()),
            other => Error::Database(other.to_string()),
        })?;
        row.into_model()
    }

    pub async fn list_runs(&self, filter: RunsFilter) -> Result<RunsResponse, Error> {
        let mut where_clauses = vec!["1=1".to_string()];
        let mut count_where = vec!["1=1".to_string()];

        if filter.job_id.is_some() {
            where_clauses.push("r.job_id = ?".to_string());
            count_where.push("r.job_id = ?".to_string());
        }
        if filter.status.is_some() {
            where_clauses.push("r.status = ?".to_string());
            count_where.push("r.status = ?".to_string());
        }
        if filter.search.is_some() {
            where_clauses.push("(j.name LIKE ? OR r.id LIKE ?)".to_string());
            count_where.push("(j.name LIKE ? OR r.id LIKE ?)".to_string());
        }

        let where_str = where_clauses.join(" AND ");
        let count_where_str = count_where.join(" AND ");

        let sort_by = filter.sort_by.as_deref().unwrap_or("started_at");
        let sort_col = match sort_by {
            "duration" | "duration_ms" => "r.duration_ms",
            "status" => "r.status",
            "job" | "job_name" => "j.name",
            _ => "r.started_at",
        };
        let sort_dir = if filter.sort_order.as_deref() == Some("asc") { "ASC" } else { "DESC" };

        // Count query
        let count_sql = format!(
            "SELECT COUNT(*) as count FROM runs r JOIN jobs j ON r.job_id = j.id WHERE {}",
            count_where_str
        );

        let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
        if let Some(ref job_id) = filter.job_id {
            count_q = count_q.bind(job_id);
        }
        if let Some(ref status) = filter.status {
            count_q = count_q.bind(status.to_string());
        }
        if let Some(ref search) = filter.search {
            count_q = count_q.bind(format!("%{}%", search));
            count_q = count_q.bind(format!("%{}%", search));
        }

        let total = count_q
            .fetch_one(self.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        // Data query
        let data_sql = format!(
            "SELECT r.id, r.job_id, j.name as job_name_from_join, r.command, r.trigger, r.status, r.exit_code, r.duration_ms, r.cpu_pct, r.mem_kb, r.started_at, r.finished_at FROM runs r JOIN jobs j ON r.job_id = j.id WHERE {} ORDER BY {} {}",
            where_str, sort_col, sort_dir
        );

        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(50);

        let data_sql = format!("{} LIMIT ? OFFSET ?", data_sql);

        let mut q = sqlx::query_as::<_, RunSummaryRow>(&data_sql);
        if let Some(ref job_id) = filter.job_id {
            q = q.bind(job_id);
        }
        if let Some(ref status) = filter.status {
            q = q.bind(status.to_string());
        }
        if let Some(ref search) = filter.search {
            q = q.bind(format!("%{}%", search));
            q = q.bind(format!("%{}%", search));
        }
        q = q.bind(limit);
        q = q.bind(offset);

        let rows = q
            .fetch_all(self.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        let runs = rows.into_iter().map(|r| r.into_model()).collect::<Result<Vec<_>, _>>()?;

        Ok(RunsResponse { runs, total })
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
    job_name: Option<String>,
    command: Option<String>,
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
}

impl RunRow {
    fn into_model(self) -> Result<Run, Error> {
        Ok(Run {
            id: self.id,
            job_id: self.job_id,
            job_name: self.job_name,
            command: self.command,
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
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
struct RunSummaryRow {
    id: String,
    job_id: String,
    job_name_from_join: String,
    command: Option<String>,
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
            job_name: self.job_name_from_join,
            command: self.command.unwrap_or_default(),
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