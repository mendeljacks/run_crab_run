use crate::Database;
use rcr_core::models::job::{CreateJob, Job, UpdateJob};
use rcr_core::Error;

impl Database {
    pub async fn create_job(&self, create: CreateJob) -> Result<Job, Error> {
        let job = Job::new(create);
        sqlx::query(
            "INSERT INTO jobs (id, name, command, schedule, enabled, max_concurrent, env_vars, webhook_secret, containerized, container_image, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&job.id)
        .bind(&job.name)
        .bind(&job.command)
        .bind(&job.schedule)
        .bind(job.enabled)
        .bind(job.max_concurrent)
        .bind(serde_json::to_string(&job.env_vars).unwrap())
        .bind(&job.webhook_secret)
        .bind(job.containerized)
        .bind(&job.container_image)
        .bind(job.created_at.to_rfc3339())
        .bind(job.updated_at.to_rfc3339())
        .execute(self.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(job)
    }

    pub async fn get_job(&self, id: &str) -> Result<Job, Error> {
        let row = sqlx::query_as::<_, JobRow>(
            "SELECT id, name, command, schedule, enabled, max_concurrent, env_vars, webhook_secret, containerized, container_image, created_at, updated_at FROM jobs WHERE id = ?"
        )
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
        let rows = sqlx::query_as::<_, JobRow>(
            "SELECT id, name, command, schedule, enabled, max_concurrent, env_vars, webhook_secret, containerized, container_image, created_at, updated_at FROM jobs ORDER BY created_at DESC"
        )
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
        let max_concurrent = update.max_concurrent.unwrap_or(existing.max_concurrent);
        let env_vars = update.env_vars.unwrap_or(existing.env_vars);
        let webhook_secret = update.webhook_secret.or(existing.webhook_secret);
        let containerized = update.containerized.unwrap_or(existing.containerized);
        let container_image = update.container_image.or(existing.container_image);

        let now = chrono::Utc::now();

        sqlx::query(
            "UPDATE jobs SET name=?, command=?, schedule=?, enabled=?, max_concurrent=?, env_vars=?, webhook_secret=?, containerized=?, container_image=?, updated_at=? WHERE id=?"
        )
        .bind(&name)
        .bind(&command)
        .bind(&schedule)
        .bind(enabled)
        .bind(max_concurrent)
        .bind(serde_json::to_string(&env_vars).unwrap())
        .bind(&webhook_secret)
        .bind(containerized)
        .bind(&container_image)
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
            "SELECT id, name, command, schedule, enabled, max_concurrent, env_vars, webhook_secret, containerized, container_image, created_at, updated_at FROM jobs WHERE enabled = 1 AND schedule IS NOT NULL"
        )
        .fetch_all(self.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        rows.into_iter().map(|r| r.into_model()).collect()
    }
}

// Row type for SQLx mapping - only includes columns we use
#[derive(Debug, sqlx::FromRow)]
struct JobRow {
    id: String,
    name: String,
    command: String,
    schedule: Option<String>,
    enabled: bool,
    max_concurrent: i32,
    env_vars: String,
    webhook_secret: Option<String>,
    containerized: bool,
    container_image: Option<String>,
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
            max_concurrent: self.max_concurrent,
            env_vars: serde_json::from_str(&self.env_vars).unwrap_or_default(),
            webhook_secret: self.webhook_secret,
            containerized: self.containerized,
            container_image: self.container_image,
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .map_err(|e| Error::Database(e.to_string()))?
                .into(),
            updated_at: chrono::DateTime::parse_from_rfc3339(&self.updated_at)
                .map_err(|e| Error::Database(e.to_string()))?
                .into(),
        })
    }
}