use crate::Database;
use rcr_core::models::webhook::{CreateWebhookSubscription, WebhookSubscription};
use rcr_core::Error;

impl Database {
    pub async fn create_webhook(&self, create: CreateWebhookSubscription) -> Result<WebhookSubscription, Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let arg_mapping = serde_json::to_string(&create.arg_mapping).unwrap();

        sqlx::query(
            "INSERT INTO webhook_subscriptions (id, job_id, name, secret, arg_mapping) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&create.job_id)
        .bind(&create.name)
        .bind(&create.secret)
        .bind(&arg_mapping)
        .execute(self.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(WebhookSubscription {
            id,
            job_id: create.job_id,
            name: create.name,
            secret: create.secret,
            arg_mapping: create.arg_mapping,
        })
    }

    pub async fn get_webhook_by_name(&self, name: &str) -> Result<WebhookSubscription, Error> {
        let row = sqlx::query_as::<_, WebhookRow>(
            "SELECT * FROM webhook_subscriptions WHERE name = ?"
        )
        .bind(name)
        .fetch_one(self.pool())
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::Database(format!("Webhook not found: {}", name)),
            other => Error::Database(other.to_string()),
        })?;
        row.into_model()
    }

    pub async fn list_webhooks(&self) -> Result<Vec<WebhookSubscription>, Error> {
        let rows = sqlx::query_as::<_, WebhookRow>(
            "SELECT * FROM webhook_subscriptions ORDER BY name"
        )
        .fetch_all(self.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        rows.into_iter().map(|r| r.into_model()).collect()
    }

    pub async fn list_webhooks_for_job(&self, job_id: &str) -> Result<Vec<WebhookSubscription>, Error> {
        let rows = sqlx::query_as::<_, WebhookRow>(
            "SELECT * FROM webhook_subscriptions WHERE job_id = ? ORDER BY name"
        )
        .bind(job_id)
        .fetch_all(self.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        rows.into_iter().map(|r| r.into_model()).collect()
    }

    pub async fn delete_webhook(&self, id: &str) -> Result<(), Error> {
        let result = sqlx::query("DELETE FROM webhook_subscriptions WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(Error::Database(format!("Webhook not found: {}", id)));
        }
        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct WebhookRow {
    id: String,
    job_id: String,
    name: String,
    secret: String,
    arg_mapping: String,
}

impl WebhookRow {
    fn into_model(self) -> Result<WebhookSubscription, Error> {
        Ok(WebhookSubscription {
            id: self.id,
            job_id: self.job_id,
            name: self.name,
            secret: self.secret,
            arg_mapping: serde_json::from_str(&self.arg_mapping).unwrap_or_default(),
        })
    }
}