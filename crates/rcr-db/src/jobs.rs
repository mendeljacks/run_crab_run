use crate::Database;
use rcr_core::models::job::{CreateJob, Job, UpdateJob};
use rcr_core::Error;

impl Database {
    pub async fn create_job(&self, create: CreateJob) -> Result<Job, Error> {
        let job = Job::new(create);
        sqlx::query(
            "INSERT INTO jobs (id, name, command, schedule, enabled, timeout_secs, max_concurrent, env_vars, webhook_secret, tags, notify_on, notify_email, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&job.id)
        .bind(&job.name)
        .bind(&job.command)
        .bind(&job.schedule)
        .bind(job.enabled)
        .bind(job.timeout_secs)
        .bind(job.max_concurrent)
        .bind(serde_json::to_string(&job.env_vars).unwrap())
        .bind(&job.webhook_secret)
        .bind(serde_json::to_string(&job.tags).unwrap())
        .bind(&job.notify_on)
        .bind(&job.notify_email)
        .bind(job.created_at.to_rfc3339())
        .bind(job.updated_at.to_rfc3339())
        .execute(self.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(job)
    }

    pub async fn get_job(&self, id: &str) -> Result<Job, Error> {
        let row = sqlx::query_as::<_, JobRow>("SELECT * FROM jobs WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::JobNotFound(id.to_string()),
                other => Error::Database(other.to_string()),
            })?;
        row.into_model()
    }

    pub async fn list_jobs(&self) -> Result<Vec<Job>, Error> {
        let rows = sqlx::query_as::<_, JobRow>("SELECT * FROM jobs ORDER BY created_at DESC")
            .fetch_all(self.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        rows.into_iter().map(|r| r.into_model()).collect()
    }

    pub async fn update_job(&self, id: &str, update: UpdateJob) -> Result<Job, Error> {
        let existing = self.get_job(id).await?;

        let name = update.name.unwrap_or(existing.name);
        let command = update.command.unwrap_or(existing.command);
        let schedule = update.schedule.or(existing.schedule);
        let enabled = update.enabled.unwrap_or(existing.enabled);
        let timeout_secs = update.timeout_secs.or(existing.timeout_secs);
        let max_concurrent = update.max_concurrent.unwrap_or(existing.max_concurrent);
        let env_vars = update.env_vars.unwrap_or(existing.env_vars);
        let webhook_secret = update.webhook_secret.or(existing.webhook_secret);
        let tags = update.tags.unwrap_or(existing.tags);
        let notify_on = update.notify_on.or(existing.notify_on);
        let notify_email = update.notify_email.or(existing.notify_email);

        let now = chrono::Utc::now();

        sqlx::query(
            "UPDATE jobs SET name=?, command=?, schedule=?, enabled=?, timeout_secs=?, max_concurrent=?, env_vars=?, webhook_secret=?, tags=?, notify_on=?, notify_email=?, updated_at=? WHERE id=?"
        )
        .bind(&name)
        .bind(&command)
        .bind(&schedule)
        .bind(enabled)
        .bind(timeout_secs)
        .bind(max_concurrent)
        .bind(serde_json::to_string(&env_vars).unwrap())
        .bind(&webhook_secret)
        .bind(serde_json::to_string(&tags).unwrap())
        .bind(&notify_on)
        .bind(&notify_email)
        .bind(now.to_rfc3339())
        .bind(id)
        .execute(self.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        self.get_job(id).await
    }

    pub async fn delete_job(&self, id: &str) -> Result<(), Error> {
        let result = sqlx::query("DELETE FROM jobs WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(Error::JobNotFound(id.to_string()));
        }
        Ok(())
    }

    pub async fn get_enabled_scheduled_jobs(&self) -> Result<Vec<Job>, Error> {
        let rows = sqlx::query_as::<_, JobRow>(
            "SELECT * FROM jobs WHERE enabled = 1 AND schedule IS NOT NULL"
        )
        .fetch_all(self.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        rows.into_iter().map(|r| r.into_model()).collect()
    }
}

// Row type for SQLx mapping
#[derive(Debug, sqlx::FromRow)]
struct JobRow {
    id: String,
    name: String,
    command: String,
    schedule: Option<String>,
    enabled: bool,
    timeout_secs: Option<i64>,
    max_concurrent: i32,
    env_vars: String,
    webhook_secret: Option<String>,
    tags: String,
    notify_on: Option<String>,
    notify_email: Option<String>,
    created_at: String,
    updated_at: String,
}

impl JobRow {
    fn into_model(self) -> Result<Job, Error> {
        Ok(Job {
            id: self.id,
            name: self.name,
            command: self.command,
            schedule: self.schedule,
            enabled: self.enabled,
            timeout_secs: self.timeout_secs,
            max_concurrent: self.max_concurrent,
            env_vars: serde_json::from_str(&self.env_vars).unwrap_or_default(),
            webhook_secret: self.webhook_secret,
            tags: serde_json::from_str(&self.tags).unwrap_or_default(),
            notify_on: self.notify_on,
            notify_email: self.notify_email,
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .map_err(|e| Error::Database(e.to_string()))?
                .into(),
            updated_at: chrono::DateTime::parse_from_rfc3339(&self.updated_at)
                .map_err(|e| Error::Database(e.to_string()))?
                .into(),
        })
    }
}